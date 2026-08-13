# Recurring Audit-and-Release Loop Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Land a durable, agent-runnable maintainer loop (runbook + repo-local skill + fetch/probe/gate helpers + tripwires) that can repeat the 2026-08-13 freshness pass and **hard-stops** before every last-mile publish action.

**Architecture:** Canonical process lives in `docs/maintenance/audit-and-release-loop.md` (phase list, file lists, STOP packet). The Grok project skill `.grok/skills/audit-and-release/SKILL.md` is the agent prompt for the same phases (invocation, authorization, forbidden prefixes, report shape) and points at the checklist rather than duplicating it. Maintainer-only POSIX scripts under `scripts/freshness/` and `scripts/maintenance/` do network probes and wrap required gates; they are never imported by `harness-guard-core`, CLI scan, or CI product jobs. Constitution files gain short pointers. Tripwires pin cadence, supported-version, and “scripts stay out of scan” invariants.

**Tech Stack:** Markdown runbook + Grok `SKILL.md` (YAML frontmatter `name` + `description`). POSIX `sh` (`set -eu`) using existing `jq`, `curl`, `scripts/freshness/{extract-urls,normalize}.sh`. Rust tripwires in `crates/harness-guard-rules/tests/tripwires.rs` (no new crates, no new cargo deps, no xtask, no product CLI).

**Spec:** `docs/superpowers/specs/2026-08-13-audit-and-release-loop-design.md` (read it before starting any task; section references below are to that spec).

## Global Constraints

Copied from the spec and binding constitution; every task’s requirements implicitly include all of these.

