# Contributing to Harness Guard

Thanks for helping improve Harness Guard. The project is intentionally narrow:
it is a local, execution-free, per-finding-cited config auditor. Safety and
evidence quality are part of the product contract, not optional polish.

## Set up

You need Git, Rust 1.85 or newer with `rustfmt` and Clippy, and `cargo-deny`.
On macOS, the runtime no-egress test also uses the system `sandbox-exec` tool.

```bash
git clone https://github.com/Coriou/harness-guard.git
cd harness-guard
cargo build -p harness-guard-cli
cargo run -p harness-guard-cli -- --help
```

Do not test against your real Codex installation or config. All development
tests use the synthetic trees under `fixtures/`.

## Choose the right contribution

- Bug, UX, documentation, or test fix: open a focused pull request.
- Suspected vendor-doc or release drift: open an issue first with the official
  URL, the before/after claim, affected versions, and the date you checked it.
- New rule, new tool, networking, write/fix behavior, or output-contract
  expansion: get maintainer approval before implementation.

Never attach real config, prompts, transcripts, audited-project source code,
usernames, home paths, tokens, `.env` contents, shell history, or other secrets
to an issue or pull request.

## Safe change workflow

1. Read `CONTEXT.md`, this file, and the root `AGENTS.md`.
2. Create a focused branch and add a failing synthetic test or fixture first.
3. Make the smallest change that fixes it. Preserve crate boundaries: core
   receives explicit discovery roots; only the CLI resolves ambient paths;
   rules are consumed through their schema contract.
4. Confirm output contains only normalized, allowlisted observations and
   redacted paths. Never echo raw config or unsafe parse snippets.
5. Run the checks below and include the results in the pull request.

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo deny check
cargo test --workspace
```

On macOS, also run the instrumented proof:

```bash
scripts/no-egress/run-macos.sh
```

If a workflow changed, run `actionlint`. CI repeats formatting, Clippy,
dependency policy, tests on macOS/Linux/Windows, and macOS/Linux no-egress checks.

## Rules and evidence

Rule changes require more than a plausible config key:

1. Re-retrieve the relevant official primary source; never use quarantined
   legacy `data/` or old research reports as application ground truth.
2. Record the actual retrieval date, official URL, semantic-text hash produced
   by `scripts/freshness/normalize.sh`, and a Wayback snapshot when available.
3. Keep tested version ranges explicit. No matching range must degrade
   conservatively to `stale-ruleset` or `unknown`, never a confident pass.
4. Every non-unknown outcome must retain its source and retrieval date.
5. Add or update synthetic fixture goldens, including hostile and degradation
   cases. Raw unrecognized values must never appear in output.
6. Update the relevant `freshness/` state and bump `rules/ruleset.json` when a
   rule changes. Follow `docs/maintenance/runbook.md`.

Auth-method-dependent policy is user-confirmed or unknown; never infer it from
local artifacts. Automation may flag drift, but only a human certifies a rule.

## Working with a coding agent

Give the agent a clean branch or worktree, a bounded task, and this prompt:

```text
Read AGENTS.md, CONTRIBUTING.md, and CONTEXT.md first. Work only on <task>.
Use synthetic fixtures; never inspect ambient HOME, CODEX_HOME, ~/.codex, or
other sensitive stores. Preserve no-network/no-execution scan guarantees and
redaction. Add focused tests, run the repository validation commands, then
report the changed files, test evidence, and any unresolved risk. Do not add a
new rule, tool, dependency, workflow activation, or publishing action unless
the task explicitly authorizes it.
```

Review an agent's diff exactly as you would a human's. In particular, inspect
new fixture content, evidence provenance, diagnostic text, dependency changes,
and whether tests could touch ambient machine state.

## Issues and pull requests

Keep one concern per issue or pull request. Explain the user-visible behavior,
the safety/evidence impact, and how it was tested. Link the official source for
any vendor claim and state what remains unknown. Please do not claim a public
verification cadence: the freshness workflows are authored but default-off.

Contributions are Apache-2.0 licensed. There is no CLA.
