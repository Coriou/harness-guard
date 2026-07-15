# CONTEXT.md — Read this first in every session

**Project:** Harness Guard  
**Purpose:** Local-only, source-cited privacy and configuration posture auditor for AI coding tools.  
**Context date:** 2026-07-14  
**Current phase:** Product direction decided; pre-implementation research (competition, maintainability, UX) complete with binding constraints; implementation is gated by evidence-schema work and a narrow CLI validation slice.

## Current decision

Proceed with a **free/open-source, cross-platform CLI/core first**. Start read-only with Claude Code, OpenAI Codex CLI, and GitHub Copilot CLI. A GUI, automatic fixes, App Store distribution, broad tool coverage, and a paid product are deferred until user evidence justifies them.

The canonical product decision is [`docs/product/decision-and-strategy.md`](docs/product/decision-and-strategy.md). The production path is [`docs/product/implementation-plan.md`](docs/product/implementation-plan.md).

## Required reading order

1. `README.md`
2. `CONTEXT.md`
3. `docs/product/decision-and-strategy.md`
4. `docs/research/verification-audit-2026-07-13.md`
5. `docs/product/implementation-plan.md`
6. `docs/research/synthesis-2026-07-14.md` (binding constraints from competition/maintainability/UX research)
7. Only then, relevant legacy report/data or per-tool notes

## Critical data-quality warning

The original report, comparison JSON, audit-command YAML, and config examples are **research artifacts, not application ground truth**. The verification audit found material stale/unsupported claims, including Grok mitigation keys absent from current official documentation and oversimplified account/plan behavior across Cursor, Claude Code, Copilot CLI, and Codex CLI.

Do not execute `data/audit-commands.yaml`, build rules directly from `data/tools-comparison.json`, or repeat the old executive table without re-verifying the exact claim, version, OS, plan/auth context, and source.

## Product safety invariants

- Normal scans make no network requests.
- Never execute a detected harness, MCP server, skill, plugin, hook, or arbitrary command.
- Do not read source code, prompt transcripts, shell history, `.env` files, or secret values in the initial product.
- Store/report normalized findings only; redact usernames, home paths, tokens, and raw configuration.
- Separate inference transfer, retention, training, telemetry, feedback, sync/sharing, permissions, sandbox, and network access.
- Report locally unknowable account/remote state as `unknown` with an official verification link.
- Every rule is version-bounded, source-cited, dated, fixture-tested, and explicit about limitations.
- Prefer concrete findings and actions over a false-precision numeric risk score.
- Never position publicly as an "AI agent security scanner"; the claim is "local, execution-free, per-finding-cited privacy/configuration posture auditor."
- Do not publicly claim a verification cadence (e.g., "verified monthly") before the automated freshness pipeline (release-watch, doc-drift detection, fixture tripwires) is operational.

## Next authorized work package

Phase 0 plus one thin vertical slice:

1. Define source, rule, finding, fixture, and sanitized-report schemas.
2. Create the Rust library/CLI workspace.
3. Implement safe Codex config discovery/parsing against synthetic fixtures.
4. Evaluate one source-cited rule such as history persistence.
5. Render terminal and JSON output and prove the scan has no egress.
6. Review the slice before adding rules or tools.

This does not authorize a full desktop build.

## Session continuity

- Log completed work in `notes/session-history.md`.
- Record product/research choices in `notes/research-decisions.md`.
- Preserve retrieved dates and exact source links for research changes.
- Treat upstream tool behavior as volatile; degrade to `unknown` outside tested versions.
- This directory is not currently a Git repository. Establish version control before implementation/release work.

## Business direction

The core should be free and open source (Apache-2.0 is the current recommendation). Use the tool, evidence pages, and a transparent case study on `benjsmin.com` to demonstrate product/security engineering and offer a fixed-scope team AI-coding posture review. Build paid team policy/fleet capabilities only after repeated requests or paid pilots. Do not begin with a $0.99 App Store app.
