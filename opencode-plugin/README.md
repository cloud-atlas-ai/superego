# Superego OpenCode Plugin

TypeScript adapter for running superego with [OpenCode](https://opencode.ai).

## Status: In Development

This plugin enables superego's metacognitive oversight for OpenCode users, using the same `.superego/` configuration as the Claude Code plugin.

## Architecture

Superego uses a **shared core with adapters** pattern:

```
.superego/                    # Shared core (language-agnostic)
├── prompt.md                 # Evaluation criteria
├── config.yaml               # Settings (threshold, model, etc.)
├── sessions/<id>/            # Per-session state & decisions
│   ├── state.json
│   ├── feedback
│   └── decisions/
└── ...

plugin/                       # Claude Code adapter (shell scripts)
opencode-plugin/              # OpenCode adapter (TypeScript)
```

### What's Shared

- Evaluation prompt (`prompt.md`)
- Configuration schema (`config.yaml`)
- Decision format: `DECISION: ALLOW|BLOCK\n\n<feedback>`
- Session state and decision journal structure

### What Adapters Handle

| Concern | Claude Code | OpenCode |
|---------|-------------|----------|
| Hook registration | Shell scripts in `plugin/` | TypeScript in `.opencode/plugin/` |
| LLM invocation | Claude CLI | Configurable (Gemini, etc.) |
| Transcript access | `$CLAUDE_TRANSCRIPT_PATH` env var | `client.session.messages()` SDK |
| Feedback delivery | Block hook with JSON | TBD |

## Hook Mapping

| Superego Hook | Claude Code | OpenCode |
|---------------|-------------|----------|
| Session start (inject contract) | `SessionStart` | `session.created` |
| Pre-tool evaluation | `PreToolUse` | `tool.execute.before` |
| Final evaluation | `Stop` | `session.idle` |

## Installation

```bash
# Copy plugin to OpenCode plugin directory
cp -r opencode-plugin/.opencode ~/.config/opencode/
```

## Configuration

Uses the same `.superego/config.yaml` as Claude Code:

```yaml
# LLM backend for evaluation (OpenCode adapter)
eval_model: gemini-2.5-pro  # or: claude, gpt-4, ollama/llama3, etc.
```

## Development

```bash
cd opencode-plugin
bun install
bun test
```
