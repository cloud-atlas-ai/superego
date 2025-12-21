//! Open Horizons integration for superego
//!
//! Optional integration that logs superego decisions to OH endeavors.
//! Enabled when OH_API_URL and OH_API_KEY environment variables are set.
//!
//! AIDEV-NOTE: This is completely optional - if OH is not configured,
//! superego works exactly as before. The integration enables higher-level
//! coordination by connecting metacognitive feedback to strategic context.

use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::Path;

/// OH API configuration from environment
#[derive(Debug, Clone)]
pub struct OhConfig {
    pub api_url: String,
    pub api_key: String,
}

impl OhConfig {
    /// Try to load configuration from environment variables
    /// Returns None if OH_API_KEY is not set (OH_API_URL has default)
    pub fn from_env() -> Option<Self> {
        let api_key = env::var("OH_API_KEY").ok()?;
        let api_url = env::var("OH_API_URL").unwrap_or_else(|_| "http://localhost:3001".to_string());
        Some(OhConfig { api_url, api_key })
    }
}

/// Error type for OH operations
#[derive(Debug)]
pub enum OhError {
    /// HTTP request failed
    RequestFailed(String),
    /// Failed to parse response
    ParseError(String),
    /// OH not configured (not an error, just skip)
    NotConfigured,
    /// API returned an error
    ApiError(u16, String),
}

impl std::fmt::Display for OhError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OhError::RequestFailed(msg) => write!(f, "OH request failed: {}", msg),
            OhError::ParseError(msg) => write!(f, "Failed to parse OH response: {}", msg),
            OhError::NotConfigured => write!(f, "OH not configured"),
            OhError::ApiError(status, msg) => write!(f, "OH API error ({}): {}", status, msg),
        }
    }
}

impl std::error::Error for OhError {}

/// A context (personal or shared space) in OH
#[derive(Debug, Clone, Deserialize)]
pub struct OhContext {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
}

/// An endeavor (mission, aim, initiative, task) in OH
#[derive(Debug, Clone, Deserialize)]
pub struct OhEndeavor {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub node_type: Option<String>,
}

/// Response from creating a log entry
#[derive(Debug, Clone, Deserialize)]
pub struct LogResponse {
    pub log: Option<LogEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LogEntry {
    pub id: String,
}

/// OH API client
#[derive(Debug, Clone)]
pub struct OhClient {
    config: OhConfig,
}

impl OhClient {
    /// Create a new OH client if configuration is available
    pub fn new() -> Result<Self, OhError> {
        let config = OhConfig::from_env().ok_or(OhError::NotConfigured)?;
        Ok(OhClient { config })
    }

    /// Check if OH is available and reachable
    pub fn is_available(&self) -> bool {
        // Simple health check - try to get contexts
        self.get_contexts().is_ok()
    }

