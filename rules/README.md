# Harness Guard rules — standalone data package

Machine-readable, source-cited audit rules for the privacy/retention/telemetry
posture of AI coding harness configuration. Codex CLI, Claude Code, and Grok
Build (local-posture rules) are currently represented. This package is consumed by
[Harness Guard](../README.md), a local, execution-free, per-finding-cited
config auditor for privacy/retention/telemetry posture — but this directory is
an **independently usable, forkable, permissively licensed data package from
day one**, not a folder convention.

- License: Apache-2.0 (see `LICENSE` in this directory).
- Contract: every per-rule file validates against
  `../schemas/rule.schema.json` (JSON Schema draft 2020-12), **schema version
  1.1**. Consume rules only through that schema — the Harness Guard binary does
  exactly this and nothing more. `ruleset.json` is the package manifest, not a
  rule file.
- Layout:
  - `ruleset.json` — package manifest with CalVer `ruleset_version`
  - `rules/<tool-id>/<short-name>.json` — one file per rule
  - Current tool ids (closed set for the 0.0.1 binary): `claude-code`,
    `codex`, `grok-build`
- Rule id convention: each rule's `id` field must be prefixed with its tool id
  (for example `codex-history-persist-01`, `claude-code-telemetry-opt-out-01`,
  `grok-build-telemetry-01`). The `tool` field inside the rule must match the
  parent directory name.
- Guarantees encoded in the schema: non-`unknown` outcomes structurally
  require a source with `url` + `retrieved`; `tested_versions` ranges are
  explicit; `limitations` and `unknown_conditions` are required. Schema 1.1
  adds declarative `match` primitives and optional integer observations with
  `integer_bounds`.

No verification cadence is claimed for this data. Check each rule's
`retrieved` dates.
