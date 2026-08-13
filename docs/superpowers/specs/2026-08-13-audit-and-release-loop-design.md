# Recurring Audit-and-Release Loop — Design Spec

**Date:** 2026-08-13
**Status:** Approved decision pack → spec. Inputs: `AGENTS.md`, `CONTEXT.md`, `CONTRIBUTING.md`, `docs/maintenance/runbook.md`, `docs/superpowers/handoffs/2026-07-17-0.0.1-release-checklist.md`, `docs/superpowers/specs/2026-07-16-harness-guard-0.0.1-multi-harness-design.md` (§9–§10), `.github/workflows/{release-watch,doc-drift,ci}.yml`, `CHANGELOG.md`, `SECURITY.md`, `scripts/freshness/{normalize,extract-urls}.sh`, `crates/harness-guard-rules/tests/tripwires.rs`, `docs/research/maintainability-strategy-2026-07-14.md`, owner decisions of 2026-08-13 (system shape, last-mile gates, bots-never-edit-rules, no public cadence, freshness workflows stay off).
**Positioning (binding, verbatim, test-pinned):** Harness Guard is a *local, execution-free, per-finding-cited config auditor*. "AI agent security scanner" appears nowhere in user-facing text.

This spec does not replace the 0.0.1 multi-harness design. It adds a **repeatable maintainer loop** so the 2026-08-13 freshness pass (ruleset `2026.08.13`, binary `0.0.2`, annotated tag `0.0.2`) can be run again by an agent that follows a runbook plus a repo-local skill, and that **hard-stops** before every last-mile publish action.

## 1. Goal

Turn the completed 2026-08-13 pass into a durable, agent-runnable process: thoroughly audit, maintain, fix, and improve the public repo on a regular basis, through a prepared last-mile packet — and then **stop** until the owner gives a fresh explicit go for each of push, annotated tag, and GitHub Release.

Owner decisions this spec implements (final, 2026-08-13):

1. **System shape is runbook + skill**, not checklists-only and not auto-opening PRs. A Grok/Claude-invocable skill runs the loop locally and hard-stops before push/tag/Release.
2. **Last-mile is owner-gated per action.** The system may document and prepare commands, but must not execute push, annotated tag, GitHub Release, package publish, or repo-settings changes. Tag and GitHub Release are separate checkpoints. Do not move `0.0.1`, `0.0.2`, or any existing tag.
3. **Bots never edit rules.** GitHub Actions may only open triage issues (or flag drift). Only a human, or an agent session the owner authorized to do that exact re-verify, certifies a rule after live primary-source re-verification. Bots never set verdicts, never edit `rules/`, never bump CalVer, never update `freshness/` as a certified fact.
4. **No public cadence claim.** Internal runbook reminders are fine. README, `docs/agent-guide.md`, and other user-facing text must not say the project is verified monthly/weekly/continuously.
5. **Freshness workflows stay default-off.** This system does not set `ENABLE_FRESHNESS_WORKFLOWS`.
6. **Default last-mile is git tag + optional GitHub Release** from a CHANGELOG excerpt. No crates.io or npm unless separately authorized.
7. **Two version axes.** Ruleset CalVer always bumps on a certified rule/evidence change. Binary semver is independent. The tag name is owner-chosen.
8. **Extend the existing maintenance home.** Canonical process lives under `docs/maintenance/`. Do not replace the triage flow in `runbook.md`. Do not add an xtask crate or a new product CLI.

## 2. Scope

**In scope (exactly, no more):**

1. Extend `docs/maintenance/runbook.md` with a short pointer section; add sibling `docs/maintenance/audit-and-release-loop.md` as the phased checklist (§6).
2. A repo-local skill at `.grok/skills/audit-and-release/SKILL.md` that runs the loop and hard-stops (§7).
3. Maintainer-only fetch/probe helpers under `scripts/freshness/` and a gates wrapper under `scripts/maintenance/` — never in `harness-guard-core` or the scan path (§8).
4. Constitution pointers in `AGENTS.md`, `CONTEXT.md`, and `CONTRIBUTING.md` so every coding agent finds the loop without a root `CLAUDE.md` (§9).
5. Docs-vs-shipped-state sweep, including `SECURITY.md` supported-version language, capabilities goldens, and version-pin lists (§6.4).
6. Safety / tripwire / CI / deny hygiene that protects the loop's invariants (§10).
7. CHANGELOG + version-bump rules and the STOP last-mile packet (§6.6–§6.8).

