---
name: superego
description: Metacognitive oversight. Invoke with "$superego" to evaluate, "$superego init" to set up, "$superego remove" to uninstall.
---

# Superego - Metacognitive Oversight

## $superego (evaluate)

Evaluate the current conversation for alignment, focus, and proportionality.

**When to use:**
- Before committing to a plan or approach
- When choosing between alternatives
- Before non-trivial implementations
- When the task feels complex or uncertain
- Before claiming work is "done"

**Run:**
```bash
sg evaluate-codex
```

**Handle results:**
- `has_concerns: true` → STOP, show feedback to user verbatim, ask before proceeding
- `has_concerns: false` → Tell user "Superego: No concerns", continue
- `skipped: true` → Proceed normally (not an error)

## $superego init

Set up superego for this project. Run these steps:

**Step 1:** Install sg binary if not present:
```bash
command -v sg || cargo install superego
```

**Step 2:** Initialize .superego/ directory:
```bash
sg init
```

**Step 3:** Add this section to the project's AGENTS.md (create the file if needed):

```markdown
## Superego Metacognitive Oversight

This project uses superego. You have the `$superego` skill available.

**Use $superego:**
- Before committing to a plan or approach
- When choosing between alternatives
- Before non-trivial implementations
- When the task feels complex or uncertain
- Before claiming work is "done"

Superego catches strategic mistakes. Use it at **decision points**.

**Results:** `has_concerns: true` = STOP and show user; `skipped: true` = proceed normally.
```

**Step 4:** Confirm to user: "Superego initialized. I'll use $superego at decision points."

## $superego remove

Remove superego from this project. Run these steps:

**Step 1:** Remove .superego/ directory:
```bash
rm -rf .superego/
```

**Step 2:** Remove the "Superego Metacognitive Oversight" section from AGENTS.md (if present).

**Step 3:** Confirm to user: "Superego removed from this project."
