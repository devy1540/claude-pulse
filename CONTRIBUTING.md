# Contributing to claude-pulse

## Branch Strategy

- `main` — 안정 브랜치. 직접 push 금지, PR만 허용
- `feat/*`, `fix/*`, `docs/*` — 작업 브랜치

```
main ← PR ← feat/add-sparkline-color
```

## Commit Convention

[Conventional Commits](https://www.conventionalcommits.org/) 를 따릅니다:

```
<type>: <description>

[optional body]
```

| Type | Description |
|------|-------------|
| `feat` | 새 기능 (새 플레이스홀더, 설정 옵션 등) |
| `fix` | 버그 수정 |
| `refactor` | 기능 변경 없는 코드 개선 |
| `docs` | 문서 변경 (README, CONTRIBUTING, commands/) |
| `test` | 테스트 추가/수정 |
| `chore` | 빌드, CI, 의존성 등 |

예시:
```
feat: {network} 플레이스홀더 추가
fix: sparkline 세션 리셋 안 되는 문제
docs: README에 labels 설정 예시 추가
```

## Pull Request

1. 작업 브랜치에서 개발
2. `cargo test` 통과 확인
3. PR 생성 — 제목은 커밋 컨벤션 형식
4. 리뷰 후 squash merge

### PR Checklist

- [ ] `cargo build --release` 성공
- [ ] `cargo test` 전체 통과
- [ ] 새 플레이스홀더 추가 시 `commands/configure.md`에 설명 추가
- [ ] 새 설정 옵션 추가 시 `README.md` 업데이트

## Versioning

[Semantic Versioning](https://semver.org/) 을 따릅니다:

```
MAJOR.MINOR.PATCH
```

| 변경 유형 | 버전 | 예시 |
|-----------|------|------|
| 호환 안 되는 config 변경 | MAJOR | 플레이스홀더 이름 변경, config 키 삭제 |
| 새 기능 (하위 호환) | MINOR | 새 플레이스홀더, 새 설정 옵션 |
| 버그 수정 | PATCH | 색상 오류, 파싱 버그 |

## Release

릴리스는 **자동**입니다:

1. `Cargo.toml`의 `version` 을 올린다
2. `main`에 merge한다
3. CI가 자동으로:
   - 버전 태그 생성 (`v0.2.0`)
   - 5개 플랫폼 바이너리 빌드
   - GitHub Release 생성

**직접 태그를 만들 필요 없습니다.**

## Development Setup

```bash
# Clone
git clone https://github.com/devy1540/claude-pulse.git
cd claude-pulse

# Build
cargo build --release

# Test
cargo test

# Local install (for testing)
cp target/release/claude-pulse ~/.claude/bin/claude-pulse
```

## Architecture

```
src/
├── main.rs           # 진입점
├── types.rs          # 전체 타입 정의
├── stdin.rs          # stdin JSON 파싱
├── transcript.rs     # JSONL 트랜스크립트 파싱 + 캐싱
├── config.rs         # 설정 로드/병합
├── config_reader.rs  # CLAUDE.md/rules/MCP/hooks 카운팅
├── speed.rs          # 토큰 출력 속도 추적
├── cost.rs           # 세션 비용 추정
├── sparkline.rs      # 컨텍스트 추이 시각화
├── extra_cmd.rs      # --extra-cmd 외부 명령 실행
├── git.rs            # git 상태
├── memory.rs         # 시스템 메모리
├── terminal.rs       # 터미널 너비 감지
├── version.rs        # Claude Code 버전
└── render/
    ├── mod.rs        # 렌더 진입 + 줄 래핑
    ├── template.rs   # 핵심 템플릿 엔진 (resolve, rules)
    ├── colors.rs     # ANSI 색상
    ├── tools.rs      # 도구 활동 라인
    ├── agents.rs     # 에이전트 상태 라인
    └── todos.rs      # TODO 진행률 라인
```

### 새 플레이스홀더 추가하기

1. `src/render/template.rs`의 `resolve()` 함수에 매치 추가
2. 필요하면 `RuleVars`와 `auto_var_for_target()`에 규칙 변수 추가
3. `default_lines()`에 디폴트 라인 반영 여부 결정
4. `commands/configure.md`의 플레이스홀더 목록에 설명 추가
5. `README.md`에 추가
6. 테스트 작성