**Explicitly not in this loop** (deferred, not authorized here): a new harness or new shipped rule (including the held 2026-08-13 candidates in `CONTEXT.md`); `--fix` or any write operation in the product; any networking in `harness-guard-core` or scan; a new output format or GUI; Windows; crates.io / npm publish; enabling `ENABLE_FRESHNESS_WORKFLOWS`; any public verification-cadence claim; an xtask crate; a Grok `.rhai` workflow as the primary driver; auto-opening PRs or issues from the local loop; moving or retagging `0.0.1` / `0.0.2` / any existing tag; a root `CLAUDE.md`.

**Unchanged and binding:** the product is a local, execution-free, per-finding-cited config auditor; `rules/` is an Apache-2.0 data package consumed only through the schema contract; scans make zero network requests and execute nothing discovered; tests use synthetic fixtures only and never inspect `~/.codex`, `~/.claude`, `~/.grok`, or override vars; automation flags drift and only a certified re-verify writes rules; freshness workflows remain authored-off; required gates stay `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo deny check`, `cargo test --workspace`, `scripts/no-egress/run-macos.sh` on macOS, and `actionlint` if workflows changed.

## 3. Guardrails (copied, binding)

- Binding rules are `AGENTS.md` / `CONTEXT.md` / `CONTRIBUTING.md`.
- Positioning: local, execution-free, per-finding-cited config auditor. Not an agent-security scanner.
- New tool / rule / `--fix` / network / output format / public claim needs explicit approval.
- `rules/` is an Apache-2.0 data package via schema only.
- Never inspect real `~/.codex`, `~/.claude`, `~/.grok`.
- Scans make zero network requests; never execute discovered binaries.
- Automation flags drift; only a human certifies a rule (an agent may do the mechanical re-verify when the owner authorized that session).
- Freshness workflows stay default-off unless separately enabled.
- Do not publish / push / tag / release unless the user requests that exact action.
- No public verification-cadence claim.
- Required gates: `cargo fmt --check`, clippy `-D warnings`, `cargo deny`, `cargo test --workspace`, `scripts/no-egress/run-macos.sh`, `actionlint` if workflows changed.
- Redact usernames / homes / raw config.
- Never derive rules from `data/` or legacy research.
- Auth-dependent policy is unknown. No matching `tested_versions` ⇒ `stale-ruleset` / `unknown`.
- Tripwires: no weekly / daily / continuously-verified language in `docs/agent-guide.md`; no canary-upload claims.
- Grok stays local-posture / `official-documentation` unless a lab run exists.

## 4. Approaches considered

Three shapes were on the table. The owner picked (b). Recorded so implementers do not re-open it.

| | (a) Checklists only | (b) Runbook + local skill (chosen) | (c) Scheduled Actions that execute the loop |
|---|---|---|---|
| Who runs it | Human walking docs | Owner-invoked agent, locally | GitHub cron / `workflow_dispatch` |
| Rule edits | Human only | Authorized session may certify after live re-verify | Forbidden (bots-never-edit-rules) |
| Last-mile | Human reads a list | Skill prepares commands, then STOP | Easy to accidentally tag/release |
| Cadence claim risk | Low | Low if skill stays internal | High if a public workflow "runs monthly" |
| Fit with authored-off freshness | Neutral | Complements it: local probes replace the off jobs | Conflicts: would want `ENABLE_FRESHNESS_WORKFLOWS` |

**(a)** is not enough: the 2026-08-13 pass was a multi-hour, multi-file mechanical loop (fetch, hash, Wayback, goldens, CalVer, binary pins, gates). A checklist without an agent prompt will drift from what agents actually do.

**(c)** is banned here. Freshness workflows stay off. Actions that edit `rules/` or push tags would violate bots-never-edit-rules and no-auto-tag-push-release. Auto-opening PRs was also rejected.

**(b)** matches how the owner already works: invoke an agent in this repo, let it retrieve vendor docs *outside* the product, apply certified updates on a clean branch, run gates, and wait. A Grok `.rhai` workflow is **not** the primary driver. The loop is sequential, judgment-heavy, and must halt on last-mile; a multi-agent orchestrator adds fan-out without making the STOP safer. A later read-only fetch fan-out may be proposed separately. It is not this spec.

## 5. Layout deltas