- Positioning (verbatim, test-pinned): Harness Guard is a *local, execution-free, per-finding-cited config auditor*. “AI agent security scanner” appears nowhere in user-facing text.
- Binding constitution: `AGENTS.md`, `CONTEXT.md`, `CONTRIBUTING.md`. No root `CLAUDE.md`.
- System shape is runbook + repo-local skill. No auto-PRs, no Grok `.rhai` primary workflow, no xtask, no new product CLI, no new rules/harnesses.
- Last-mile is owner-gated **per action**. This plan prepares files only. Do **not** execute `git push`, `git tag`, `gh release`, `gh repo`, `cargo publish`, `npm publish`, or any write to GitHub repository variables/settings. Do not move tags `0.0.1`, `0.0.2`, or any existing tag. A later “just ship it” is not a go.
- Bots never edit `rules/`. This implementation must not ship a new rule or widen shipped coverage. Held 2026-08-13 candidates in `CONTEXT.md` stay held.
- Freshness workflows stay default-off. Do not set `ENABLE_FRESHNESS_WORKFLOWS`. Do not add a scheduled workflow that edits rules or tags. New files may mention the variable only as “do not enable.”
- New fetch/probe/gate scripts are maintainer-only, never in `harness-guard-core` or the scan path. Scans make zero network requests and execute nothing discovered.
- Extend `docs/maintenance/`; do not replace the triage flow in `docs/maintenance/runbook.md`.
- No crates.io, no npm publish, no public verification-cadence claim. Do not ban the bare word `monthly` in research/history files. Do not scan `docs/maintenance/runbook.md` or the spec for cadence words.
- Synthetic fixtures only. Never inspect `~/.codex`, `~/.claude`, `~/.grok`, `CODEX_HOME`, or other ambient harness stores.
- Landing this system does **not** bump workspace `0.0.2` or ruleset CalVer `2026.08.13` unless the owner separately asks. No new tag is part of this plan.
- Required validation after any code, script, or CI-affecting change: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo deny check`, `cargo test --workspace`; on macOS `scripts/no-egress/run-macos.sh`; `actionlint` if `.github/workflows/*.yml` changed. This plan does not require a workflow edit.
- `actionlint` after any workflow edit remains required. Do not edit workflows in this plan.

## Explicit assumptions

The spec is complete enough to implement. These mechanical defaults are locked here so executors do not invent others.

1. **No version bump for this landing.** Spec §12: landing docs/scripts/skill on `main` is itself a last-mile that needs an ordinary owner go to push and does not require a version bump. Leave `Cargo.toml` at `0.0.2` and `rules/ruleset.json` at `2026.08.13`. Put a short note under `CHANGELOG.md` `## [Unreleased]` only.
2. **Grok channel pointer body** is a single-line version string (confirmed by `docs/research/evidence/grok-build/2026-08-13/raw/stable` = `1.0.3`). `probe-releases.sh` trims the first line and compares it to `channels.grok-build.version`.
3. **“CLI scan modules”** for the no-helper tripwire means `crates/harness-guard-core/**` plus `crates/harness-guard-cli/src/**` plus `.github/workflows/ci.yml`. Do not scan `crates/harness-guard-cli/tests/**` (explain goldens already contain `web.archive.org` as a cited archive host, which is not `web.archive.org/save`).
4. **Historical checklist banner** is in scope for spec §5 (“do not make this the live last-mile”). Add a pointer; do not append new owner-go boxes there.
5. **Network scripts are not run by `cargo test`.** Hygiene is source-grep + `sh -n`. Live `probe-releases.sh` / `fetch-cited.sh` are optional maintainer smokes, not a gate of this plan.

## File structure

```text
docs/maintenance/audit-and-release-loop.md          # NEW: canonical phases A–H + STOP packet
docs/maintenance/runbook.md                         # EXTEND: pointer section only
.grok/skills/audit-and-release/SKILL.md             # NEW: agent prompt
.grok/skills/README.md                              # NEW: five-line note
scripts/freshness/probe-releases.sh                 # NEW: npm + Grok pointer vs last-seen
scripts/freshness/fetch-cited.sh                    # NEW: cited URLs vs url-hashes
scripts/maintenance/run-gates.sh                    # NEW: required validation wrapper
crates/harness-guard-rules/tests/tripwires.rs       # EXTEND: loop invariants
SECURITY.md                                         # FIX: supported-version sentence
AGENTS.md / CONTEXT.md / CONTRIBUTING.md            # EXTEND: pointers + last-mile
docs/superpowers/handoffs/2026-07-17-0.0.1-release-checklist.md  # BANNER only
CHANGELOG.md                                        # EXTEND: [Unreleased] note
notes/session-history.md                            # EXTEND: this landing
```

No new crate. No change under `crates/harness-guard-core/src`, CLI scan I/O, `rules/`, `fixtures/`, or `.github/workflows/`.

## Sequencing

```text
Task 1  tripwires (cadence + SECURITY phrase)     # TDD red
Task 2  SECURITY.md supported-version             # TDD green
Task 3  audit-and-release-loop.md + runbook + historical banner
Task 4  AGENTS.md / CONTEXT.md / CONTRIBUTING.md
Task 5  probe-releases.sh + fetch-cited.sh + script tripwires
Task 6  scripts/maintenance/run-gates.sh
Task 7  .grok/skills/audit-and-release/SKILL.md + .grok/skills/README.md
Task 8  self-check + CHANGELOG [Unreleased] + session-history
```

Task 7 must not start before Tasks 3, 5, and 6 exist (skill names real paths). Do not parallel Tasks 1 and 5 — both edit `tripwires.rs`.

---

### Task 1: Cadence and supported-version tripwires

**Files:**
- Modify: `crates/harness-guard-rules/tests/tripwires.rs`
- Test: `crates/harness-guard-rules/tests/tripwires.rs`

**Interfaces:**
- Consumes: existing `repo_root()` helper in that file.
- Produces: `readme_and_agent_guide_carry_positioning_and_no_cadence_claims` (replaces `agent_guide_carries_positioning_and_no_cadence_claims`) and `security_md_does_not_claim_versioned_releases_are_absent`. Later tasks rely on the cadence list staying exactly `weekly`, `daily re-verification`, `continuously verified`, `always up to date`.

- [ ] **Step 1: Write the failing / extended tripwires**

Replace `agent_guide_carries_positioning_and_no_cadence_claims` and append the SECURITY test. Keep the existing canary-upload and lifted-ban tests unchanged.

```rust
#[test]
fn readme_and_agent_guide_carry_positioning_and_no_cadence_claims() {
    let root = repo_root();
    let files = ["docs/agent-guide.md", "README.md"];
    let forbidden_phrase = ["AI agent", "security scanner"].join(" ");
    for rel in files {
        let path = root.join(rel);
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("{path:?} must be readable UTF-8"));
        assert!(
            text.contains("local, execution-free, per-finding-cited config auditor"),
            "{rel} must carry the positioning phrase"
        );
        assert!(
            !text.contains(&forbidden_phrase),
            "{rel} must not contain {forbidden_phrase:?}"
        );
        for cadence in [
            "weekly",
            "daily re-verification",
            "continuously verified",
            "always up to date",
        ] {
            assert!(
                !text.to_lowercase().contains(cadence),
                "cadence claim {cadence:?} found in {rel}"
            );
        }
    }
}

#[test]
fn security_md_does_not_claim_versioned_releases_are_absent() {
    let text = std::fs::read_to_string(repo_root().join("SECURITY.md")).unwrap();
    assert!(
        !text.to_lowercase().contains("until versioned releases exist"),
        "SECURITY.md must name the current tagged preview now that 0.0.1 / 0.0.2 exist"
    );
}
```

Do **not** walk `docs/maintenance/runbook.md`, `docs/superpowers/specs/`, or research/history files in these tests.

- [ ] **Step 2: Run the tripwire file and confirm the expected red**

Run: `cargo test -p harness-guard-rules --test tripwires -- --nocapture`

Expected:
- `security_md_does_not_claim_versioned_releases_are_absent` **FAIL** because `SECURITY.md` line 23 still says “until versioned releases exist”.
- `readme_and_agent_guide_carry_positioning_and_no_cadence_claims` **PASS** on the current tree (`README.md` already carries the positioning phrase and has none of the four cadence strings).
- Existing canary / lifted-ban tests still PASS.

- [ ] **Step 3: Commit**

```bash
git add crates/harness-guard-rules/tests/tripwires.rs
git commit -m "test: pin README cadence and SECURITY supported-version tripwires"
```

---

### Task 2: Fix SECURITY.md supported-version language

**Files:**
- Modify: `SECURITY.md` (the “Supported version” section, currently lines 20–23)
- Test: `crates/harness-guard-rules/tests/tripwires.rs`

**Interfaces:**
- Consumes: the tripwire from Task 1.
- Produces: `SECURITY.md` names the current tagged preview (`0.0.2`) as the supported line. No cadence words.

- [ ] **Step 1: Replace the stale sentence**

Change the “Supported version” section to exactly:

```markdown
## Supported version

Harness Guard is an early preview. Security fixes target the latest tagged
preview on `main` (currently `0.0.2`). There is no long-term support line.
```

Do not mention monthly/weekly/continuous verification. Do not mention `ENABLE_FRESHNESS_WORKFLOWS`.

- [ ] **Step 2: Re-run tripwires**

Run: `cargo test -p harness-guard-rules --test tripwires`

Expected: PASS, including `security_md_does_not_claim_versioned_releases_are_absent`.

- [ ] **Step 3: Commit**

```bash
git add SECURITY.md
git commit -m "docs: name tagged preview 0.0.2 as the supported line"
```

---

### Task 3: Canonical loop checklist and runbook pointer

**Files:**
- Create: `docs/maintenance/audit-and-release-loop.md`
- Modify: `docs/maintenance/runbook.md` (append one section at the end; do not rewrite “Scheduled workflows”, “Triage flow”, “Grok Build channel notes”, or “Cadence claims”)
- Modify: `docs/superpowers/handoffs/2026-07-17-0.0.1-release-checklist.md` (banner only)

**Interfaces:**
- Consumes: spec §6.0–§6.8 (this file is the one home for phases, file lists, and the STOP packet).
- Produces: the path later named by the skill and constitution pointers.

- [ ] **Step 1: Write `docs/maintenance/audit-and-release-loop.md`**

Create the file with this exact body (no cadence SLA; internal “when you next sit down” reminder is allowed):

```markdown
# Audit-and-release loop

Canonical maintainer process for repeating a certified freshness / docs /
release-prep pass. Agent prompt: `.grok/skills/audit-and-release/SKILL.md`.
Triage of authored-off GitHub freshness jobs stays in `runbook.md`.

This loop does **not** enable `ENABLE_FRESHNESS_WORKFLOWS`, ship a new rule
or harness, move an existing tag, or authorize last-mile publish actions.

Start from current public `main` at the latest existing tag plus any already
merged work. Work on `audit/YYYY-MM-DD` (UTC date of the session). Never
amend a published tag. Never inspect ambient harness stores (`~/.codex`,
`~/.claude`, `~/.grok`, `CODEX_HOME`).

The historical file
`docs/superpowers/handoffs/2026-07-17-0.0.1-release-checklist.md` is the
0.0.1/0.0.2 execution record. It is not the live last-mile doc. New passes
write `docs/superpowers/handoffs/YYYY-MM-DD-audit-and-release.md` (or append
`notes/session-history.md` if the pass produced no tree change).

## Session authorization

Invoking `/audit-and-release`, or an explicit “run the audit-and-release
loop” request, authorizes **this session** to:

- retrieve vendor docs via `scripts/freshness/probe-releases.sh` and
  `scripts/freshness/fetch-cited.sh` (network lives here, not in scan);
- certify existing shipped rules after live primary-source re-verification;
- update `rules/`, synthetic fixtures / goldens, `freshness/`, Grok evidence
  packs, docs, CHANGELOG, and version pins in the working tree;
- run `scripts/maintenance/run-gates.sh`;
- write the STOP packet.

It does **not** authorize: `git push`; `git tag`; GitHub Release; crates.io
/ npm; setting `ENABLE_FRESHNESS_WORKFLOWS`; repo-settings changes; moving
an existing tag; shipping a new rule or harness; `--fix`; network-in-product;
a public cadence sentence.

A later “just ship it” / “finish the release” is **not** a go. Each last-mile
action needs a fresh explicit owner sentence that names that action. Tag go
and Release go are never the same sentence.

Default mode is a **full pass** (phases A–H). A narrower owner request
(“probe only”, “docs sweep only”) may stop after the named phase and must
refuse to bump CalVer or prepare a tag unless the owner expands the task.
Do not half-refresh `retrieved` / `verified_on` dates without a CalVer bump.

## Phase A — Inventory (read-only)

Record, without editing:

- workspace version from `Cargo.toml` (`0.0.2` as of 2026-08-13);
- `rules/ruleset.json` `ruleset_version` (`2026.08.13` as of 2026-08-13);
- existing tags (`git tag -l`); refuse any plan that moves one;
- shipped rule inventory via `capabilities --json` after a local build, not
  by hardcoding counts;
- `freshness/last-seen.json` package and `channels.grok-build` pins;
- open questions already parked in `CONTEXT.md` (held 2026-08-13 candidates
  stay held);
- whether `SECURITY.md` still names the current tagged preview.

## Phase B — Release probe (maintainer network, no writes)

Run `scripts/freshness/probe-releases.sh`. Compare live npm dist-tags and
the Grok CLI pointer against `freshness/last-seen.json`:

| Probe | Source | Compare to |
| --- | --- | --- |
| `@anthropic-ai/claude-code` `stable` | npm abbreviated registry | `packages["@anthropic-ai/claude-code"].version` |
| `@openai/codex` `latest` | npm abbreviated registry | `packages["@openai/codex"].version` |
| `@github/copilot` `latest` | npm abbreviated registry | `packages["@github/copilot"].version` (watched only; not a shipped harness) |
| `@xai-official/grok` `latest` | npm abbreviated registry | `packages["@xai-official/grok"].version` |
| Grok channel pointer | `https://x.ai/cli/stable` | `channels.grok-build.version` |

Print the script’s drift table. Do not write `last-seen.json` here. Copilot
movement is watch-only; it does not widen shipped coverage and does not
justify a new rule.

When Grok versions disagree across npm, the CLI pointer, or OSS
`SOURCE_REV`, follow `runbook.md`: prefer the CLI pointer as the install
channel, re-check the monorepo `SOURCE_REV`, and do not widen
`tested_versions` until those agree or the disagreement is written as a
limitation / `unknown`.

A **probe-only** run stops after this phase (and the fetch report if the
owner asked for it) and writes nothing.

## Phase C — Cited-source re-verify (maintainer network, then certified writes)

Run `scripts/freshness/fetch-cited.sh` (URLs from
`scripts/freshness/extract-urls.sh`, hashes via
`scripts/freshness/normalize.sh`). Optionally
`scripts/freshness/fetch-cited.sh --save-wayback`. The script never edits
`rules/` or `freshness/`.

A **full pass** (the default) re-reads every cited page even when hashes
match, refreshes `retrieved` / `verified_on` / `archived_url` to the session
date, and therefore bumps CalVer. A probe-only or no-drift no-op run (owner
asked to check, not re-certify) stops after the reports and writes nothing.

Then, on a full pass, apply certified updates rule by rule:

1. Re-read the live official page (and Grok OSS blobs at the pinned
   `SOURCE_REV` when the citation is a GitHub blob). Never use `data/` or
   legacy research as evidence.
2. If the claim still holds: refresh `retrieved`, `content_hash`,
   `archived_url`, `tested_versions` (`verified_on` = UTC date of this
   session; widen `max` only when the live version was actually checked).
   Keep observation keys and outcomes unchanged unless the official text
   changed the documented key or default.
3. If the claim no longer holds: change the rule data (outcomes,
   limitations, `unknown_conditions`) conservatively. Auth-dependent policy
   stays unknown. No matching range ⇒ `stale-ruleset` / `unknown`, never an
   inferred pass. Grok stays local-posture / `official-documentation` unless
   a new lab pack exists under `docs/research/evidence/grok-build/<date>/`.
4. If a new key appears (the 2026-08-13 held list, or anything like it):
   **do not ship it**. Record it in `CONTEXT.md` under a dated “evaluated,
   not shipped” table. New rules need a separate owner approval.
5. Update `freshness/url-hashes.json` and, when versions moved,
   `freshness/last-seen.json` (`checked` = session date).
6. If any rule JSON, certified freshness fact, or fixture fact derived from
   those changed: bump `rules/ruleset.json` `ruleset_version` to today’s
   CalVer (`YYYY.MM.DD`). Same-day second pass keeps that CalVer; a later
   calendar day gets a new CalVer. Never leave a certified rule change on
   the previous CalVer.
7. Grok channel or `SOURCE_REV` movement: add
   `docs/research/evidence/grok-build/<date>/` following the 2026-08-13 pack
   shape (`README.md` + `raw/` artifacts + semantic hashes). Do not delete
   prior dated packs.

After rule edits, update every `fixtures/**/expected.json` that embeds
`ruleset_version`, `rules_verified_date`, or `rules_last_verified_version`,
plus `crates/harness-guard-cli/tests/goldens/capabilities.expected.json`. A
golden that still names the previous CalVer after a bump is a failed pass.

Fetch failure (timeout, non-200, empty body, invalid hash): treat that URL
as **unverified**, do not refresh `retrieved` / `content_hash` from a
partial body, list it in the STOP packet risk section, and never invent a
hash. Hash drift without a readable semantic change is still drift: open
the live page, decide if the *claim* moved, and record the new hash only
after that read. Version newer than every `tested_versions.max`: do not
silently widen.

## Phase D — Docs-vs-shipped-state

Keep these in lockstep with the tree that would be tagged. Do not claim
coverage the binary does not have.

| Surface | Must match |
| --- | --- |
| `CONTEXT.md` current phase, rule counts, verified-through versions, CalVer, binary version, tag state | shipped `capabilities` + `ruleset.json` + `Cargo.toml` + `git tag` |
| `README.md` scope blurb and example versions | same |
| `docs/agent-guide.md` sample `capabilities` JSON | same; still no cadence words |
| `CHANGELOG.md` | Keep a Changelog; dated section only when a binary bump is proposed |
| `SECURITY.md` “Supported version” | current tagged preview (latest tag on `main`), not “until versioned releases exist” |
| `crates/harness-guard-cli/tests/cli_surface.rs` | `harness-guard <binary>` string |
| `crates/harness-guard-cli/src/render_term.rs` unit-test literals | binary version |
| `notes/session-history.md` | append-only summary of this pass |

Cadence language remains forbidden in user-facing files. This runbook may
say “run this loop when you next sit down to maintain the repo” without
promising a public SLA.

## Phase E — Safety / tripwire / CI / deny hygiene

Read-only unless a real defect is in scope for this pass:

- `deny.toml` still bans network crates from the workspace graph (including
  dev-deps).
- `crates/harness-guard-core/clippy.toml` disallowed methods still cover
  process / env / net.
- `tripwires.rs`: canary-upload claims still absent; README and agent-guide
  still carry the positioning phrase and still lack weekly / daily /
  continuously-verified language; `SECURITY.md` does not say versioned
  releases are absent.
- `.github/workflows/{release-watch,doc-drift}.yml` still job-gated on
  `vars.ENABLE_FRESHNESS_WORKFLOWS == 'true'`. Do not flip the variable.
  Do not add a third scheduled workflow that edits rules or tags.
- Maintainer scripts are not referenced from core, CLI scan, or CI product
  jobs.
- Hostile / redaction / real-home-refusal tests still exist; do not weaken
  a golden to make a pass.

In-scope fixes: broken tripwires, stale comments that re-introduce canary
or cadence claims, a workflow edit that accidentally drops the enablement
gate, deny-policy drift, docs that contradict shipped capabilities. Out of
scope: new harness support, new product flags, rewriting CI.

## Phase F — Version bumps

Two axes, never collapsed:

| Event | Ruleset CalVer | Workspace / binary semver | Tag |
| --- | --- | --- | --- |
| Certified rule, source, `tested_versions`, or `freshness/` fact change | **Always** today’s CalVer | Unchanged unless a tag is planned or a version-pin file must move | Owner-chosen; skill proposes next patch |
| Code, docs, tests, scripts, hygiene only | No | Only if `CARGO_PKG_VERSION` pins would lie | Usually none |
| Owner wants a public snapshot of this tree | CalVer already bumped if rules changed | Propose next **patch** unless the owner already named a version | Owner-chosen; never reuse or move a tag |

Binary minor/major is not invented by this loop. A product-surface change
(new command, schema bump, new harness) needs its own approved spec.

When the binary version moves, update every pin in one commit-set:
`Cargo.toml` workspace version, `cli_surface.rs`, `render_term.rs` test
literals, `capabilities.expected.json` `harness_guard_version`, README /
CONTEXT / agent-guide samples.

CHANGELOG: put certified ruleset work and binary bumps under a dated
`## [X.Y.Z] - YYYY-MM-DD` when a tag is being prepared; otherwise leave
notes under `## [Unreleased]`. Use the 0.0.2 entry as the tone model: what
was re-verified, which versions widened, what did *not* change (observation
keys, outcomes), and that freshness automation remains authored-off.

