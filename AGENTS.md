# Instructions for coding agents

These instructions apply to the entire repository. Read `CONTEXT.md` and
`CONTRIBUTING.md` before changing code, rules, fixtures, workflows, or public
documentation.

## Product and scope

- Describe Harness Guard as a **local, execution-free, per-finding-cited config
  auditor**. Avoid broad category labels that frame it as agent-security
  scanning.
- Keep changes within the task the user or maintainer authorized. A new tool,
  rule, `--fix`/write behavior, network feature, database, output format, or
  public claim requires explicit approval before implementation.
- `rules/` is an independently usable Apache-2.0 data package. Consume it only
  through the schema contract; do not couple core code to rule internals.

## Non-negotiable safety rules

- Never inspect or ingest the developer's real `~/.codex`, ambient
  `CODEX_HOME`, other harness stores, source projects, prompt/session
  transcripts, shell history, `.env` files, credentials, or secrets.
- Every test and development scan must use an explicit synthetic root under
  `fixtures/` or a temporary directory derived from one. Do not run an ambient
  `harness-guard scan` as a test.
- Scan paths must make zero network requests. Do not add network-capable scan
  dependencies or APIs.
- Never execute anything discovered: no harness binary, package shim, MCP
  server, skill, plugin, hook, shell command, or config-provided command.
- Reads must stay bounded, regular-file-only, symlink/reparse-point refusing,
  depth-bounded, and resistant to path swaps. Extract only rule-relevant keys
  and discard raw input immediately.
- Reports and diagnostics must redact usernames and home paths, withhold raw
  config values, and omit source snippets. Render observations only through a
  rule's allowlist.

## Evidence and conservative behavior

- Retrieve vendor evidence freshly from official primary sources. Never use
  `data/` or legacy research artifacts as rule inputs.
- For each source, preserve its actual retrieval date, semantic-text hash, and
  archived URL when available. Non-unknown outcomes require a source URL and
  retrieval date structurally.
- Keep tested version ranges explicit. If no range matches, degrade to
  `stale-ruleset`/`unknown`; never infer support or emit a confident pass.
- Treat authentication-method-dependent policy as user-confirmed or unknown,
  never inferred.
- Use synthetic fixtures for every rule branch, hostile input, redaction case,
  and conservative degradation path. Never copy a real config into a fixture.
- Rule changes must update fixture goldens, relevant `freshness/` state, and
  the CalVer ruleset version. Follow `docs/maintenance/runbook.md`.

## Architecture

- `harness-guard-core`: explicit `DiscoveryRoot` only; no ambient environment,
  home lookup, process spawning, or network APIs.
- `harness-guard-rules`: schema-mirroring types, validation, and bundled rule
  loading.
- `harness-guard-cli`: argv, environment/home resolution, and rendering. JSON
  and terminal output must use the same validated report structs.

## Required validation

Run these before handing off a code, rule, fixture, or schema change:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo deny check
cargo test --workspace
```

On macOS also run:

```bash
scripts/no-egress/run-macos.sh
```

Run `actionlint` after changing `.github/workflows/*.yml`. Add focused tests for
the change; do not weaken a safety gate or golden merely to make it pass.

## External actions

- Freshness automation is triage-only and default-off. Do not enable scheduled
  workflows or set `ENABLE_FRESHNESS_WORKFLOWS` without explicit user approval.
- Do not publish packages, push branches, create releases, make a repository
  public, or change repository settings unless the user explicitly requests
  that exact external action.
- Never make a public verification-cadence claim until the pipeline has
  demonstrably run on schedule.
