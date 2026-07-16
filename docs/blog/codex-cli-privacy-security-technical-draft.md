# Codex CLI privacy: local history, configuration, and data controls

Status: preliminary technical draft for later editorial and SEO work. This is
not a published post. Facts were checked on 2026-07-16 and are scoped to Codex
CLI 0.144.5 unless stated otherwise.

## Editorial metadata

Proposed SEO titles:

1. Codex CLI Privacy: Local History, Config, and Data Controls
2. What Codex CLI Stores Locally—and How to Disable History
3. Codex CLI `history.jsonl`: Defaults, Paths, and Safe Setup

Recommended slug: `codex-cli-privacy-local-history`

Meta description: Learn what Codex CLI stores in `history.jsonl`, how
`history.persistence` and `CODEX_HOME` work, and how to check local history
settings without exposing config.

Primary query: `Codex CLI privacy`

Secondary queries: `Codex CLI security`, `Codex CLI history`, `Codex
history.jsonl`, `disable Codex history`, `Codex CODEX_HOME`, `Codex
config.toml`, `what does Codex CLI store`, `Codex data retention`, and `secure
Codex CLI setup`.

## Answer first

Codex CLI stores local message history by default. The documented setting is
`history.persistence`, its default is `save-all`, and the documented history
file is `CODEX_HOME/history.jsonl`. `CODEX_HOME` defaults to `~/.codex`.
Setting the effective value to `none` prevents new writes by the Codex
message-history component in version 0.144.5.

That is a local-storage control. It does not establish what a request sends to
OpenAI, whether vendor systems collect or retain data, whether data is used for
training, or which workspace or API policies apply. Those remote questions
depend on at least the authentication route and applicable account or
organization controls, and can also depend on plan, project, endpoint, feature,
agreement, and geography.

## What Codex CLI stores locally

OpenAI's [advanced configuration documentation](https://learn.chatgpt.com/docs/config-file/config-advanced#history-persistence)
describes `history.jsonl` as local session-transcript storage. At the Codex CLI
0.144.5 release tag, the message-history implementation writes one JSON entry
per line and constructs the file path by joining the configured Codex home with
`history.jsonl`.

This file is not the whole Codex state model. OpenAI also documents config,
credentials when file-based credential storage is selected, sessions, logs,
caches, and SQLite-backed state under or alongside the Codex state root. The
exact set varies by configuration, feature, credential store, and Codex
surface. Disabling message-history persistence does not make all local Codex
state ephemeral.

### Exact setting and default

The configuration belongs under the `[history]` TOML table:

```toml
[history]
persistence = "save-all"
```

The documented values are:

- `save-all`: persist local message history. This is the documented default.
- `none`: do not persist new local message-history entries.

