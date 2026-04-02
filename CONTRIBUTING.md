# Contributing to claude-pulse

[한국어](docs/CONTRIBUTING.ko.md)

## Branch Strategy

- `main` — Stable branch. No direct push, PR only
- `feat/*`, `fix/*`, `docs/*` — Working branches

```
main ← PR ← feat/add-sparkline-color
```

## Commit Convention

We follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>: <description>

[optional body]
```

| Type | Description |
|------|-------------|
| `feat` | New feature (placeholder, config option, etc.) |
| `fix` | Bug fix |
| `refactor` | Code improvement without behavior change |
| `docs` | Documentation (README, CONTRIBUTING, commands/) |
| `test` | Add/update tests |
| `chore` | Build, CI, dependencies |

Examples:
```
feat: add {network} placeholder
fix: sparkline session reset not working
docs: add labels config example to README
```

## Pull Request

1. Develop on a working branch
2. Ensure `cargo test` passes
3. Create PR — title must follow commit convention format
4. Review then squash merge

### PR Checklist

- [ ] `cargo build --release` passes
- [ ] `cargo test` passes
- [ ] New placeholder → added to `commands/configure.md` and `README.md`
- [ ] New config option → added to `README.md`

## Versioning

We follow [Semantic Versioning](https://semver.org/):

```
MAJOR.MINOR.PATCH
```

| Change Type | Version | Example |
|-------------|---------|---------|
| Breaking config change | MAJOR | Rename placeholder, remove config key |
| New feature (backward compatible) | MINOR | New placeholder, new config option |
| Bug fix | PATCH | Color bug, parsing error |

## Release

Automated via [release-please](https://github.com/googleapis/release-please):

1. When `feat:` / `fix:` commits are merged into main
2. release-please automatically creates a **Release PR**
   - Auto-updates `Cargo.toml` version
   - Auto-generates `CHANGELOG.md`
3. When the Release PR is merged:
   - Creates version tag (`v0.2.0`)
   - Builds binaries for 5 platforms
   - Creates GitHub Release

**No need to manually bump versions or create tags.**

### Commit Type → Version Bump

| Commit Type | Version Change |
|-------------|---------------|
| `fix:` | PATCH (0.1.0 → 0.1.1) |
| `feat:` | MINOR (0.1.0 → 0.2.0) |
| `feat!:` or `BREAKING CHANGE:` | MAJOR (0.1.0 → 1.0.0) |
| `docs:`, `chore:`, `refactor:`, `test:` | No version change |

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
├── main.rs           # Entry point
├── types.rs          # Type definitions
├── stdin.rs          # Stdin JSON parsing
├── transcript.rs     # JSONL transcript parsing + caching
├── config.rs         # Config loading/merging
├── config_reader.rs  # CLAUDE.md/rules/MCP/hooks counting
├── speed.rs          # Token output speed tracking
├── cost.rs           # Session cost estimation
├── sparkline.rs      # Context trend visualization
├── extra_cmd.rs      # --extra-cmd external command execution
├── git.rs            # Git status
├── memory.rs         # System memory
├── terminal.rs       # Terminal width detection
├── version.rs        # Claude Code version
└── render/
    ├── mod.rs        # Render entry + line wrapping
    ├── template.rs   # Core template engine (resolve, rules)
    ├── colors.rs     # ANSI colors
    ├── tools.rs      # Tool activity line
    ├── agents.rs     # Agent status line
    └── todos.rs      # TODO progress line
```

### Adding a New Placeholder

1. Add match arm in `resolve()` in `src/render/template.rs`
2. If needed, add rule variable to `RuleVars` and `auto_var_for_target()`
3. Decide whether to include in `default_lines()`
4. Add description to placeholder list in `commands/configure.md`
5. Update `README.md`
6. Write tests
