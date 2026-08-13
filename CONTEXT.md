# CONTEXT.md — Read this first in every session

**Project:** Harness Guard
**Purpose:** Local, execution-free, per-finding-cited config auditor.
**Context date:** 2026-08-13
**Current phase:** 0.0.1 multi-harness content complete; ruleset re-verified
2026-08-13. The owner-gated release-gate run and tag checklist (Task 25)
remain — no tag, GitHub Release, package publish, or workflow enablement
without exact owner authorization.

## Current implemented scope

Harness Guard is a free/open-source, read-only Rust CLI and core, built on a
declarative rule engine (rules are data over a closed set of typed match
primitives; totality is proven at load time). Runtime code supports **three
harnesses**:

- **Codex CLI** — 4 source-cited rules (history persistence, analytics,
  feedback, OpenTelemetry prompt logging); categories retention, telemetry,
  transfer; verified through **0.147.0**.
- **Claude Code** — 5 source-cited rules (session-history cleanup period,
  telemetry / error-reporting / feedback-command / feedback-survey opt-outs);
  categories retention, telemetry; verified through stable **2.1.223**.
- **Grok Build** — 4 local-posture rules (`features.telemetry`,
  `features.feedback`, `telemetry.trace_upload`,
  `telemetry.otel_log_user_prompts`); categories telemetry, transfer; tested
  on **1.0.3** (npm + `https://x.ai/cli/stable`) with evidence under
  `docs/research/evidence/grok-build/2026-08-13/` (prior: 2026-07-20,
  2026-07-17). Detection uses PATH binary `grok`, npm package
  `@xai-official/grok`, and a managed-install symlink basename version
  fallback; `GROK_HOME` is honored. Rules describe local config posture only
  — not wire-level upload behavior.

The `capabilities` subcommand and `docs/agent-guide.md` expose this inventory
machine-readably so it never needs to be hardcoded by a consumer. The bundled
ruleset CalVer is **2026.08.13**. Workspace / binary version is **0.0.2**.

GitHub Copilot CLI (watched at npm `1.0.79`), Gemini CLI, Cursor, OpenCode,
and the other tools discussed in early research and product-strategy
documents are not implemented or supported. Those documents describe possible
sequencing, not shipped coverage.
Adding a harness, rule, write/fix behavior, network feature, database, output
format, GUI, or new public claim requires explicit approval and fresh primary
evidence.

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
6. The design specs, implementation plans, and review findings under
   `docs/superpowers/`, and any handoff under `docs/superpowers/handoffs/`
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

The repository is public. Freshness workflows remain triage-only and disabled.
Do not publish packages, create a GitHub Release, or make other external
changes without the exact authorization required by `AGENTS.md`. Public GitHub
already has annotated tag `0.0.1` at the 2026-07-17 tip. This freshness
commit is tagged `0.0.2`. Do not move `0.0.1`. Do not create a GitHub
Release or enable freshness workflows without a separate owner go.

## 2026-08-13 candidate-rule decisions (evaluated, not shipped)

Official sources still document extra keys. None were added in this pass:

| Candidate | Why not shipped |
| --- | --- |
| Grok `/privacy` coding-data / training | Account/settings UI, not a user-scope `config.toml` key this product can observe. |
| Grok `telemetry.mixpanel_enabled` | Documented sub-switch, but the OSS type is a non-optional `bool` whose default is baked-token-dependent. Needs its own fixture matrix and a careful unset/default write-up. |
| Grok `telemetry.otel_log_tool_details` | Clean sibling of `otel_log_user_prompts`. Held so freshness of the existing 13 stays a single reviewable change. |
| Claude `DO_NOT_TRACK` | Already noted as an alternate survey-disable path in `claude-code-feedback-survey-opt-out-01`. Not a distinct observation. |
| Claude `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC` | Documented umbrella; still no explicit `=1` token. Existing limitations already refuse to evaluate it. |
| Claude `skipWebFetchPreflight` / `feedbackSurveyRate` | Security-blocklist / survey-frequency, outside current retention/telemetry/transfer categories. |

## Session continuity

- Follow `docs/maintenance/runbook.md` for evidence and rule changes.
- Preserve actual retrieval dates, exact official URLs, semantic hashes, archive
  URLs when available, and version evidence.
- Treat upstream behavior as volatile. When no verified range matches, degrade
  to `stale-ruleset`/`unknown`; never infer support.
- Keep changes within the currently authorized work package and record unresolved
  safety, evidence, or release risks in the handoff.
