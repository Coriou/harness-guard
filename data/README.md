# Data Status

**Important:** The files in this directory were created from an initial research synthesis. They are **legacy research artifacts, not verified application inputs**.

- `tools-comparison.json` flattens plan-, version-, evidence-, and setting-specific behavior into fields that are too broad for reliable findings.
- `audit-commands.yaml` contains shell-oriented discovery and mitigation ideas. Some Grok telemetry keys are not present in current official documentation. Do not execute these commands or expose them to users as fixes.
- `config-examples/` has not yet been validated against the versioned evidence and fixture system.

Read [`docs/research/verification-audit-2026-07-13.md`](../docs/research/verification-audit-2026-07-13.md) before using any of this material.

The production application should consume new, schema-validated, source-cited, version-bounded rules. Legacy data should be migrated rule by rule and removed only after review.
