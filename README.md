# Harness Guard

Harness Guard is a planned local-only, source-cited posture auditor for AI coding tools. It will help developers understand locally observable privacy, retention, synchronization, telemetry, permission, sandbox, and network settings without reading their code or executing the tools being audited.

## Status — 2026-07-13

The project has completed a second-pass feasibility and verification review. The decision is to proceed conditionally with a narrow, read-only CLI validation slice—not a full desktop application yet.

- No implementation exists yet.
- Start with Claude Code, OpenAI Codex CLI, and GitHub Copilot CLI.
- Build a cross-platform Rust core/CLI before considering Tauri.
- Make scans offline, non-executing, and sanitized.
- Release the core free/open source if user validation succeeds.
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
