# Harness Guard rules — standalone data package

Machine-readable, source-cited audit rules for the privacy/retention/telemetry
posture of AI coding harness configuration. Codex CLI, Claude Code, and Grok
Build (local-posture rules) are currently represented. This package is consumed by
[Harness Guard](../README.md), a local, execution-free, per-finding-cited
config auditor for privacy/retention/telemetry posture — but this directory is
an **independently usable, forkable, permissively licensed data package from
day one**, not a folder convention.

- License: Apache-2.0 (see `LICENSE` in this directory).
- Contract: every per-rule file under `<tool>/` validates against
  `../schemas/rule.schema.json` (JSON Schema draft 2020-12). Consume rules
  only through that schema — the Harness Guard binary does exactly this and
  nothing more. `ruleset.json` is the package manifest, not a rule file.
- Layout: `ruleset.json` (CalVer `ruleset_version`) + one JSON file per rule
  under `<tool>/<rule>.json`.
- Guarantees encoded in the schema: non-`unknown` outcomes structurally
  require a source with `url` + `retrieved`; `tested_versions` ranges are
  explicit; `limitations` and `unknown_conditions` are required.

No verification cadence is claimed for this data. Check each rule's
`retrieved` dates.