## Phase G — Gates at the exact commit

Run `scripts/maintenance/run-gates.sh`. Re-run after any late fix so the
STOP packet names a commit that actually passed. CI is a second opinion;
do not skip local gates because CI is green.

## Phase H — STOP (last-mile packet)

Terminal state of an unauthorized last-mile. Print a packet and **halt**.

Packet contents (all concrete, no placeholders):

- Branch name and `git rev-parse HEAD`.
- Existing tags that must not move.
- Proposed tag name (suggestion only) and why (ruleset-only vs binary bump).
- Whether a GitHub Release is recommended (yes when a new tag is proposed;
  still a separate go).
- CHANGELOG excerpt for that version (the section body, not the whole file).
- Files changed and a one-paragraph risk list (unverified URLs, held
  candidates, Grok-without-lab, docs that still disagree).
- Exact commands, **not executed**:

```bash
git push -u origin audit/YYYY-MM-DD          # only after: owner go to push this branch
# after merge to main, only after: owner go to push main
git tag -a X.Y.Z <sha> -m "Harness Guard X.Y.Z"   # only after: owner go to tag X.Y.Z at <sha>
git push origin X.Y.Z                         # only after: owner go to push that tag
gh release create X.Y.Z --title "X.Y.Z" --notes-file <excerpt>  # only after: separate owner go for GitHub Release
```

