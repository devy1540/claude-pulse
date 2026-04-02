---
description: Configure HUD display options (layout, template, bar, icons, colors, rules)
allowed-tools: Bash, Read, Edit, AskUserQuestion
---

## Overview

This command configures claude-pulse display options. Configuration is stored in:
- **macOS/Linux**: `${CLAUDE_CONFIG_DIR:-$HOME/.claude}/plugins/claude-pulse/config.json`
- **Windows**: `Join-Path $HOME ".claude" "plugins\claude-pulse\config.json"`

All changes apply immediately (no restart needed).

## Step 1: Detect Current Config

Read the config file if it exists:

```bash
CLAUDE_DIR="${CLAUDE_CONFIG_DIR:-$HOME/.claude}"
CONFIG_PATH="$CLAUDE_DIR/plugins/claude-pulse/config.json"
cat "$CONFIG_PATH" 2>/dev/null || echo "{}"
```

Determine current mode:
- If `"lines"` key exists → **Template mode**
- Otherwise → **Classic mode** (expanded/compact)

## Step 2: Choose Configuration Flow

Use AskUserQuestion:
- header: "Configuration"
- question: "What would you like to configure?"
- options:
  - "Quick preset" — Choose a preset configuration
  - "Template layout" — Define custom line format with placeholders
  - "Bar style" — Customize progress bar characters and width
  - "Icons" — Customize status icons
  - "Colors" — Customize color scheme
  - "Conditional rules" — Show/hide elements based on thresholds
  - "Display elements" — Toggle individual HUD elements (classic mode)
  - "Reset to defaults" — Remove config file

---

### Flow A: Quick Presets

Use AskUserQuestion:
- header: "Presets"
- question: "Choose a preset:"
- options:
  - "Minimal" — Model + context only
  - "Standard" — Model + context + usage (default)
  - "Overview" — 2-line compact with cost/7d + emoji + line bar
  - "Full" — Everything enabled
  - "Compact" — Single-line classic layout
  - "Developer" — Template with tools/agents/git stats
  - "Dashboard" — Template with all metrics and memory

Preset configs:

**Minimal**:
```json
{
  "lines": [
    "{model} {context_bar} {context_pct}"
  ]
}
```

**Standard** (default — just delete the config file):
Remove config file to use defaults.

**Overview**:
```json
{
  "lines": [
    "{model} │ {project} {git} │ {duration}",
    "{context} │ {usage} │ {seven_day} │ {cost}"
  ],
  "bar": { "filled": "━", "empty": "╌", "width": 10 },
  "icons": {
    "running": "🔄", "completed": "✅", "error": "❌",
    "todoActive": "▶️", "todoDone": "✅", "dirty": "📝",
    "ahead": "⬆️", "behind": "⬇️", "timer": "⏱️ ", "warning": "⚠️"
  }
}
```

**Full**:
```json
{
  "lines": [
    "{model} │ {project} {git} │ {version} │ {duration}",
    "{context} │ {usage}",
    "{seven_day}",
    "{env}",
    "{tools}",
    "{agents}",
    "{todos}"
  ]
}
```

**Compact**:
```json
{
  "lineLayout": "compact",
  "display": {
    "showTools": true,
    "showAgents": true,
    "showTodos": true,
    "showDuration": true,
    "showConfigCounts": true
  }
}
```

**Developer**:
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

**Dashboard**:
```json
{
  "lines": [
    "{model} │ {project} {git}",
    "{context} │ {usage} │ {seven_day}",
    "{memory}",
    "{env} │ {duration} │ {version}",
    "{tools}",
    "{agents}",
    "{todos}"
  ],
  "bar": { "width": 12 }
}
```

---

### Flow B: Template Layout

Explain to the user that template mode uses `"lines"` — an array of format strings with `{placeholder}` variables. Each line with all-empty placeholders is auto-skipped.

Use AskUserQuestion:
- header: "Template Layout"
- question: "How would you like to set up your template?"
- options:
  - "Start from a preset" — Pick a preset then customize
  - "Build from scratch" — Choose placeholders interactively
  - "Edit raw JSON" — Show current template for manual editing

**If "Build from scratch":**

Show available placeholders grouped by category, with descriptions:

**Identity**
- `{model}` — 모델명 배지 `[Opus 4.6 (1M context)]`
- `{model_name}` — 모델명만 (배지 없이)
- `{project}` — 프로젝트 경로
- `{git}` — git 브랜치 + 상태 `git:(main*)`
- `{session_name}` — 세션 이름/슬러그
- `{version}` — Claude Code 버전 `CC v2.1.6`