```text
harness-guard/
├── .grok/skills/audit-and-release/SKILL.md   # NEW: agent prompt (Grok auto-discovers)
├── docs/maintenance/
│   ├── runbook.md                            # EXTEND: pointer + last-mile reminder
│   └── audit-and-release-loop.md             # NEW: canonical phased checklist
├── scripts/
│   ├── freshness/
│   │   ├── extract-urls.sh                   # existing
│   │   ├── normalize.sh                      # existing
│   │   ├── fetch-cited.sh                    # NEW: maintainer fetch + hash report
│   │   └── probe-releases.sh                 # NEW: npm + Grok channel vs last-seen
│   └── maintenance/
│       └── run-gates.sh                      # NEW: wrap required validation
├── AGENTS.md / CONTEXT.md / CONTRIBUTING.md  # EXTEND: invoke the skill; STOP last-mile
├── crates/harness-guard-rules/tests/tripwires.rs  # EXTEND: loop invariants
└── docs/superpowers/handoffs/2026-07-17-0.0.1-release-checklist.md
                                              # HISTORICAL: do not make this the live last-mile
```

No new crate. No new product subcommand. No change to `harness-guard-core` scan I/O. `.grok/skills/` is committed (not gitignored) so any clone has the skill.

## 6. The loop (canonical process)

Canonical human-and-agent process: `docs/maintenance/audit-and-release-loop.md`. The skill is the agent-facing encoding of the same phases. One home per fact: the checklist owns the phase list, file lists, and STOP packet; the skill owns invocation, authorization, forbidden commands, and the report format.

Starting point of every run: current public `main` at the latest existing tag (`0.0.2` as of 2026-08-13) plus any already-merged work. Work on a focused branch `audit/YYYY-MM-DD` (UTC date of the session). Never amend a published tag. Never inspect ambient harness stores.

### 6.0 Session authorization (what invoking the skill allows)

Invoking `/audit-and-release`, or an explicit "run the audit-and-release loop" request, authorizes **this session** to:

- retrieve vendor docs via maintainer scripts (network lives here, not in scan);
- certify existing shipped rules after live primary-source re-verification;
- update `rules/`, synthetic fixtures / goldens, `freshness/`, Grok evidence packs, docs, CHANGELOG, and version pins in the working tree;
- run full gates;
- write the STOP packet.

It does **not** authorize: `git push`; `git tag`; GitHub Release; crates.io / npm; setting `ENABLE_FRESHNESS_WORKFLOWS`; repo-settings changes; moving an existing tag; shipping a new rule or harness; `--fix`; network-in-product; a public cadence sentence.

A later "just ship it" / "finish the release" is **not** a go. Each last-mile action needs a fresh explicit owner sentence that names that action (examples: "push `audit/2026-09-10` to origin", "create annotated tag `0.0.3` at `<sha>`", "create the GitHub Release for `0.0.3` from the CHANGELOG excerpt"). Tag go and Release go are never the same sentence.

### 6.1 Phase A — Inventory (read-only)

Record, without editing:

- workspace version from `Cargo.toml` (`0.0.2` today);
- `rules/ruleset.json` `ruleset_version` (`2026.08.13` today);
- existing tags (`git tag -l`); refuse any plan that moves one;
- shipped rule inventory via `capabilities --json` after a local build, not by hardcoding counts;
- `freshness/last-seen.json` package and `channels.grok-build` pins;
- open questions already parked in `CONTEXT.md` (held 2026-08-13 candidates stay held);
- whether `SECURITY.md` still claims versioned releases do not exist (it did on 2026-08-13 — a docs-sweep item, not a product defect).

### 6.2 Phase B — Release probe (maintainer network, no writes)

Run `scripts/freshness/probe-releases.sh`. Compare live npm dist-tags and the Grok CLI pointer against `freshness/last-seen.json`:

| Probe | Source | Compare to |
|---|---|---|
| `@anthropic-ai/claude-code` `stable` | npm abbreviated registry | `packages["@anthropic-ai/claude-code"].version` |
| `@openai/codex` `latest` | npm abbreviated registry | `packages["@openai/codex"].version` |
| `@github/copilot` `latest` | npm abbreviated registry | `packages["@github/copilot"].version` (watched only; not a shipped harness) |
| `@xai-official/grok` `latest` | npm abbreviated registry | `packages["@xai-official/grok"].version` |
| Grok channel pointer | `https://x.ai/cli/stable` | `channels.grok-build.version` |

Print a drift table. Do not write `last-seen.json` here. Copilot movement is recorded as watch-only; it does not widen shipped coverage and does not justify a new rule.

When Grok versions disagree across npm, the CLI pointer, or OSS `SOURCE_REV`, follow the existing runbook: prefer the CLI pointer as the install channel, re-check the monorepo `SOURCE_REV`, and do not widen `tested_versions` until those agree or the disagreement is written as a limitation / `unknown`.

