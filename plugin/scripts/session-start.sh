#!/bin/bash
set -u  # Exit on undefined variables
# SessionStart hook for superego
# Handles three scenarios:
# 1. .superego/ exists + binary present → inject full contract
# 2. .superego/ exists + binary missing → offer to install binary
# 3. .superego/ doesn't exist → offer to initialize superego
#
# AIDEV-NOTE: Uses additionalContext to inject context into Claude's session.

# Skip entirely if superego is disabled
if [ "${SUPEREGO_DISABLED:-}" = "1" ]; then
    exit 0
fi

# Use CLAUDE_PROJECT_DIR if available, otherwise current directory
PROJECT_DIR="${CLAUDE_PROJECT_DIR:-.}"

# Validate PROJECT_DIR exists and is a directory
if [ ! -d "$PROJECT_DIR" ]; then
    exit 0
fi

# SCENARIO 3: .superego/ doesn't exist - offer to initialize
if [ ! -d "$PROJECT_DIR/.superego" ]; then
    cat << 'EOFINNER'
{
  "hookSpecificOutput": {
    "hookEventName": "SessionStart",
    "additionalContext": "SUPEREGO AVAILABLE: The superego plugin is installed but not initialized for this project. Superego is a metacognitive advisor that provides feedback on your work.\n\nTo enable it, ask the user which mode they prefer:\n- **always** (default): Automatic evaluation at checkpoints (Stop, large changes, ExitPlanMode)\n- **pull**: You decide when to call `sg review` at decision points (less intrusive)\n\nThen run: `sg init` followed by `sg mode <chosen_mode>`"
  }
}
EOFINNER
    exit 0
fi

# Fetch latest prompt from GitHub (silent, non-blocking)
# Updates .superego/prompt.md if fetch succeeds, otherwise keeps existing
PROMPT_URL="https://raw.githubusercontent.com/cloud-atlas-ai/superego/main/default_prompt.md"
PROMPT_PATH="$PROJECT_DIR/.superego/prompt.md"
curl -sS --max-time 5 "$PROMPT_URL" -o "$PROMPT_PATH.tmp" 2>/dev/null && \
    mv "$PROMPT_PATH.tmp" "$PROMPT_PATH" 2>/dev/null || \
    rm -f "$PROMPT_PATH.tmp" 2>/dev/null

# .superego/ exists - log session start
echo "[$(date '+%H:%M:%S')] [session] Session started" >> "$PROJECT_DIR/.superego/hook.log" 2>/dev/null

# Clean up stale pending_change.txt files from previous sessions
# These can persist if a session crashes mid-evaluation, causing "phantom edit" warnings
rm -f "$PROJECT_DIR/.superego/pending_change.txt" 2>/dev/null
find "$PROJECT_DIR/.superego/sessions" -name "pending_change.txt" -delete 2>/dev/null

# SCENARIO 2: .superego/ exists but binary missing - offer to install
if ! command -v sg &> /dev/null; then
    echo "[$(date '+%H:%M:%S')] [session] sg binary not found - requesting install" >> "$PROJECT_DIR/.superego/hook.log" 2>/dev/null
    cat << 'EOFINNER'
{
  "hookSpecificOutput": {
    "hookEventName": "SessionStart",
    "additionalContext": "SUPEREGO SETUP REQUIRED: The superego plugin is active (.superego/ exists) but the 'sg' binary is missing. The binary is needed for LLM-based evaluation. Install options: 'brew install cloud-atlas-ai/superego/superego' (macOS, ~10s) or 'cargo install superego' (cross-platform, ~30s, requires Rust). Ask the user which they prefer, or if they want to skip superego this session."
  }
}
EOFINNER
    exit 0
fi

# SCENARIO 1: Everything present - check mode and inject appropriate context
MODE=$(sg mode 2>/dev/null || echo "always")
echo "[$(date '+%H:%M:%S')] [session] Mode: $MODE" >> "$PROJECT_DIR/.superego/hook.log" 2>/dev/null

if [ "$MODE" = "pull" ]; then
    # Pull mode: Claude decides when to evaluate using `sg review`
    cat << 'EOFINNER'
{
  "hookSpecificOutput": {
    "hookEventName": "SessionStart",
    "additionalContext": "SUPEREGO AVAILABLE (pull mode): This project has superego for metacognitive oversight. Use `sg review` at decision points:\n- Before committing to a plan or approach\n- When choosing between alternatives\n- Before non-trivial implementations\n- When the task feels complex or uncertain\n- Before claiming work is done\n\nSuperego catches strategic mistakes (wrong approach, over-engineering, scope creep). Call it when you need a second opinion, not automatically.\n\nTo switch to automatic mode: `sg mode always`"
  }
}
EOFINNER
else
    # Always mode: automatic evaluation at checkpoints
    cat << 'EOFINNER'
{
  "hookSpecificOutput": {
    "hookEventName": "SessionStart",
    "additionalContext": "SUPEREGO ACTIVE: This project uses superego, a metacognitive advisor that monitors your work. When you receive SUPEREGO FEEDBACK, critically evaluate it: if you agree, incorporate it into your approach; if you disagree on non-trivial feedback, escalate to the user explaining both perspectives. Superego feedback reflects concerns about your reasoning, approach, or alignment with the user's goals - it deserves serious consideration, not just acknowledgment.\n\nTo switch to pull mode (you decide when to review): `sg mode pull`"
  }
}
EOFINNER
fi
