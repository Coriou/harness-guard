# Session History

This file tracks work done in successive sessions for continuity.

## 2026-07-13 (Initial)

- Project directory was empty.
- Created clean organization structure.
- Persisted full research report from previous deep research session.
- Created:
  - README.md
  - CONTEXT.md
  - `docs/research/AI_CODING_TOOLS_PRIVACY_RESEARCH_REPORT.md`
  - `docs/research/sources-and-references.md`
  - `docs/research/app-building-recommendations.md`
  - `data/tools-comparison.json`
  - `data/audit-commands.yaml`
  - Several config examples
- Goal achieved: Research is now cleanly written down and organized for future self-healing sessions.
- Next: Review, refine, fill any gaps, then move toward spec / implementation planning.

## 2026-07-13 (Verification, feasibility, and strategy)

- Re-read the full project corpus and audited decision-critical claims against current official documentation and identifiable independent evidence.
- Found material issues in the original “ground truth” model: unsupported current Grok mitigation keys, version-dependent Grok upload behavior, oversimplified Cursor privacy modes, consumer/commercial/ZDR conflation for Claude Code, omitted Copilot CLI session synchronization, and overly broad Codex data-policy wording.
- Added early-product coverage research for Gemini CLI and OpenCode.
- Reviewed adjacent/direct competitors including Snyk Agent Scan, Cisco IDE AI Security Scanner, Armor1, Skarn, and Code Insights. Chose a narrower non-executing, no-content, evidence-cited posture niche.
- Reviewed Apple sandbox/notarization constraints and rejected a $0.99 App Store-first strategy.
- Reviewed `~/Projects/benjsmin-v2` positioning. The project fits as public proof of secure product/AI-tooling work and as a funnel to a fixed-scope team posture review, while keeping the scanner free, accountless, and telemetry-free.
- Created:
  - `docs/product/decision-and-strategy.md`
  - `docs/product/implementation-plan.md`
  - `docs/research/verification-audit-2026-07-13.md`
  - `data/README.md`
- Updated README, CONTEXT, research index, legacy-document warnings, and decision continuity.
- Current next step: establish version control, complete Phase 0 schemas/evidence inventory, then build one Codex CLI read-only rule end to end. No full desktop implementation is authorized yet.

## 2026-07-14 (Competition, maintainability, and UX research)

- Ran three parallel research streams (Sonnet subagents) to answer the pre-implementation questions: is the niche occupied, can a solo maintainer keep this accurate, and what does a clean audit-CLI UX look like.
- Competitive result: niche is real but narrow — closest analog is `claudit-sec` (Claude-only, no citations/unknowns, stalled ~2 months); category positioning must avoid "AI agent security scanner"; top threat is privacy-feature creep from Snyk Agent Scan / Cisco; Voibe AI Privacy Tracker owns the policy-level content layer (our wedge: your actual local config).
- Maintainability result: monthly verification of 3 tools is solo-viable only with an automated freshness pipeline (npm dist-tag release-watch, doc-drift hashing + Wayback anchoring, fixture tripwires, human-only verdicts). Public cadence claims are blocked until that pipeline ships. Concrete rule-schema constraints identified (structurally required source_url/retrieved_date, version ranges, conservative fixture-tested unknown-degradation).
- UX result: v1 command surface (`scan`/`list`/`explain`/`version`), 3-way exit codes, first-class `unknown` status with honest rendering, SARIF deferred, Rust crate stack chosen, terminal/JSON mockups produced.
- Created:
  - `docs/research/synthesis-2026-07-14.md` (decision-grade synthesis)
  - `docs/research/competitive-landscape-2026-07-14.md`
  - `docs/research/maintainability-strategy-2026-07-14.md`
  - `docs/research/cli-ux-research-2026-07-14.md`
- Updated CONTEXT.md (context date, reading order, cadence-claim invariant), research INDEX, and research-decisions log.
- Conclusion: no blockers found; recommended proceeding to brainstorm → spec → plan for the v1 thin slice with the three reports as Decision Pack inputs.
- Ran the brainstorm-pipeline end-to-end (context-digester → brainstorm-worker → spec-to-plan workflow with adversarial review):
  - Spec: `docs/superpowers/specs/2026-07-14-harness-guard-v1-thin-slice-design.md`
  - Plan: `docs/superpowers/plans/2026-07-14-harness-guard-v1-thin-slice.md` (17 tasks, review verdict: approve, zero blocking findings)
  - Review nits for the executor: `docs/superpowers/plans/2026-07-14-harness-guard-v1-thin-slice-review-findings.md`
  - Orchestrator-resolved escalations (user-ratified at handoff): freshness workflows authored locally with repo publication deferred to an explicit user decision; Apache-2.0 for code and rules/, no CLA.
  - Execution mode chosen: subagent-driven development in a fresh session; prerequisite step 1 of the plan is `git init` (docs committed first).

## Future sessions

(Add dated entries here with summary of what was accomplished, decisions made, and open questions.)

## 2026-07-15 — v1 thin slice implemented

Implemented per `docs/superpowers/plans/2026-07-14-harness-guard-v1-thin-slice.md`
(spec: `docs/superpowers/specs/2026-07-14-harness-guard-v1-thin-slice-design.md`):
git repo initialized (docs-first commit); Phase 0 schemas; 3-crate workspace
with cargo-deny + core-scoped clippy no-egress gates; `codex-history-persist-01`
end-to-end with fresh evidence (retrieved 2026-07-15, hashed, archived); 13-case
synthetic fixture matrix with goldens; scan/list/explain/version CLI with
terminal + JSON views from shared structs; sandbox-exec no-egress proof run
locally (strace job authored in CI); freshness workflows + runbook authored,
NOT enabled. Slice is now at the human review gate — no second rule or tool
until review.

