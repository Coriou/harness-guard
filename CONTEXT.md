# CONTEXT.md — Read this first in every session

**Project:** Harness Guard
**Purpose:** Local, execution-free, per-finding-cited config auditor.
**Context date:** 2026-07-16
**Current phase:** Codex CLI thin slice implemented; release, security, evidence,
and public-readiness validation in progress.

## Current implemented scope

Harness Guard is a free/open-source, read-only Rust CLI and core. Runtime code
currently supports **Codex CLI only**, with a single source-cited rule for local
history persistence. The bundled ruleset is the authoritative record of the
tested Codex version range.

Claude Code, GitHub Copilot CLI, Grok, and the other tools discussed in early
research and product-strategy documents are not implemented or supported. Those
documents describe possible sequencing, not shipped coverage. Adding a harness,
rule, write/fix behavior, network feature, database, output format, GUI, or new
public claim requires explicit approval and fresh primary evidence.

The CLI supports macOS and Linux. Unsupported build targets fail at compile
time rather than falling back to an unhardened filesystem open or being
reported as another operating system. Windows is deferred until its full path
traversal can meet the same race-resistant, reparse-point-refusing invariant.

## Required reading order

1. `AGENTS.md`
2. `CONTRIBUTING.md`
3. `README.md`
4. `SECURITY.md`
5. `docs/maintenance/runbook.md`
6. The thin-slice design, plan, and review findings under `docs/superpowers/`
7. Relevant production code, schemas, bundled rules, freshness state, synthetic
   fixtures, workflows, and no-egress scripts

Historical product and research documents remain useful context, but they are
not proof of runtime support or current vendor behavior.

## Critical data-quality warning

The original reports, comparison JSON, audit-command YAML, and config examples
under legacy research areas are quarantined artifacts, not application inputs
or rule evidence. Never derive a rule from `data/` or repeat a historical claim
without freshly verifying its exact version, operating system, product,
plan/auth context, and official primary source.

## Product safety invariants

- Scans make no network requests and execute nothing discovered.
- Core receives an explicit `DiscoveryRoot`; it never resolves ambient homes or
  environment variables.
- Never test against a developer's real harness store. Use synthetic roots under
  `fixtures/` or temporary directories derived from them.
- Do not read source code, prompt/session transcripts, history contents, shell
  history, `.env` files, credentials, or secret values.
- Reads are bounded, regular-file-only, symlink/reparse-point refusing,
  depth-bounded, and resistant to path replacement.
- Reports contain only normalized, allowlisted observations. Redact usernames,
  home paths, raw config values, and source snippets.
- Keep local storage distinct from data transmission and vendor-side
  collection, training, telemetry, and retention.
- Report locally unknowable account/auth/remote state as `unknown`; never infer
  authentication method from local artifacts.
- Every non-unknown finding is version-bounded, source-cited, dated,
  fixture-tested, and explicit about limitations.
- Never position Harness Guard as an agent-security scanner. Do not claim a
  public verification cadence while freshness workflows remain default-off.

## Architecture and release state

- `harness-guard-core`: explicit discovery roots, bounded reads, parsing, and
  evaluation; no environment, process, or network APIs.
- `harness-guard-rules`: schema-mirroring types, validation, and bundled rule
  loading. The top-level `rules/` directory is an independently usable
  Apache-2.0 data package.
- `harness-guard-cli`: argument parsing, environment/home resolution, sanitized
  rendering, and exit-code semantics.

The repository is under Git version control. Release work must preserve private
visibility until local gates and private CI pass. Freshness workflows remain
triage-only and disabled. Do not publish packages, create a GitHub Release, or
make other external changes without the exact authorization required by
`AGENTS.md`.

## Session continuity

- Follow `docs/maintenance/runbook.md` for evidence and rule changes.
- Preserve actual retrieval dates, exact official URLs, semantic hashes, archive
  URLs when available, and version evidence.
- Treat upstream behavior as volatile. When no verified range matches, degrade
  to `stale-ruleset`/`unknown`; never infer support.
- Keep changes within the currently authorized work package and record unresolved
  safety, evidence, or release risks in the handoff.
