# Maintenance runbook

## Scheduled workflows: authored, not enabled

`release-watch.yml` and `doc-drift.yml` exist in-tree but are default-off. Both
their scheduled and manual-dispatch jobs require the repository variable
`ENABLE_FRESHNESS_WORKFLOWS` to equal the lower-case string `true`. Publishing
the repository alone does not run either freshness job.

After the separate decision to publish, opt in at **Settings → Secrets and
variables → Actions → Variables → New repository variable**. Set the name to
`ENABLE_FRESHNESS_WORKFLOWS` and the value to `true`. To disable the jobs again,
change that value to `false` or delete the variable. The job-level guard treats
both scheduled events and manual dispatches as disabled when the variable is
absent or is not exactly lower-case `true`; when it is `true`, both event types
may run.

For public repositories, GitHub automatically disables scheduled workflows
after 60 days without repository activity. Re-enable the workflow itself at
**Actions → select workflow → Enable workflow**, then confirm the repository
variable remains intentionally set to `true`. Check this whenever public-repo
activity has lapsed; a private calendar reminder after 45 days is the cheapest
guard.

## Triage flow (drift or release detected)

1. Automation opens a triage issue (`release-watch` or `doc-drift`). Bots never
   set verdicts and never edit rules.
2. A human re-verifies the claim against the live official page and the linked
   Wayback snapshots (old = the rule's `archived_url`; new = the issue link).
3. If the rule needs changing, edit its JSON with a new `retrieved` date and
   `content_hash` (via `scripts/freshness/normalize.sh`), refresh
   `archived_url`, and update `tested_versions` with the re-verified range.
4. Update `freshness/url-hashes.json` and/or `freshness/last-seen.json`.
5. Bump `rules/ruleset.json` `ruleset_version` (CalVer, date of change).
6. Run the full test suite. Fixture goldens are the second staleness signal: a
   rule silently failing to match config shape is stronger drift evidence than
   a document hash.

## Grok Build channel notes

Grok Build's primary install channel is the CLI channel pointer
`https://x.ai/cli/stable` (and sibling `alpha` / `enterprise` pointers), not
npm-first. Evidence pack: `docs/research/evidence/grok-build/2026-07-17/`.

- `freshness/last-seen.json` records both the cli-pointer under `channels.grok-build`
  and the npm package `@xai-official/grok` under `packages` (same version when
  they agree).
- `release-watch.yml` probes the npm package only (dist-tag `latest`) because
  the channel pointer is not an npm registry object. When triage fires, also
  re-check `https://x.ai/cli/stable` and the OSS monorepo `SOURCE_REV` before
  widening `tested_versions`.
- Local-posture rules cite OSS user guide + telemetry types
  (`evidence_class: official-documentation`). Behavior claims still require a
  lab run per `docs/research/protocols/grok-build-cleanroom.md`.

## Cadence claims

No public verification-cadence claim ("verified monthly", badges, and similar)
is made until the freshness pipeline has actually run on a schedule. This is a
hard rule from the product decision record.
