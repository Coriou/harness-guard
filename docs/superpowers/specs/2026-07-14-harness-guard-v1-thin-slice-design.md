# Harness Guard v1 Thin Slice — Design Spec

**Date:** 2026-07-14
**Status:** Approved decision pack → spec. Inputs: `CONTEXT.md`, `docs/product/decision-and-strategy.md`, `docs/product/implementation-plan.md`, `docs/research/verification-audit-2026-07-13.md`, `docs/research/synthesis-2026-07-14.md` (binding), `docs/research/cli-ux-research-2026-07-14.md`, `docs/research/maintainability-strategy-2026-07-14.md`.
**Positioning (binding, verbatim):** Harness Guard is a *local, execution-free, per-finding-cited config auditor for privacy/retention/telemetry posture*. It is never described as an "AI agent security scanner" in any user-facing text, including `--help`, README fragments, and rule prose.

## 1. Goal

Prove the evidence schema, output contract, and safety boundaries end-to-end at minimum cost of change: Phase 0 schemas + a Rust CLI workspace + safe Codex CLI config discovery/parsing against synthetic fixtures + **one** source-cited rule (history persistence) evaluated and rendered to terminal and JSON + an instrumented no-egress proof + hostile-input tests, plus the automated freshness pipeline **authored** as workflow files (runs deferred). The slice ends with a human review gate before any new rules or tools.

## 2. Scope

**In scope (exactly, no more):**

1. JSON Schemas for: source, rule, finding/sanitized-report, fixture (§5).
2. Rust workspace: `crates/harness-guard-core`, `crates/harness-guard-rules`, `crates/harness-guard-cli`, plus top-level `rules/`, `fixtures/`, `schemas/`, `freshness/` (§4).
3. Safe Codex CLI config discovery and TOML parsing against synthetic fixtures only (§9).
4. One rule end-to-end: `codex-history-persist-01` (§8).
5. CLI: `scan`, `list`, `explain <rule-id>`, `version`; exit codes 0/1/2 (§6).
6. Terminal + JSON rendering from the same structs (§7).
7. Instrumented no-egress proof + hostile-input test suite (§10).
8. Freshness pipeline workflow files + runbook, authored locally, **not enabled** (§11).
9. Prerequisite: initialize the git repository before any implementation work (CONTEXT.md requirement). First commit should capture the existing docs tree; implementation lands as subsequent commits.

**Explicitly not in this slice** (deferred, not authorized here): GUI/Tauri, any second tool or second rule before slice review, `--fix` or any write operation, `rules update` or any networking in the product, SQLite/history, SARIF export, rule signing/out-of-band ruleset updates (rules are compiled into the binary for now), enabling scheduled workflow runs, publishing the repository, and any public "verified monthly"-style cadence claim (blocked until the pipeline has actually run).

**Quarantined inputs:** `data/tools-comparison.json`, `data/audit-commands.yaml`, and `AI_CODING_TOOLS_PRIVACY_RESEARCH_REPORT.md` are legacy research artifacts. They are never read by code, never used as rule evidence, and never migrated implicitly. The new `freshness/` directory is deliberately separate from legacy `data/`.

## 3. Licensing (resolved)

Apache-2.0 for everything — code and the `rules/` data package — single license, no CLA. `rules/` carries its own `LICENSE` copy and a `README.md` stating the hard invariant: **`rules/` is an independently usable, forkable, permissively licensed data package from day one**, not a folder convention. Code must load rules only through the schema contract (no reaching into rule internals from `core`), so the data package remains consumable without the binary.

## 4. Repository and workspace layout

