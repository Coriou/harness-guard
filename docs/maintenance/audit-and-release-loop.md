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