- Explicit non-actions: no `cargo publish`, no npm, no
  `ENABLE_FRESHNESS_WORKFLOWS`, no repo-settings, no moving `0.0.1` /
  `0.0.2`.

If the owner later gives a go for **one** of those lines, run **that line
only**, then STOP again. A tag go is not a Release go. A Release go is not
a crates.io go.
```

- [ ] **Step 2: Append the runbook pointer section**

Append this section at the end of `docs/maintenance/runbook.md`. Leave every existing section byte-stable aside from the new trailing heading.

```markdown
## Audit-and-release loop

The repeatable certified pass (inventory → probe → fetch/hash → certify
existing rules → docs sweep → hygiene → version/CHANGELOG → gates → STOP)
lives in `docs/maintenance/audit-and-release-loop.md`. Agents invoke it
through `.grok/skills/audit-and-release/SKILL.md`.

Invoking that loop does not replace this file’s triage flow, does not
enable `ENABLE_FRESHNESS_WORKFLOWS`, and does not authorize push, annotated
tag, GitHub Release, package publish, or repo-settings changes. Run this
loop when you next sit down to maintain the repo.
```

- [ ] **Step 3: Banner the historical last-mile file**

Insert this block as the new first lines of
`docs/superpowers/handoffs/2026-07-17-0.0.1-release-checklist.md`, above
the existing title:

```markdown
> **Historical.** This is the 0.0.1 / 0.0.2 execution record. Live last-mile
> process: `docs/maintenance/audit-and-release-loop.md`. Do not append new
> owner-go boxes here.

```

- [ ] **Step 4: Confirm the new checklist does not claim a public cadence**

Run:

```bash
python3 - <<'PY'
from pathlib import Path
text = Path("docs/maintenance/audit-and-release-loop.md").read_text().lower()
# Internal "when you next sit down" is fine. Do not scan bare "weekly" here:
# Phase E names the tripwire list. Ban public-SLA wording only.
for needle in [
    "daily re-verification",
    "continuously verified",
    "always up to date",
    "verified monthly",
    "verified weekly",
]:
    assert needle not in text, needle
print("loop-doc cadence check OK")
PY
```

Expected: `loop-doc cadence check OK`. Do **not** put the four tripwire cadence strings into `README.md` or `docs/agent-guide.md`.

- [ ] **Step 5: Commit**

```bash
git add docs/maintenance/audit-and-release-loop.md docs/maintenance/runbook.md \
  docs/superpowers/handoffs/2026-07-17-0.0.1-release-checklist.md
git commit -m "docs: add audit-and-release loop checklist and runbook pointer"
```

---

### Task 4: Constitution pointers

**Files:**
- Modify: `AGENTS.md` (after the existing `## External actions` section)
- Modify: `CONTEXT.md` (`**Current phase:**` paragraph and `## Session continuity`)
- Modify: `CONTRIBUTING.md` (the boxed prompt under “Working with a coding agent”)

**Interfaces:**
- Consumes: paths created in Task 3.
- Produces: every coding agent finds the loop without a root `CLAUDE.md`.

- [ ] **Step 1: Add the AGENTS.md pointer**

Append this section after the existing `## External actions` bullets in `AGENTS.md`. Do not turn `AGENTS.md` into a second checklist.

```markdown
## Audit and release

When asked to audit, re-verify, run a freshness pass, or prepare a release,
follow `.grok/skills/audit-and-release/SKILL.md` and
`docs/maintenance/audit-and-release-loop.md`. Invoking the loop authorizes
certified re-verify of existing shipped rules and a STOP packet; it does
not authorize `git push`, `git tag`, GitHub Release, package publish,
moving an existing tag, or setting `ENABLE_FRESHNESS_WORKFLOWS`. Each of
those needs a fresh explicit owner sentence that names that exact action.
```

- [ ] **Step 2: Update CONTEXT.md current phase and session continuity**

Replace the `**Current phase:**` paragraph (currently the 0.0.1-complete / Task 25 wording) with:

```markdown
**Current phase:** 0.0.2 is tagged; ruleset CalVer is 2026.08.13. Repeat a
freshness / docs / release-prep pass via
`docs/maintenance/audit-and-release-loop.md` and
`.grok/skills/audit-and-release/SKILL.md`. Tags `0.0.1` and `0.0.2` exist;
do not move them. Last-mile (push, annotated tag, GitHub Release) needs a
fresh explicit owner go per action.
```

In `## Session continuity`, add this bullet at the top of the list (keep the existing bullets):

```markdown
- Follow `docs/maintenance/audit-and-release-loop.md` to repeat the
  2026-08-13 certified pass. Triage of authored-off freshness jobs stays in
  `docs/maintenance/runbook.md`.
```

Keep the existing tag-state sentences under `## Architecture and release state` true (`0.0.1` and `0.0.2` exist; do not move them). Do not ship any held 2026-08-13 candidate.

- [ ] **Step 3: Extend the CONTRIBUTING boxed prompt**

Replace the fenced `text` block under “Working with a coding agent” with:

```text
Read AGENTS.md, CONTRIBUTING.md, and CONTEXT.md first. Work only on <task>.
If the task is an audit, re-verify, freshness pass, or release prep, follow
`.grok/skills/audit-and-release/SKILL.md` and
`docs/maintenance/audit-and-release-loop.md`. Use synthetic fixtures; never
inspect ambient HOME, CODEX_HOME, ~/.codex, ~/.claude, ~/.grok, or other
sensitive stores. Preserve no-network/no-execution scan guarantees and
redaction. Add focused tests, run the repository validation commands, then
report the changed files, gate evidence (which gates ran and their exit
status), and any unresolved risk. Hard-stop before push, annotated tag, or
GitHub Release unless the task names that exact action. Do not add a new
rule, tool, dependency, workflow activation, or publishing action unless
the task explicitly authorizes it.
```

- [ ] **Step 4: Confirm no root CLAUDE.md and no cadence slip**

Run:

```bash
test ! -e CLAUDE.md && echo "no root CLAUDE.md"
python3 - <<'PY'
from pathlib import Path
needles = ["weekly", "daily re-verification", "continuously verified", "always up to date"]
for rel in ["AGENTS.md", "CONTEXT.md", "CONTRIBUTING.md", "README.md", "docs/agent-guide.md"]:
    text = Path(rel).read_text().lower()
    for n in needles:
        assert n not in text, f"{rel}: {n}"
print("constitution cadence check OK")
PY
cargo test -p harness-guard-rules --test tripwires
```

Expected: `no root CLAUDE.md`, `constitution cadence check OK`, tripwires PASS.

- [ ] **Step 5: Commit**

```bash
git add AGENTS.md CONTEXT.md CONTRIBUTING.md
git commit -m "docs: point constitution files at the audit-and-release loop"
```

---

### Task 5: Maintainer probe and fetch scripts

**Files:**
- Create: `scripts/freshness/probe-releases.sh`
- Create: `scripts/freshness/fetch-cited.sh`
- Modify: `crates/harness-guard-rules/tests/tripwires.rs`
- Test: `crates/harness-guard-rules/tests/tripwires.rs`
- Unchanged (call only): `scripts/freshness/extract-urls.sh`, `scripts/freshness/normalize.sh`, `freshness/last-seen.json`, `freshness/url-hashes.json`

**Interfaces:**
- Consumes: `extract-urls.sh` (prints unique cited URLs), `normalize.sh` (prints 64 hex chars), `freshness/last-seen.json` shape `{packages, channels.grok-build}`, `freshness/url-hashes.json` shape `{hashes}`.
- Produces:
  - `scripts/freshness/probe-releases.sh` — exit `0` all match, `1` any drift, `2` fetch/parse failure; stdout TSV table; writes nothing under `rules/` or `freshness/`.
  - `scripts/freshness/fetch-cited.sh [--save-wayback]` — exit `0` all match, `1` drift or missing, `2` fetch/hash failure; stdout `url old new status`; optional `wayback\turl\tsnapshot-or-unavailable`; writes nothing under `rules/` or `freshness/`.
- Probe pairs (hardcoded, same as `.github/workflows/release-watch.yml`): `@anthropic-ai/claude-code`/`stable`, `@openai/codex`/`latest`, `@github/copilot`/`latest`, `@xai-official/grok`/`latest`, plus GET `https://x.ai/cli/stable`.

- [ ] **Step 1: Write the failing script tripwires**

Append to `crates/harness-guard-rules/tests/tripwires.rs`:

```rust
fn line_writes_into_rules_or_freshness(line: &str) -> bool {
    let compact: String = line
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
        .to_lowercase();
    let after_redirect = compact.contains(">freshness/")
        || compact.contains(">>freshness/")
        || compact.contains(">rules/")
        || compact.contains(">>rules/");
    let tee_into = compact.contains("teefreshness/")
        || compact.contains("tee-afreshness/")
        || compact.contains("teerules/")
        || compact.contains("tee-arules/");
    after_redirect || tee_into
}

#[test]
fn maintainer_fetch_scripts_are_strict_and_write_nothing_under_rules_or_freshness() {
    let root = repo_root();
    let scripts = [
        root.join("scripts/freshness/probe-releases.sh"),
        root.join("scripts/freshness/fetch-cited.sh"),
    ];
    for script in scripts {
        let text = std::fs::read_to_string(&script)
            .unwrap_or_else(|_| panic!("maintainer script {script:?} must exist"));
        assert!(
            text.contains("set -eu"),
            "{script:?} must contain set -eu"
        );
        for line in text.lines() {
            assert!(
                !line_writes_into_rules_or_freshness(line),
                "{script:?} must not redirect into rules/ or freshness/: {line}"
            );
        }
    }
}

#[test]
fn core_cli_scan_and_ci_do_not_reference_maintainer_fetch_helpers() {
    let root = repo_root();
    let mut files = Vec::new();
    walk_files(&root.join("crates/harness-guard-core"), &mut files);
    walk_files(&root.join("crates/harness-guard-cli/src"), &mut files);
    files.push(root.join(".github/workflows/ci.yml"));
    assert!(files.iter().any(|p| p.ends_with("scan.rs")));
    let needles = ["fetch-cited", "probe-releases", "web.archive.org/save"];
    for file in files {
        let text = std::fs::read_to_string(&file)
            .unwrap_or_else(|_| panic!("{file:?} must be readable UTF-8"));
        for needle in needles {
            assert!(
                !text.contains(needle),
                "{needle:?} must not appear in product/CI tree {file:?}"
            );
        }
    }
}
```

- [ ] **Step 2: Run tripwires and confirm the scripts-exist test is red**

Run: `cargo test -p harness-guard-rules --test tripwires -- --nocapture`

Expected: `maintainer_fetch_scripts_are_strict_and_write_nothing_under_rules_or_freshness` **FAIL** (`must exist`). `core_cli_scan_and_ci_do_not_reference_maintainer_fetch_helpers` **PASS** on the current tree.

- [ ] **Step 3: Write `scripts/freshness/probe-releases.sh`**

Create the file, `chmod +x`. Header comments must not put `>` on the same compact line as `freshness/` or `rules/` (the tripwire would fire). Use temp files under `/tmp` via `mktemp`, never under `freshness/` or `rules/`.

```sh
#!/bin/sh
# probe-releases.sh — compare live npm dist-tags and the Grok CLI pointer
# against last-seen.json. Maintainer-only. Does not write the freshness
# tree or the rules tree.
# Exit 0 all match; 1 any drift; 2 fetch/parse failure.
set -eu
cd "$(dirname "$0")/../.."

LAST_SEEN="freshness/last-seen.json"

if ! jq -e \
  'type == "object" and (.packages | type == "object") and (.channels["grok-build"] | type == "object")' \
  "$LAST_SEEN" >/dev/null; then
  echo "FAIL: last-seen schema missing packages or channels.grok-build" >&2
  exit 2
fi

status=0

probe_npm() {
  pkg="$1"
  tag="$2"
  encoded=$(printf '%s' "$pkg" | sed 's|/|%2F|')
  tmp=$(mktemp)
  if ! curl --fail --silent --show-error \
    -H 'Accept: application/vnd.npm.install-v1+json' \
    "https://registry.npmjs.org/$encoded" \
    --output "$tmp"; then
    echo "FAIL: npm fetch failed for $pkg" >&2
    rm -f "$tmp"
    exit 2
  fi
  if ! jq -e 'type == "object" and (."dist-tags" | type == "object")' "$tmp" >/dev/null; then
    echo "FAIL: npm document shape wrong for $pkg" >&2
    rm -f "$tmp"
    exit 2
  fi
  live=$(jq -er --arg tag "$tag" \
    '."dist-tags"[$tag] | strings | select(length > 0)' "$tmp") || {
    echo "FAIL: dist-tag $tag missing for $pkg" >&2
    rm -f "$tmp"
    exit 2
  }
  seen=$(jq -er --arg pkg "$pkg" \
    '.packages[$pkg].version | strings | select(length > 0)' "$LAST_SEEN") || {
    echo "FAIL: last-seen missing version for $pkg" >&2
    rm -f "$tmp"
    exit 2
  }
  rm -f "$tmp"
  if [ "$seen" = "$live" ]; then
    row_status="match"
  else
    row_status="drift"
    status=1
  fi
  printf '%s\t%s\t%s\t%s\t%s\n' "$pkg" "$tag" "$seen" "$live" "$row_status"
}

printf '%s\t%s\t%s\t%s\t%s\n' "package" "tag" "seen" "live" "status"
probe_npm "@anthropic-ai/claude-code" "stable"
probe_npm "@openai/codex" "latest"
probe_npm "@github/copilot" "latest"
probe_npm "@xai-official/grok" "latest"

tmp=$(mktemp)
if ! curl --fail --silent --show-error --location \
  "https://x.ai/cli/stable" --output "$tmp"; then
  echo "FAIL: fetch of the Grok CLI pointer failed" >&2
  rm -f "$tmp"
  exit 2
fi
live=$(sed -n '1p' "$tmp" | tr -d ' \t\r\n')
rm -f "$tmp"
if [ -z "$live" ]; then
  echo "FAIL: empty Grok channel pointer" >&2
  exit 2
fi
seen=$(jq -er \
  '.channels["grok-build"].version | strings | select(length > 0)' \
  "$LAST_SEEN") || {
  echo "FAIL: last-seen missing channels.grok-build.version" >&2
  exit 2
}
if [ "$seen" = "$live" ]; then
  row_status="match"
else
  row_status="drift"
  status=1
fi
printf '%s\t%s\t%s\t%s\t%s\n' "grok-build" "cli-pointer" "$seen" "$live" "$row_status"

exit "$status"
```

