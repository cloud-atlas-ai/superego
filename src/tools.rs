/// Tool classification for superego gating
///
/// Read tools are always allowed (no phase check needed).
/// Write tools require READY phase or override.

/// Tools that only read - always allowed
const READ_TOOLS: &[&str] = &[
    "Glob",
    "Grep",
    "Read",
    "LS",
    "WebFetch",
    "WebSearch",
    "TaskOutput",
];

/// Tools that modify state - require READY phase
const WRITE_TOOLS: &[&str] = &[
    "Edit",
    "Write",
    "Bash",
    "Task",
    "NotebookEdit",
    "KillShell",
    "TodoWrite",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolClass {
    Read,
    Write,
    Unknown,
}

/// Classify a tool by name
pub fn classify(tool_name: &str) -> ToolClass {
    if READ_TOOLS.contains(&tool_name) {
        ToolClass::Read
    } else if WRITE_TOOLS.contains(&tool_name) {
        ToolClass::Write
    } else {
        // AIDEV-NOTE: Unknown tools are treated as write tools for safety
        // This ensures new tools are gated until explicitly classified
        ToolClass::Unknown
    }
}

/// Check if a tool requires phase gating
pub fn requires_gating(tool_name: &str) -> bool {
    match classify(tool_name) {
        ToolClass::Read => false,
        ToolClass::Write | ToolClass::Unknown => true,
    }
}

/// Check if a tool is a read-only tool
pub fn is_read_only(tool_name: &str) -> bool {
    classify(tool_name) == ToolClass::Read
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_tools() {
        assert_eq!(classify("Read"), ToolClass::Read);
        assert_eq!(classify("Glob"), ToolClass::Read);
        assert_eq!(classify("Grep"), ToolClass::Read);
        assert_eq!(classify("WebSearch"), ToolClass::Read);
        assert!(!requires_gating("Read"));
    }

    #[test]
    fn test_write_tools() {
        assert_eq!(classify("Edit"), ToolClass::Write);
        assert_eq!(classify("Write"), ToolClass::Write);
        assert_eq!(classify("Bash"), ToolClass::Write);
        assert_eq!(classify("Task"), ToolClass::Write);
        assert!(requires_gating("Bash"));
    }

    #[test]
    fn test_unknown_tools_are_gated() {
        // Unknown tools should be treated as write for safety
        assert_eq!(classify("SomeNewTool"), ToolClass::Unknown);
        assert!(requires_gating("SomeNewTool"));
    }
}
