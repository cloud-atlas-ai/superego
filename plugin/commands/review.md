# Superego Review

On-demand review of changes using the current superego prompt.

## Usage

- `/superego:review` — Review staged changes (falls back to uncommitted if nothing staged)
- `/superego:review staged` — Review only staged changes
- `/superego:review pr` — Review PR diff vs base branch
- `/superego:review <file>` — Review changes in a specific file

## How It Works

1. Run `sg review [target]` to invoke the review
2. Superego uses the current prompt (code or writing) to evaluate the changes
3. Returns constructive feedback (advisory, not blocking)

## Examples

```bash
# Review what you're about to commit
sg review

# Review your entire PR before requesting review
sg review pr

# Review changes to a specific file
sg review src/main.rs
```

## Notes

- Uses the currently active prompt (`sg prompt show` to check)
- For writing projects, switch to writing prompt first: `sg prompt switch writing`
- This is advisory feedback, not a blocking hook