```text
harness-guard/
├── Cargo.toml                     # workspace, version 0.1.0, edition 2024, rust-version pinned
├── LICENSE                        # Apache-2.0
├── crates/
│   ├── harness-guard-core/        # discovery, safe reads, parsing, evaluation — no env, no network
│   ├── harness-guard-rules/       # schema types, rule loading + validation, bundled rules
│   └── harness-guard-cli/         # clap surface, terminal + JSON rendering, env/home resolution
├── rules/                         # data package: one JSON file per rule + ruleset.json (Apache-2.0)
│   ├── LICENSE
│   ├── README.md
│   ├── ruleset.json               # { "ruleset_version": "2026.MM.DD", "generated_note": ... }
│   └── codex/history-persist-01.json
├── schemas/                       # JSON Schema (draft 2020-12): source, rule, fixture, report
├── fixtures/                      # synthetic-only per-case trees + expected.json goldens
│   └── codex/<case>/{files/, expected.json}
├── freshness/                     # committed pipeline state: last-seen.json, url-hashes.json
├── scripts/freshness/             # cited-URL extraction, semantic-text normalize+hash helpers
├── .github/workflows/             # ci.yml, release-watch.yml, doc-drift.yml (authored, not enabled)
└── docs/maintenance/runbook.md
```

**Crate boundaries (hard):**

- `harness-guard-core` performs discovery, bounded reads, parsing, and rule evaluation. It takes an explicit `DiscoveryRoot` argument for every operation and is **forbidden** from `std::env`, home-directory resolution, process spawning, and all network APIs. Enforced by lint/dependency gates (§10), not convention.
- `harness-guard-rules` owns the Rust types mirroring the schemas, loads/validates rule JSON, and embeds `rules/` into the binary at compile time (`include_str!`-style). Signature verification is deferred until rules ship separately from the binary.
- `harness-guard-cli` is the only crate that touches the real environment (`CODEX_HOME`, home dir, TTY, argv) and the only crate with rendering dependencies. Terminal and JSON views serialize from the same report structs.

No `xtask` crate in this slice; freshness scripting is shell/jq under `scripts/freshness/`.

## 5. Phase 0 schemas

All four schemas live in `schemas/` as JSON Schema draft 2020-12, versioned via a `schema_version` field (start `"1.0"`). Rust types in `harness-guard-rules` mirror them; a test validates every file in `rules/` and `fixtures/` against the schemas, and validates a sample `--json` report against the report schema.

### 5.1 Source

```json
{
  "schema_version": "1.0",
  "url": "https://…",
  "publisher": "OpenAI",
  "title": "Codex configuration reference",
  "evidence_class": "official-documentation",
  "retrieved": "2026-07-XX",
  "content_hash": "sha256:…",
  "archived_url": null,
  "notes": null
}
```

- `evidence_class` enum (closed): `local-observation | official-documentation | official-policy | independent-reproduction | inference`.
- `content_hash` is the **semantic-text** hash (normalize HTML to text, strip nav/timestamps, then SHA-256) — the same normalization the doc-drift job uses, so citation anchors and drift tripwires share one definition (`scripts/freshness/normalize`).
- `archived_url` holds a Wayback snapshot URL when one exists.

### 5.2 Rule

One JSON file per rule, e.g. `rules/codex/history-persist-01.json`:

```json
{
  "schema_version": "1.0",
  "id": "codex-history-persist-01",
  "tool": "codex",
  "category": "retention",
  "title": "Session history persistence",
  "why_it_matters": "…plain-English, context-specific…",
  "os": ["macos", "linux", "windows"],
  "scopes": ["user"],
  "auth_prerequisites": null,
  "observation": {
    "file": "config.toml",
    "key": "history.persistence",
    "type": "enum",
    "allowed_render": ["<documented values>", "unset"]
  },
  "outcomes": [ { "when": "…", "status": "finding", "severity": "warning",
                  "confidence": "high", "message": "…", "remediation": {…} } ],
  "tested_versions": [ { "min": "<=0.144.4", "max": "0.144.4", "verified_on": "2026-07-XX" } ],
  "sources": [ { …Source… } ],
  "limitations": ["…"],
  "unknown_conditions": ["…"]
}
```

Structural constraints (schema-enforced, not review discipline — MDN `spec_url` pattern):

