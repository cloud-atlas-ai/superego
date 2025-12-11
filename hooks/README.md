# Superego Hooks

These hooks integrate superego with Claude Code for bd-based phase gating.

## Architecture

```
Stop hook ──────────► sg evaluate-bd ──────► Updates state + writes feedback
                      (runs when Claude finishes)

User types...

UserPromptSubmit ───► sg has-feedback ─────► If blocking, surfaces to Claude
                      (runs when user submits)

PreToolUse ─────────► sg check ────────────► Gates writes based on state
                      (runs before write tools)
```

## Phase Detection via bd

Phase is determined by bd task state, not LLM conversation analysis:
- **No tasks in_progress** → read-only mode (exploring)
- **Tasks in_progress** → ready phase (writes allowed)

This is observable, auditable, and fast (no LLM calls).

## Installation

1. Build and install `sg`:
   ```bash
   cargo build --release
   cp target/release/sg /usr/local/bin/  # or add to PATH
   ```

2. Initialize superego in your project:
   ```bash
   cd /path/to/your/project
   sg init
   ```

3. Add hooks to your Claude Code settings (`.claude/settings.json`):
   ```json
   {
     "hooks": {
       "Stop": [
         {
           "hooks": [
             {
               "type": "command",
               "command": "/path/to/higher-peak/hooks/stop.sh"
             }
           ]
         }
       ],
       "UserPromptSubmit": [
         {
           "hooks": [
             {
               "type": "command",
               "command": "/path/to/higher-peak/hooks/user-prompt-submit.sh"
             }
           ]
         }
       ],
       "PreToolUse": [
         {
           "matcher": "Edit|Write|Bash|Task|NotebookEdit",
           "hooks": [
             {
               "type": "command",
               "command": "/path/to/higher-peak/hooks/pre-tool-use.sh"
             }
           ]
         }
       ]
     }
   }
   ```

## How It Works

### Stop Hook
- Runs when Claude finishes responding (before user types)
- Calls `sg evaluate-bd` to check bd state
- Updates `.superego/state.json` and writes feedback if issues

### UserPromptSubmit Hook
- Runs when user submits a prompt
- Calls `sg has-feedback` to check for pending feedback
- If blocking feedback exists, surfaces it to Claude
- If non-blocking, injects as context

### PreToolUse Hook
- Runs before write tools (Edit, Write, Bash, Task, NotebookEdit)
- Calls `sg check` which uses state set by evaluate-bd
- Blocks if no tasks are in progress

## Commands

```bash
# Fast bd-based evaluation (triggered by Stop hook)
sg evaluate-bd

# Check for pending feedback (instant, for hooks)
sg has-feedback

# Get and clear pending feedback
sg get-feedback

# Check if tool action is allowed
sg check --tool-name Edit

# Set manual override for next blocked action
sg override "user approved"

# View decision history
sg history --limit 5
```

## Environment Variables

- `SUPEREGO_DISABLED=1` - Bypass all superego checks

## Troubleshooting

Check superego state:
```bash
cat .superego/state.json
```

Check bd task status:
```bash
bd list --status in_progress
```

View decision history:
```bash
sg history --limit 5
```

Manually set override:
```bash
sg override "user approved"
```