    /// Get all contexts the user has access to
    pub fn get_contexts(&self) -> Result<Vec<OhContext>, OhError> {
        let url = format!("{}/api/contexts", self.config.api_url);

        let response = attohttpc::get(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .send()
            .map_err(|e| OhError::RequestFailed(e.to_string()))?;

        if !response.is_success() {
            let status = response.status().as_u16();
            let body = response.text().unwrap_or_default();
            return Err(OhError::ApiError(status, body));
        }

        // OH returns { contexts: [...] } or just [...]
        let body = response.text().map_err(|e| OhError::ParseError(e.to_string()))?;

        // Try parsing as { contexts: [...] } first
        #[derive(Deserialize)]
        struct ContextsResponse {
            contexts: Vec<OhContext>,
        }

        if let Ok(wrapper) = serde_json::from_str::<ContextsResponse>(&body) {
            return Ok(wrapper.contexts);
        }

        // Fall back to direct array
        serde_json::from_str(&body).map_err(|e| OhError::ParseError(format!("{}: {}", e, body)))
    }

    /// Get endeavors in a context
    pub fn get_endeavors(&self, context_id: &str) -> Result<Vec<OhEndeavor>, OhError> {
        let url = format!(
            "{}/api/dashboard?contextId={}",
            self.config.api_url,
            urlencoding::encode(context_id)
        );

        let response = attohttpc::get(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .send()
            .map_err(|e| OhError::RequestFailed(e.to_string()))?;

        if !response.is_success() {
            let status = response.status().as_u16();
            let body = response.text().unwrap_or_default();
            return Err(OhError::ApiError(status, body));
        }

        let body = response.text().map_err(|e| OhError::ParseError(e.to_string()))?;

        // Dashboard returns { nodes: [...] }
        #[derive(Deserialize)]
        struct DashboardResponse {
            nodes: Vec<OhEndeavor>,
        }

        if let Ok(wrapper) = serde_json::from_str::<DashboardResponse>(&body) {
            return Ok(wrapper.nodes);
        }

        // Fall back to direct array
        serde_json::from_str(&body).map_err(|e| OhError::ParseError(format!("{}: {}", e, body)))
    }

    /// Log a decision to an endeavor
    pub fn log_decision(
        &self,
        endeavor_id: &str,
        content: &str,
        log_date: Option<&str>,
    ) -> Result<String, OhError> {
        let url = format!("{}/api/logs", self.config.api_url);

        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let date = log_date.unwrap_or(&today);

        #[derive(Serialize)]
        struct LogRequest<'a> {
            entity_type: &'a str,
            entity_id: &'a str,
            content: &'a str,
            content_type: &'a str,
            log_date: &'a str,
        }

        let request = LogRequest {
            entity_type: "endeavor",
            entity_id: endeavor_id,
            content,
            content_type: "markdown",
            log_date: date,
        };

        let response = attohttpc::post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .map_err(|e| OhError::RequestFailed(e.to_string()))?
            .send()
            .map_err(|e| OhError::RequestFailed(e.to_string()))?;

        if !response.is_success() {
            let status = response.status().as_u16();
            let body = response.text().unwrap_or_default();
            return Err(OhError::ApiError(status, body));
        }

        let body = response.text().map_err(|e| OhError::ParseError(e.to_string()))?;
        let log_response: LogResponse =
            serde_json::from_str(&body).map_err(|e| OhError::ParseError(format!("{}: {}", e, body)))?;

        Ok(log_response
            .log
            .map(|l| l.id)
            .unwrap_or_else(|| "unknown".to_string()))
    }
}

/// Check if OH integration is available (env vars set)
pub fn is_configured() -> bool {
    OhConfig::from_env().is_some()
}

/// Parse oh_endeavor_id from config file content
/// Extracted for testability (avoids env var interference in tests)
fn parse_config_for_endeavor_id(content: &str) -> Option<String> {
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("oh_endeavor_id:") {
            if let Some(value) = line.strip_prefix("oh_endeavor_id:") {
                let value = value.trim().trim_matches('"').trim_matches('\'');
                if !value.is_empty() {
                    return Some(value.to_string());
                }
            }
        }
    }
    None
}

/// Get the configured OH endeavor ID from environment or config file
///
/// Priority:
/// 1. OH_ENDEAVOR_ID environment variable (for overrides)
/// 2. oh_endeavor_id in .superego/config.yaml
///
/// Returns None if not configured (OH integration will be skipped)
pub fn get_endeavor_id(superego_dir: &Path) -> Option<String> {
    // First check env var (allows override)
    if let Ok(id) = env::var("OH_ENDEAVOR_ID") {
        if !id.is_empty() {
            return Some(id);
        }
    }

    // Then check config.yaml
    let config_path = superego_dir.join("config.yaml");
    if config_path.exists() {
        if let Ok(content) = fs::read_to_string(&config_path) {
            return parse_config_for_endeavor_id(&content);
        }
    }

    None
}

/// Full OH integration configuration
/// Combines API config with endeavor targeting
#[derive(Debug, Clone)]
pub struct OhIntegration {
    pub client: OhClient,
    pub endeavor_id: String,
}

impl OhIntegration {
    /// Try to create a fully configured OH integration
    /// Returns None if either API is not configured or endeavor ID is not set
    pub fn new(superego_dir: &Path) -> Option<Self> {
        let client = OhClient::new().ok()?;
        let endeavor_id = get_endeavor_id(superego_dir)?;
        Some(OhIntegration { client, endeavor_id })
    }

    /// Log superego feedback to the configured endeavor
    pub fn log_feedback(&self, feedback: &str) -> Result<String, OhError> {
        let content = format!("## Superego Feedback\n\n{}", feedback);
        self.client.log_decision(&self.endeavor_id, &content, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_from_env_missing() {
        // Clear env vars for test
        env::remove_var("OH_API_KEY");
        env::remove_var("OH_API_URL");

        assert!(OhConfig::from_env().is_none());
    }

    #[test]
    fn test_is_configured_false_when_no_env() {
        env::remove_var("OH_API_KEY");
        env::remove_var("OH_API_URL");

        assert!(!is_configured());
    }

    #[test]
    fn test_client_new_fails_when_not_configured() {
        env::remove_var("OH_API_KEY");
        env::remove_var("OH_API_URL");

        let result = OhClient::new();
        assert!(matches!(result, Err(OhError::NotConfigured)));
    }

    // Tests for parse_config_for_endeavor_id (no env var interference)

    #[test]
    fn test_parse_config_extracts_endeavor_id() {
        let content = "# Config\neval_interval_minutes: 5\noh_endeavor_id: my-endeavor-123\n";
        let result = parse_config_for_endeavor_id(content);
        assert_eq!(result, Some("my-endeavor-123".to_string()));
    }

    #[test]
    fn test_parse_config_strips_double_quotes() {
        let content = "oh_endeavor_id: \"quoted-value\"";
        let result = parse_config_for_endeavor_id(content);
        assert_eq!(result, Some("quoted-value".to_string()));
    }

    #[test]
    fn test_parse_config_strips_single_quotes() {
        let content = "oh_endeavor_id: 'single-quoted'";
        let result = parse_config_for_endeavor_id(content);
        assert_eq!(result, Some("single-quoted".to_string()));
    }

    #[test]
    fn test_parse_config_returns_none_when_missing() {
        let content = "eval_interval_minutes: 5\nmodel: sonnet";
        let result = parse_config_for_endeavor_id(content);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_config_returns_none_for_empty_value() {
        let content = "oh_endeavor_id: ";
        let result = parse_config_for_endeavor_id(content);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_config_handles_whitespace() {
        let content = "  oh_endeavor_id:   spaced-value  \n";
        let result = parse_config_for_endeavor_id(content);
        assert_eq!(result, Some("spaced-value".to_string()));
    }
}