**Context**
- `{context}` — 라벨 + 바 + 퍼센트 `Context ━━━╌╌╌ 25%`
- `{context_bar}` — 바만 `━━━╌╌╌`
- `{context_pct}` — 퍼센트만 `25%`
- `{token_breakdown}` — 토큰 상세 `in: 50k, cache: 171k`
- `{sparkline}` — 컨텍스트 추이 그래프 `▁▂▃▄▅▆▇`

**Usage**
- `{usage}` — 5시간 사용량 라벨+바+% `Usage ━━╌╌ 26% (2h 34m)`
- `{usage_bar}` — 5시간 바만
- `{usage_pct}` — 5시간 %만
- `{seven_day}` — 7일 사용량 `7d ━╌╌ 10%`
- `{usage_reset}` — 리셋까지 남은 시간

**Activity**
- `{tools}` — 도구 사용 내역 `✅ Read ×3 | ✅ Edit ×2`
- `{agents}` — 에이전트 상태 (타입, 모델, 소요시간)
- `{todos}` — TODO 진행 상황 `▶️ Task name (2/5)`
- `{todo_bar}` — TODO 진행률 바 `[━━━╌╌╌] 3/5`

**Environment**
- `{env}` — 환경 요약 `1 CLAUDE.md | 3 rules | 2 MCPs`
- `{claude_md}` — CLAUDE.md 파일 수
- `{rules}` — 규칙 파일 수
- `{mcps}` — MCP 서버 수
- `{hooks}` — 훅 수

**System**
- `{memory}` — RAM 사용량 `RAM ━━━╌ 12.3GB / 16GB (77%)`
- `{memory_bar}` — RAM 바만
- `{memory_pct}` — RAM %만

**claude-pulse 전용**
- `{speed}` — 토큰 출력 속도 `~142 tok/s`
- `{cost}` — 세션 비용 추정 `~$0.29`
- `{predict}` — autocompact까지 남은 메시지 `~15 msgs left`
- `{extra}` — 외부 명령 라벨 (`--extra-cmd` 연동)

**기타**
- `{duration}` — 세션 경과 시간 `⏱️ 46m`
- `{custom}` — 사용자 정의 텍스트 (config의 customLine)

구분자(`│`, `|`, `-` 등)와 일반 텍스트를 자유롭게 배치할 수 있습니다.

Ask the user to define each line. Use AskUserQuestion for each line:
- question: "Line N (enter placeholders, or 'done' to finish):"
- freeform text input

Collect all lines into the `"lines"` array.

**If "Edit raw JSON":**

Show the current `"lines"` value and let the user edit it directly.

---

### Flow C: Bar Style

Use AskUserQuestion:
- header: "Bar Style"
- question: "Choose bar style:"
- options:
  - "Default" — `█░` (filled block + light shade)
  - "Line" — `━╌` (heavy line + dashed)
  - "Dot" — `●○` (filled circle + empty)
  - "Arrow" — `▶▷` (filled arrow + empty)
  - "Braille" — `⣿⣀` (braille patterns)
  - "ASCII" — `#-` (hash + dash, for simple terminals)
  - "Custom" — Enter your own characters

Also ask for bar width:
- question: "Bar width (4-30, default 10):"
- Default: 10

Write the `"bar"` config:
```json
{
  "bar": {
    "filled": "━",
    "empty": "╌",
    "width": 12
  }
}
```

---

### Flow D: Icons

Use AskUserQuestion:
- header: "Icon Set"
- question: "Choose icon set:"
- options:
  - "Keep current" — Don't change icons (skip this step)
  - "Default" — `◐ ✓ ✗ ▸ * ↑ ↓`
  - "Emoji" — `🔄 ✅ ❌ ▶️ 📝 ⬆️ ⬇️`
  - "ASCII" — `> v x > * ^ v` (simple terminals)

If "Keep current" is selected, skip writing `"icons"` to config entirely. Do not remove existing icon settings.

**If "Custom"**: Ask for each icon using AskUserQuestion (running, completed, error, todoActive, todoDone, dirty, ahead, behind, timer, warning).

