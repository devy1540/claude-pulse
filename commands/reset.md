---
description: Reset claude-pulse config to defaults
allowed-tools: Bash, AskUserQuestion
---

## Step 1: Confirm

Use AskUserQuestion:
- header: "Reset"
- question: "Remove all claude-pulse configuration and restore defaults?"
- options:
  - "Yes, reset everything" — Delete config file
  - "Cancel" — Do nothing

If "Cancel", stop here.

## Step 2: Delete Config

```bash
CLAUDE_DIR="${CLAUDE_CONFIG_DIR:-$HOME/.claude}"
rm -f "$CLAUDE_DIR/plugins/claude-pulse/config.json"
```

Tell the user: "✅ Reset complete. Default settings apply immediately."
