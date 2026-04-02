---
description: Uninstall claude-pulse binary, config, and statusline setting
allowed-tools: Bash, Read, Edit, AskUserQuestion
---

## Step 1: Confirm

Use AskUserQuestion:
- header: "Uninstall"
- question: "This will remove the claude-pulse binary, config, and statusline setting. Continue?"
- options:
  - "Yes, uninstall everything"
  - "Cancel"

If "Cancel", stop here.

## Step 2: Remove Binary

```bash
CLAUDE_DIR="${CLAUDE_CONFIG_DIR:-$HOME/.claude}"
rm -f "$CLAUDE_DIR/bin/claude-pulse"
```

## Step 3: Remove Config

```bash
CLAUDE_DIR="${CLAUDE_CONFIG_DIR:-$HOME/.claude}"
rm -rf "$CLAUDE_DIR/plugins/claude-pulse"
```

## Step 4: Remove statusLine from settings.json

Read `${CLAUDE_CONFIG_DIR:-$HOME/.claude}/settings.json`, remove the `"statusLine"` key, and write back. Preserve all other settings.

If the current statusLine command does not contain "claude-pulse", warn the user and skip this step — it belongs to a different plugin.

## Step 5: Done

Tell the user:

> ✅ Uninstalled. Restart Claude Code to fully remove the statusline.
>
> To also remove the plugin itself: `/plugin uninstall claude-pulse`
