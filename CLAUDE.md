# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Superego is a metacognitive advisor for Claude Code. It monitors conversations, evaluates Claude's approach, and provides feedback via Claude Code hooks before Claude finishes or makes large edits.

**Binary:** `sg` (short for superego)

## Build & Test Commands

```bash
cargo build              # Development build
cargo build --release    # Release build
cargo test               # Run all tests
cargo test <test_name>   # Run single test
cargo run -- <args>      # Run with args (e.g., cargo run -- init)
```

## Architecture

### Core Flow
1. **SessionStart hook** → Injects superego contract into Claude's context
2. **PreToolUse hook** → Evaluates large Edit/Write operations (≥20 lines by default)
3. **Stop/PreCompact hooks** → Runs LLM evaluation before Claude finishes

### Module Structure

- `main.rs` - CLI entry point using clap, defines all subcommands
- `init.rs` - Creates `.superego/` directory structure and configures Claude Code hooks
- `evaluate.rs` - LLM-based evaluation logic; calls Claude to review conversation transcripts
- `claude.rs` - Wrapper for invoking Claude CLI (`claude -p --output-format json`)
- `audit.rs` - Audit command: aggregates decisions and runs LLM analysis
- `transcript/` - Parses Claude Code JSONL transcript files
  - `types.rs` - Serde structs for transcript entries (User, Assistant, Summary, etc.)
  - `reader.rs` - Reads and filters transcript messages since last evaluation
- `bd.rs` - Integration with beads (`bd`) task tracking; provides current task context
- `state.rs` - Manages `.superego/state.json` (last_evaluated timestamp)
- `decision.rs` - Decision journal for audit trail (`.superego/decisions/`); includes `read_all_sessions()`
- `feedback.rs` - Feedback queue (`.superego/feedback` file)

### Hook Scripts (embedded in binary via `include_str!`)

Located in `hooks/`:
- `session-start.sh` - Injects superego contract via `additionalContext`
- `evaluate.sh` - Runs `sg evaluate-llm`, blocks if concerns found
- `pre-tool-use.sh` - Evaluates pending changes before large Edit/Write operations

### Key Design Patterns

**Recursion prevention:** Superego's own Claude calls set `SUPEREGO_DISABLED=1` to prevent hooks from triggering on themselves.

**Decision format:** LLM responses must follow `DECISION: ALLOW|BLOCK\n\n<feedback>` format. Unknown decisions default to BLOCK for safety.

**State tracking:** `last_evaluated` timestamp in state.json ensures only new conversation content is evaluated.

## Environment Variables

- `SUPEREGO_DISABLED=1` - Disables superego entirely
- `SUPEREGO_CHANGE_THRESHOLD=N` - Lines required to trigger PreToolUse evaluation (default: 20)

## Files Created by `sg init`

```
.superego/
├── prompt.md          # Customizable system prompt for evaluation
├── state.json         # Evaluation state (last_evaluated timestamp)
├── config.yaml        # Placeholder config
├── decisions/         # Decision journal (audit trail) - JSON files
├── sessions/          # Per-session state and decisions
│   └── <session-id>/
│       ├── state.json
│       ├── decisions/
│       └── superego_session
└── feedback           # Pending feedback queue (transient)

.claude/
├── settings.json      # Hook configuration
└── hooks/superego/    # Hook scripts
```

## CLI Commands

- `sg init` - Initialize superego for a project
- `sg audit` - Analyze decision history with LLM (patterns, timeline, insights)
- `sg audit --json` - JSON output for programmatic use
- `sg history --limit N` - Show recent decisions
- `sg check` - Verify hooks are up to date
- `sg reset` - Remove superego configuration

## Decision Journal

Decisions are stored as JSON files in `.superego/decisions/` (base) and `.superego/sessions/<id>/decisions/` (per-session).

**Format:**
```json
{
  "timestamp": "2025-12-17T22:16:39.368740Z",
  "session_id": "855f6568-...",
  "type": "feedback_delivered",
  "context": "The feedback text...",
  "trigger": null
}
```

**YAML Migration:** Legacy `.yaml` decision files can be converted to JSON:
```bash
# Simple converter (requires python3 for JSON escaping)
find .superego -name "*.yaml" -path "*/decisions/*" | while read f; do
  # Extract fields and output JSON
  timestamp=$(grep "^timestamp:" "$f" | cut -d' ' -f2-)
  session_id=$(grep "^session_id:" "$f" | cut -d' ' -f2-)
  type=$(grep "^type:" "$f" | cut -d' ' -f2-)
  context=$(awk '/^context:/{flag=1;next}/^[a-z_]+:/{flag=0}flag' "$f" | sed 's/^  //')
  context_json=$(echo "$context" | python3 -c 'import sys,json;print(json.dumps(sys.stdin.read().strip()))')
  echo "{\"timestamp\":\"$timestamp\",\"session_id\":\"$session_id\",\"type\":\"$type\",\"context\":$context_json,\"trigger\":null}" > "${f%.yaml}.json"
done
```

## Debugging

### Evaluation failures
Check `.superego/hook.log` for recent activity:
```bash
tail -50 .superego/hook.log
```

### Common issues

**"EOF while parsing" error:** stdout not piped in `claude.rs`. The `invoke()` function MUST have:
```rust
cmd.stdout(Stdio::piped());
cmd.stderr(Stdio::piped());
```
Without this, `wait_with_output()` returns empty and JSON parsing fails.

**No decisions recorded:** Check that evaluations complete successfully in hook.log. Look for "Evaluation complete" not "ERROR".

**SKIP messages in log:**
- `SUPEREGO_DISABLED=1` - Normal for superego's own Claude calls (recursion prevention)
- `stop_hook_active=true` - Normal, prevents infinite loops after blocking once

### Testing the full flow
```bash
# Trigger evaluation manually
sg evaluate-llm --transcript-path <path-to-jsonl> --session-id test

# Check for new decision files
find .superego -name "*.json" -path "*/decisions/*" -mmin -5

# Verify hook receives transcript path
grep "Running:" .superego/hook.log | tail -5
```
