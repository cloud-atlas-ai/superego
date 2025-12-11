# Claude Code Hooks Reference

> Source: https://code.claude.com/docs/en/hooks

## Hook Events

| Event | Trigger | Key Use Cases |
|-------|---------|---------------|
| **SessionStart** | New/resumed session, `/clear`, post-compact | Load context, set env vars |
| **SessionEnd** | Session ends | Cleanup, logging |
| **PreToolUse** | Before tool executes | Validate/modify/block tool calls |
| **PostToolUse** | After tool completes | React to results, add context to Claude |
| **PermissionRequest** | Permission dialog shown | Auto-allow/deny permissions |
| **UserPromptSubmit** | User sends prompt | Validate prompts, inject context |
| **PreCompact** | Before context compaction | Save state before compaction |
| **Stop** | Main agent finishes | Force continuation |
| **SubagentStop** | Subagent (Task) finishes | Force subagent continuation |
| **Notification** | Various notifications | Custom alerts |

## Configuration Locations

```
~/.claude/settings.json          # Global (user)
.claude/settings.json            # Project (committed)
.claude/settings.local.json      # Project (local, not committed)
```

## Hook Structure

```json
{
  "hooks": {
    "EventName": [
      {
        "matcher": "ToolPattern",  // regex, empty = all (PreToolUse/PostToolUse/PermissionRequest only)
        "hooks": [
          {
            "type": "command",     // or "prompt" for LLM evaluation
            "command": "your-command",
            "timeout": 60          // optional, seconds
          }
        ]
      }
    ]
  }
}
```

## Hook Types

1. **Command** (`type: "command"`) - Runs shell command
2. **Prompt** (`type: "prompt"`) - LLM evaluates decision (Stop/SubagentStop mainly)

## Exit Code Semantics

| Exit Code | Meaning | stdout | stderr |
|-----------|---------|--------|--------|
| **0** | Success | Shown in verbose mode; injected as context for SessionStart/UserPromptSubmit | Ignored |
| **2** | Blocking error | Ignored | Fed back to Claude as error |
| **Other** | Non-blocking error | Ignored | Shown in verbose mode |

## JSON Output Schema

All hooks can return JSON to stdout (exit 0):

```json
{
  "continue": true,              // false stops Claude entirely
  "stopReason": "string",        // shown to user when continue=false
  "suppressOutput": false,       // hide from verbose mode
  "systemMessage": "string",     // warning shown to user

  "decision": "block",           // PostToolUse/Stop/SubagentStop
  "reason": "explanation",       // shown to Claude when blocked

  "hookSpecificOutput": {
    "hookEventName": "PreToolUse",
    "permissionDecision": "allow|deny|ask",
    "permissionDecisionReason": "...",
    "updatedInput": { },         // modify tool inputs
    "additionalContext": "..."   // SessionStart/UserPromptSubmit
  }
}
```

## Event-Specific Details

### SessionStart

**Matchers:** `startup`, `resume`, `clear`, `compact`

**Context Injection:** stdout (plain text or JSON `additionalContext`) is added to conversation context.

**Environment Persistence:** Write to `$CLAUDE_ENV_FILE` to set env vars for session:
```bash
echo 'export MY_VAR=value' >> "$CLAUDE_ENV_FILE"
```

### PreToolUse

**Common Matchers:** `Bash`, `Write`, `Edit`, `Read`, `Glob`, `Grep`, `Task`, `WebFetch`, `WebSearch`

**Decisions:**
- `"allow"` - Bypass permission system
- `"deny"` - Block with reason shown to Claude
- `"ask"` - Show permission dialog

**Modify Inputs:** Use `updatedInput` to change tool parameters before execution.

### PostToolUse

**Decisions:**
- `"block"` - Prompts Claude with `reason`
- `undefined` - No action

### UserPromptSubmit

**Context Injection:** stdout is added as context (plain text or JSON `additionalContext`).

**Blocking:** Use `"decision": "block"` with `"reason"` to reject prompts.

### Stop / SubagentStop

**Decisions:**
- `"block"` - Prevent stopping, must provide `reason` for Claude to continue
- `undefined` - Allow stop

### Notification

**Matchers:** `permission_prompt`, `idle_prompt`, `auth_success`, `elicitation_dialog`

### PreCompact

**Matchers:** `manual` (from `/compact`), `auto` (context full)

## Tool Matchers

- Exact match: `Write`
- Regex: `Edit|Write`, `Notebook.*`
- All tools: `*` or `""`
- MCP tools: `mcp__<server>__<tool>` (e.g., `mcp__memory__.*`)

## Environment Variables

| Variable | Availability | Description |
|----------|--------------|-------------|
| `CLAUDE_PROJECT_DIR` | All hooks | Absolute path to project root |
| `CLAUDE_ENV_FILE` | SessionStart only | File path to persist env vars |
| `CLAUDE_PLUGIN_ROOT` | Plugin hooks | Plugin directory path |
| `CLAUDE_CODE_REMOTE` | All hooks | `"true"` if web environment |

## Execution Details

- **Timeout:** 60 seconds default, configurable per command
- **Parallelization:** All matching hooks run in parallel
- **Deduplication:** Identical commands are deduplicated

## Example: SessionStart Context Injection

```json
{
  "hooks": {
    "SessionStart": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "bd prime"
          }
        ]
      }
    ]
  }
}
```

## Example: PreToolUse Auto-Approve

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Read",
        "hooks": [
          {
            "type": "command",
            "command": "$CLAUDE_PROJECT_DIR/.claude/hooks/auto-approve-docs.sh"
          }
        ]
      }
    ]
  }
}
```

## Example: Stop Hook to Force Continuation

```json
{
  "hooks": {
    "Stop": [
      {
        "hooks": [
          {
            "type": "prompt",
            "prompt": "Check if all tasks complete. Input: $ARGUMENTS. Return {\"decision\": \"approve\" or \"block\", \"reason\": \"...\"}"
          }
        ]
      }
    ]
  }
}
```

## Debugging

```bash
claude --debug  # See hook execution details
```

Check `/hooks` in Claude Code to see registered hooks.
