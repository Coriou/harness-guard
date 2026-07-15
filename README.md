# Harness Guard

Harness Guard is a local, execution-free, per-finding-cited config auditor for privacy/retention/telemetry posture. It helps developers understand locally observable privacy, retention, synchronization, telemetry, permission, sandbox, and network settings without reading their code or executing the tools being audited.

## Status — 2026-07-15

The one-rule Codex CLI thin slice is implemented and is now at the human review gate. It remains a narrow, read-only validation slice—not a full desktop application.

- The implemented slice covers Codex CLI history persistence only.
- The cross-platform Rust core/CLI and source-cited rules data package are in place.
- Scans are offline, non-executing, and sanitized.
- Repository publishing and workflow enablement remain separate user decisions.
- Defer the App Store, automatic fixes, session/secret scanning, and team product.

Read [the product decision](docs/product/decision-and-strategy.md) and [implementation plan](docs/product/implementation-plan.md).

## Why the narrower scope

A local scan can reliably inspect some files and settings. It cannot prove which bytes a vendor received, whether data trained a model, or which opaque account toggle is enabled. Harness Guard will distinguish observed, documented, independently reproduced, inferred, and unknown facts instead of compressing them into a generic risk score.

The product's safety promise is equally important: no network during scans, no execution of harnesses/MCP/plugins, no source or transcript ingestion, and no raw secrets or paths in reports.

## Repository map

- `docs/product/decision-and-strategy.md` — build/no-build, scope, audience, distribution, and business decision
- `docs/product/implementation-plan.md` — phased, gated path to a production-ready product
- `docs/research/verification-audit-2026-07-13.md` — corrections and source-quality findings
- `docs/research/AI_CODING_TOOLS_PRIVACY_RESEARCH_REPORT.md` — original broad research artifact; not canonical product data
- `data/README.md` — warning and migration status for legacy structured data
- `notes/` — session and decision continuity

Read `CONTEXT.md` before doing project work.