### 6.3 Phase C — Cited-source re-verify (maintainer network, then certified writes)

Run `scripts/freshness/fetch-cited.sh` (URLs from `scripts/freshness/extract-urls.sh`, hashes via `scripts/freshness/normalize.sh`). The script prints old vs new `sha256:…` against `freshness/url-hashes.json` and optionally requests a Wayback SPN2 snapshot (`--save-wayback`). It never edits `rules/` or `freshness/`.

A **full pass** (the default) re-reads every cited page even when hashes match, refreshes `retrieved` / `verified_on` / `archived_url` to the session date, and therefore bumps CalVer. That is how 2026-08-13 worked. A **probe-only** or **no-drift no-op** run (owner asked to check, not re-certify) stops after the reports and writes nothing. Do not half-refresh dates without a CalVer bump.

Then, on a full pass, the authorized session applies certified updates, rule by rule, using the existing runbook steps:

1. Re-read the live official page (and Grok OSS blobs at the pinned `SOURCE_REV` when the citation is a GitHub blob). Never use `data/` or legacy research as evidence.
2. If the claim still holds: refresh `retrieved`, `content_hash`, `archived_url`, `tested_versions` (`verified_on` = UTC date of this session; widen `max` only when the live version was actually checked). Keep observation keys and outcomes unchanged unless the official text changed the documented key or default.
3. If the claim no longer holds: change the rule data (outcomes, limitations, `unknown_conditions`) conservatively. Auth-dependent policy stays unknown. No matching range ⇒ `stale-ruleset` / `unknown`, never an inferred pass. Grok stays local-posture / `official-documentation` unless a new lab pack exists under `docs/research/evidence/grok-build/<date>/`.
4. If a new key appears (the 2026-08-13 held list, or anything like it): **do not ship it**. Record it in `CONTEXT.md` under a dated "evaluated, not shipped" table. New rules need a separate owner approval.
5. Update `freshness/url-hashes.json` and, when versions moved, `freshness/last-seen.json` (`checked` = session date).
6. If any rule JSON, certified freshness fact, or fixture fact derived from those changed: bump `rules/ruleset.json` `ruleset_version` to today's CalVer (`YYYY.MM.DD`). Same-day second pass uses `YYYY.MM.DD` still if it is the same certified date; a later calendar day gets a new CalVer. Never leave a certified rule change on the previous CalVer.
7. Grok channel or `SOURCE_REV` movement: add `docs/research/evidence/grok-build/<date>/` following the 2026-08-13 pack shape (`README.md` + `raw/` artifacts + semantic hashes). Do not delete prior dated packs.

Fixture goldens are the second staleness signal. After rule edits, update every `fixtures/**/expected.json` that embeds `ruleset_version`, `rules_verified_date`, or `rules_last_verified_version`, plus `crates/harness-guard-cli/tests/goldens/capabilities.expected.json`. A golden that still names the previous CalVer after a bump is a failed pass.

### 6.4 Phase D — Docs-vs-shipped-state

Keep these in lockstep with the tree that would be tagged. Do not claim coverage the binary does not have.

| Surface | Must match |
|---|---|
| `CONTEXT.md` current phase, rule counts, verified-through versions, CalVer, binary version, tag state | shipped `capabilities` + `ruleset.json` + `Cargo.toml` + `git tag` |
| `README.md` scope blurb and example versions | same |
| `docs/agent-guide.md` sample `capabilities` JSON | same; still no cadence words |
| `CHANGELOG.md` | Keep a Changelog; dated section only when a binary bump is proposed |
| `SECURITY.md` "Supported version" | current tagged preview (today: `0.0.2` / latest tag on `main`), not "until versioned releases exist" |
| `crates/harness-guard-cli/tests/cli_surface.rs` | `harness-guard <binary>` string |
| `crates/harness-guard-cli/src/render_term.rs` unit-test literals | binary version |
| `notes/session-history.md` | append-only summary of this pass |

Cadence language remains forbidden in user-facing files. Internal runbook may say "run this loop when you next sit down to maintain the repo" without promising a public SLA.

### 6.5 Phase E — Safety / tripwire / CI / deny hygiene

Read-only unless a real defect is in scope for this pass:

- `deny.toml` still bans network crates from the workspace graph (including dev-deps).
- `crates/harness-guard-core/clippy.toml` disallowed methods still cover process / env / net.
- `tripwires.rs`: canary-upload claims still absent; agent-guide still carries the positioning phrase and still lacks weekly / daily / continuously-verified language.
- `.github/workflows/{release-watch,doc-drift}.yml` still job-gated on `vars.ENABLE_FRESHNESS_WORKFLOWS == 'true'`. Do not flip the variable. Do not add a third scheduled workflow that edits rules or tags.
- New maintainer scripts are not referenced from core, CLI scan, or CI product jobs.
- Hostile / redaction / real-home-refusal tests still exist; do not weaken a golden to make a pass.

In-scope fixes: broken tripwires, stale comments that re-introduce canary or cadence claims, a workflow edit that accidentally drops the enablement gate, deny-policy drift, docs that contradict shipped capabilities. Out of scope: new harness support, new product flags, rewriting CI.

### 6.6 Phase F — Version bumps

Two axes, never collapsed:

| Event | Ruleset CalVer | Workspace / binary semver | Tag |
|---|---|---|---|
| Certified rule, source, `tested_versions`, or `freshness/` fact change | **Always** today's CalVer | Unchanged unless a tag is planned or a version-pin file must move | Owner-chosen; skill proposes next patch |
| Code, docs, tests, scripts, hygiene only | No | Only if `CARGO_PKG_VERSION` pins would lie | Usually none |
| Owner wants a public snapshot of this tree | CalVer already bumped if rules changed | Propose next **patch** (`0.0.2` → `0.0.3`) unless the owner already named a version | Owner-chosen; never reuse or move a tag |

Binary minor/major is not invented by the skill. A product-surface change (new command, schema bump, new harness) needs its own approved spec; this loop does not create one.

When the binary version moves, update every pin in one commit-set: `Cargo.toml` workspace version, `cli_surface.rs`, `render_term.rs` test literals, `capabilities.expected.json` `harness_guard_version`, README / CONTEXT / agent-guide samples.

CHANGELOG: put certified ruleset work and binary bumps under a dated `## [X.Y.Z] - YYYY-MM-DD` when a tag is being prepared; otherwise leave notes under `## [Unreleased]`. Use the 0.0.2 entry as the tone model: what was re-verified, which versions widened, what did *not* change (observation keys, outcomes), and that freshness automation remains authored-off.

### 6.7 Phase G — Gates at the exact commit

