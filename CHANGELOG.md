# Changelog

All notable changes to this project will be documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [0.0.1] - 2026-07-17

First release: the reviewed Codex CLI thin slice generalized to three
co-equal audited harnesses.

### Added
- Harnesses: Claude Code, Codex CLI, and Grok Build — user-scope config
  auditing with per-finding citations, execution-free version detection, and
  conservative degradation (`unknown` / `stale-ruleset`).
- Declarative rule engine: rules are pure data over a closed set of typed
  match primitives; totality (exhaustiveness, overlap freedom, status
  legality) is proven at rule load time.
- `capabilities` subcommand (`schemas/capabilities.schema.json` 1.0) and
  `docs/agent-guide.md` for agent consumers.
- Grok Build local-posture rules citing the 2026-07-17 OSS source-reading
  intake (config surface, defaults, and detection from official source and
  docs at the pinned release); clean-room protocol documents a lab path for
  future behavior claims (none ship in 0.0.1).
- JSON config parsing (Claude Code `settings.json`) at the same hostile-input
  rigor as TOML: bounded reads, depth limits, value-free diagnostics.

### Changed
- Rule and report schemas: 1.0 → 1.1 (`match` primitives, integer
  observations with `integer_bounds`, widened `tool`/`scopes` enums).
- Workspace version 0.1.0 → 0.0.1 (owner decision 2026-07-16; nothing was
  ever published, so the backwards move has no consumers).

### Notes
- No network requests are ever made by a scan; nothing discovered is
  executed. Freshness automation ships authored but disabled
  (authored-off; not scheduled by default).
