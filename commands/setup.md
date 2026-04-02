---
description: Install claude-pulse binary and configure statusline
allowed-tools: Bash, Read, Edit, AskUserQuestion
---

## Step 1: Detect Platform and Architecture

Use the environment context values (`Platform:` and `Shell:`).

| Platform | Architecture | Binary target |
|----------|-------------|---------------|
| `darwin` | `arm64` | `claude-pulse-aarch64-apple-darwin` |
| `darwin` | `x86_64` | `claude-pulse-x86_64-apple-darwin` |
| `linux` | `x86_64` | `claude-pulse-x86_64-unknown-linux-gnu` |
| `linux` | `aarch64` | `claude-pulse-aarch64-unknown-linux-gnu` |
| `win32` | `x86_64` | `claude-pulse-x86_64-pc-windows-msvc.exe` |

Detect architecture:

**macOS/Linux**:
```bash
uname -m
```
Map: `arm64` → `aarch64`, `x86_64` → `x86_64`.

**Windows (PowerShell)**:
```powershell
$env:PROCESSOR_ARCHITECTURE
```
Map: `AMD64` → `x86_64`.

## Step 2: Download Binary

Set these variables based on Step 1:

```bash
CLAUDE_DIR="${CLAUDE_CONFIG_DIR:-$HOME/.claude}"
BIN_DIR="$CLAUDE_DIR/bin"
mkdir -p "$BIN_DIR"
```

Download from GitHub Releases:

```bash
REPO="devy1540/claude-pulse"
TARGET="{detected_target}"
DOWNLOAD_URL="https://github.com/$REPO/releases/latest/download/$TARGET"

curl -fSL "$DOWNLOAD_URL" -o "$BIN_DIR/claude-pulse"
chmod +x "$BIN_DIR/claude-pulse"
```

**Windows (PowerShell)**:
```powershell
$claudeDir = if ($env:CLAUDE_CONFIG_DIR) { $env:CLAUDE_CONFIG_DIR } else { Join-Path $HOME ".claude" }
$binDir = Join-Path $claudeDir "bin"
New-Item -ItemType Directory -Path $binDir -Force | Out-Null

$repo = "devy1540/claude-pulse"
$target = "{detected_target}"
$url = "https://github.com/$repo/releases/latest/download/$target"

Invoke-WebRequest -Uri $url -OutFile (Join-Path $binDir "claude-pulse.exe")
```

If download fails (404), the binary hasn't been released yet. Ask user to check releases page or build from source with `cargo install --path .`.

## Step 3: Verify Binary

```bash
"$BIN_DIR/claude-pulse" --version 2>/dev/null || echo '{}' | "$BIN_DIR/claude-pulse"
```

Should output HUD lines or version info within 1 second. If it errors, do not proceed.

## Step 4: Apply Configuration

Read the settings file and merge in the statusLine config, preserving all existing settings:

- **macOS/Linux**: `${CLAUDE_CONFIG_DIR:-$HOME/.claude}/settings.json`
- **Windows**: `$env:CLAUDE_CONFIG_DIR` or `Join-Path $HOME ".claude"` + `settings.json`

If the file doesn't exist, create it. If it contains invalid JSON, report the error and do not overwrite.

The generated command is simply the absolute path to the binary:

**macOS/Linux**:
```json
{
  "statusLine": {
    "type": "command",
    "command": "{BIN_DIR}/claude-pulse"
  }
}
```

**Windows**:
```json
{
  "statusLine": {
    "type": "command",
    "command": "{BIN_DIR}\\claude-pulse.exe"
  }
}
```

After successfully writing the config, tell the user:

> ✅ Config written. **Please restart Claude Code now** — quit and run `claude` again in your terminal.
> Once restarted, the HUD will appear below your input field.

## Step 5: Choose Profile

Use AskUserQuestion:
- header: "Profile"
- question: "Choose a HUD profile to get started:"
- options:
  - "⚡ Quick Install (recommended)" — Standard 2-line HUD, no extra config needed
  - "🛠 Developer" — Tools, agents, todos tracking + smart token breakdown
  - "📊 Dashboard" — All metrics: usage, memory, environment, version
  - "🎯 Minimal" — Just model + context percentage, nothing else
  - "🎨 Customize" — Fine-tune everything with /claude-pulse:configure

### If "Quick Install":
Do not create a config file. Tell the user defaults are ready.

### If "Developer":
Write `plugins/claude-pulse/config.json`:
```json
{
  "lines": [
    "{model} │ {project} {git} │ {duration}",
    "{context} {token_breakdown} │ {usage}",
    "{tools}",
    "{agents}",
    "{todos}"
  ],
  "rules": [
    { "show": "token_breakdown", "when": "context_pct >= 85" }
  ]
}
```

### If "Dashboard":
Write `plugins/claude-pulse/config.json`:
```json
{
  "lines": [
    "{model} │ {project} {git} │ {version}",
    "{context} │ {usage} │ {seven_day}",
    "{memory}",
    "{env} │ {duration}",
    "{tools}",
    "{agents}",
    "{todos}"
  ],
  "bar": { "width": 12 },
  "rules": [
    { "show": "token_breakdown", "when": "context_pct >= 85" },
    { "show": "seven_day", "when": "seven_day_pct >= 70" },
    { "show": "memory", "when": "memory_pct >= 50" }
  ]
}
```

### If "Minimal":
Write `plugins/claude-pulse/config.json`:
```json
{
  "lines": [
    "{model} {context_bar} {context_pct}"
  ]
}
```

### If "Customize":
Tell the user: "Run `/claude-pulse:configure` after restart to set up template, bar style, icons, colors, and rules."

---

After writing the profile config (if any), create directories if needed:
```bash
CLAUDE_DIR="${CLAUDE_CONFIG_DIR:-$HOME/.claude}"
mkdir -p "$CLAUDE_DIR/plugins/claude-pulse"
```

## Step 6: Verify & Finish

Tell the user:

> ✅ Setup complete! **Please restart Claude Code now** — quit and run `claude` again.
> After restart, the HUD will appear below your input field.
>
> You can change settings anytime with `/claude-pulse:configure`, or reset with `/claude-pulse:reset`.

Use AskUserQuestion:
- question: "After restarting, is the HUD working?"
- options: "Yes, it's working" / "No, something's wrong" / "I'll check later"

**If "Yes"**: Done!

**If "I'll check later"**: Done!

**If "No"**: Debug:

1. Verify config: Read settings.json and check statusLine.command path
2. Test manually: `{BIN_DIR}/claude-pulse < /dev/null 2>&1`
3. Common issues:
   - **"Permission denied"**: `chmod +x {BIN_DIR}/claude-pulse`
   - **Binary not found**: Redownload from Step 2
   - **HUD not visible**: Must fully restart Claude Code (not just `/reload-plugins`)
