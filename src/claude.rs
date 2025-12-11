/// Claude CLI invocation
///
/// Calls the Claude Code CLI for superego evaluation.

use serde::{Deserialize, Serialize};
use std::process::Command;
use std::time::Duration;

/// Response from Claude CLI in JSON format
#[derive(Debug, Clone, Deserialize)]
pub struct ClaudeResponse {
    #[serde(rename = "type")]
    pub response_type: String,
    pub subtype: Option<String>,
    pub is_error: bool,
    pub duration_ms: u64,
    pub result: String,
    pub session_id: String,
    pub total_cost_usd: f64,
}

/// Error type for Claude invocation
#[derive(Debug)]
pub enum ClaudeError {
    CommandFailed(String),
    ParseError(serde_json::Error),
    IoError(std::io::Error),
    Timeout,
}

impl std::fmt::Display for ClaudeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClaudeError::CommandFailed(msg) => write!(f, "Claude command failed: {}", msg),
            ClaudeError::ParseError(e) => write!(f, "Failed to parse Claude response: {}", e),
            ClaudeError::IoError(e) => write!(f, "IO error: {}", e),
            ClaudeError::Timeout => write!(f, "Claude command timed out"),
        }
    }
}

impl std::error::Error for ClaudeError {}

impl From<std::io::Error> for ClaudeError {
    fn from(e: std::io::Error) -> Self {
        ClaudeError::IoError(e)
    }
}

impl From<serde_json::Error> for ClaudeError {
    fn from(e: serde_json::Error) -> Self {
        ClaudeError::ParseError(e)
    }
}

/// Options for Claude invocation
#[derive(Debug, Clone, Default)]
pub struct ClaudeOptions {
    /// Model to use (default: sonnet)
    pub model: Option<String>,
    /// Session ID for continuation
    pub session_id: Option<String>,
    /// Timeout in seconds
    pub timeout_secs: Option<u64>,
    /// Don't persist session to disk
    pub no_session_persistence: bool,
}

/// Invoke Claude CLI with a system prompt and user message
///
/// # Arguments
/// * `system_prompt` - System prompt for Claude
/// * `message` - User message / context
/// * `options` - Invocation options
///
/// # Returns
/// * `Ok(ClaudeResponse)` - Successful response
/// * `Err(ClaudeError)` - Error during invocation
pub fn invoke(
    system_prompt: &str,
    message: &str,
    options: ClaudeOptions,
) -> Result<ClaudeResponse, ClaudeError> {
    let mut cmd = Command::new("claude");

    // Non-interactive mode with JSON output
    cmd.arg("-p").arg("--output-format").arg("json");

    // System prompt
    cmd.arg("--system-prompt").arg(system_prompt);

    // Model (default to sonnet for cost efficiency)
    let model = options.model.unwrap_or_else(|| "sonnet".to_string());
    cmd.arg("--model").arg(&model);

    // Session handling
    if let Some(session_id) = &options.session_id {
        cmd.arg("--resume").arg(session_id);
    }

    // Don't persist session by default for superego
    if options.no_session_persistence {
        cmd.arg("--no-session-persistence");
    }

    // The message is passed as the prompt argument
    cmd.arg(message);

    // Execute the command
    let output = cmd.output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ClaudeError::CommandFailed(stderr.to_string()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse JSON response
    let response: ClaudeResponse = serde_json::from_str(&stdout)?;

    Ok(response)
}

/// Parse superego evaluation result from Claude response
///
/// The result field should contain JSON like:
/// {"phase": "ready", "confidence": 0.9, ...}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SuperegoEvaluation {
    pub phase: String,
    pub confidence: Option<f64>,
    pub approved_scope: Option<String>,
    pub concerns: Option<Vec<Concern>>,
    pub suggestion: Option<String>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Concern {
    #[serde(rename = "type")]
    pub concern_type: String,
    pub description: String,
}

/// Extract JSON from Claude's result, handling markdown code blocks
pub fn extract_json(result: &str) -> Option<&str> {
    // Try to find JSON in markdown code block
    if let Some(start) = result.find("```json") {
        let content_start = start + 7;
        if let Some(end) = result[content_start..].find("```") {
            return Some(result[content_start..content_start + end].trim());
        }
    }

    // Try to find JSON in generic code block
    if let Some(start) = result.find("```") {
        let content_start = start + 3;
        // Skip the optional language identifier
        let content_start = result[content_start..]
            .find('\n')
            .map(|n| content_start + n + 1)
            .unwrap_or(content_start);
        if let Some(end) = result[content_start..].find("```") {
            return Some(result[content_start..content_start + end].trim());
        }
    }

    // Try to parse the whole thing as JSON
    if result.trim().starts_with('{') {
        return Some(result.trim());
    }

    None
}

/// Parse the evaluation from Claude's result
pub fn parse_evaluation(result: &str) -> Result<SuperegoEvaluation, ClaudeError> {
    let json_str = extract_json(result).ok_or_else(|| {
        ClaudeError::CommandFailed("No JSON found in Claude response".to_string())
    })?;

    serde_json::from_str(json_str).map_err(ClaudeError::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_markdown() {
        let result = r#"Here's my evaluation:
```json
{"phase": "ready", "confidence": 0.9}
```
Done."#;
        let json = extract_json(result).unwrap();
        assert!(json.contains("ready"));
    }

    #[test]
    fn test_extract_json_raw() {
        let result = r#"{"phase": "discussing", "confidence": 0.7}"#;
        let json = extract_json(result).unwrap();
        assert!(json.contains("discussing"));
    }

    #[test]
    fn test_parse_evaluation() {
        let result = r#"{"phase": "ready", "confidence": 0.9, "approved_scope": "implement auth"}"#;
        let eval = parse_evaluation(result).unwrap();
        assert_eq!(eval.phase, "ready");
        assert_eq!(eval.confidence, Some(0.9));
        assert_eq!(eval.approved_scope, Some("implement auth".to_string()));
    }

    #[test]
    fn test_parse_evaluation_with_concerns() {
        let result = r#"{
            "phase": "discussing",
            "confidence": 0.8,
            "concerns": [
                {"type": "local_maxima", "description": "Haven't explored alternatives"}
            ],
            "reason": "User hasn't confirmed yet"
        }"#;
        let eval = parse_evaluation(result).unwrap();
        assert_eq!(eval.phase, "discussing");
        assert_eq!(eval.concerns.as_ref().unwrap().len(), 1);
    }
}
