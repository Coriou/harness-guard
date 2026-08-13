# Changelog

All notable changes to this project will be documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Added
- Maintainer audit-and-release loop: `docs/maintenance/audit-and-release-loop.md`,
  `.grok/skills/audit-and-release/SKILL.md`, and maintainer helpers
  `scripts/freshness/probe-releases.sh`, `scripts/freshness/fetch-cited.sh`,
  `scripts/maintenance/run-gates.sh`. Hard-stops before push, annotated tag,
  or GitHub Release. Scan behavior, shipped rules, and freshness workflow
  enablement are unchanged.

## [0.0.2] - 2026-08-13

Ruleset re-verification on current vendor releases. Binary 0.0.1 remains
the first tagged commit; this tag is the first public tree that matches
Codex 0.147.0, Claude Code stable 2.1.223, and Grok Build 1.0.3.

### Changed
- Workspace version **0.0.1 → 0.0.2**.
- Ruleset CalVer **2026.07.20 → 2026.08.13**: re-verified all 13 rules against
  live official primary sources; widened `tested_versions` to Codex **0.147.0**,
  Claude Code stable **2.1.223**, Grok Build **1.0.3** (npm + cli/stable).
  Observation keys and outcomes unchanged; source hashes, Wayback archives,
  fixtures, and Grok evidence pack `2026-08-13` refreshed. Grok crossed a
  major (`0.2.106 → 1.0.3`); local-posture keys remain documented.

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
- Ruleset CalVer **2026.07.17 → 2026.07.20**: re-verified all 13 rules against
  live official primary sources; widened `tested_versions` to Codex **0.144.6**,
  Claude Code stable **2.1.205**, Grok Build **0.2.106** (npm + cli/stable).
  Observation keys and outcomes unchanged; source hashes, Wayback archives,
  fixtures, and Grok evidence pack `2026-07-20` refreshed.

- Rule and report schemas: 1.0 → 1.1 (`match` primitives, integer
  observations with `integer_bounds`, widened `tool`/`scopes` enums).
- Workspace version 0.1.0 → 0.0.1 (owner decision 2026-07-16; nothing was
  ever published, so the backwards move has no consumers).

### Notes
- No network requests are ever made by a scan; nothing discovered is
  executed. Freshness automation ships authored but disabled
  (authored-off; not scheduled by default).
