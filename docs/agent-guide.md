# Driving Harness Guard from an agent

This is the deep reference for an agent (or the operator scripting one) that
drives the `harness-guard` binary as an external tool. If you are instead
contributing code, rules, or docs to this repository, read
[AGENTS.md](../AGENTS.md) instead — different audience, different rules.

Harness Guard is a **local, execution-free, per-finding-cited config auditor**
for the privacy/retention/telemetry posture of AI coding harness
configuration. It reads a small allowlist of local config keys, cites the
documentation behind every result, and never tells you a score — only
individually-sourced findings you can act on or hand to a human.

## Installing

There is no published package yet: 0.0.1 ships as source only, no crates.io
or npm distribution. An operator installs it once with

```bash
git clone https://github.com/Coriou/harness-guard.git
cd harness-guard && cargo install --path crates/harness-guard-cli --locked
```

Everything below this point is what an agent needs once `harness-guard` is on
`PATH`. None of it requires the repository to be present, checked out, or
readable — `capabilities`, `scan`, and `explain` are self-describing and
work offline from the installed binary alone.

## Step 1: discover what's audited — `capabilities --json`

Before invoking `scan`, run `capabilities --json`. It is offline, deterministic,
and makes no network requests — the right first call for an agent deciding
whether/how to invoke the tool, and the only supported discovery entrypoint
(don't hardcode rule counts or tool names; read them from here):

```json
{
  "schema_version": "1.0",
  "harness_guard_version": "0.1.0",
  "ruleset_version": "2026.07.17",
  "report_schema_version": "1.1",
  "tools": [
    { "tool": "claude-code", "rules": 5, "categories": ["retention", "telemetry"],
      "rules_last_verified_version": "2.1.204", "rules_verified_date": "2026-07-17" },
    { "tool": "codex", "rules": 4, "categories": ["retention", "telemetry", "transfer"],
      "rules_last_verified_version": "0.144.5", "rules_verified_date": "2026-07-16" },
    { "tool": "grok-build", "rules": 4, "categories": ["telemetry", "transfer"],
      "rules_last_verified_version": "0.2.102", "rules_verified_date": "2026-07-17" }
  ],
  "commands": ["scan", "list", "explain", "version", "capabilities", "completions"],
  "exit_codes": { "0": "no findings at/above --fail-on", "1": "findings at/above --fail-on", "2": "degraded or internal/usage error" }
}
```

Read this, don't assume it:

- `tools[].rules` is the number of bundled rules for that tool *right now*.
  `grok-build` ships four local-posture rules (telemetry master switch,
  feedback, session/trace upload sub-switch, external OTEL prompt log) cited
  from OSS primary sources for version `0.2.102` — not wire-level behavior.
- `harness_guard_version` (the binary) and `ruleset_version` (the rule data,
  CalVer) move independently. A `scan` can report stale findings on a
  perfectly current binary if the *rules* haven't been re-verified for a new
  harness release — see stale-ruleset below.
- `tools[].rules_last_verified_version` is the newest harness version any
  bundled rule has been checked against. A detected install newer than that
  is exactly the case that produces `stale-ruleset`, not a guessed pass.

## Step 2: run the scan — `scan --json`

`scan --json` emits a sanitized report against
[`schemas/report.schema.json`](../schemas/report.schema.json) (schema `1.1`).
Two things about the contract an agent should rely on:

- `tools[]` is ordered alphabetically by tool id, and `findings[]` within each
  tool by rule id. This ordering is contractual, not incidental — diff two
  reports directly, don't re-sort first.
- `network_requests_made` is always `0`. It exists so a report is
  self-certifying without an agent having to trust a claim outside the JSON.

A trimmed, real example (fixture data, redacted paths) — a finding, a benign
"config doesn't say" unknown, and the alphabetical tool ordering, side by side:

```json
{
  "schema_version": "1.1",
  "network_requests_made": 0,
  "tools": [
    {
      "tool": "claude-code",
      "config_paths": ["~/.claude/settings.json"],
      "findings": [
        { "rule_id": "claude-code-cleanup-period-01", "status": "finding",
          "severity": "warning", "observation": "cleanupPeriodDays = 365" },
        { "rule_id": "claude-code-telemetry-opt-out-01", "status": "unknown",
          "unknown_reason": "env.DISABLE_TELEMETRY is unset in the user-level config; …" }
      ]
    },
    {
      "tool": "codex",
      "config_paths": ["~/.codex/config.toml"],
      "findings": [
        { "rule_id": "codex-history-persist-01", "status": "finding",
          "severity": "warning", "observation": "history.persistence = \"save-all\"" }
      ]
    }
  ]
}
```

The human-readable form (`scan` with no flags) renders the same findings with
citations inline; run `harness-guard scan --help` for `--tool` (repeatable
filter), `--min-severity`, `--fail-on`, `--quiet`/`--verbose`, and `--color`.

## Exit codes: read them before the JSON

| Code | Meaning | What it does *not* mean |
| ---: | --- | --- |
| `0` | No findings at/above `--fail-on` (default: `warning`) | Not "everything passed" — `unknown`/`stale-ruleset` results can still be present at exit `0` |
| `1` | A finding at/above `--fail-on` was reported | The scan ran cleanly; treat this like a normal lint failure |
| `2` | The scan **degraded** — a config file was unreadable or unparseable — or a usage error occurred | **Not** a safety signal. Exit `2` means Harness Guard saw *less* than usual, not that the harness is *less safe*. Never read exit `2` as "worse" than `1` |

Degraded (exit `2`) always wins over any finding, and it fires only when a
config file itself could not be read or parsed — not merely because a value
was unset. Verified contrast, both against the same rule set:

```text
$ harness-guard scan --quiet     # config.toml has an unclosed [table]
harness_guard::config_parse
  × config not safely parseable: unclosed table, expected `]`
  help: line 1, column 9 in ~/codex-home/config.toml — fix the file and re-run;
        raw file content is never shown
$ echo $?
2
```

