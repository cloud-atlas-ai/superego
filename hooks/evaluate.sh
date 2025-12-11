#!/bin/bash
# Superego evaluation hook
# Used by: Stop (after response), PreCompact (before context truncation)
#
# AIDEV-NOTE: Single script for all evaluation triggers. Evaluates
# everything since last_evaluated timestamp.

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

# Extract transcript path from hook input
TRANSCRIPT_PATH=$(echo "$INPUT" | jq -r '.transcript_path // .transcriptPath // ""')

# Skip if no transcript path
if [ -z "$TRANSCRIPT_PATH" ] || [ "$TRANSCRIPT_PATH" = "null" ]; then
    exit 0
fi

# Skip if this is superego's own transcript (recursion prevention)
if [[ "$TRANSCRIPT_PATH" == *".superego"* ]]; then
    exit 0
fi

# Run LLM evaluation
# Output goes to stderr for debugging, stdout is ignored
sg evaluate-llm --transcript-path "$TRANSCRIPT_PATH" >&2 2>&1 || true

# Always exit 0 - evaluation shouldn't block Claude from stopping
exit 0
