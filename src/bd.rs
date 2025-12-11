/// Beads (bd) integration for task state
///
/// Task state comes from bd, not LLM conversation analysis.
/// AIDEV-NOTE: Simplified - removed unused functions.

use std::process::Command;
use serde::Deserialize;

/// Issue from bd list --json
#[derive(Debug, Clone, Deserialize)]
pub struct BdIssue {
    pub id: String,
    pub title: String,
}

/// Error type for bd operations
#[derive(Debug)]
pub enum BdError {
    CommandFailed(String),
    ParseError(String),
    NotInitialized,
}

impl std::fmt::Display for BdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BdError::CommandFailed(msg) => write!(f, "bd command failed: {}", msg),
            BdError::ParseError(msg) => write!(f, "Failed to parse bd output: {}", msg),
            BdError::NotInitialized => write!(f, "bd not initialized in this project"),
        }
    }
}

impl std::error::Error for BdError {}

/// Get issues in progress
fn get_in_progress() -> Result<Vec<BdIssue>, BdError> {
    let output = Command::new("bd")
        .args(["list", "--status", "in_progress", "--json"])
        .output()
        .map_err(|e| BdError::CommandFailed(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("not initialized") || stderr.contains("No database") {
            return Err(BdError::NotInitialized);
        }
        return Err(BdError::CommandFailed(stderr.to_string()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Handle empty output
    if stdout.trim().is_empty() || stdout.trim() == "[]" {
        return Ok(Vec::new());
    }

    serde_json::from_str(&stdout)
        .map_err(|e| BdError::ParseError(format!("{}: {}", e, stdout)))
}

/// Check if bd is initialized
fn is_initialized() -> bool {
    Command::new("bd")
        .args(["stats"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Evaluation result based on bd state
#[derive(Debug)]
pub struct BdEvaluation {
    /// Are we in read-only mode (no tasks in progress)?
    pub read_only: bool,
    /// Current task if any
    pub current_task: Option<BdIssue>,
    /// Feedback message if there's an issue
    pub feedback: Option<String>,
}

/// Evaluate current state based on bd
pub fn evaluate() -> Result<BdEvaluation, BdError> {
    if !is_initialized() {
        // No bd = no constraints (for now)
        return Ok(BdEvaluation {
            read_only: false,
            current_task: None,
            feedback: None,
        });
    }

    let tasks = get_in_progress()?;

    if tasks.is_empty() {
        Ok(BdEvaluation {
            read_only: true,
            current_task: None,
            feedback: Some(
                "No task in progress. Claim a task with `bd update <id> --status in_progress` before making changes.".to_string()
            ),
        })
    } else if tasks.len() > 1 {
        let task_list: Vec<_> = tasks.iter().map(|t| format!("{}: {}", t.id, t.title)).collect();
        Ok(BdEvaluation {
            read_only: false, // Allow but warn
            current_task: Some(tasks[0].clone()),
            feedback: Some(format!(
                "Multiple tasks in progress ({}). Consider focusing on one at a time.",
                task_list.join(", ")
            )),
        })
    } else {
        Ok(BdEvaluation {
            read_only: false,
            current_task: Some(tasks[0].clone()),
            feedback: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_initialized() {
        // This will depend on whether bd is installed and initialized
        // Just verify the function doesn't panic
        let _ = is_initialized();
    }
}