`scripts/maintenance/run-gates.sh` runs, in order:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo deny check
cargo test --workspace
```

On macOS it then runs `scripts/no-egress/run-macos.sh`. On Linux it skips that script (CI already runs the Linux strace job); it does not invent a weaker local substitute. If any `.github/workflows/*.yml` file is in the diff, it runs `actionlint` and fails closed if `actionlint` is not on `PATH`. The wrapper must not talk to the network beyond what those tools already do, must not tag, and must not push. CI remains the second opinion; the loop does not skip local gates because CI is green.

Re-run gates after any late fix so the STOP packet names a commit that actually passed.

### 6.8 Phase H — STOP (last-mile packet)

This is the terminal state of an unauthorized last-mile. The skill prints a packet and **halts**.

Packet contents (all concrete, no placeholders):

- Branch name and `git rev-parse HEAD`.
- Existing tags that must not move.
- Proposed tag name (suggestion only) and why (ruleset-only vs binary bump).
- Whether a GitHub Release is recommended (yes when a new tag is proposed; still a separate go).
- CHANGELOG excerpt for that version (the section body, not the whole file).
- Files changed and a one-paragraph risk list (unverified URLs, held candidates, Grok-without-lab, docs that still disagree).
- Exact commands, **not executed**:

```bash
git push -u origin audit/YYYY-MM-DD          # only after: owner go to push this branch
# after merge to main, only after: owner go to push main
git tag -a X.Y.Z <sha> -m "Harness Guard X.Y.Z"   # only after: owner go to tag X.Y.Z at <sha>
git push origin X.Y.Z                         # only after: owner go to push that tag
gh release create X.Y.Z --title "X.Y.Z" --notes-file <excerpt>  # only after: separate owner go for GitHub Release
```

- Explicit non-actions: no `cargo publish`, no npm, no `ENABLE_FRESHNESS_WORKFLOWS`, no repo-settings, no moving `0.0.1` / `0.0.2`.

If the owner later gives a go for **one** of those lines, the agent may run **that line only**, then STOP again. A tag go is not a Release go. A Release go is not a crates.io go.

The historical file `docs/superpowers/handoffs/2026-07-17-0.0.1-release-checklist.md` stays as the 0.0.1/0.0.2 execution record. It is not the live last-mile doc. New passes do not keep appending owner-go boxes there; they write a short dated handoff under `docs/superpowers/handoffs/YYYY-MM-DD-audit-and-release.md` (or append `notes/session-history.md` if the pass produced no tree change).

## 7. Skill

**Path:** `.grok/skills/audit-and-release/SKILL.md`

Grok project-skill layout per `create-skill`: directory + `SKILL.md` with YAML frontmatter `name` + `description`. No user-scoped copy under `~/.grok/skills/`. No second body under `.claude/skills/` (dual source would drift; constitution is `AGENTS.md`, not a root `CLAUDE.md`).

**Frontmatter `name`:** `audit-and-release`

**Frontmatter `description` (invocation surface):** state that the skill runs the maintainer audit-and-release loop (re-verify cited evidence, update fixtures/freshness/docs, run gates, prepare CHANGELOG and version bumps) and **hard-stops before push, annotated tag, or GitHub Release**. Include trigger phrases: audit, re-verify, freshness pass, ruleset bump, prepare release, tag, GitHub Release. Include `/audit-and-release`. State that it never pushes, tags, or publishes without a fresh explicit owner go for that exact action.

**Body is an agent prompt, not a second runbook.** It must:

1. Instruct the agent to read `AGENTS.md`, `CONTRIBUTING.md`, `CONTEXT.md`, then `docs/maintenance/audit-and-release-loop.md`, and to obey those over improvisation.
2. Restate session authorization (§6.0) and the default-forbidden command prefixes: `git push`, `git tag`, `gh release`, `gh repo`, `cargo publish`, `npm publish`, and any write to GitHub repository variables/settings. After a fresh owner sentence that names **one** of those exact actions, that one command is allowed once; then the prefix list applies again. "Ship it" does not lift the list.
3. Require a clean branch `audit/YYYY-MM-DD` from current `main`.
4. Walk phases A–H in order. Allow skipping a phase only when its precondition is already satisfied and the report says so (example: probe-releases shows no version movement **and** fetch-cited shows no hash drift → still refresh `retrieved` / `verified_on` only if the owner asked for a full re-verify; a "probe-only" run may stop after B with a no-op report).
5. Use `scripts/freshness/probe-releases.sh` and `scripts/freshness/fetch-cited.sh` rather than ad-hoc curl that hashes differently from `normalize.sh`.
6. Use synthetic fixtures only. Never read ambient `HOME` harness stores.
7. Treat held 2026-08-13 candidates as out of scope.
8. End with the CONTRIBUTING report shape: changed files, test evidence (which gates ran, exit status), unresolved risk, and the STOP packet. Do not claim a public cadence in that report either.

Default mode is a **full pass** (A–H). The skill may accept an explicit narrower task ("probe only", "docs sweep only") and must then refuse to bump CalVer or prepare a tag unless the owner expands the task.

## 8. Maintainer scripts

All three are POSIX `sh`, `set -eu`, repo-root aware (same `cd "$(dirname "$0")/../.."` pattern as `extract-urls.sh`). They are not product surface. CI product jobs do not run the two network scripts.

### 8.1 `scripts/freshness/probe-releases.sh`

- Read `freshness/last-seen.json`; fail if the schema shape is wrong (object with `packages` and `channels.grok-build`).
- For each package/tag pair already encoded in `release-watch.yml` (`claude-code`/`stable`, `codex`/`latest`, `copilot`/`latest`, `@xai-official/grok`/`latest`), GET the npm abbreviated document (`Accept: application/vnd.npm.install-v1+json`) and print `seen → live`.
- GET `https://x.ai/cli/stable` and compare to `channels.grok-build.version`.
- Exit `0` if every probe matches; exit `1` if any drifted; exit `2` on fetch/parse failure. Print a stable table to stdout. Write nothing under `freshness/` or `rules/`.

### 8.2 `scripts/freshness/fetch-cited.sh`

- Build the URL list from `scripts/freshness/extract-urls.sh`.
- For each URL: fetch to a temp file (`curl --fail --location`), hash with `scripts/freshness/normalize.sh`, compare to `freshness/url-hashes.json`.
- Print `url old new status` (`match` / `drift` / `missing-from-freshness`).
- `--save-wayback` (optional): request `https://web.archive.org/save/$url` and print the snapshot URL or `unavailable`. Same best-effort behavior as `doc-drift.yml`.
- Exit `0` if every cited URL matches a known hash; `1` on any drift or missing entry; `2` on fetch/hash failure. Write nothing under `freshness/` or `rules/`.
- Document the same JS-rendered-page limitation already in `doc-drift.yml` and `normalize.sh`. No Playwright fallback in this spec.

### 8.3 `scripts/maintenance/run-gates.sh`

Wraps the required validation set in §6.7. Detects macOS for the no-egress script. Detects workflow diffs via `git diff --name-only` (uncommitted) plus `git diff --name-only origin/main...HEAD` when that ref exists, and runs `actionlint` if any `*.yml` under `.github/workflows/` appears. Does not call `fetch-cited.sh` or `probe-releases.sh`. Does not look at `~/.codex` / `~/.claude` / `~/.grok`.

## 9. Constitution pointers

No root `CLAUDE.md`. Three files gain short, non-duplicative pointers:

- **`AGENTS.md` — External actions / new "Audit and release" paragraph:** when asked to audit, re-verify, freshness-pass, or prepare a release, follow `.grok/skills/audit-and-release/SKILL.md` and `docs/maintenance/audit-and-release-loop.md`. Last-mile still requires a fresh explicit owner go per action. Do not turn `AGENTS.md` into a second checklist.
- **`CONTEXT.md` — Current phase + session continuity:** once this system ships, the current phase names the loop as the way to repeat the 2026-08-13 pass, and points at the sibling checklist. Keep tag-state sentences true (`0.0.1` and `0.0.2` exist; do not move them).
- **`CONTRIBUTING.md` — Working with a coding agent:** extend the boxed prompt so an audit/release task also says: follow the audit-and-release skill; synthetic fixtures only; report files + gate evidence + unresolved risk; hard-stop before push/tag/Release unless the task names that exact action.

`docs/maintenance/runbook.md` keeps the existing "Scheduled workflows: authored, not enabled", triage flow, Grok channel notes, and cadence-claim rule. Add one section at the end, **Audit-and-release loop**, that points at the sibling checklist and the skill, and restates that invoking the loop does not enable freshness workflows and does not authorize last-mile.

`.grok/skills/README.md` is a five-line note: project-local maintainer skills; not product surface; `audit-and-release` is the only skill this spec adds.

## 10. Testing and tripwires

No product behavior changes, so the existing fixture matrix stays the source of truth for scan outcomes. This spec adds **loop-invariant** tests and script smoke:

1. **`tripwires.rs` extensions (no network):**
   - Cadence phrases already banned in `docs/agent-guide.md` also banned in `README.md` (the public front door). Keep the same list: `weekly`, `daily re-verification`, `continuously verified`, `always up to date`. Do **not** scan `docs/maintenance/runbook.md` or this spec — internal reminders are allowed. Do not ban the bare word `monthly` in research/history files.
   - `SECURITY.md` must not contain `until versioned releases exist` now that `0.0.1` / `0.0.2` exist. Replace that sentence in the same change that adds the tripwire.
   - Maintainer fetch scripts must not be referenced from `crates/harness-guard-core` or from CLI scan modules (string scan of those trees for `fetch-cited` / `probe-releases` / `web.archive.org/save`).
2. **Script hygiene:** `probe-releases.sh` and `fetch-cited.sh` contain `set -eu` and never redirect output into `rules/` or `freshness/`. Pin that by grepping the script sources (no `>` / `tee` into those trees). Do not add a dry-run flag in this spec and do not hit the network from `cargo test`.
3. **`run-gates.sh`:** invoked in the implementation plan's self-check; not a CI-required job in this spec (CI already runs the same tools). If the wrapper is wrong, local agents will notice on the first pass.
4. **Skill / docs:** the new skill description and `audit-and-release-loop.md` must not introduce user-facing cadence claims. Positioning phrase stays intact wherever user-facing text is touched.

`actionlint` after any workflow edit remains required. This spec does not require a workflow edit; if one happens anyway, the gate applies.

## 11. Error handling and conservative defaults

- Fetch failure (timeout, non-200, empty body, invalid hash): treat that URL as **unverified**, do not refresh `retrieved` / `content_hash` from a partial body, and list it in the STOP packet's risk section. Never invent a hash.
- Hash drift without a readable semantic change (JS chrome, CDN): still a drift. The certifying session must open the live page, decide if the *claim* moved, and record the new hash only after that read. When unsure, keep the old rule outcomes and write the uncertainty in `notes` + the handoff.
- Version newer than every `tested_versions.max`: do not silently widen. Either re-verify against that version or leave the range and accept `stale-ruleset` for that install.
- Probe shows Copilot (or any non-shipped tool) moved: update `last-seen.json` only if this pass is already writing certified freshness facts; never add rules.
- Agent tempted to "help" by pushing because gates are green: forbidden. Green gates are a precondition for the STOP packet, not a substitute for owner go.
- Same-day repeat of the loop with no drift: allowed to produce an empty diff and a STOP packet that says "no tag recommended".

## 12. Sequencing

```text
WP1  docs/maintenance/audit-and-release-loop.md
     + runbook pointer section
     + AGENTS.md / CONTEXT.md / CONTRIBUTING.md pointers
WP2  scripts/freshness/probe-releases.sh
     scripts/freshness/fetch-cited.sh
     scripts/maintenance/run-gates.sh
WP3  .grok/skills/audit-and-release/SKILL.md
     + .grok/skills/README.md
WP4  tripwires.rs + SECURITY.md supported-version sentence
     (SECURITY.md is already stale vs tagged 0.0.1/0.0.2; this WP owns that fix)
WP5  implementation self-check: run-gates.sh green; tripwires green;
     skill forbids last-mile; no ENABLE_FRESHNESS_WORKFLOWS mention as an action
```

WP1 before WP3 so the skill can point at a real checklist. WP2 before WP3 so the skill names real scripts. WP4 can parallel WP2. No tag is part of shipping this system; landing the docs/scripts/skill on `main` is itself a last-mile that needs the ordinary owner go to push, and does **not** require a version bump unless the owner wants one.

## 13. Acceptance criteria

1. An owner can say `/audit-and-release` (Grok) or "follow `.grok/skills/audit-and-release/SKILL.md`" (any coding agent) and the agent has a complete, ordered loop that ends in a STOP packet.
2. `docs/maintenance/runbook.md` still describes authored-off workflows and human triage; the new loop is a sibling, not a replacement.
3. `probe-releases.sh` and `fetch-cited.sh` exist, share `normalize.sh` / `extract-urls.sh`, write nothing to `rules/` or `freshness/`, and are unused by core/scan/CI product jobs.
4. `run-gates.sh` runs the required local gates and `actionlint` when workflows changed.
5. Constitution files point at the skill and restate the per-action last-mile go. No root `CLAUDE.md`.
6. Tripwires fail if `README.md` or `docs/agent-guide.md` gain cadence claims, if `SECURITY.md` again says versioned releases do not exist, or if core/scan reference the fetch helpers.
7. `SECURITY.md` names the current tagged preview as the supported line.
8. No workflow in this change sets or documents enabling `ENABLE_FRESHNESS_WORKFLOWS` as part of the loop.
9. No user-facing file claims a verification cadence.
10. Held 2026-08-13 candidate rules remain unshipped. No new harness, `--fix`, network-in-product, crates.io, or tag move is specified as a loop action.
11. The 2026-08-13 pass is reproducible from the checklist: inventory → probe → fetch/hash/Wayback → certify rules + goldens + freshness + CalVer → docs sweep → hygiene → version/CHANGELOG → gates → STOP.

## 14. Resolved decision log

- **(a) System shape:** runbook + repo-local skill; not checklists-only; not auto-PRs; not a Grok `.rhai` primary workflow. §4, §7.
- **(b) Skill home:** `.grok/skills/audit-and-release/SKILL.md` only. Claude and other agents are pointed there by `AGENTS.md` / `CONTRIBUTING.md`. No `.claude/skills/` duplicate. No root `CLAUDE.md`. §7, §9.
- **(c) Process home:** extend `docs/maintenance/runbook.md`; add sibling `docs/maintenance/audit-and-release-loop.md`. Historical 0.0.1 checklist stays historical. §5, §6.8.
- **(d) Last-mile:** prepare then STOP. Push, annotated tag, GitHub Release, package publish, and repo-settings each need a fresh explicit go. Tag and Release are separate. Never move existing tags. Default publish is tag + optional GitHub Release; no crates.io. §6.0, §6.8.
- **(e) Who may edit rules:** GitHub bots never. An owner-invoked audit session may certify after live primary-source re-verify. New rules still need separate approval. §6.0, §6.3.
- **(f) Freshness workflows:** stay off. Local `probe-releases.sh` / `fetch-cited.sh` are the on-demand substitute. §8.
- **(g) Version axes:** CalVer always on certified rule/evidence change; binary semver independent; skill proposes next patch when a tag is wanted; tag name is owner-chosen. §6.6.
- **(h) Cadence:** no public claim; tripwire on README + agent-guide; runbook may remind internally. §3, §10.
- **(i) Scripts, not product:** POSIX helpers under `scripts/`; no xtask; no scan networking. §8.
- **(j) Held candidates:** remain held. Loop records new keys in `CONTEXT.md`, does not ship them. §2, §6.3.
