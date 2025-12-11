#!/bin/bash
# UserPromptSubmit hook for superego
# Evaluates conversation phase on each user message

# Read hook input from stdin
INPUT=$(cat)

# Skip if superego is disabled
if [ "$SUPEREGO_DISABLED" = "1" ]; then
    exit 0
fi

# Extract transcript path from input
TRANSCRIPT_PATH=$(echo "$INPUT" | jq -r '.transcript_path // .transcriptPath // ""')

# Skip if no transcript path (shouldn't happen)
if [ -z "$TRANSCRIPT_PATH" ] || [ "$TRANSCRIPT_PATH" = "null" ]; then
    echo "Warning: No transcript path provided" >&2
    exit 0
fi

# Skip if transcript is superego's own (recursion prevention)
if [[ "$TRANSCRIPT_PATH" == *".superego"* ]]; then
    exit 0
fi

# Check if superego is initialized
if [ ! -d ".superego" ]; then
    echo "Superego not initialized in this project" >&2
    exit 0
fi

# Call sg evaluate (output goes to stderr for visibility, doesn't affect Claude)
sg evaluate --transcript-path "$TRANSCRIPT_PATH" >&2

# Always exit 0 - evaluation failures shouldn't block user
exit 0
