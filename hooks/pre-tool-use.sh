#!/bin/bash
# PreToolUse hook for superego
# Gates write tools based on conversation phase

# Read tool input from stdin
INPUT=$(cat)

# Extract tool name from input JSON
TOOL_NAME=$(echo "$INPUT" | jq -r '.tool_name // .toolName // "unknown"')

# Skip if superego is disabled
if [ "$SUPEREGO_DISABLED" = "1" ]; then
    echo '{"permissionDecision": "allow", "reason": "superego disabled"}'
    exit 0
fi

# Skip check for superego's own transcript (recursion prevention)
TRANSCRIPT_PATH=$(echo "$INPUT" | jq -r '.tool_input.transcript_path // .tool_input.file_path // ""')
if [[ "$TRANSCRIPT_PATH" == *".superego"* ]]; then
    echo '{"permissionDecision": "allow", "reason": "superego internal"}'
    exit 0
fi

# Call sg check
RESULT=$(sg check --tool-name "$TOOL_NAME" 2>/dev/null)
EXIT_CODE=$?

# Parse sg check output
DECISION=$(echo "$RESULT" | jq -r '.decision // "allow"')
REASON=$(echo "$RESULT" | jq -r '.reason // "unknown"')
PHASE=$(echo "$RESULT" | jq -r '.phase // ""')

if [ "$DECISION" = "allow" ]; then
    echo "{\"permissionDecision\": \"allow\", \"reason\": \"$REASON\"}"
else
    # Build deny message with context
    if [ -n "$PHASE" ]; then
        MSG="Phase is $PHASE. Confirm approach before writing code."
    else
        MSG="$REASON"
    fi
    echo "{\"permissionDecision\": \"deny\", \"reason\": \"$MSG\"}"
fi