1. **Citations are structural.** Any outcome whose `status` is not `unknown` requires ≥1 source with non-empty `url` and `retrieved` (JSON Schema conditional). In Rust, a non-unknown outcome cannot be constructed without a `Source` — the type system repeats the guarantee.
2. **Tested version ranges are explicit.** `tested_versions` entries carry `min`/`max`/`verified_on`. The `min` string may use the MDN `<=` prefix (`"<=0.144.4"` = "confirmed at this version, possibly applicable earlier") instead of a fake-precise origin.
3. **Unknown is absence, never a sentinel.** No magic value flows through evaluation. A detected version is "verified" iff some `tested_versions` entry matches; no match ⇒ the result degrades per §5.4 (anti-precedent: vercel/next.js#92091, where an implicit "unknown ⇒ assume supported" fallthrough inverted the safety property).
4. `allowed_render` is the complete allowlist of value renderings; anything not in it is never printed (§7.3).
5. `limitations` and `unknown_conditions` are required, non-empty arrays.

### 5.3 Fixture

`fixtures/codex/<case>/` = `files/` (a synthetic `CODEX_HOME` tree) + `expected.json` (`{ "schema_version", "case", "description", "expected_report": <report subset> }`). Fixtures are **synthetic only** — never real user config, sessions, usernames, tokens, or paths. This is load-bearing on the dev machine, which has a real `~/.codex` (§10.3).

### 5.4 Finding / sanitized report

The report schema is simultaneously the `--json` output contract and the only persisted/emitted artifact shape — there is no separate internal report containing raw config. Top level:

```json
{
  "schema_version": "1.0",
  "harness_guard_version": "0.1.0",
  "ruleset_version": "2026.07.14",
  "scanned_at": "RFC3339 local",
  "network_requests_made": 0,
  "platform": { "os": "macos" },
  "tools": [ { "tool": "codex", "detected_version": "0.144.4",
               "config_paths": ["~/.codex/config.toml"], "detection_confidence": "high",
               "rules_last_verified_version": "0.144.4", "rules_verified_date": "2026-07-XX",
               "version_in_range": true, "findings": [ … ] } ],
  "summary": { "tools_scanned": 1, "warning": 1, "info": 0, "unknown": 0, "stale": 0, "passed": 3 }
}
```

Per-finding fields: `rule_id`, `status`, `severity`, `confidence`, `evidence_class`, `message`, `observation` (allowlisted rendering or null), `remediation` (`{summary, command}` or null), `source` (`{url, retrieved}` or null), `valid_from`/`valid_until`, `limitations`, `unknown_reason`, `verify_url`, `stale_reason`.

**Status model — the reconciled enum.** This resolves the divergence between the implementation plan's `pass|action|review|unknown|unsupported` and the UX/synthesis enum. The UX enum wins (it is newer and binding); the old values are folded in deliberately:

| `status` | Meaning | `severity` | `confidence` | Source required | Fails scan |
|---|---|---|---|---|---|
| `pass` | Rule applicable and verified for this version; observation matches the safe condition | `null` | non-null | yes | never |
| `finding` | Rule applicable and verified; observation warrants attention | `warning` \| `info` | non-null | yes | if ≥ `--fail-on` |
| `unknown` | The rule's question is not answerable from local files (account/server state), or a declared `unknown_condition` fired (unreadable file, symlink, unrecognized value) | `null` | `null` | no (`unknown_reason` required; `verify_url` when one exists) | never (by default) |
| `stale-ruleset` | Detected version matches no `tested_versions` entry, or version undetected. The rule is still evaluated best-effort and the indicative outcome appears **in the message**, phrased as unverified (`stale_reason` required) | `null` | `null` | yes (last-known rule's source, shown as unverified) | never (by default) |

- Old `action`/`review` become `finding` + the shape of `remediation`: a config-change command (action) vs. a manual verification pointer (review). No separate `kind` field in v1.
- Old `unsupported` splits: a per-rule version mismatch is `stale-ruleset`; "tool not supported at all" is a detection-level concern, not a finding status.
- **Degradation direction is conservative and fixture-tested:** out-of-range and undetected-version results are surfaced and flagged for attention — never silently treated as pass, never dropped, never asserted with confidence. Dedicated fixtures (§10.2) pin this default so it cannot silently invert.
- `stale-ruleset` is triggered only by version mismatch/absence in v1. Calendar age of the ruleset is shown via the ruleset date in the header, not a per-finding status; a "verified as of DATE" staleness badge is deferred until the freshness pipeline produces last-checked timestamps.
- No aggregate numeric score anywhere — counts and an ordered list only.

## 6. CLI surface

Binary: `harness-guard`. Subcommands (clap v4 derive, `noun verb` style):

- `harness-guard scan [--tool codex]... [--json] [--min-severity <info|warning>] [--fail-on <never|info|warning>] [--color <auto|always|never>] [--quiet] [--verbose]`
  Default: scan every detected supported tool (v1: Codex only; `--tool` accepts only implemented IDs, so others are a clap usage error).
- `harness-guard list` — detection-only: tool, detected version (or "version not detected"), redacted config path, detection confidence. No rule evaluation. Exit 0 (2 on internal error).
- `harness-guard explain <rule-id>` — full evidence record from bundled data (works offline): all sources with class/retrieved/hash/archive, tested ranges with `verified_on`, all outcomes, limitations, unknown conditions, why-it-matters. Unknown rule-id ⇒ exit 2 with a nearest-match suggestion.
- `harness-guard version` / `--version` — binary version and ruleset version (CalVer from `rules/ruleset.json`) reported separately; they will diverge once rules update independently.
- `--help` everywhere, examples-first; `clap_complete` shell completions.

**Flag semantics:**

- `--min-severity` (default `info`, i.e. show everything) filters the display of `finding`-status blocks only. It **never** hides `unknown` or `stale-ruleset` blocks — silent omission must not read as "verified safe" (anti-pattern: "we don't check this" and "passed" looking identical).
- `--fail-on` (default `warning`) counts only `status: finding` at/above the threshold. `info`, `unknown`, and `stale-ruleset` never fail a scan by default; a flag to fail on unknowns is deferred.
- `--quiet` suppresses header, detection block, and banners; keeps finding/unknown/stale blocks and the one-line summary.
- `--verbose` additionally itemizes `pass` results (default output shows passes only as a count).

**Exit codes** (ruff/ESLint 3-way, fixed set — no growth per condition):

- `0` — scan completed; nothing at/above `--fail-on`.
- `1` — findings at/above `--fail-on`.
- `2` — the tool itself failed or degraded: permission error, config present but not safely parseable, internal error, usage error. A discovered-but-unreadable/unparseable config produces **both** a rendered report (affected rules ⇒ `unknown` with reason) **and** exit 2 — the output stays useful while the exit code signals a degraded scan. Stale ruleset is a status, never an exit code.

**Flag-convention verification (resolves decision-pack discrepancy c):** verified 2026-07-14 against `cargo-audit` source (`rustsec/rustsec`, `cargo-audit/src/commands/audit.rs`): cargo-audit's threshold flag is actually `-D/--deny <warnings|unmaintained|unsound|yanked>` — advisory *kinds*, not severities — so it is **not** the precedent for our threshold flag; `--fail-on <severity>` follows grype. Our `--json`, `--quiet`, and `--color` spellings do match cargo-audit's. The "cargo-audit style" citation in the UX report applies to the ~6-line finding block with source+date, which stands.

## 7. Rendering

### 7.1 Terminal (default human view)

Adopt the UX report's mockup 1 structure as the starting spec (its versions/paths are fictional placeholders):

1. Header line: binary version · ruleset version · scan time (local) · "no network requests made".
2. Detection block: `●` detected (with version) / `○` not detected, one line per supported tool.
3. Per-tool section (tools alphabetical), header with detected version and a version-bound line: in range ⇒ `rules verified ≤X · DATE`; out of range/undetected ⇒ a hint-styled banner ("rules verified ≤X — you have Y, showing last-known rules as unverified"), never an error.
4. Section opens with a plain-word status line + counts by kind (warning · info · unknown · stale · passed).
5. Finding blocks, ~6 lines each (cargo-audit shape): status label + message; `rule <id> · <evidence-class>`; allowlisted observation; remediation command when applicable; compact citation `= source: <url> (<retrieved>)`; `= harness-guard explain <rule-id>` pointer. Citations, unknown-reasons, and version bounds appear in **default** output; only the full evidence record is behind `explain`.
6. `unknown` blocks use the `??` glyph and spell out *why* it is unknowable, including the reframe sentence for account/server state ("no local file can confirm it either way — that's not a limitation of this scan, it's what local-only means") plus an official verify link.
7. `stale-ruleset` blocks use the `~` glyph with label `UNVERIFIED (stale ruleset)` and state the indicative outcome as unverified.
8. Footer: totals · "0 network requests made" · the no-score line ("No numeric score is produced — read findings individually.").

**Color discipline:** exactly one strong color: red+bold marks the highest severity tier in use (v1: `warning`). `info` default weight, `??` cyan, `~` yellow-dim, pass green appears only in count lines. `anstream` + `owo-colors` + `colorchoice-clap` handle TTY/`NO_COLOR`/`--color`; never hand-roll TTY detection.

### 7.2 JSON

`--json` emits the §5.4 report — same structs, `serde::Serialize`, so terminal and JSON cannot drift. `status` is a first-class enum; `severity`/`confidence` are legitimately `null` per the §5.4 matrix. Output is deterministic: tools alphabetical, findings ordered by rule id — golden tests depend on this.

### 7.3 Redaction (applies to both views and all diagnostics)

- Home directories render as `~`; usernames never appear anywhere in output.
- Config values are only ever rendered through a rule's `allowed_render` allowlist (bools/enums). Unrecognized values are never echoed.
- Parse-failure diagnostics (miette, config-parse failures only — not the findings list) report line/column and structural key path but **never attach the raw file text as `source_code`** — no snippet may leak values. This deliberately forgoes miette's snippet rendering to preserve the invariant.

## 8. The slice rule: `codex-history-persist-01`

**Question:** does the local Codex CLI configuration persist session history to disk (`history.persistence` in `config.toml`), i.e., local retention posture — deliberately *not* the auth/data-policy question.

**Evidence protocol (binding):** there is no per-tool research note for Codex, and the legacy `data/` files are quarantined. During implementation, the maintainer retrieves the official Codex config reference (`learn.chatgpt.com/docs/config-file/config-reference`) **fresh**, records `retrieved` (the actual retrieval date), computes the semantic-text `content_hash`, fires a Wayback snapshot for `archived_url`, and transcribes the documented `history.persistence` values and default into `observation.allowed_render` and the outcomes. This spec intentionally does not pre-assert those values — placeholders like `<documented values>` in §5.2 are filled only from the fresh retrieval. Evidence classes: `official-documentation` for documented values/default, `local-observation` for the observed key.

**Outcomes** (exact messages/values finalized against the fresh retrieval):

- Persistence explicitly set to the documented no-persist value ⇒ `pass`.
- Persistence unset (documented default applies) or explicitly set to a persisting value ⇒ `finding`, severity `warning`, confidence `high`, with a why-it-matters (prompts/commands retained in plaintext on disk, no expiry) and a copy-pasteable remediation (set the documented no-persist value).
- `unknown_conditions`: config unreadable (permissions), config is a symlink/non-regular file, value outside `allowed_render` ("set to an unrecognized value — raw values are never displayed"), file exceeds parse bounds. Each yields `unknown` + `unknown_reason`.
- Detected version outside `tested_versions`, or undetected ⇒ `stale-ruleset` per §5.4.

**Limitations (required, in the rule):** project-level config is not inspected in this slice; cannot confirm any server-side/remote retention; auth method (ChatGPT sign-in vs API key) changes data-policy interpretation and is user-confirmed-or-unknown — never inferred by this rule (per verification audit: parse observable local controls individually; never reduce to a training opt-out boolean).

**Version anchor:** Codex `latest` = 0.144.4 as of 2026-07-14; `tested_versions` uses `"<=0.144.4"` unless the fresh retrieval justifies a real lower bound.

## 9. Codex discovery and parsing

- **Injected roots, no ambient state:** every core API takes `DiscoveryRoot { codex_home: PathBuf, path_dirs: Vec<PathBuf> }`. Only the CLI crate constructs it from the real environment: `CODEX_HOME` if set, else `<home>/.codex` (home via `directories`). Tests always pass fixture roots — core cannot read the real home even by accident.
- **Discovery:** `codex_home/config.toml` (user scope only in this slice). Missing dir/file ⇒ tool not detected / no config, handled gracefully (fixture-covered).
- **Version detection without execution (hard invariant — never run the tool):** best effort, read-only: locate a `codex` entry in `path_dirs`, resolve symlinks with bounded hops, and read the `version` field of the owning npm package's `package.json` if the resolved path sits inside a package directory. Not found or ambiguous ⇒ `detected_version: null`, `detection_confidence` lowered, results flagged `stale-ruleset` ("version not detected") — which conveniently exercises the stale path in fixtures.
- **Bounded, refusing reads:** `symlink_metadata` before open — only regular files; symlinked config is not followed (⇒ `unknown` with reason). Max file size 1 MiB; TOML nesting depth ≤ 32; exceeding either ⇒ treated as not safely parseable (⇒ report with `unknown` findings + exit 2). Non-UTF-8 or malformed TOML ⇒ miette structural diagnostic (§7.3) + same degradation. `toml` crate (serde-compatible) is added to the stack for this — the decision-pack crate list covered the CLI/rendering layer; a TOML parser is a necessity for Codex config.
- **Parsing extracts only rule-relevant keys** into typed `ConfigLayer` values; raw text and unrelated keys are dropped immediately and never stored in any report struct.

## 10. Safety invariants and test plan

### 10.1 No-egress proof (instrumented, three layers)

1. **Dependency gate:** `cargo-deny` config bans network-capable crates (reqwest, hyper, ureq, curl, tokio net features, …) from the entire workspace graph; runs in CI and locally.
2. **Lint gate:** clippy `disallowed-methods`/`disallowed-types` forbids `std::net`, `std::process::Command`, and `std::env` in `harness-guard-core` (env allowed only in the CLI crate).
3. **Runtime instrumented test:** on macOS (the dev machine — runnable now): execute `harness-guard scan` against fixtures under a `sandbox-exec` profile denying all network; assert normal completion and expected exit codes (any egress attempt would error visibly). For Linux CI (authored now, runs when CI is enabled): the same scan under `strace -f -e trace=network`, asserting zero socket/connect syscalls.

### 10.2 Fixture matrix (all synthetic; golden `expected.json` per case)

`missing` (no CODEX_HOME), `minimal` (empty config), `hardened` (no-persist ⇒ pass), `risky-unset`, `risky-explicit` (⇒ warning finding), `malformed-toml` (exit 2 + unknown), `unrecognized-value` (⇒ unknown, value never echoed), `symlink-config` (not followed ⇒ unknown), `oversized` (> 1 MiB ⇒ bounded refusal), `deep-nesting` (depth bound), `permission-denied` (perms set at test runtime, not committed), `unknown-version` (no version marker ⇒ stale-ruleset), `version-out-of-range` (version marker beyond tested range ⇒ stale-ruleset). The two stale cases plus `unrecognized-value` are the **conservative-degradation pins** required by §5.4 — they are the regression guard against the next.js#92091 failure mode.

### 10.3 Real-config protection

The dev machine has a real `~/.codex`. Defenses: core takes only explicit roots (§9); tests construct `DiscoveryRoot` exclusively from fixture paths and never from `HOME`/`CODEX_HOME`; a test asserts the fixture tree contains no absolute paths escaping the fixture dir. Nothing from a real machine is ever committed (fixture reviews check for usernames/paths/tokens).

### 10.4 Other verifications in the slice

- Schema validation tests: every `rules/` and `fixtures/` file validates; a generated report validates against the report schema; a rule with a non-unknown outcome missing a source **fails** schema validation (negative test — proves the structural citation constraint).
- Golden output tests: terminal (via snapshot) and JSON for each fixture; deterministic ordering per §7.2.
- Exit-code tests per §6.
- CI workflow (`ci.yml`, authored now): fmt, clippy (with the §10.1 lint config), cargo-deny, test matrix macOS + Linux + Windows (build+test; strace job Linux-only, sandbox-exec job macOS-only). Until CI is enabled, the same steps run locally via `cargo fmt/clippy/deny/test` + the sandbox script.

## 11. Freshness pipeline (gated deliverable — authored, not enabled)

Workflow files are written into `.github/workflows/` in the local repo now; **enabling scheduled runs and publishing the repository are deferred, user-triggered steps** — nothing in this slice assumes a public repo exists. Automation only ever opens triage issues; **bots never set verdicts** or edit rules.

1. **`release-watch.yml`** — weekly cron: GET `https://registry.npmjs.org/{pkg}` with `Accept: application/vnd.npm.install-v1+json` for `@anthropic-ai/claude-code` (watch `stable` — not `latest`/`next`), `@openai/codex` (watch `latest` only; ignore its ~16 other dist-tags; filter `-alpha.` if reading releases.atom), `@github/copilot` (watch `latest`, not `prerelease`). Diff against committed `freshness/last-seen.json`; on change, open a triage issue titled with tool + old/new version, linking the changelog/release notes (Codex: use the GitHub release body — its CHANGELOG.md is a stub).
2. **`doc-drift.yml`** — weekly cron over the cited URLs extracted from `rules/**/*.json` sources: `lychee` dead-link pass, then semantic-text content-hash pass (`scripts/freshness/normalize` — filter to semantic text *before* hashing, the same normalization as §5.1) diffed against `freshness/url-hashes.json`. On drift: fire a Wayback SPN2 snapshot and open a triage issue linking the rule's `archived_url` (old) and the new snapshot as evidence anchors. Known pitfalls, documented in the workflow comments: JS-rendered vendor docs may need a per-page Playwright fallback later (not default); CDN edge-caching causes occasional staggered detection.
3. **Fixture tripwire** — the §10.2 suite doubles as the second staleness signal: a rule silently failing to match config shape is stronger drift evidence than a doc hash.
4. **`docs/maintenance/runbook.md`** — records: GitHub scheduled workflows auto-disable after 60 days without repo activity (re-enable steps + cadence note); the triage flow (issue → human re-verification → rule edit with new `retrieved` date → ruleset version bump); the rule that no public verification-cadence claim is made until the pipeline has actually run.

## 12. Crate stack (resolved)

clap v4 (derive) + clap_complete; anstream + owo-colors + colorchoice-clap; serde + serde_json; **toml** (addition, §9); comfy-table (tabular sub-views only, e.g. `list`); directories (CLI crate only); time (RFC3339, `retrieved`/`verified_on` handling); miette **only** for config-parse failures under the §7.3 no-snippet constraint. Dev/CI tooling: cargo-deny, insta (or equivalent) for snapshots.

## 13. Acceptance criteria (slice review gate)

1. `git init` done; docs committed before implementation commits.
2. `cargo test` green: schema validation (incl. the negative citation test), full fixture matrix with goldens, exit codes, hostile inputs.
3. `harness-guard scan` on the `risky-unset` fixture renders the §7.1 shape with citation + retrieved date in default output; `--json` validates against `schemas/report`; both views come from the same structs.
4. `explain codex-history-persist-01` shows the full evidence record, including a real `retrieved` date and `content_hash` from the fresh retrieval, and an `archived_url`.
5. No-egress: cargo-deny + clippy gates pass; the sandbox-exec instrumented scan passes locally; the strace job is authored in CI.
6. `unknown` and `stale-ruleset` render as designed (reason + verify link; unverified banner) — visually distinct from both pass and failure.
7. Freshness workflows + runbook exist locally with committed `freshness/` state files; schedules not enabled; no cadence claim anywhere in user-facing text.
8. `rules/` stands alone: own LICENSE/README, loadable purely via the schema contract.
9. All user-facing text uses the binding positioning phrase; "AI agent security scanner" appears nowhere.
10. `notes/session-history.md` updated; slice presented for human review **before** any second rule or tool.

## 14. Resolved decision log (discrepancies from the decision pack)

- **(a) Status enum:** UX/synthesis enum adopted (`pass|finding|unknown|stale-ruleset`); `action`/`review` folded into `finding` + remediation shape; `unsupported` split into `stale-ruleset` (version) vs detection-level (tool). §5.4.
- **(b) `explain` argument:** `rule-id` (rustc `--explain` pattern), not finding-id. §6.
- **(c) cargo-audit flags:** verified 2026-07-14 from source; `--fail-on` is grype's convention (cargo-audit uses `-D/--deny` with advisory kinds); our `--json/--quiet/--color` spellings match cargo-audit. Flag set finalized as specified. §6.