The [configuration reference](https://learn.chatgpt.com/docs/config-file/config-reference)
and tagged 0.144.5 configuration schema agree on the setting name, allowed
values, and default.

### Default and custom paths

The default state root is `~/.codex`, so the documented default history path is
`~/.codex/history.jsonl`. If `CODEX_HOME` is set, the history path instead
becomes:

```text
$CODEX_HOME/history.jsonl
```

The user configuration audited in this article follows the same root:

```text
$CODEX_HOME/config.toml
```

with `~/.codex/config.toml` as the normal default. Do not assume that a custom
`CODEX_HOME` lives below the user's home directory.

The examples here target Bash and Zsh on macOS and Linux. Harness Guard's
current binary does not support Windows because its filesystem traversal has
not yet met the same race-resistant path-refusal invariant. The OpenAI docs use
`~/.codex` as their general shorthand; this draft does not invent an expanded
Windows path.

## How to disable new local history writes

Add or update this value in the configuration layer you intend to control:

```toml
[history]
persistence = "none"
```

For the user layer, edit `CODEX_HOME/config.toml`, normally
`~/.codex/config.toml`, with a trusted local editor. Preserve all unrelated
settings. Do not print, paste, upload, or replace an existing config merely to
apply this example.

If the file already contains a `[history]` table, update that table rather than
adding a duplicate. If the file does not exist, create it with permissions
appropriate for user-private configuration and add the two lines above. The
OpenAI documentation establishes this setting for future writes; it does not
establish a safe universal procedure for deleting existing history. This draft
therefore provides no deletion command.

### Configuration precedence matters

OpenAI documents this precedence, from highest to lowest:

1. CLI overrides
2. trusted-project configuration
3. the selected profile
4. user configuration
5. system configuration
6. built-in defaults

Consequently, a user-file setting describes only that layer. A trusted-project
setting or one-off CLI override can determine a different effective value for a
particular invocation. An unset user file is not enough to conclude that the
built-in `save-all` default wins, because other layers may provide a value.
See OpenAI's [configuration precedence documentation](https://learn.chatgpt.com/docs/config-file/config-basic#configuration-precedence).

## What `history.max_bytes` does—and does not do

`history.max_bytes` is an optional positive byte-size limit for local
`history.jsonl`. In tagged Codex CLI 0.144.5 source, exceeding the configured
limit triggers compaction: older JSON lines are dropped and a retained tail is
rewritten toward a soft target. The newest entry is retained, including when a
single newest entry is larger than the configured limit.

It is not:

- a time-based retention period;
- a switch that disables history persistence;
- a guarantee that every file or backup is below a hard cap;
- a control for sessions, logs, caches, credentials, or other Codex state; or
- evidence about vendor-side deletion or retention.

Tagged source treats a zero value as no enforcement, while the public prose
does not foreground that edge case. Use positive values when relying on this
control and keep the Codex version scope explicit.

## Local storage is not remote collection

The `history.persistence` control governs a local write path. It is not
documented as a request-payload, telemetry, training, data-residency, or remote
retention control.

Keep four questions separate:

| Question | What this setting establishes |
| --- | --- |
| What is stored locally? | Whether the effective setting allows new writes to local `history.jsonl`. |
| What is transmitted? | Not established by this setting or by this research; no traffic capture was performed. |
| What does a vendor collect, train on, or retain? | Not established by local history configuration. |
| Which policy applies? | Depends on auth and account context; it must be confirmed rather than inferred from local files. |

OpenAI's [Codex authentication documentation](https://learn.chatgpt.com/docs/auth)
says local Codex can use ChatGPT sign-in or API-key sign-in. ChatGPT sign-in
follows the selected workspace's controls; API-key use follows the API
organization's controls. Do not infer either route by inspecting credentials.

For API-key use, OpenAI's [API data-controls documentation](https://developers.openai.com/api/docs/guides/your-data)
says API data is not used to train OpenAI models unless the customer explicitly
opts in. The same page separately describes abuse-monitoring logs, application
state, conditional retention controls, and endpoint- or feature-specific
behavior. Default abuse-monitoring retention can be up to 30 days, subject to
the page's legal and safety qualifications. Modified Abuse Monitoring and Zero
Data Retention require eligibility and approval. These API statements are not
a universal retention answer for ChatGPT-authenticated Codex, and this research
does not assign one API endpoint row to every Codex session.

## Verify the local posture without exposing config

Avoid commands that print `config.toml`, `auth.json`, `history.jsonl`, session
files, logs, or environment secrets. A useful review should report only the
specific normalized setting and its evidence.

[Harness Guard](https://github.com/Coriou/harness-guard) is an optional local,
execution-free, per-finding-cited config auditor for this check. It reads only
an allowlisted config key and bounded package-version evidence; a scan makes no
network requests and never executes Codex.

Run the terminal report:

```bash
harness-guard scan
```

Or preserve the sanitized JSON even when a finding produces exit code 1:

```bash
report_dir=$(mktemp -d "${TMPDIR:-/tmp}/harness-guard-report.XXXXXX") || exit 2
report_path="$report_dir/report.json"
if harness-guard scan --json > "$report_path"; then
  scan_status=0
else
  scan_status=$?
fi
printf 'Harness Guard exit code: %s\nSanitized report: %s\n' \
  "$scan_status" "$report_path"
```

The output deliberately normalizes observations to `none`, `save-all`, or
`unset`; it does not echo unrecognized raw values. It also treats later,
uncertified Codex versions conservatively as `stale-ruleset` rather than
silently passing them.

### Sanitized terminal example

This abbreviated synthetic example represents an explicit user-level
`save-all` value and exits 1:

```text
detected tools
  ● codex 0.144.5 · config ~/codex-home/config.toml · confidence high

!! WARNING: The inspected user-level config explicitly enables local history persistence (history.persistence = "save-all").
   observed: history.persistence = "save-all"
   fix: Add to CODEX_HOME/config.toml (normally ~/.codex/config.toml):
        [history]
        persistence = "none"
   = source: https://learn.chatgpt.com/docs/config-file/config-reference (2026-07-16)
```

### Sanitized JSON example

This abbreviated synthetic example represents an explicit user-level `none`
value and exits 0:

```json
{
  "schema_version": "1.0",
  "ruleset_version": "2026.07.16",
  "network_requests_made": 0,
  "tools": [
    {
      "tool": "codex",
      "detected_version": "0.144.5",
      "config_paths": ["~/codex-home/config.toml"],
      "version_in_range": true,
      "findings": [
        {
          "rule_id": "codex-history-persist-01",
          "status": "pass",
          "message": "The inspected user-level config sets history.persistence = \"none\".",
          "observation": "history.persistence = \"none\"",
          "source": {
            "url": "https://learn.chatgpt.com/docs/config-file/config-reference",
            "retrieved": "2026-07-16"
          }
        }
      ]
    }
  ]
}
```

The complete schema, fixtures, and rule evidence live in the public
[Harness Guard repository](https://github.com/Coriou/harness-guard/tree/main/rules).

### Exit codes for humans and agents

| Code | Interpretation | Required handling |
| ---: | --- | --- |
| `0` | Scan completed with no finding at or above the selected `--fail-on` threshold. | Still review `unknown` and `stale-ruleset`; they do not fail by default. |
| `1` | Scan completed and reported a finding at or above the threshold. | Preserve and review the report; this is not a crash. |
| `2` | Usage failed or the scan degraded, for example because config was unreadable or malformed. | Preserve any emitted report and diagnostic; do not treat it as a pass. |

Automation should parse `status`, `unknown_reason`, `stale_reason`, `source`,
and the summary instead of reducing the report to a score. The tool does not
produce a security score.

## Project and rule limitations

Harness Guard currently supports Codex CLI only. It does not support Claude
Code, Grok, or any other harness. Its only current rule inspects the user-level
`history.persistence` value on macOS and Linux for Codex versions certified
through 0.144.5.

The rule does not inspect:

- system, profile, trusted-project, or CLI configuration layers;
- `history.max_bytes`;
- existing history contents or any transcript;
- credentials or authentication method;
- sessions, SQLite state, logs, caches, or source trees; or
- network payloads, telemetry, vendor collection, training, or remote
  retention.

An explicit `none` can therefore be a cited pass about the inspected user
layer without proving the effective merged setting. An unset user-level value
is `unknown`. A version outside the certified range is `stale-ruleset`. None of
those results proves remote behavior or that Codex as a whole is secure.

## Troubleshooting

### The user config is unset, but Codex documents `save-all` as the default

The built-in default is only the lowest-precedence layer. A system setting,
selected profile, trusted-project setting, or CLI override may determine the
effective value. A user-file-only audit should report `unknown`, not infer the
merged result.

### The scanner reports `stale-ruleset`

The detected Codex version is missing or outside the human-certified range.
This is a conservative block, not evidence of a bad setting. Check the
installed version and wait for or perform fresh evidence certification before
treating that version as covered.

### The scanner refuses the path

Symlinks, non-regular files, unreadable files, oversized input, invalid UTF-8,
and excessive TOML nesting are refused rather than followed or guessed. Review
the symbolic path and file type without exposing its contents. Do not replace a
real config with a synthetic example.

### A custom `CODEX_HOME` is set

Apply and verify the user-layer setting under that root. Remediation should
refer to `CODEX_HOME/config.toml`, not assume `~/.codex/config.toml`. Do not
print the absolute custom path when sharing a report.

## FAQ

### Does Codex CLI save history by default?

Yes. For Codex CLI 0.144.5, `history.persistence` defaults to `save-all`.
Configuration precedence still determines the effective merged value.

### Where is `history.jsonl`?

It is documented at `CODEX_HOME/history.jsonl`. With the default Codex home,
that is `~/.codex/history.jsonl`.

### Does `persistence = "none"` delete existing history?

The reviewed primary sources establish that it stops new message-history
writes when effective. They do not establish deletion of an existing file, so
this draft does not provide a deletion command.

### Does disabling history change OpenAI retention or training?

No such conclusion follows. Local persistence and remote data policy are
separate. Confirm the applicable ChatGPT workspace or API organization controls
for the actual authentication route.

### Is `history.max_bytes` a retention period?

No. It is a byte-size compaction control, not a duration and not a remote
retention policy.

### Can I verify the setting by printing my config?

Do not share or dump the config. Use a trusted local editor for changes and a
normalized, allowlisted check for verification. Even then, remember that a
user-file audit cannot prove higher-precedence project or CLI values.

### Does this guidance apply to Claude Code or Grok?

No. This research and the current Harness Guard runtime cover Codex CLI only.

## Claim/source appendix

| Draft claim | Primary source | Evidence date and reproducibility |
| --- | --- | --- |
| `history.persistence` accepts `save-all` and `none`; default `save-all` | [Configuration reference](https://learn.chatgpt.com/docs/config-file/config-reference), [advanced configuration](https://learn.chatgpt.com/docs/config-file/config-advanced), tagged 0.144.5 types/schema | 2026-07-16; semantic hashes `de1707…185d`, `85675e…2a0f`; tagged hashes in research dossier C03 |
| `CODEX_HOME` defaults to `~/.codex`; local history is `CODEX_HOME/history.jsonl` | [Advanced configuration](https://learn.chatgpt.com/docs/config-file/config-advanced#config-and-state-locations), [environment variables](https://learn.chatgpt.com/docs/config-file/environment-variables), tagged 0.144.5 history source | 2026-07-16; semantic hashes `85675e…2a0f`, `0df124…a7e`; tagged hash in dossier C04 |
| Effective `none` prevents new message-history writes | [Advanced configuration](https://learn.chatgpt.com/docs/config-file/config-advanced#history-persistence), tagged 0.144.5 implementation | 2026-07-16; semantic hash `85675e…2a0f`; tagged hashes in dossier C06 |
| Positive `history.max_bytes` compacts older entries; it is not time retention | [Configuration reference](https://learn.chatgpt.com/docs/config-file/config-reference), [advanced configuration](https://learn.chatgpt.com/docs/config-file/config-advanced), tagged 0.144.5 implementation/tests | 2026-07-16; registered hashes in dossier C07 |
| Precedence is CLI, trusted project, profile, user, system, built-in | [Configuration basics](https://learn.chatgpt.com/docs/config-file/config-basic#configuration-precedence) | 2026-07-16; semantic hash `9823d3…165` |
| ChatGPT and API-key auth route to different control contexts | [Codex authentication](https://learn.chatgpt.com/docs/auth) | 2026-07-16; semantic hash `f48eae…8eb` |
| API data is not used for training unless opted in; retention controls are conditional | [OpenAI API data controls](https://developers.openai.com/api/docs/guides/your-data) | 2026-07-16; semantic hash `03d408…c07`; scope and qualifications in dossier C12 |
| Latest certified Codex version is 0.144.5 | [npm latest metadata](https://registry.npmjs.org/@openai%2Fcodex/latest), [official 0.144.5 release](https://github.com/openai/codex/releases/tag/rust-v0.144.5), tag commit `87db9bc18ba5bc82c1cb4e4381b44f693ee35623` | 2026-07-16; npm response hash `1fab33…94d`; full release evidence in dossier S07-S09 |
| Harness Guard covers only user-level Codex persistence on macOS/Linux through 0.144.5 | Final public rule, runtime, schemas, and synthetic fixtures | Pin to final public commit after private CI and public-clone validation |

The complete claim ledger, full hashes, evidence classes, limitations, search-
intent notes, and unresolved unknowns are in
`docs/blog/codex-cli-privacy-security-research.md`.
