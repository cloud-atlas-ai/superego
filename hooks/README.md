# Superego Hooks

These hooks integrate superego with Claude Code for LLM-based evaluation.

## Architecture

```
Stop / PreCompact ──► sg evaluate-llm ──► Feedback queue + decision journal
                     (evaluates new messages since last_evaluated)

User types...

UserPromptSubmit ───► sg get-feedback ──► Injects pending feedback as context
                     (surfaces superego advice to Claude)
```

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
               "command": "/path/to/higher-peak/hooks/evaluate.sh"
             }
           ]
         }
       ],
       "PreCompact": [
         {
           "hooks": [
             {
               "type": "command",
               "command": "/path/to/higher-peak/hooks/evaluate.sh"
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
       ]
     }
   }
   ```

## How It Works

### Evaluate Hook (Stop + PreCompact)
- Runs when Claude finishes responding OR before context compaction
- Calls `sg evaluate-llm` with transcript path
- Evaluates all messages since `last_evaluated` timestamp
- Writes feedback to queue and decision journal if concerns found

### UserPromptSubmit Hook
- Runs when user submits a prompt
- Calls `sg get-feedback` to retrieve pending feedback
- Injects feedback as context for Claude to consider

## Commands

```bash
# LLM-based evaluation (triggered by hooks)
sg evaluate-llm --transcript-path /path/to/transcript.jsonl

# Check for pending feedback (instant)
sg has-feedback

# Get and clear pending feedback
sg get-feedback

# View decision history
sg history --limit 5
```

## Environment Variables

- `SUPEREGO_DISABLED=1` - Bypass all superego evaluation

## Troubleshooting

Check superego state:
```bash
cat .superego/state.json
```

View decision history:
```bash
sg history --limit 5
```