Write the `"icons"` config:
```json
{
  "icons": {
    "running": "⟳",
    "completed": "✔",
    "error": "✖",
    "todoActive": "→",
    "todoDone": "✔",
    "dirty": "●",
    "ahead": "↑",
    "behind": "↓",
    "timer": "⏱ ",
    "warning": "⚠"
  }
}
```

---

### Flow E: Colors

Use AskUserQuestion:
- header: "Color Scheme"
- question: "Choose a color scheme:"
- options:
  - "Keep current" — Don't change colors (skip this step)
  - "Default" — Green/cyan/yellow theme (remove color overrides)
  - "Ocean" — Blue/cyan tones
  - "Sunset" — Orange/red warm tones

If "Keep current" is selected, skip writing `"colors"` to config entirely. Do not remove existing color settings.
If "Default" is selected, remove the `"colors"` key from config to restore built-in defaults.

**Monochrome**:
```json
{ "colors": { "context": "dim", "usage": "dim", "model": "dim", "project": "dim", "git": "dim", "gitBranch": "dim", "label": "dim" } }
```

**Ocean**:
```json
{ "colors": { "context": "#5DADE2", "usage": "#3498DB", "model": "#2E86C1", "project": "#85C1E9", "git": "#2980B9", "gitBranch": "#AED6F1", "warning": "#F39C12", "critical": "#E74C3C" } }
```

**Sunset**:
```json
{ "colors": { "context": "#E67E22", "usage": "#D35400", "model": "#F39C12", "project": "#F5B041", "git": "#DC7633", "gitBranch": "#FAD7A0", "warning": "#E74C3C", "critical": "#C0392B" } }
```

**Custom**: Ask for hex values for key colors using AskUserQuestion.

---

### Flow F: Conditional Rules

Explain: Rules control when specific placeholders appear, based on variable thresholds.

Available variables:
- `context_pct` (0-100) — Context window usage
- `usage_pct` (0-100) — 5-hour usage
- `seven_day_pct` (0-100) — 7-day usage
- `tools_count` — Number of tool entries
- `agents_count` — Number of agent entries
- `todos_count` — Number of todo items
- `memory_pct` (0-100) — RAM usage

Operators: `>`, `>=`, `<`, `<=`, `==`, `!=`

Use AskUserQuestion:
- header: "Rules"
- question: "Choose common rules or create custom:"
- multiSelect: true
- options:
  - "Show token breakdown at high context" → `{ "show": "token_breakdown", "when": "context_pct >= 85" }`
  - "Show 7-day usage when high" → `{ "show": "seven_day", "when": "seven_day_pct >= 70" }`
  - "Hide usage when zero" → `{ "show": "usage", "when": "usage_pct > 0" }`
  - "Show memory only when high" → `{ "show": "memory", "when": "memory_pct >= 80" }`
  - "Custom rule" — Define your own

**If "Custom rule"**: Ask for target placeholder, variable, operator, and value.

Write the `"rules"` array:
```json
{
  "rules": [
    { "show": "token_breakdown", "when": "context_pct >= 85" },
    { "show": "seven_day", "when": "seven_day_pct >= 70" }
  ]
}
```

---

### Flow G: Display Elements (Classic Mode)

This flow is for users not using template mode. It toggles individual elements in the classic expanded/compact layout.

Use AskUserQuestion with multiSelect (mark currently enabled):
- "Model name", "Context bar", "Usage bar", "Tools activity", "Agents tracking", "Todo progress", "Session duration", "Config counts", "Memory usage", "Claude Code version"

Write selected as `true`, unselected as `false` in `display.*` keys.

Also ask for layout:
- "Expanded" (multi-line) or "Compact" (single-line)

---

### Flow H: Reset

Delete the config file:

```bash
CLAUDE_DIR="${CLAUDE_CONFIG_DIR:-$HOME/.claude}"
rm -f "$CLAUDE_DIR/plugins/claude-pulse/config.json"
```

Tell user: "Reset to defaults. Changes apply immediately."

---

## Step 3: Write Config

Create the directory if needed:
```bash
CLAUDE_DIR="${CLAUDE_CONFIG_DIR:-$HOME/.claude}"
CONFIG_DIR="$CLAUDE_DIR/plugins/claude-pulse"
mkdir -p "$CONFIG_DIR"
```

**IMPORTANT**: Always merge new settings with existing config. Read the current file first, then update only the changed keys. Do not overwrite unrelated settings.

Tell the user: "✅ Configuration updated. Changes apply immediately (no restart needed)."
