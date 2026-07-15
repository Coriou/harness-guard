# Maintenance runbook

## Scheduled workflows: authored, not enabled

`release-watch.yml` and `doc-drift.yml` exist in-tree but do not run: the
repository is local-only. Publishing the repository and enabling workflow
schedules are separate, user-triggered decisions.

GitHub automatically disables scheduled workflows after 60 days without
repository activity. To re-enable one: open the repository's **Actions** tab,
select the workflow, then select **Enable workflow**. Check this whenever
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

## Cadence claims

No public verification-cadence claim ("verified monthly", badges, and similar)
is made until the freshness pipeline has actually run on a schedule. This is a
hard rule from the product decision record.