- [ ] **Step 4: Write `scripts/freshness/fetch-cited.sh`**

Create the file, `chmod +x`. Document the same JS-rendered-page limitation already in `doc-drift.yml` and `normalize.sh`. No Playwright fallback.

```sh
#!/bin/sh
# fetch-cited.sh — fetch every cited rule URL, hash via normalize.sh, compare
# to url-hashes.json. Maintainer-only. Does not write the freshness tree or
# the rules tree.
# Usage: fetch-cited.sh [--save-wayback]
# Known limitation (same as doc-drift.yml / normalize.sh): regex tag stripping
# is approximate; JS-rendered pages may need a Playwright fallback later.
# Exit 0 all match; 1 any drift or missing entry; 2 fetch/hash failure.
set -eu
cd "$(dirname "$0")/../.."

SAVE_WAYBACK=0
if [ "${1:-}" = "--save-wayback" ]; then
  SAVE_WAYBACK=1
elif [ -n "${1:-}" ]; then
  echo "Usage: $0 [--save-wayback]" >&2
  exit 2
fi

HASHES="freshness/url-hashes.json"
if ! jq -e 'type == "object" and (.hashes | type == "object")' \
  "$HASHES" >/dev/null; then
  echo "FAIL: url-hashes schema missing hashes object" >&2
  exit 2
fi

tmp_dir=$(mktemp -d)
trap 'rm -rf "$tmp_dir"' EXIT
scripts/freshness/extract-urls.sh > "$tmp_dir/urls.txt"
if [ ! -s "$tmp_dir/urls.txt" ]; then
  echo "FAIL: no cited URLs" >&2
  exit 2
fi

status=0
printf '%s\t%s\t%s\t%s\n' "url" "old" "new" "status"

while IFS= read -r url; do
  [ -n "$url" ] || continue
  document=$(mktemp "$tmp_dir/document.XXXXXX")
  if ! curl --fail --silent --show-error --location "$url" --output "$document"; then
    echo "FAIL: fetch failed: $url" >&2
    exit 2
  fi
  if [ ! -s "$document" ]; then
    echo "FAIL: empty document: $url" >&2
    exit 2
  fi
  hex=$(scripts/freshness/normalize.sh "$document") || {
    echo "FAIL: normalize failed: $url" >&2
    exit 2
  }
  case "$hex" in
    *[!0-9a-f]* | "")
      echo "FAIL: invalid hash for $url" >&2
      exit 2
      ;;
  esac
  if [ "${#hex}" -ne 64 ]; then
    echo "FAIL: invalid hash length for $url" >&2
    exit 2
  fi
  new="sha256:$hex"
  old=$(jq -r --arg url "$url" '.hashes[$url] // empty' "$HASHES")
  if [ -z "$old" ]; then
    row_status="missing-from-freshness"
    old="-"
    status=1
  elif [ "$old" = "$new" ]; then
    row_status="match"
  else
    row_status="drift"
    status=1
  fi
  printf '%s\t%s\t%s\t%s\n' "$url" "$old" "$new" "$row_status"

  if [ "$SAVE_WAYBACK" -eq 1 ]; then
    snapshot=$(curl --fail --silent --show-error \
      "https://web.archive.org/save/$url" \
      --output /dev/null --write-out '%{redirect_url}' || true)
    if [ -z "$snapshot" ]; then
      snapshot="unavailable"
    fi
    printf 'wayback\t%s\t%s\n' "$url" "$snapshot"
  fi
done < "$tmp_dir/urls.txt"

exit "$status"
```

Note: `scripts/freshness/extract-urls.sh > "$tmp_dir/urls.txt"` writes a temp file, not `freshness/` or `rules/`. That is allowed.

- [ ] **Step 5: Syntax-check and make tripwires green (no network)**

Run:

```bash
chmod +x scripts/freshness/probe-releases.sh scripts/freshness/fetch-cited.sh
sh -n scripts/freshness/probe-releases.sh
sh -n scripts/freshness/fetch-cited.sh
cargo test -p harness-guard-rules --test tripwires
```

Expected: both `sh -n` silent; tripwires PASS.

Do **not** run the two scripts as part of this task (they hit the network). Do **not** add them to `.github/workflows/ci.yml`.

- [ ] **Step 6: Confirm CI product jobs still ignore them**

Run:

```bash
! grep -E 'fetch-cited|probe-releases' .github/workflows/ci.yml
! grep -E 'fetch-cited|probe-releases|web.archive.org/save' \
  crates/harness-guard-core/src/* crates/harness-guard-cli/src/*
echo "no product references"
```

Expected: `no product references`.

- [ ] **Step 7: Commit**

```bash
git add scripts/freshness/probe-releases.sh scripts/freshness/fetch-cited.sh \
  crates/harness-guard-rules/tests/tripwires.rs
git commit -m "feat: add maintainer probe-releases and fetch-cited helpers"
```

---

### Task 6: Gates wrapper

**Files:**
- Create: `scripts/maintenance/run-gates.sh`