versus an absent/unset value, which is `unknown` but not degraded:

```text
$ harness-guard scan --quiet     # config.toml exists and parses; the key is just absent
?? UNKNOWN: Cannot determine history persistence posture: …
$ echo $?
0
```

If your agent's policy is "never fail the run, just collect findings," pass
`--fail-on never` — findings still render and the JSON is unchanged, only the
exit code stops mapping findings to `1`. Exit `2` still fires on genuine
degradation regardless of `--fail-on`.

## Reading finding status

Four statuses appear in `findings[].status`. Two of them are easy to
misread as failures — they are not:

- **`unknown`** — locally unknowable, *not* "unchecked" and *not* "safe".
  Either the relevant key is absent/unset in the one config layer Harness
  Guard inspects (uninspected system/profile/project/CLI layers, or the
  ambient shell environment, may set the effective value) — or the config
  itself was unreadable/unparseable (the degraded case above). Read
  `unknown_reason` to tell which; do not collapse this into "not a finding."
- **`stale-ruleset`** — the detected tool version falls outside every bundled
  rule's `tested_versions` range (see `version_in_range` on the tool
  object). This is the ruleset telling the truth about its own coverage
  instead of guessing a pass on an unverified version — it is unverified,
  not failing. `stale_reason` says why; `explain <rule-id>` gives the last
  known evaluation logic.
- **`pass`** / **`finding`** are conventional: the config layer inspected
  explicitly matches a documented safe or unsafe value, cited and sourced.

## Full evidence record — `explain <rule-id>`

For any rule id seen in a report, `harness-guard explain <rule-id>` (offline,
no flags needed) prints the complete evidence record: why it matters, every
outcome branch with its message and confidence, tested-version ranges,
primary sources with retrieval dates and content hashes, archived copies,
limitations, and every condition that produces `unknown`. This is the right
target when a report cites a rule id and an agent (or the human it's
reporting to) wants to verify the claim rather than take it on faith.

## What Harness Guard can never tell you

- **No network requests, ever**, during a scan — `network_requests_made` in
  every report proves it, it isn't a claim to take on faith.
- **Never executes anything it discovers** — no harness binary, shim, plugin,
  hook, or config-provided command. It only reads bounded, regular files.
  It cannot tell you what a tool *does*, only what its config *says*.
  Discovery version markers (`list`, the `detected_version` field) come from
  a package manifest, not from running the tool.
  **Do not build a workflow that shells out to a tool Harness Guard reports
  on based on that report** — that's outside what this tool observed.
- **Never emits raw config values.** Observations are rendered only through a
  rule's declared allowlist. An agent must not expect config file contents,
  paths outside the redacted `~`/`$TOKEN` form, or secrets in any report —
  asking for them is asking for something this tool structurally withholds.
- **No numeric score, anywhere.** Summaries are finding counts by status, not
  a composite risk number. Read findings individually.
- **No server-side or account-level state** — training opt-in, data-retention
  tier, Zero Data Retention agreements, and similar policy facts depend on
  the account and vendor backend, not on any local file, and are never
  inferred. Where a rule's outcome depends on one, its limitations/
  `verify_url` say so explicitly instead of guessing.

## A minimal agent loop

```text
1. harness-guard capabilities --json   → decide which --tool values exist and how many rules back them
2. harness-guard scan --json           → parse the report; branch on the process exit code
     exit 0 → no findings at/above --fail-on (still check summary.unknown/summary.stale)
     exit 1 → surface findings at/above --fail-on to the user, cited
     exit 2 → surface degraded tools separately from findings; don't conflate with "unsafe"
3. For any unknown/stale-ruleset entry the user asks about, run
   harness-guard explain <rule_id> for the full evidence record before
   claiming anything about it.
```
