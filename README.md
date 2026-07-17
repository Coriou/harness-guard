# Harness Guard

[![CI](https://github.com/Coriou/harness-guard/actions/workflows/ci.yml/badge.svg)](https://github.com/Coriou/harness-guard/actions/workflows/ci.yml)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](./LICENSE)
[![Rust 1.85+](https://img.shields.io/badge/Rust-1.85%2B-orange.svg?logo=rust)](./Cargo.toml)

Harness Guard is a local, execution-free, per-finding-cited config auditor for
privacy, retention, and telemetry posture.

It reads a small allowlist of local configuration keys, explains every result,
and cites the exact documentation behind each finding. Scans make no network
requests, never execute the tools they discover, and never read source code,
session transcripts, shell history, `.env` files, or secrets.

> **Current scope:** three harnesses are supported —
> [Codex CLI](./rules/codex/) (4 bundled rules),
> [Claude Code](./rules/claude-code/) (5 bundled rules), and
> [Grok Build](./rules/grok-build/) (4 local-posture rules). See
> [Current scope and limits](#current-scope-and-limits) below.

## Quick start

Prerequisites: macOS or Linux, Git, and Rust 1.85 or newer.

```bash
git clone https://github.com/Coriou/harness-guard.git
cd harness-guard
cargo install --path crates/harness-guard-cli --locked

harness-guard scan
```

Exit code `1` means a finding was reported—not that the scan crashed. A typical
result looks like this:

```text
detected tools
  ● codex 0.144.5 · config ~/.codex/config.toml · confidence high

!! WARNING: The inspected user-level config explicitly enables local history persistence (history.persistence = "save-all").
   observed: history.persistence = "save-all"
   fix: Add to CODEX_HOME/config.toml (normally ~/.codex/config.toml):
        [history]
        persistence = "none"
   = source: https://learn.chatgpt.com/docs/config-file/config-reference (2026-07-16)
```

No numeric score is produced. Read findings individually; `unknown` and
`stale-ruleset` results are deliberately not presented as passes.

### Use it with an agent

The JSON report is the safest handoff: it contains normalized observations and
citations, never raw config. Give your coding agent this prompt from the cloned
repository:

```text
Install Harness Guard with `cargo install --path crates/harness-guard-cli --locked`,
then run `harness-guard scan --json`. Preserve and inspect the JSON even when the
command exits 1. Summarize every finding, unknown, and stale-ruleset result with
its cited source. Do not read my config directly and do not change anything
without asking me first.
```

Exit `2` means the scan degraded; the JSON report still explains why.

See [docs/agent-guide.md](docs/agent-guide.md) for driving Harness Guard from an agent.

## Commands

```bash
# Audit detected supported tools
harness-guard scan

# Machine-readable, sanitized report
harness-guard scan --json

# Detection only—no rule evaluation
harness-guard list

# Full bundled evidence, hashes, archives, limitations, and tested versions
harness-guard explain codex-history-persist-01

# Machine-readable tool/rule/category inventory (the discovery entrypoint for agents)
harness-guard capabilities --json

# Binary and ruleset versions
harness-guard version

# Shell completions
harness-guard completions zsh > _harness-guard
```

Run `harness-guard --help` or `harness-guard scan --help` for filtering,
color, verbosity, and failure-threshold options.

### Exit codes

| Code | Meaning |
| ---: | --- |
| `0` | Scan completed with nothing at or above `--fail-on` |
| `1` | Scan completed and reported a finding at or above `--fail-on` |
| `2` | Usage error or degraded scan, such as unreadable/malformed config |

`unknown` and `stale-ruleset` do not fail by default. Use
`--fail-on never` when you want findings rendered without exit code `1`.

## Test it without using your config

All committed fixtures are synthetic. This exercises the complete fixture
matrix without touching your real Codex, Claude Code, or Grok Build config:

```bash
cargo test --workspace
```

To inspect a synthetic warning end to end, isolate `HOME` as well as
`CODEX_HOME`—otherwise the scan will also look for a real `~/.claude` or
`~/.grok` on your machine, since three harnesses are now detected, not one:

```bash
cargo build -p harness-guard-cli

env -i \
  HOME="$PWD/fixtures/codex/risky-explicit/files/codex-home" \
  CODEX_HOME="$PWD/fixtures/codex/risky-explicit/files/codex-home" \
  PATH="$PWD/fixtures/codex/risky-explicit/files/path:/usr/bin:/bin" \
  ./target/debug/harness-guard scan --color never
```

That command intentionally exits `1` because the fixture contains a warning.

## Safety model

During a scan, Harness Guard may read only, per detected harness:

- the harness's user-scope config file — `CODEX_HOME/config.toml`,
  `~/.claude/settings.json`, or `~/.grok/config.toml` (each bounded to 1 MiB;
  Grok also honors `GROK_HOME`), and
- a nearby version marker when one exists without executing the harness:
  npm `package.json` for Codex and Claude Code, and for Grok Build either
  npm `@xai-official/grok` or a managed-install symlink whose target basename
  encodes the version (each marker bounded to 64 KiB).

Reads refuse symlinks and non-regular files, use a pinned opened handle, and
discard unrelated/raw config data before reporting. Usernames, home paths, and
unrecognized values are redacted. Every report includes
`network_requests_made: 0`.

The no-egress claim is enforced in three layers: dependency bans, core lints,
and instrumented runtime proofs. On macOS, run:

```bash
scripts/no-egress/run-macos.sh
```

## Current scope and limits

| Tool | Rules | Categories | Status |
| --- | ---: | --- | --- |
| Codex CLI | 4 | retention, telemetry, transfer | Implemented |
| Claude Code | 5 | retention, telemetry | Implemented |
| Grok Build | 4 | telemetry, transfer | Implemented (local posture) |

Run `harness-guard capabilities --json` for the live, authoritative count
instead of relying on this table — it is regenerated from the same bundled
rule data and will not drift from what a scan actually evaluates.

- Grok Build rules describe **local config posture only** (telemetry master
  switch, feedback, session/trace upload sub-switch, external OTEL prompt
  log), cited from OSS primary sources for version `0.2.102`. They do not
  claim wire-level upload behavior.
- Only the user-level config for each harness is inspected. System config,
  selected profiles, trusted-project config, and CLI overrides are not
  inspected, so an unset user-level value is reported as `unknown`.
- Auth method and server-side policy are never inferred from local files.
- An unknown or untested harness version produces an unverified result, never
  a pass.
- Windows is not supported in this release because its filesystem traversal has
  not yet met the same race-resistant path-refusal invariant as macOS/Linux.
- Rules carry source URLs, retrieval dates, semantic hashes, archive links,
  explicit tested-version ranges, limitations, and unknown conditions.
- No public rule-verification cadence is claimed.

## Development and contributing

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo deny check
cargo test --workspace
```

See [CONTRIBUTING.md](./CONTRIBUTING.md) for the safe change workflow and
[AGENTS.md](./AGENTS.md) for agent-assisted contributions. Security issues
should follow [SECURITY.md](./SECURITY.md).

The machine-readable [`rules/`](./rules/) directory is also a standalone,
Apache-2.0 data package with its own README and license.

## Project status

This is a deliberately narrow v1 slice under maintainer review. It does not
apply fixes, read session content, contact vendors, or claim to prove remote
vendor behavior. Product decisions and the longer-term plan live in
[`docs/product/`](./docs/product/).

## License

Apache-2.0. See [LICENSE](./LICENSE).
