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