## 2026-07-16 — Codex 0.144.5 certification and public-release preparation

- Enumerated runtime and bundled-rule support: Codex CLI remains the only
  implemented harness. Claude Code, Copilot CLI, Grok, and other researched
  tools remain unsupported future scope.
- Human-certified `codex-history-persist-01` through Codex CLI 0.144.5 using
  freshly retrieved official documentation, npm/release/tag evidence, semantic
  hashes, and tagged-source reproduction. Ruleset `2026.07.16` now carries the
  certified evidence and coverage.
- Narrowed the user-file claim: explicit `none` and `save-all` describe only
  the inspected user layer; an unset value is `unknown` because system,
  profile, trusted-project, or CLI layers may determine the effective value.
- Added exact-latest synthetic coverage while preserving a future out-of-range
  fixture, tightened rule/report schema invariants, and kept freshness
  workflows authored but disabled.
- Hardened macOS/Linux reads with handle-relative, component-by-component
  no-follow traversal and race tests. Windows is deferred until it can meet the
  same path-refusal invariant.
- Completed formatting, Clippy, dependency policy, 136 workspace tests, Rust
  1.85 locked checks, actionlint, macOS no-egress proof, clean release install,
  installed hostile-input matrix, isolated terminal/JSON first runs, and
  Bash/Zsh completion checks.
- Added the Codex-only research dossier and preliminary technical article
  draft under `docs/blog/`. Independent agents reproduced all official hashes,
  validated every material claim, and exercised tutorial commands in isolated
  Bash and Zsh roots.
- Secret scans of reachable history and the current worktree found no leaks.
  The pre-public provenance cleanup is limited to the already-approved absolute
  checkout path and author email normalization.

## 2026-07-17 — Multi-harness architecture and engine shipped; public push

- Executed `docs/superpowers/plans/2026-07-16-harness-guard-0.0.1-multi-harness.md`
  via subagent-driven development: 20 of its 25 tasks complete and
  adversarially reviewed (~28 commits since `bc6610e`).
- Shipped a declarative rule engine (rules are data over a closed set of
  typed match primitives; totality — exhaustiveness, overlap-freedom, status
  legality — proven at load time), the `HarnessId`/descriptor abstraction,
  and JSON config parsing at TOML-equivalent hostile rigor.
- Generalized runtime coverage from Codex-only to three harnesses: Codex CLI
  (4 rules: history persistence, analytics, feedback, OTel prompt logging)
  and Claude Code (5 rules: cleanup period, telemetry/error-reporting/
  feedback-command/feedback-survey opt-outs), both with fresh-evidence
  fixture matrices. Grok Build is detection-only — recognized as a supported
  harness with zero bundled rules, pending its clean-room evidence run.
- Added the `capabilities` subcommand (schema 1.0) and `docs/agent-guide.md`
  as the agent-facing discovery surface.
- Remaining work (dependency order): Grok Build rules (release-gating, owner
  lab-run evidence), the deferred `capabilities` goldens, the multi-harness
  no-egress proof, the 0.0.1 version/CHANGELOG bump, the full documentation-
  corrections pass, a final whole-branch review, then the owner-gated release
  tag. Full status, per-task commit ranges, and the next starting prompt are
  in `docs/superpowers/handoffs/2026-07-17-0.0.1-multi-harness-handoff.md`.
- Did an interim documentation-truthfulness pass (README, CONTEXT.md,
  AGENTS.md) ahead of the full Task 24 sweep, then merged this branch to
  `main` and pushed to the now-public `origin` with owner authorization.

## 2026-07-17 (later) — Grok rules shipped; 0.0.1 content complete through Task 24

- **Grok Build local-posture rules (Task 19+):** four bundled rules for
  `features.telemetry`, `features.feedback`, `telemetry.trace_upload`, and
  `telemetry.otel_log_user_prompts`, tested on **0.2.102**, citing
  `docs/research/evidence/grok-build/2026-07-17/` (OSS primary + install
  channel). Local posture only — not wire-level behavior. Clean-room protocol
  has a source-reading path plus a lab path.
- **Detection:** PATH binary `grok`, npm `@xai-official/grok`, managed-install
  symlink basename version fallback; `GROK_HOME` honored. Synthetic fixture
  matrix under `fixtures/grok-build/`.
- **Capabilities goldens** pin Claude 5 / Codex 4 / Grok 4 rule counts
  (ruleset CalVer **2026.07.17**).
- **Version:** workspace **0.0.1** (not 0.1.0); `CHANGELOG.md` present.
  Multi-harness no-egress proof green on macOS.
- **Task 24 documentation corrections:** README/CONTEXT/`rules/README`/
  agent-guide reflect three co-equal harnesses with Grok rules (removed all
  "detection-only / zero bundled rules" public claims); strategy doc gets a
  dated third-tool supersession note; AGENTS.md already generalized real-store
  language for `~/.codex` / `~/.claude` / `~/.grok`. Freshness workflows remain
  default-off.
- **Unresolved / next:** Task 25 owner-gated release-gate sweep and tag —
  requires explicit owner authorization for `git tag 0.0.1` and any GitHub
  Release. No package publish. No `ENABLE_FRESHNESS_WORKFLOWS`. Wire-level
  Grok behavior claims remain out of scope until a targeted lab run lands.