**Interfaces:**
- Consumes: existing repo gates; `uname -s`; `git diff --name-only` and, when `origin/main` exists, `git diff --name-only origin/main...HEAD`.
- Produces: `scripts/maintenance/run-gates.sh` that runs, in order, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo deny check`, `cargo test --workspace`; then `scripts/no-egress/run-macos.sh` only when `uname -s` is `Darwin`; then `actionlint` if any `.github/workflows/*.yml` appears in those diffs, failing closed if `actionlint` is not on `PATH`. Must not call `fetch-cited.sh` or `probe-releases.sh`. Must not look at `~/.codex` / `~/.claude` / `~/.grok`. Must not tag or push.

- [ ] **Step 1: Create `scripts/maintenance/run-gates.sh`**

```sh
#!/bin/sh
# run-gates.sh — required local validation wrapper.
# Does not tag, push, or call the maintainer network scripts.
set -eu
cd "$(dirname "$0")/../.."

cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo deny check
cargo test --workspace

os=$(uname -s)
if [ "$os" = "Darwin" ]; then
  scripts/no-egress/run-macos.sh
fi

workflow_changed=0
if git diff --name-only | grep -q '^\.github/workflows/.*\.yml$'; then
  workflow_changed=1
fi
if git rev-parse --verify origin/main >/dev/null 2>&1; then
  if git diff --name-only origin/main...HEAD | grep -q '^\.github/workflows/.*\.yml$'; then
    workflow_changed=1
  fi
fi

if [ "$workflow_changed" -eq 1 ]; then
  if ! command -v actionlint >/dev/null 2>&1; then
    echo "FAIL: workflow files changed and actionlint is not on PATH" >&2
    exit 1
  fi
  actionlint
fi
```

`chmod +x scripts/maintenance/run-gates.sh`.

- [ ] **Step 2: Syntax-check and confirm it does not call network helpers or home stores**

Run:

```bash
chmod +x scripts/maintenance/run-gates.sh
sh -n scripts/maintenance/run-gates.sh
python3 - <<'PY'
from pathlib import Path
text = Path("scripts/maintenance/run-gates.sh").read_text()
for needle in ["fetch-cited", "probe-releases", "~/.codex", "~/.claude", "~/.grok", "git tag", "git push"]:
    assert needle not in text, needle
print("run-gates hygiene OK")
PY
```

Expected: `run-gates hygiene OK`.

Do not run the wrapper yet if you want a faster task cycle; Task 8 runs it as the implementation self-check. If you do run it here, it must be green before commit.

- [ ] **Step 3: Commit**

```bash
git add scripts/maintenance/run-gates.sh
git commit -m "feat: add maintainer run-gates wrapper"
```

---

### Task 7: Repo-local audit-and-release skill

**Files:**
- Create: `.grok/skills/audit-and-release/SKILL.md`
- Create: `.grok/skills/README.md`

**Interfaces:**
- Consumes: Task 3 checklist path; Task 5–6 script paths; spec §6.0 and §7.
- Produces: Grok-discoverable skill `name: audit-and-release` invoked as `/audit-and-release`. No copy under `~/.grok/skills/` or `.claude/skills/`.

- [ ] **Step 1: Write `.grok/skills/README.md`**

Exactly five lines of substance (blank line after the title is fine):

```markdown
# Project-local maintainer skills

These skills are for maintainers working in this clone. They are not
product surface and are not invoked by `harness-guard` scans.

`audit-and-release` is the only skill in this directory.
```

- [ ] **Step 2: Write `.grok/skills/audit-and-release/SKILL.md`**

This is an agent prompt, not a second runbook. One home per fact: phases, file lists, and the STOP packet stay in `docs/maintenance/audit-and-release-loop.md`.

```markdown
---
name: audit-and-release
description: >
  Runs the maintainer audit-and-release loop (re-verify cited evidence,
  update fixtures/freshness/docs, run gates, prepare CHANGELOG and version
  bumps) and hard-stops before push, annotated tag, or GitHub Release.
  Use when the user asks to audit, re-verify, run a freshness pass, bump
  the ruleset, prepare a release, tag, or create a GitHub Release, or runs
  /audit-and-release. Never pushes, tags, or publishes without a fresh
  explicit owner go for that exact action.
---

# Audit-and-release

You are running the maintainer loop for Harness Guard, a local,
execution-free, per-finding-cited config auditor.

## Read first, then obey

Read `AGENTS.md`, `CONTRIBUTING.md`, `CONTEXT.md`, then
`docs/maintenance/audit-and-release-loop.md`. Obey those over improvisation.
The checklist owns phases A–H, file lists, and the STOP packet. This skill
owns invocation, authorization, forbidden commands, and the report format.

## Session authorization

Invoking this skill, `/audit-and-release`, or an explicit “run the
audit-and-release loop” request authorizes **this session** to retrieve
vendor docs via the maintainer scripts, certify existing shipped rules
after live primary-source re-verification, update `rules/`, synthetic
fixtures / goldens, `freshness/`, Grok evidence packs, docs, CHANGELOG,
and version pins in the working tree, run full gates, and write the STOP
packet.

It does **not** authorize last-mile publish, new rules/harnesses, `--fix`,
network-in-product, setting `ENABLE_FRESHNESS_WORKFLOWS`, moving a tag, or
a public cadence sentence.

Default-forbidden command prefixes (do not run):

- `git push`
- `git tag`
- `gh release`
- `gh repo`
- `cargo publish`
- `npm publish`
- any write to GitHub repository variables or settings

After a fresh owner sentence that names **one** of those exact actions,
that one command is allowed once; then the prefix list applies again.
“Ship it” / “finish the release” does not lift the list. Tag go and
Release go are never the same sentence.

## Branch

Require a clean branch `audit/YYYY-MM-DD` (UTC date of this session) from
current `main`. Never amend a published tag. Never inspect ambient `HOME`
harness stores (`~/.codex`, `~/.claude`, `~/.grok`, `CODEX_HOME`).
Synthetic fixtures only.

## Walk phases A–H

Follow `docs/maintenance/audit-and-release-loop.md` in order.

- Default is a **full pass** (A–H).
- Skip a phase only when its precondition is already satisfied **and** the
  report says so. Example: probe-releases shows no version movement **and**
  fetch-cited shows no hash drift → still refresh `retrieved` /
  `verified_on` only if the owner asked for a full re-verify; a
  “probe-only” run may stop after B with a no-op report.
- A narrower owner task (“probe only”, “docs sweep only”) must refuse to
  bump CalVer or prepare a tag unless the owner expands the task.
- Use `scripts/freshness/probe-releases.sh` and
  `scripts/freshness/fetch-cited.sh` rather than ad-hoc curl that hashes
  differently from `scripts/freshness/normalize.sh`.
- Use `scripts/maintenance/run-gates.sh` for Phase G.
- Held 2026-08-13 candidates in `CONTEXT.md` stay out of scope. Do not
  ship a new rule or harness.

## Report shape (then STOP)

End with the CONTRIBUTING report:

- changed files
- test evidence (which gates ran, exit status)
- unresolved risk
- the Phase H STOP packet from the checklist (concrete branch, SHA,
  tags that must not move, proposed tag, Release recommendation,
  CHANGELOG excerpt, risk paragraph, exact unexecuted commands,
  explicit non-actions)

Do not claim a public verification cadence in that report either.
```

Do not list the words `weekly`, `daily re-verification`, `continuously verified`, or `always up to date` in the skill (even as a ban list); say “public verification cadence” instead so a future tripwire on this file would stay green.

- [ ] **Step 3: Confirm layout, no duplicate skill homes, no last-mile as an action**

Run:

```bash
test -f .grok/skills/audit-and-release/SKILL.md
test ! -e .claude/skills/audit-and-release/SKILL.md
test ! -e CLAUDE.md
python3 - <<'PY'
from pathlib import Path
text = Path(".grok/skills/audit-and-release/SKILL.md").read_text()
assert text.startswith("---\nname: audit-and-release\n")
assert "/audit-and-release" in text
for needle in ["git push", "git tag", "gh release", "cargo publish", "npm publish"]:
    assert needle in text, needle
# Must not instruct enabling freshness workflows:
lower = text.lower()
assert "enable_freshness_workflows" in lower
assert "does **not** authorize" in text or "does not authorize" in lower
for bad in ["set ENABLE_FRESHNESS_WORKFLOWS=true", "enable ENABLE_FRESHNESS"]:
    assert bad.lower() not in lower
for cadence in ["weekly", "daily re-verification", "continuously verified", "always up to date"]:
    assert cadence not in lower, cadence
print("skill checks OK")
PY
```

Expected: `skill checks OK`.

- [ ] **Step 4: Commit**

```bash
git add .grok/skills/README.md .grok/skills/audit-and-release/SKILL.md
git commit -m "feat: add repo-local audit-and-release skill"
```

`.grok/` is not gitignored (`.gitignore` only has `/target`, `**/.DS_Store`, and worktree dirs). The skill must be committed.

---

### Task 8: Implementation self-check and landing notes

**Files:**
- Modify: `CHANGELOG.md` (`## [Unreleased]` only)
- Modify: `notes/session-history.md` (append-only)
- Test: `scripts/maintenance/run-gates.sh` plus the grep checks below

