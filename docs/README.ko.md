# claude-pulse

[English](../README.md)

Claude Code를 위한 초고속 Rust statusline HUD.

컨텍스트 사용량, 토큰 비용, 출력 속도, 도구 활동, 사용량 제한을 실시간으로 statusline에서 모니터링합니다.

![claude-pulse overview](../screenshots/overview.png)

## 왜 claude-pulse인가

- **5배 빠른 실행** (~35ms vs Node.js ~180ms)
- **15배 적은 메모리** (3.6MB vs 56MB)
- **런타임 의존성 없음** — 단일 바이너리, Node.js 불필요
- **50+ 플레이스홀더** 템플릿 엔진
- **고유 기능**: `{speed}`, `{cost}`, `{sparkline}`, `{predict}`, `{todo_bar}`, 조건부 규칙

## 설치

```bash
/install github:devy1540/claude-pulse
```

설치 후:

```
/cp:setup
```

## 명령어

| 명령어 | 설명 |
|--------|------|
| `/cp:setup` | 바이너리 다운로드 + statusline 설정 |
| `/cp:configure` | 레이아웃, 바, 아이콘, 색상, 규칙 커스터마이즈 |
| `/cp:reset` | 설정 초기화 |
| `/cp:uninstall` | 바이너리, 설정, statusline 제거 |

## 프리셋

| 프리셋 | 설명 |
|--------|------|
| **Minimal** | 모델 + 컨텍스트 바만 |
| **Standard** | 모델 + 컨텍스트 + 사용량 (기본값) |
| **Overview** | 2줄 컴팩트 + speed/cost/7d + 이모지 아이콘 |
| **Full** | 모든 요소 활성화 |
| **Developer** | 도구/에이전트/TODO + git 상태 |
| **Dashboard** | 메모리 + 환경 포함 전체 메트릭 |

## 플레이스홀더

### 식별 정보
| 플레이스홀더 | 출력 |
|-------------|------|
| `{model}` | `[Opus 4.6 (1M context)]` |
| `{project}` | 프로젝트 경로 |
| `{git}` | `git:(main*)` |
| `{version}` | `CC v2.1.6` |
| `{session_name}` | 세션 이름/슬러그 |

### 컨텍스트
| 플레이스홀더 | 출력 |
|-------------|------|
| `{context}` | `ctx ━━━╌╌╌╌ 25%` |
| `{context_bar}` | `━━━╌╌╌╌` |
| `{context_pct}` | `25%` |
| `{token_breakdown}` | `in: 50k, cache: 171k` |
| `{sparkline}` | `▁▂▃▄▅▆▇` |

### 사용량
| 플레이스홀더 | 출력 |
|-------------|------|
| `{usage}` | `5h ━━╌╌ 26% (2h 34m)` |
| `{seven_day}` | `7d ━━━╌╌ 45% (2d 11h)` |
| `{usage_bar}` / `{usage_pct}` | 바 또는 퍼센트만 |

### 활동
| 플레이스홀더 | 출력 |
|-------------|------|
| `{tools}` | `✅ Read ×3 \| ✅ Edit ×2` |
| `{agents}` | 에이전트 타입, 모델, 소요시간 |
| `{todos}` | `▶️ Task name (2/5)` |
| `{todo_bar}` | `[━━━╌╌╌] 3/5` |

### 시스템 & 메타
| 플레이스홀더 | 출력 |
|-------------|------|
| `{speed}` | `~142 tok/s` — 토큰 출력 속도 |
| `{cost}` | `~$0.29` — 세션 비용 추정 |
| `{predict}` | `~15 msgs left` — autocompact 예측 |
| `{memory}` | `mem ━━━╌ 12.3GB / 16GB (77%)` |
| `{env}` | `1 CLAUDE.md \| 3 rules \| 2 MCPs` |
| `{duration}` | `⏱️ 46m` |
| `{extra}` | 외부 셸 명령 라벨 (`--extra-cmd`) |

## 설정

설정 파일: `~/.claude/plugins/claude-pulse/config.json`

### 템플릿 라인

```json
{
  "lines": [
    "{model} │ {project} {git} │ {speed} │ {duration}",
    "{context} │ {usage} │ {seven_day} │ {cost}"
  ]
}
```

### 커스텀 라벨

```json
{
  "labels": {
    "context": "Context",
    "usage": "Usage",
    "sevenDay": "Weekly",
    "memory": "RAM"
  }
}
```

### 바 스타일

```json
{
  "bar": { "filled": "━", "empty": "╌", "width": 10 }
}
```

### 조건부 규칙

임계값 기반으로 플레이스홀더 표시/숨김:

```json
{
  "rules": [
    { "show": "token_breakdown", "when": "context_pct >= 85" },
    { "show": "seven_day", "when": "seven_day_pct >= 70" }
  ]
}
```

### 색상

Named (`green`, `cyan`, `red`), ANSI 256 (`178`), 또는 hex (`#5DADE2`):

```json
{
  "colors": {
    "context": "green",
    "usage": "brightBlue",
    "sevenDay": "magenta",
    "model": "#2E86C1"
  }
}
```

### 외부 명령

셸 명령으로 커스텀 라벨 삽입:

```bash
# statusLine 설정에서:
claude-pulse --extra-cmd "my-script.sh"
```

스크립트는 `{ "label": "텍스트" }` 형식으로 출력해야 합니다.

## 플랫폼

| 플랫폼 | 바이너리 |
|--------|---------|
| macOS ARM (Apple Silicon) | `claude-pulse-aarch64-apple-darwin` |
| macOS Intel | `claude-pulse-x86_64-apple-darwin` |
| Linux x86-64 | `claude-pulse-x86_64-unknown-linux-gnu` |
| Linux ARM64 | `claude-pulse-aarch64-unknown-linux-gnu` |
| Windows | `claude-pulse-x86_64-pc-windows-msvc.exe` |

## 라이선스

MIT
