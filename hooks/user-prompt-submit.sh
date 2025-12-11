#!/bin/bash
# UserPromptSubmit hook for superego
# Injects pending feedback as context for Claude
#
# AIDEV-NOTE: No blocking, no severity checks. Just inject feedback
# as context and let Claude decide how to handle it.

# Read hook input from stdin
INPUT=$(cat)

# Skip if superego is disabled
if [ "$SUPEREGO_DISABLED" = "1" ]; then
    exit 0
fi

# Check if superego is initialized
if [ ! -d ".superego" ]; then
    exit 0
fi

# Skip if this is superego's own transcript (recursion prevention)
TRANSCRIPT_PATH=$(echo "$INPUT" | jq -r '.transcript_path // .transcriptPath // ""')
if [[ "$TRANSCRIPT_PATH" == *".superego"* ]]; then
    exit 0
fi

# Check for pending feedback (exit 0 = yes, exit 1 = no)
if sg has-feedback 2>/dev/null; then
    # Get and inject feedback as context
    FEEDBACK=$(sg get-feedback 2>/dev/null)
    echo "SUPEREGO FEEDBACK:"
    echo "$FEEDBACK"
fi

exit 0