**Interfaces:**
- Consumes: every artifact from Tasks 1–7.
- Produces: a tree that meets spec §13 and stops before last-mile.

- [ ] **Step 1: Add the Unreleased changelog note**

Under the existing `## [Unreleased]` heading in `CHANGELOG.md`, add:

```markdown
### Added
- Maintainer audit-and-release loop: `docs/maintenance/audit-and-release-loop.md`,
  `.grok/skills/audit-and-release/SKILL.md`, and maintainer helpers
  `scripts/freshness/probe-releases.sh`, `scripts/freshness/fetch-cited.sh`,
  `scripts/maintenance/run-gates.sh`. Hard-stops before push, annotated tag,
  or GitHub Release. Scan behavior, shipped rules, and freshness workflow
  enablement are unchanged.
```

Do not bump `0.0.2`. Do not add a dated `## [0.0.3]` section.

- [ ] **Step 2: Append session history**

Append to `notes/session-history.md`:

```markdown
## 2026-08-13 — audit-and-release loop landed

- Added the repeatable maintainer loop (checklist + repo-local skill +
  probe/fetch/gate scripts + tripwires). `SECURITY.md` now names tagged
  preview `0.0.2` as the supported line.
- Did not bump binary `0.0.2` or ruleset `2026.08.13`. Did not ship held
  2026-08-13 candidate rules. Did not enable `ENABLE_FRESHNESS_WORKFLOWS`.
- Last-mile (push / tag / GitHub Release) still needs a fresh explicit
  owner go per action. Do not move `0.0.1` or `0.0.2`.
```

- [ ] **Step 3: Acceptance greps (spec §13)**

Run:

```bash
# 1. Skill + checklist exist and skill forbids last-mile
test -f docs/maintenance/audit-and-release-loop.md
test -f .grok/skills/audit-and-release/SKILL.md
grep -q 'git push' .grok/skills/audit-and-release/SKILL.md
grep -q 'git tag' .grok/skills/audit-and-release/SKILL.md
grep -q 'gh release' .grok/skills/audit-and-release/SKILL.md

# 2. Runbook triage flow still present; loop is a sibling
grep -q 'Triage flow (drift or release detected)' docs/maintenance/runbook.md
grep -q 'Audit-and-release loop' docs/maintenance/runbook.md
grep -q 'authored, not enabled' docs/maintenance/runbook.md

# 3. Network scripts exist, share helpers, unused by product/CI
test -x scripts/freshness/probe-releases.sh
test -x scripts/freshness/fetch-cited.sh
grep -q 'extract-urls.sh' scripts/freshness/fetch-cited.sh
grep -q 'normalize.sh' scripts/freshness/fetch-cited.sh
! grep -E 'fetch-cited|probe-releases' .github/workflows/ci.yml

# 4. Gates wrapper
test -x scripts/maintenance/run-gates.sh
grep -q 'cargo fmt --all -- --check' scripts/maintenance/run-gates.sh
grep -q 'scripts/no-egress/run-macos.sh' scripts/maintenance/run-gates.sh
grep -q 'actionlint' scripts/maintenance/run-gates.sh

# 5. No root CLAUDE.md; constitution points at the skill
test ! -e CLAUDE.md
grep -q '.grok/skills/audit-and-release/SKILL.md' AGENTS.md
grep -q '.grok/skills/audit-and-release/SKILL.md' CONTEXT.md
grep -q '.grok/skills/audit-and-release/SKILL.md' CONTRIBUTING.md

# 6–9. Cadence / supported-version / no enable-as-action
python3 - <<'PY'
from pathlib import Path
cadence = ["weekly", "daily re-verification", "continuously verified", "always up to date"]
for rel in ["README.md", "docs/agent-guide.md", "SECURITY.md"]:
    text = Path(rel).read_text().lower()
    for n in cadence:
        assert n not in text, f"{rel}: {n}"
sec = Path("SECURITY.md").read_text().lower()
assert "until versioned releases exist" not in sec
assert "0.0.2" in Path("SECURITY.md").read_text()
# New loop files must not treat enabling freshness workflows as an action.
for rel in [
    "docs/maintenance/audit-and-release-loop.md",
    ".grok/skills/audit-and-release/SKILL.md",
]:
    text = Path(rel).read_text()
    assert "ENABLE_FRESHNESS_WORKFLOWS" in text
    assert "set ENABLE_FRESHNESS_WORKFLOWS=true" not in text
    assert "ENABLE_FRESHNESS_WORKFLOWS=true" not in text or "do not" in text.lower()
print("acceptance greps OK")
PY

# 10. Held candidates still unshipped; no new rule files in this branch
git diff --name-only origin/main...HEAD | grep -v '^rules/' || true
test "$(git diff --name-only origin/main...HEAD | grep -c '^rules/' || true)" -eq 0
```

Expected: `acceptance greps OK` and zero paths under `rules/` in the branch diff.

- [ ] **Step 4: Run the required gates via the new wrapper**

Run: `scripts/maintenance/run-gates.sh`

Expected: exit 0. On macOS this includes `scripts/no-egress/run-macos.sh`. This plan does not change workflows, so `actionlint` should not run. If the wrapper incorrectly fires `actionlint`, fix the diff detection before committing.

- [ ] **Step 5: Commit and STOP**

```bash
git add CHANGELOG.md notes/session-history.md
git commit -m "docs: record audit-and-release loop landing under Unreleased"
```

Print a STOP packet for **this** implementation branch (do not push):

- Branch name and `git rev-parse HEAD`
- Existing tags that must not move (`0.0.1`, `0.0.2`, plus `git tag -l`)
- Proposed tag: **none** (docs/scripts/skill landing; no binary bump unless the owner asks)
- GitHub Release: **not recommended** for this landing
- CHANGELOG excerpt: the `[Unreleased]` body added above
- Risk: loop is not yet proven on a live full pass; network scripts unexercised by CI; Grok still local-posture / no lab; held 2026-08-13 candidates remain unshipped
- Commands **not executed**: `git push -u origin <this-branch>` only after an owner go that names this branch

Do not `git push`. Do not `git tag`. Do not `gh release`. Do not `cargo publish`.

---

## Spec coverage (self-review)

| Spec section | Task |
| --- | --- |
| §1 goal / owner decisions 1–8 | Global Constraints + Tasks 3, 7, 8 |
| §2 in-scope items 1–7 | Tasks 3–8 |
| §2 not-in-scope | Global Constraints; Task 8 grep of `rules/` |
| §5 layout | File structure; Tasks 3, 5, 6, 7 |
| §6.0 authorization | Task 3 checklist + Task 7 skill |
| §6.1–§6.8 phases A–H + STOP | Task 3 (one home); Task 7 walks them |
| §7 skill home / frontmatter / body | Task 7 |
| §8.1–§8.3 scripts | Tasks 5–6 |
| §9 constitution + runbook pointer + skills README | Tasks 3, 4, 7 |
| §10 tripwires (README cadence, SECURITY sentence, no core/scan refs, script hygiene) | Tasks 1, 2, 5 |
| §10.3 run-gates self-check | Task 8 |
| §11 conservative defaults | Task 3 Phase C / H |
| §12 WP1–WP5 / no tag for this landing | task order; Task 8 STOP |
| §13 acceptance 1–11 | Task 8 greps + gates |
| §14 decision log | not re-opened |

No placeholders remain. No product scan I/O change. No workflow enablement.
