# Superego Hooks

These hooks integrate superego with Claude Code.

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
       "UserPromptSubmit": [
         {
           "type": "command",
           "command": "/path/to/higher-peak/hooks/user-prompt-submit.sh"
         }
       ],
       "PreToolUse": [
         {
           "matcher": "Edit|Write|Bash|Task|NotebookEdit",
           "type": "command",
           "command": "/path/to/higher-peak/hooks/pre-tool-use.sh"
         }
       ]
     }
   }
   ```

## How It Works

### UserPromptSubmit Hook
- Runs on every user message
- Calls `sg evaluate` with the transcript path
- Infers conversation phase (EXPLORING/DISCUSSING/READY)
- Updates `.superego/state.json`

### PreToolUse Hook
- Runs before write tools (Edit, Write, Bash, Task, NotebookEdit)
- Calls `sg check` to verify phase
- Blocks if phase is not READY
- Returns allow/deny decision to Claude

## Environment Variables

- `SUPEREGO_DISABLED=1` - Bypass all superego checks

## Troubleshooting

Check superego state:
```bash
cat .superego/state.json
```

View decision history:
```bash
sg history --limit 5
```

Manually set override:
```bash
sg override "user approved"
```
