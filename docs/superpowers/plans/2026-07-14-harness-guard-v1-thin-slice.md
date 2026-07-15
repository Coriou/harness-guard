# Harness Guard v1 Thin Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Spec:** `./docs/superpowers/specs/2026-07-14-harness-guard-v1-thin-slice-design.md` (read it once before starting; this plan implements it exactly — no scope growth).

**Goal:** Prove the evidence schema, output contract, and safety boundaries end-to-end: Phase 0 JSON Schemas, a 3-crate Rust workspace, safe Codex config discovery/parsing against synthetic fixtures, ONE rule (`codex-history-persist-01`), terminal+JSON rendering, an instrumented no-egress proof, hostile-input fixtures, and freshness workflows authored-but-not-enabled.

**Architecture:** `harness-guard-core` (discovery/reads/parsing/evaluation, takes injected `DiscoveryRoot`, forbidden from env/net/process by per-crate clippy gates) → `harness-guard-rules` (schema-mirroring types, embedded rule data, report structs) → `harness-guard-cli` (clap surface, env/home resolution, terminal+JSON rendering from the same structs). `rules/` is a standalone Apache-2.0 data package.

**Tech Stack:** Rust (edition 2024, rust-version 1.85), clap v4 derive + clap_complete, anstream + owo-colors + colorchoice-clap, serde/serde_json, toml (pinned 1.1.2, MSRV 1.85; never enable the `unbounded` feature), comfy-table, directories, time, miette (config-parse failures only). Dev: cargo-deny, insta, jsonschema (default-features=false), tempfile.

## Global Constraints

Every task implicitly includes these. Violating any one is a task failure.

- **No network requests in scans.** Three-layer proof: cargo-deny bans network crates workspace-graph-wide; clippy `disallowed-methods`/`disallowed-types` gates on `harness-guard-core`; runtime `sandbox-exec` (macOS, run now) / `strace` (Linux, authored in CI).
- **Never execute** any detected harness/MCP server/plugin/hook/command. **Never read** source code, transcripts, shell history, `.env`, or secret values. **Redact**: home dirs render as `~`; usernames never appear; config values render only via a rule's `allowed_render` allowlist; unrecognized values are never echoed.
- **Synthetic fixtures only.** Never commit or read real user config. The dev machine has a real `~/.codex` that must never be ingested — tests construct `DiscoveryRoot` exclusively from fixture paths.
- **Rule schema:** `source.url` + `retrieved` structurally required for any non-`unknown` outcome; explicit `tested_versions` ranges (MDN `<=` prefix allowed on `min`); unknown = absence of a matching version-range entry, degrading conservatively (fixture-pinned).
- **License:** Apache-2.0 everywhere, single license, no CLA. `rules/` carries its own LICENSE + README (standalone data package).
- **Scope:** exactly this slice. NOT authorized: GUI, second tool/rule, `--fix`, `rules update`/networking, SQLite, SARIF, enabling scheduled workflows, publishing the repo, any public verification-cadence claim.
- **Positioning (verbatim, everywhere user-facing):** "local, execution-free, per-finding-cited config auditor for privacy/retention/telemetry posture". The phrase "AI agent security scanner" appears NOWHERE.
- **git init is Task 1**; docs committed before implementation. Nothing assumes a public repo.
- **Crate stack is fixed** (§12 of spec): clap v4 derive, clap_complete, anstream, owo-colors, colorchoice-clap, serde, serde_json, toml (pinned `1.1.2` — verified on crates.io 2026-07-14; the `toml` crate has NO 0.22.x series, that series belongs to `toml_edit`), comfy-table, directories, time, miette (parse failures only). Do not add other runtime crates. (Version comparison is hand-rolled — no `semver` crate; see Task 8.)
- **CLI:** `scan`/`list`/`explain <rule-id>`/`version`; exit codes 0/1/2 ruff-style; `info`/`unknown`/`stale-ruleset` never fail by default; status enum `pass|finding|unknown|stale-ruleset`; no aggregate numeric score.
- **Codex evidence must be freshly retrieved and dated during implementation** (Task 5). Quarantined legacy `data/` files are never inputs.
- **Auth-method data policy (ChatGPT vs API key) is user-confirmed-or-unknown, never inferred.**

## Research findings folded in (already decision-ready — do not re-research)

1. **`history.persistence` values** (corroborated 2026-07-14): a `[history]` table with `persistence` key; exactly two documented values `"save-all"` and `"none"`; documented default when unset is `"save-all"` (persists transcripts to `history.jsonl` under `CODEX_HOME`). So: `"none"` ⇒ pass; unset or `"save-all"` ⇒ finding/warning/high. `allowed_render = ["save-all", "none", "unset"]`. Canonical doc location: `https://developers.openai.com/codex/config-reference` (the spec-cited `learn.chatgpt.com/docs/config-file/config-reference` is live and mirrors it). The `save-all` default is stated explicitly on the `developers.openai.com/codex/config-*` pages — anchor the default's citation there. Task 5 still performs the fresh retrieval for dates/hashes.
2. **Per-crate clippy scoping works natively:** a `clippy.toml` inside `crates/harness-guard-core/` applies only to that crate under a single workspace-wide `cargo clippy` (clippy config resolves via `CARGO_MANIFEST_DIR`, first-file-found, no merging). Ban lists live in core's `clippy.toml`; severity (`deny`) via `[lints.clippy]` in core's `Cargo.toml`. Do NOT create a workspace-root `clippy.toml`.
3. **toml crate — pin `1.1.2`** (all claims re-verified against crates.io + the 1.1.2 source, 2026-07-14). Correction: an earlier draft cited "toml >= 0.22.8" — that conflated `toml_edit`'s version series with `toml`; the `toml` crate has zero 0.22.x releases (its series runs 0.8.x → 0.9.x → 1.x, latest 1.1.2). 1.1.2's MSRV is exactly 1.85, matching our `rust-version`. On 1.1.2: the parser has a built-in recursion limit of 80 (`const LIMIT` in `src/de/parser/mod.rs`, enforced via `RecursionGuard`) that makes hostile deep nesting a parse `Error`, not a stack overflow — the guard is compiled out only under the `unbounded` feature, which exists on 1.1.2 and must NEVER be enabled. The crate limit (80) sits safely above our own depth ≤ 32 post-parse bound and the 40-deep fixtures, so for those it is OUR check that trips, and the 20k-deep hostile case hits the crate limit as backstop. `toml::de::Error` exposes `message()` and `span()` (byte range) — both confirmed present on 1.1.2; line/col computed by counting newlines; structural key path is NOT available from the error — we only attach a key path for extraction-stage issues where we know the key.
4. **Codex version detection:** only npm installs have a `package.json` (`<prefix>/bin/codex` → shim → walk up to `@openai/codex/package.json`, clean semver). Standalone installer and Homebrew have NO `package.json` ⇒ `detected_version: null` ⇒ `stale-ruleset` ("version not detected") — this is correct spec behavior, and it is what the dev machine (standalone 0.144.3) will produce. Trap: the platform sub-package (`@openai/codex-darwin-arm64`) has a suffixed version (`0.144.4-darwin-arm64`) — reject non-`X.Y.Z` versions. npm `latest` = 0.144.4 (matches spec anchor).

## File structure (locked in)

```text
harness-guard/
├── .gitignore
├── Cargo.toml                      # workspace
├── LICENSE                         # Apache-2.0
├── deny.toml                       # cargo-deny: network-crate bans + license allowlist
├── crates/
│   ├── harness-guard-core/
│   │   ├── Cargo.toml              # [lints.clippy] disallowed_* = "deny"
│   │   ├── clippy.toml             # the ban lists (per-crate scope)
│   │   └── src/
│   │       ├── lib.rs              # pub mod discovery; pub mod readfs; pub mod parse; pub mod version; pub mod evaluate; pub mod scan;
│   │       ├── discovery.rs        # DiscoveryRoot, config discovery
│   │       ├── readfs.rs           # bounded refusing reads
│   │       ├── parse.rs            # TOML parse, depth check, line/col, key extraction
│   │       ├── version.rs          # execution-free version detection + range match
│   │       ├── evaluate.rs         # rule evaluation → FindingRecord
│   │       └── scan.rs             # scan_codex orchestrator → ScanResult
│   ├── harness-guard-rules/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs              # pub mod schema; pub mod report; pub mod loader;
│   │       ├── schema.rs           # RawRule/Source/Observation/… mirror of schemas/
│   │       ├── report.rs           # Report/ToolReport/FindingRecord/Summary (§5.4)
│   │       └── loader.rs           # include_str! embed, ValidatedRule
│   └── harness-guard-cli/
│       ├── Cargo.toml
│       ├── src/
│       │   ├── main.rs             # clap surface, exit codes, env/root resolution
│       │   ├── redact.rs           # home → ~
│       │   ├── render_term.rs      # §7.1 terminal view
│       │   ├── render_json.rs      # --json (serde on the same structs)
│       │   ├── diagnostics.rs      # miette parse-failure diagnostic (no source_code)
│       │   └── explain.rs          # explain + nearest-match
│       └── tests/
│           ├── common/mod.rs       # fixture root helpers, run_bin helper, json subset assert
│           ├── scan_fixtures.rs    # exit codes + JSON goldens per fixture
│           ├── scan_snapshots.rs   # insta terminal snapshots
│           ├── cli_surface.rs      # list/explain/version/flags
│           └── hostile.rs          # runtime-constructed cases (symlink/oversized/permission)
├── rules/
│   ├── LICENSE
│   ├── README.md
│   ├── ruleset.json
│   └── codex/history-persist-01.json
├── schemas/
│   ├── source.schema.json
│   ├── rule.schema.json
│   ├── fixture.schema.json
│   └── report.schema.json
├── fixtures/codex/<case>/{files/, expected.json}    # 13 cases, Task 10
├── freshness/{last-seen.json, url-hashes.json}
├── scripts/
│   ├── freshness/{normalize.sh, extract-urls.sh}
│   └── no-egress/{scan.sb, run-macos.sh}
├── .github/workflows/{ci.yml, release-watch.yml, doc-drift.yml}
└── docs/maintenance/runbook.md
```

**Dependency direction:** cli → {core, rules}; core → rules; rules → (serde/serde_json/time only).

---

### Task 1: git init and docs-first commit

**Files:**
- Create: `./.gitignore`
- Repo root: `.`

**Interfaces:** Produces: a git repo whose first commit is the existing docs tree. Everything later is incremental commits.

- [ ] **Step 1: Initialize the repository**

```bash
cd .
git init -b main
```

- [ ] **Step 2: Write `.gitignore`**

```gitignore
/target
**/.DS_Store
```

Note: `Cargo.lock` is committed (workspace ships a binary). Fixtures are committed — never ignore `fixtures/`.

- [ ] **Step 3: Docs-first commit (CONTEXT.md requirement)**

Commit everything that exists today — docs, notes, README, CONTEXT.md, and the quarantined legacy `data/` + `AI_CODING_TOOLS_PRIVACY_RESEARCH_REPORT.md` (they are historical record; committing them is not "using" them — they remain quarantined inputs that no code may read).

```bash
git add -A
git commit -m "docs: initial commit of research, product docs, spec, and plan (pre-implementation)"
```

- [ ] **Step 4: Verify**

Run: `git log --oneline` → exactly one commit. `git status` → clean.

---

### Task 2: Workspace scaffold + Apache-2.0 license

**Files:**
- Create: `./Cargo.toml`
- Create: `./LICENSE`
- Create: `crates/harness-guard-core/{Cargo.toml, src/lib.rs}`
- Create: `crates/harness-guard-rules/{Cargo.toml, src/lib.rs}`
- Create: `crates/harness-guard-cli/{Cargo.toml, src/main.rs}`

**Interfaces:**
- Produces: a compiling 3-crate workspace; crate names `harness-guard-core`, `harness-guard-rules`, `harness-guard-cli`; binary name `harness-guard`. Workspace-level dependency versions all later tasks reuse.

- [ ] **Step 1: Root `Cargo.toml`**

```toml
[workspace]
resolver = "3"
members = [
    "crates/harness-guard-core",
    "crates/harness-guard-rules",
    "crates/harness-guard-cli",
]

[workspace.package]
version = "0.1.0"
edition = "2024"
rust-version = "1.85"
license = "Apache-2.0"

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
# Pinned 1.1.2 (verified on crates.io 2026-07-14; MSRV 1.85 = our rust-version).
# The 0.22.x series belongs to toml_edit, NOT toml — toml has no 0.22.x release.
# NEVER enable the `unbounded` feature: it compiles out the parser's recursion
# guard (default limit 80 on 1.1.2), our overflow backstop for hostile nesting.
toml = "1.1.2"
time = { version = "0.3", features = ["formatting", "parsing", "macros", "local-offset"] }
```

- [ ] **Step 2: `LICENSE`**

Copy the canonical Apache License 2.0 text (from `https://www.apache.org/licenses/LICENSE-2.0.txt` — dev-time retrieval, not product networking) into `LICENSE` verbatim. Do not fill in the appendix placeholder fields.

- [ ] **Step 3: Crate manifests and stubs**

`crates/harness-guard-rules/Cargo.toml`:

```toml
[package]
name = "harness-guard-rules"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true

[dependencies]
serde.workspace = true
serde_json.workspace = true
time.workspace = true
```

`crates/harness-guard-core/Cargo.toml`:

```toml
[package]
name = "harness-guard-core"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true

[dependencies]
harness-guard-rules = { path = "../harness-guard-rules" }
serde_json.workspace = true   # package.json version reads only
toml.workspace = true

[dev-dependencies]
tempfile = "3"

[lints.clippy]
disallowed_methods = "deny"
disallowed_types = "deny"
```

`crates/harness-guard-cli/Cargo.toml`:

```toml
[package]
name = "harness-guard-cli"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true

[[bin]]
name = "harness-guard"
path = "src/main.rs"

[dependencies]
harness-guard-core = { path = "../harness-guard-core" }
harness-guard-rules = { path = "../harness-guard-rules" }
clap = { version = "4", features = ["derive", "wrap_help"] }
clap_complete = "4"
anstream = "0.6"
owo-colors = "4"
colorchoice-clap = "1"
comfy-table = "7"
directories = "6"
miette = { version = "7", features = ["fancy-no-backtrace"] }
serde.workspace = true
serde_json.workspace = true
time.workspace = true

[dev-dependencies]
insta = { version = "1", features = ["filters"] }
tempfile = "3"
```

Each `src/lib.rs`: `//! <one-line crate purpose>` only. `src/main.rs`: `fn main() {}`.

- [ ] **Step 4: Verify it builds, then commit**

Run: `cargo build --workspace` → succeeds. `cargo run -p harness-guard-cli` → runs, no output.

```bash
git add -A && git commit -m "feat: scaffold 3-crate workspace (core/rules/cli), Apache-2.0"
```

---

### Task 3: No-egress gates — cargo-deny + per-crate clippy bans

**Files:**
- Create: `./deny.toml`
- Create: `crates/harness-guard-core/clippy.toml`

**Interfaces:**
- Produces: `cargo deny check` and `cargo clippy --workspace --all-targets -- -D warnings` as standing gates. Later tasks must keep both green.

- [ ] **Step 1: Install tooling (dev machine, one-time)**

Run: `cargo deny --version || cargo install cargo-deny --locked`

- [ ] **Step 2: Write `deny.toml`**

```toml
# Layer 1 of the no-egress proof (§10.1): no network-capable crate may enter
# the workspace dependency graph — including dev-dependencies.
[graph]
all-features = true

[licenses]
allow = [
    "Apache-2.0",
    "MIT",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "ISC",
    "Zlib",
    "Unicode-3.0",
    "CC0-1.0",
]
exceptions = [
    # `option-ext` is MPL-2.0 (transitive: directories → dirs-sys → option-ext;
    # verified via crates.io 2026-07-14). MPL-2.0 is file-level copyleft on the
    # dependency's OWN sources; consuming the crate unmodified as a transitive
    # library imposes no obligations on our Apache-2.0 code. The exception is
    # crate-pinned on purpose — MPL-2.0 stays OFF the general allowlist.
    { crate = "option-ext", allow = ["MPL-2.0"] },
]
# Policy: Harness Guard's own code and the rules/ data package are Apache-2.0
# only — no copyleft license may ever apply to project-authored code (spec
# guarantee: single license, Apache-2.0 everywhere for OUR code). For
# dependencies: a new transitive dep should carry an OSI-approved permissive
# license (extend `allow`); a file-level-copyleft transitive dep (MPL-2.0
# class, unmodified) requires a narrowly-scoped, crate-pinned entry in
# `exceptions` like the one above — never a blanket allowlist addition.
# Strong copyleft (GPL/AGPL/LGPL) is never accepted in any form.

[bans]
multiple-versions = "warn"
deny = [
    { crate = "reqwest" },
    { crate = "hyper" },
    { crate = "hyper-util" },
    { crate = "ureq" },
    { crate = "curl" },
    { crate = "curl-sys" },
    { crate = "isahc" },
    { crate = "attohttpc" },
    { crate = "native-tls" },
    { crate = "openssl" },
    { crate = "openssl-sys" },
    { crate = "rustls" },
    { crate = "tokio" },
    { crate = "async-std" },
    { crate = "smol" },
    { crate = "mio" },
    { crate = "socket2" },
    { crate = "libssh2-sys" },
]

[advisories]
ignore = []

[sources]
unknown-registry = "deny"
unknown-git = "deny"
```

- [ ] **Step 3: Write `crates/harness-guard-core/clippy.toml`**

Per-crate file — it applies ONLY to `harness-guard-core` under a workspace-wide `cargo clippy` (clippy resolves config from each package's `CARGO_MANIFEST_DIR`, first file found, no merge). Never add a workspace-root `clippy.toml` — it would be shadowed here and invite confusion.

```toml
# Layer 2 of the no-egress proof (§10.1): core must never touch ambient
# environment, spawn processes, or open sockets. Env/home resolution is
# allowed only in harness-guard-cli.
disallowed-methods = [
    "std::env::var",
    "std::env::var_os",
    "std::env::vars",
    "std::env::vars_os",
    "std::env::args",
    "std::env::args_os",
    "std::env::current_dir",
    "std::env::current_exe",
    "std::env::home_dir",
    "std::env::set_var",
    "std::env::remove_var",
    "std::process::Command::new",
]
disallowed-types = [
    "std::process::Command",
    "std::net::TcpStream",
    "std::net::TcpListener",
    "std::net::UdpSocket",
    "std::net::SocketAddr",
]
```

- [ ] **Step 4: Verify both gates pass, and verify both gates actually trip**

Run: `cargo deny check` → passes. The workspace graph already contains `directories → dirs-sys → option-ext` (Task 2), so a green licenses check proves the `option-ext` exception works. Negative verification (do not commit): temporarily delete the `exceptions` block from `deny.toml` and run `cargo deny check licenses` → FAILS on `option-ext` (MPL-2.0 not allowed). Restore the block. If the licenses check flags `option-ext` even with the exception present, fix the exception entry — do NOT add MPL-2.0 to `allow`.

Run: `cargo clippy --workspace --all-targets -- -D warnings` → passes.

Negative verification (do not commit this change): temporarily add to `crates/harness-guard-core/src/lib.rs`:

```rust
pub fn smoke() -> Option<String> { std::env::var("HOME").ok() }
```

Run: `cargo clippy -p harness-guard-core -- -D warnings` → FAILS with `disallowed_methods`. Then confirm the CLI crate is NOT affected: the same call in `harness-guard-cli` must NOT trip. Revert the temporary code.

- [ ] **Step 5: Commit**

```bash
git add deny.toml crates/harness-guard-core/clippy.toml
git commit -m "feat: no-egress gates — cargo-deny network bans + core-scoped clippy disallow lists"
```

---

### Task 4: Phase 0 JSON Schemas

**Files:**
- Create: `schemas/source.schema.json`
- Create: `schemas/rule.schema.json`
- Create: `schemas/fixture.schema.json`
- Create: `schemas/report.schema.json`

**Interfaces:**
- Produces: four draft 2020-12 schemas. Task 6's Rust tests validate `rules/` and `fixtures/` files and a generated report against them. The `$defs/source` block inside `rule.schema.json` deliberately duplicates `source.schema.json` (self-contained schemas, no cross-file `$ref`, so the `jsonschema` dev-dep needs no resolver features); a Task 6 test pins the two copies equal.

- [ ] **Step 1: `schemas/source.schema.json`**

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "harness-guard:source:1.0",
  "title": "Harness Guard evidence source",
  "type": "object",
  "required": ["schema_version", "url", "publisher", "title", "evidence_class", "retrieved", "content_hash"],
  "additionalProperties": false,
  "properties": {
    "schema_version": { "const": "1.0" },
    "url": { "type": "string", "minLength": 1, "pattern": "^https://" },
    "publisher": { "type": "string", "minLength": 1 },
    "title": { "type": "string", "minLength": 1 },
    "evidence_class": { "enum": ["local-observation", "official-documentation", "official-policy", "independent-reproduction", "inference"] },
    "retrieved": { "type": "string", "pattern": "^\\d{4}-\\d{2}-\\d{2}$" },
    "content_hash": { "type": "string", "pattern": "^sha256:[0-9a-f]{64}$" },
    "archived_url": { "type": ["string", "null"] },
    "notes": { "type": ["string", "null"] }
  }
}
```

- [ ] **Step 2: `schemas/rule.schema.json`**

The structural-citation constraint (spec §5.2 point 1, MDN `spec_url` pattern): if ANY outcome has a non-`unknown` status, `sources` must be non-empty (each source already requires non-empty `url` + `retrieved` via `$defs/source`).

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "harness-guard:rule:1.0",
  "title": "Harness Guard rule",
  "type": "object",
  "required": ["schema_version", "id", "tool", "category", "title", "why_it_matters", "os", "scopes", "auth_prerequisites", "observation", "outcomes", "tested_versions", "sources", "limitations", "unknown_conditions"],
  "additionalProperties": false,
  "properties": {
    "schema_version": { "const": "1.0" },
    "id": { "type": "string", "pattern": "^[a-z0-9]+(-[a-z0-9]+)*$" },
    "tool": { "enum": ["codex"] },
    "category": { "enum": ["retention", "telemetry", "training", "transfer", "sync", "permissions", "sandbox", "network"] },
    "title": { "type": "string", "minLength": 1 },
    "why_it_matters": { "type": "string", "minLength": 1 },
    "os": { "type": "array", "minItems": 1, "items": { "enum": ["macos", "linux", "windows"] } },
    "scopes": { "type": "array", "minItems": 1, "items": { "enum": ["user", "project"] } },
    "auth_prerequisites": { "type": ["string", "null"] },
    "observation": {
      "type": "object",
      "required": ["file", "key", "type", "allowed_render"],
      "additionalProperties": false,
      "properties": {
        "file": { "type": "string", "minLength": 1 },
        "key": { "type": "string", "minLength": 1 },
        "type": { "enum": ["enum", "bool"] },
        "allowed_render": { "type": "array", "minItems": 1, "items": { "type": "string", "minLength": 1 } }
      }
    },
    "outcomes": {
      "type": "array",
      "minItems": 1,
      "items": {
        "type": "object",
        "required": ["when", "status", "message"],
        "additionalProperties": false,
        "properties": {
          "when": { "type": "string", "minLength": 1 },
          "status": { "enum": ["pass", "finding", "unknown"] },
          "severity": { "enum": ["info", "warning", null] },
          "confidence": { "enum": ["low", "medium", "high", null] },
          "message": { "type": "string", "minLength": 1 },
          "remediation": {
            "type": ["object", "null"],
            "required": ["summary", "command"],
            "additionalProperties": false,
            "properties": {
              "summary": { "type": "string", "minLength": 1 },
              "command": { "type": "string", "minLength": 1 }
            }
          },
          "unknown_reason": { "type": "string", "minLength": 1 },
          "verify_url": { "type": ["string", "null"] }
        },
        "allOf": [
          { "if": { "properties": { "status": { "const": "finding" } } },
            "then": { "required": ["severity", "confidence"] } },
          { "if": { "properties": { "status": { "const": "pass" } } },
            "then": { "required": ["confidence"], "properties": { "severity": { "const": null } } } },
          { "if": { "properties": { "status": { "const": "unknown" } } },
            "then": { "required": ["unknown_reason"], "properties": { "severity": { "const": null }, "confidence": { "const": null } } } }
        ]
      }
    },
    "tested_versions": {
      "type": "array",
      "minItems": 1,
      "items": {
        "type": "object",
        "required": ["min", "max", "verified_on"],
        "additionalProperties": false,
        "properties": {
          "min": { "type": "string", "pattern": "^(<=)?\\d+\\.\\d+\\.\\d+$" },
          "max": { "type": "string", "pattern": "^\\d+\\.\\d+\\.\\d+$" },
          "verified_on": { "type": "string", "pattern": "^\\d{4}-\\d{2}-\\d{2}$" }
        }
      }
    },
    "sources": { "type": "array", "items": { "$ref": "#/$defs/source" } },
    "limitations": { "type": "array", "minItems": 1, "items": { "type": "string", "minLength": 1 } },
    "unknown_conditions": { "type": "array", "minItems": 1, "items": { "type": "string", "minLength": 1 } }
  },
  "if": {
    "properties": {
      "outcomes": {
        "contains": { "properties": { "status": { "enum": ["pass", "finding"] } } }
      }
    }
  },
  "then": {
    "properties": { "sources": { "minItems": 1 } }
  },
  "$defs": {
    "source": {
      "type": "object",
      "required": ["schema_version", "url", "publisher", "title", "evidence_class", "retrieved", "content_hash"],
      "additionalProperties": false,
      "properties": {
        "schema_version": { "const": "1.0" },
        "url": { "type": "string", "minLength": 1, "pattern": "^https://" },
        "publisher": { "type": "string", "minLength": 1 },
        "title": { "type": "string", "minLength": 1 },
        "evidence_class": { "enum": ["local-observation", "official-documentation", "official-policy", "independent-reproduction", "inference"] },
        "retrieved": { "type": "string", "pattern": "^\\d{4}-\\d{2}-\\d{2}$" },
        "content_hash": { "type": "string", "pattern": "^sha256:[0-9a-f]{64}$" },
        "archived_url": { "type": ["string", "null"] },
        "notes": { "type": ["string", "null"] }
      }
    }
  }
}
```

- [ ] **Step 3: `schemas/fixture.schema.json`**

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "harness-guard:fixture:1.0",
  "title": "Harness Guard fixture expectation",
  "type": "object",
  "required": ["schema_version", "case", "description", "expected_report"],
  "additionalProperties": false,
  "properties": {
    "schema_version": { "const": "1.0" },
    "case": { "type": "string", "pattern": "^[a-z0-9]+(-[a-z0-9]+)*$" },
    "description": { "type": "string", "minLength": 1 },
    "expected_report": {
      "type": "object",
      "description": "A subset of the report schema; tests assert every key present here matches the produced report recursively."
    }
  }
}
```

- [ ] **Step 4: `schemas/report.schema.json`**

This is simultaneously the `--json` contract and the only persisted artifact shape (§5.4).

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "harness-guard:report:1.0",
  "title": "Harness Guard sanitized report",
  "type": "object",
  "required": ["schema_version", "harness_guard_version", "ruleset_version", "scanned_at", "network_requests_made", "platform", "tools", "summary"],
  "additionalProperties": false,
  "properties": {
    "schema_version": { "const": "1.0" },
    "harness_guard_version": { "type": "string" },
    "ruleset_version": { "type": "string", "pattern": "^\\d{4}\\.\\d{2}\\.\\d{2}$" },
    "scanned_at": { "type": "string" },
    "network_requests_made": { "const": 0 },
    "platform": {
      "type": "object",
      "required": ["os"],
      "additionalProperties": false,
      "properties": { "os": { "enum": ["macos", "linux", "windows"] } }
    },
    "tools": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["tool", "detected_version", "config_paths", "detection_confidence", "rules_last_verified_version", "rules_verified_date", "version_in_range", "findings"],
        "additionalProperties": false,
        "properties": {
          "tool": { "enum": ["codex"] },
          "detected_version": { "type": ["string", "null"] },
          "config_paths": { "type": "array", "items": { "type": "string" } },
          "detection_confidence": { "enum": ["high", "medium", "low"] },
          "rules_last_verified_version": { "type": ["string", "null"] },
          "rules_verified_date": { "type": ["string", "null"] },
          "version_in_range": { "type": "boolean" },
          "findings": { "type": "array", "items": { "$ref": "#/$defs/finding" } }
        }
      }
    },
    "summary": {
      "type": "object",
      "required": ["tools_scanned", "warning", "info", "unknown", "stale", "passed"],
      "additionalProperties": false,
      "properties": {
        "tools_scanned": { "type": "integer", "minimum": 0 },
        "warning": { "type": "integer", "minimum": 0 },
        "info": { "type": "integer", "minimum": 0 },
        "unknown": { "type": "integer", "minimum": 0 },
        "stale": { "type": "integer", "minimum": 0 },
        "passed": { "type": "integer", "minimum": 0 }
      }
    }
  },
  "$defs": {
    "finding": {
      "type": "object",
      "required": ["rule_id", "status", "severity", "confidence", "evidence_class", "message", "observation", "remediation", "source", "valid_from", "valid_until", "limitations", "unknown_reason", "verify_url", "stale_reason"],
      "additionalProperties": false,
      "properties": {
        "rule_id": { "type": "string" },
        "status": { "enum": ["pass", "finding", "unknown", "stale-ruleset"] },
        "severity": { "enum": ["info", "warning", null] },
        "confidence": { "enum": ["low", "medium", "high", null] },
        "evidence_class": { "type": ["string", "null"] },
        "message": { "type": "string", "minLength": 1 },
        "observation": { "type": ["string", "null"] },
        "remediation": {
          "type": ["object", "null"],
          "required": ["summary", "command"],
          "additionalProperties": false,
          "properties": {
            "summary": { "type": "string" },
            "command": { "type": "string" }
          }
        },
        "source": {
          "type": ["object", "null"],
          "required": ["url", "retrieved"],
          "additionalProperties": false,
          "properties": {
            "url": { "type": "string" },
            "retrieved": { "type": "string" }
          }
        },
        "valid_from": { "type": ["string", "null"] },
        "valid_until": { "type": ["string", "null"] },
        "limitations": { "type": "array", "items": { "type": "string" } },
        "unknown_reason": { "type": ["string", "null"] },
        "verify_url": { "type": ["string", "null"] },
        "stale_reason": { "type": ["string", "null"] }
      },
      "allOf": [
        { "if": { "properties": { "status": { "enum": ["pass", "finding"] } } },
          "then": { "properties": { "source": { "type": "object" }, "confidence": { "enum": ["low", "medium", "high"] } } } },
        { "if": { "properties": { "status": { "const": "unknown" } } },
          "then": { "properties": { "unknown_reason": { "type": "string" }, "severity": { "const": null }, "confidence": { "const": null } } } },
        { "if": { "properties": { "status": { "const": "stale-ruleset" } } },
          "then": { "properties": { "stale_reason": { "type": "string" }, "source": { "type": "object" }, "severity": { "const": null }, "confidence": { "const": null } } } }
      ]
    }
  }
}
```

- [ ] **Step 5: Sanity-check and commit**

Run: `for f in schemas/*.json; do jq -e '."$schema" == "https://json-schema.org/draft/2020-12/schema"' "$f" >/dev/null && echo "OK $f"; done` → four OK lines. (Full validation tests land in Task 6.)

```bash
git add schemas/ && git commit -m "feat: Phase 0 JSON Schemas (source, rule, fixture, report) with structural citation constraint"
```

---

### Task 5: Fresh evidence retrieval + `rules/` data package + freshness scripts

This task does dev-time manual networking (retrieving official docs) — that is allowed; the *product* never does.

**Files:**
- Create: `scripts/freshness/normalize.sh`
- Create: `scripts/freshness/extract-urls.sh`
- Create: `rules/LICENSE` (copy of the root Apache-2.0 text)
- Create: `rules/README.md`
- Create: `rules/ruleset.json`
- Create: `rules/codex/history-persist-01.json`
- Create: `freshness/url-hashes.json`

**Interfaces:**
- Produces: `rules/codex/history-persist-01.json` (validates against `schemas/rule.schema.json`), `rules/ruleset.json` with `ruleset_version` CalVer, `scripts/freshness/normalize.sh <file-or-stdin>` → sha256 hex of normalized semantic text, `scripts/freshness/extract-urls.sh` → newline list of cited URLs from `rules/**/*.json`.
- Consumes: schemas from Task 4 (shape only).

- [ ] **Step 1: Write `scripts/freshness/normalize.sh`**

One normalization definition shared by citation `content_hash` (§5.1) and the doc-drift job (§11.2): HTML → semantic text → sha256.

```bash
#!/bin/sh
# normalize.sh — semantic-text normalization + sha256 (hex to stdout).
# Usage: normalize.sh [file]   (reads stdin if no file)
# Shared definition for rule source content_hash AND doc-drift hashing (§5.1).
# 1. drop script/style/nav/header/footer blocks  2. strip tags
# 3. decode common entities  4. collapse whitespace  5. hash
# Known limitation (documented in doc-drift.yml too): regex tag stripping is
# approximate; JS-rendered pages may need a Playwright fallback later.
set -eu
INPUT="${1:-/dev/stdin}"
perl -0777 -pe '
  s/<script\b.*?<\/script>//gis;
  s/<style\b.*?<\/style>//gis;
  s/<nav\b.*?<\/nav>//gis;
  s/<header\b.*?<\/header>//gis;
  s/<footer\b.*?<\/footer>//gis;
  s/<[^>]+>/ /g;
  s/&nbsp;/ /g; s/&amp;/&/g; s/&lt;/</g; s/&gt;/>/g; s/&quot;/"/g; s/&#39;/'"'"'/g;
  s/\s+/ /g; s/^\s+|\s+$//g;
' "$INPUT" | shasum -a 256 | cut -d' ' -f1
```

(Perl is stock on macOS and ubuntu runners; this stays within the "shell/jq, no xtask crate" constraint.) Make executable: `chmod +x scripts/freshness/normalize.sh`. Determinism check: run it twice on the same saved HTML file — identical hex both times.

- [ ] **Step 2: Write `scripts/freshness/extract-urls.sh`**

```bash
#!/bin/sh
# extract-urls.sh — list every cited source URL from the rules data package.
set -eu
cd "$(dirname "$0")/../.."
find rules -name '*.json' -not -name 'ruleset.json' -print0 \
  | xargs -0 jq -r '.sources[].url' \
  | sort -u
```

`chmod +x scripts/freshness/extract-urls.sh`.

- [ ] **Step 3: Fresh retrieval (maintainer action, record real dates)**

Let `TODAY` be the actual UTC date this step runs. For each of the two evidence URLs:

```bash
mkdir -p /private/tmp/claude-501/-Users-ben-Projects-harness-guard/0604fedb-8493-4709-a322-cdb7f76284ac/scratchpad/evidence
cd /private/tmp/claude-501/-Users-ben-Projects-harness-guard/0604fedb-8493-4709-a322-cdb7f76284ac/scratchpad/evidence
curl -sL 'https://developers.openai.com/codex/config-reference' -o config-reference.html
curl -sL 'https://developers.openai.com/codex/config-advanced' -o config-advanced.html
./scripts/freshness/normalize.sh config-reference.html   # → HASH_REF
./scripts/freshness/normalize.sh config-advanced.html    # → HASH_ADV
# Wayback snapshots (SPN): open in browser or:
curl -s "https://web.archive.org/save/https://developers.openai.com/codex/config-reference" -o /dev/null -w '%{redirect_url}\n'
curl -s "https://web.archive.org/save/https://developers.openai.com/codex/config-advanced" -o /dev/null -w '%{redirect_url}\n'
# then confirm the snapshot URLs resolve, e.g. https://web.archive.org/web/<ts>/https://developers.openai.com/codex/config-reference
```

**Verify against the page text** (must hold, else STOP and re-check the rule content before authoring): `history.persistence` exists; allowed values are exactly `save-all` and `none`; the default is `save-all`; the default is stated explicitly on at least one of the two pages — cite THAT page for the default. Also re-check npm latest: `curl -s https://registry.npmjs.org/@openai/codex/latest | jq -r .version` — if it moved past `0.144.4`, keep `min: "<=0.144.4"` but only raise `max` if the retrieved docs still show the same `history.persistence` semantics at the new version (otherwise keep `max: "0.144.4"`).

- [ ] **Step 4: Author `rules/codex/history-persist-01.json`**

Fill `<TODAY>`, `<HASH_REF>`, `<HASH_ADV>`, `<ARCHIVED_REF>`, `<ARCHIVED_ADV>` from Step 3. Everything else is final text:

```json
{
  "schema_version": "1.0",
  "id": "codex-history-persist-01",
  "tool": "codex",
  "category": "retention",
  "title": "Session history persistence",
  "why_it_matters": "By default, Codex CLI saves every session transcript — your prompts and the commands run — in plaintext to history.jsonl under CODEX_HOME, with no expiry. Anything sensitive you type or paste is retained on disk until you delete it yourself.",
  "os": ["macos", "linux", "windows"],
  "scopes": ["user"],
  "auth_prerequisites": null,
  "observation": {
    "file": "config.toml",
    "key": "history.persistence",
    "type": "enum",
    "allowed_render": ["save-all", "none", "unset"]
  },
  "outcomes": [
    {
      "when": "history.persistence is explicitly \"none\"",
      "status": "pass",
      "severity": null,
      "confidence": "high",
      "message": "Session history persistence is disabled (history.persistence = \"none\").",
      "remediation": null
    },
    {
      "when": "history.persistence is unset (documented default \"save-all\" applies) or explicitly \"save-all\"",
      "status": "finding",
      "severity": "warning",
      "confidence": "high",
      "message": "Codex CLI persists full session history to disk in plaintext with no expiry.",
      "remediation": {
        "summary": "Disable local session history persistence.",
        "command": "Add to ~/.codex/config.toml:\n[history]\npersistence = \"none\""
      }
    },
    {
      "when": "history.persistence is set to a value outside the documented enum",
      "status": "unknown",
      "severity": null,
      "confidence": null,
      "message": "history.persistence is set to an unrecognized value — raw values are never displayed.",
      "unknown_reason": "Value is outside the documented enum for the tested version range; it cannot be interpreted safely.",
      "verify_url": "https://developers.openai.com/codex/config-reference"
    }
  ],
  "tested_versions": [
    { "min": "<=0.144.4", "max": "0.144.4", "verified_on": "<TODAY>" }
  ],
  "sources": [
    {
      "schema_version": "1.0",
      "url": "https://developers.openai.com/codex/config-reference",
      "publisher": "OpenAI",
      "title": "Codex configuration reference",
      "evidence_class": "official-documentation",
      "retrieved": "<TODAY>",
      "content_hash": "sha256:<HASH_REF>",
      "archived_url": "<ARCHIVED_REF>",
      "notes": "Documents [history] persistence values save-all | none. Mirrored at learn.chatgpt.com/docs/config-file/config-reference."
    },
    {
      "schema_version": "1.0",
      "url": "https://developers.openai.com/codex/config-advanced",
      "publisher": "OpenAI",
      "title": "Codex advanced configuration",
      "evidence_class": "official-documentation",
      "retrieved": "<TODAY>",
      "content_hash": "sha256:<HASH_ADV>",
      "archived_url": "<ARCHIVED_ADV>",
      "notes": "States the default persistence behavior (save-all) applied when the key is unset."
    }
  ],
  "limitations": [
    "Project-level config is not inspected in this slice; only the user-scope config.toml is read.",
    "This rule cannot confirm any server-side or remote retention — no local file can.",
    "Auth method (ChatGPT sign-in vs API key) changes data-policy interpretation and is user-confirmed-or-unknown; this rule never infers it."
  ],
  "unknown_conditions": [
    "Config file unreadable (permissions).",
    "Config file is a symlink or non-regular file (not followed).",
    "history.persistence set to a value outside allowed_render (raw values are never displayed).",
    "Config file exceeds parse bounds (size > 1 MiB or nesting depth > 32)."
  ]
}
```

If Step 3's verification found the default stated on config-reference rather than config-advanced, swap which source carries the default `notes` line accordingly. If any documented value differs from research finding 1, STOP: update `allowed_render`/outcomes to the retrieved truth (the retrieval, not this plan, is the evidence of record).

- [ ] **Step 5: Author `rules/ruleset.json`, `rules/LICENSE`, `rules/README.md`**

`rules/ruleset.json` (CalVer = the date the ruleset content was finalized, i.e. TODAY):

```json
{
  "schema_version": "1.0",
  "ruleset_version": "<TODAY as YYYY.MM.DD>",
  "generated_note": "Hand-authored. Every rule carries its own sources with retrieved dates; see each rule's JSON."
}
```

`rules/LICENSE`: byte-identical copy of the root `LICENSE`.

`rules/README.md`:

```markdown
# Harness Guard rules — standalone data package

Machine-readable, source-cited audit rules for the privacy/retention/telemetry
posture of AI coding tool configurations. Consumed by
[Harness Guard](../README.md), a local, execution-free, per-finding-cited
config auditor for privacy/retention/telemetry posture — but this directory is
an **independently usable, forkable, permissively licensed data package from
day one**, not a folder convention.

- License: Apache-2.0 (see `LICENSE` in this directory).
- Contract: every file validates against `../schemas/rule.schema.json`
  (JSON Schema draft 2020-12). Consume rules only through that schema —
  the Harness Guard binary does exactly this and nothing more.
- Layout: `ruleset.json` (CalVer `ruleset_version`) + one JSON file per rule
  under `<tool>/<rule>.json`.
- Guarantees encoded in the schema: non-`unknown` outcomes structurally
  require a source with `url` + `retrieved`; `tested_versions` ranges are
  explicit; `limitations` and `unknown_conditions` are required.

No verification cadence is claimed for this data. Check each rule's
`retrieved` dates.
```

- [ ] **Step 6: Seed `freshness/url-hashes.json`**

```json
{
  "schema_version": "1.0",
  "generated": "<TODAY>",
  "hashes": {
    "https://developers.openai.com/codex/config-reference": "sha256:<HASH_REF>",
    "https://developers.openai.com/codex/config-advanced": "sha256:<HASH_ADV>"
  }
}
```

- [ ] **Step 7: Verify and commit**

Run: `jq -e . rules/ruleset.json rules/codex/history-persist-01.json freshness/url-hashes.json >/dev/null && echo OK` → OK.
Run: `scripts/freshness/extract-urls.sh` → exactly the two developers.openai.com URLs.
Grep guard: `grep -rn "AI agent security scanner" rules/ && echo FORBIDDEN || echo CLEAN` → CLEAN. Confirm no real username/path leaked: `grep -rn "/Users/" rules/ freshness/ || echo CLEAN` → CLEAN.

```bash
git add rules/ freshness/url-hashes.json scripts/freshness/
git commit -m "feat: codex-history-persist-01 rule with fresh evidence, standalone rules/ package, freshness scripts"
```

---

### Task 6: `harness-guard-rules` crate — schema types, embedded loading, validation tests

**Files:**
- Create: `crates/harness-guard-rules/src/schema.rs`
- Create: `crates/harness-guard-rules/src/report.rs`
- Create: `crates/harness-guard-rules/src/loader.rs`
- Modify: `crates/harness-guard-rules/src/lib.rs`
- Modify: `crates/harness-guard-rules/Cargo.toml` (dev-dep `jsonschema`)
- Test: `crates/harness-guard-rules/tests/schema_validation.rs`

**Interfaces:**
- Produces (consumed by core and cli):
  - `harness_guard_rules::schema::{RawRule, Observation, RawOutcome, Remediation, Source, TestedVersion}`
  - `harness_guard_rules::report::{Report, Platform, ToolReport, FindingRecord, Status, Severity, Confidence, SourceCite, Summary}`
  - `harness_guard_rules::loader::{ValidatedRule, load_rules() -> Vec<ValidatedRule>, ruleset_version() -> String}`
  - `ValidatedRule { pub raw: RawRule, pub primary_source: Source }` — the type-level citation guarantee: constructible only via `ValidatedRule::try_from_raw`, which fails if a non-unknown outcome exists without a source.

- [ ] **Step 1: Add the schema-validation dev-dependency**

Append to `crates/harness-guard-rules/Cargo.toml`:

```toml
[dev-dependencies]
# default-features = false: MUST NOT pull an HTTP resolver into the graph
# (cargo-deny would reject it; schemas are self-contained, no remote $ref).
jsonschema = { version = "0.30", default-features = false }
```

Run `cargo deny check` immediately — if `jsonschema`'s tree trips a ban, pin a version/feature set that doesn't (the bans are non-negotiable; the dev-dep choice is).

- [ ] **Step 2: Write the failing tests** (`crates/harness-guard-rules/tests/schema_validation.rs`)

```rust
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap()
}

fn compiled(schema_file: &str) -> jsonschema::Validator {
    let raw = std::fs::read_to_string(repo_root().join("schemas").join(schema_file)).unwrap();
    let json: serde_json::Value = serde_json::from_str(&raw).unwrap();
    jsonschema::validator_for(&json).unwrap()
}

#[test]
fn every_rule_file_validates_against_rule_schema() {
    let v = compiled("rule.schema.json");
    let rules_dir = repo_root().join("rules");
    let mut seen = 0;
    for entry in walk_json(&rules_dir) {
        if entry.file_name().unwrap() == "ruleset.json" { continue; }
        let json: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&entry).unwrap()).unwrap();
        assert!(v.validate(&json).is_ok(), "schema violation in {entry:?}: {:?}",
                v.iter_errors(&json).map(|e| e.to_string()).collect::<Vec<_>>());
        seen += 1;
    }
    assert_eq!(seen, 1, "slice ships exactly one rule");
}

#[test]
fn rule_missing_source_fails_schema_validation() {
    // Negative test proving the structural citation constraint (§10.4).
    let v = compiled("rule.schema.json");
    let raw = std::fs::read_to_string(
        repo_root().join("rules/codex/history-persist-01.json")).unwrap();
    let mut json: serde_json::Value = serde_json::from_str(&raw).unwrap();
    json["sources"] = serde_json::json!([]);
    assert!(v.validate(&json).is_err(),
        "a rule with a non-unknown outcome and no sources MUST fail validation");
}

#[test]
fn embedded_source_def_matches_source_schema() {
    // rule.schema.json embeds a copy of source.schema.json in $defs — pin them equal.
    let rule: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(repo_root().join("schemas/rule.schema.json")).unwrap()).unwrap();
    let source: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(repo_root().join("schemas/source.schema.json")).unwrap()).unwrap();
    let embedded = &rule["$defs"]["source"];
    for k in ["type", "required", "additionalProperties", "properties"] {
        assert_eq!(embedded[k], source[k], "drift between rule.schema.json $defs/source and source.schema.json at `{k}`");
    }
}

#[test]
fn rules_load_and_validate_via_types() {
    let rules = harness_guard_rules::loader::load_rules();
    assert_eq!(rules.len(), 1);
    let r = &rules[0];
    assert_eq!(r.raw.id, "codex-history-persist-01");
    assert_eq!(r.raw.observation.allowed_render, vec!["save-all", "none", "unset"]);
    assert!(r.primary_source.url.starts_with("https://"));
    assert!(!r.primary_source.retrieved.is_empty());
    assert!(!r.raw.limitations.is_empty());
    assert!(!r.raw.unknown_conditions.is_empty());
}

#[test]
fn ruleset_version_is_calver() {
    let v = harness_guard_rules::loader::ruleset_version();
    let parts: Vec<&str> = v.split('.').collect();
    assert_eq!(parts.len(), 3);
    assert_eq!(parts[0].len(), 4);
}

#[test]
fn non_unknown_outcome_without_source_is_unconstructible() {
    let rules = harness_guard_rules::loader::load_rules();
    let mut raw = rules[0].raw.clone();
    raw.sources.clear();
    assert!(harness_guard_rules::loader::ValidatedRule::try_from_raw(raw).is_err());
}

fn walk_json(dir: &Path) -> Vec<PathBuf> {
    let mut out = vec![];
    for e in std::fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() { out.extend(walk_json(&p)); }
        else if p.extension().is_some_and(|x| x == "json") { out.push(p); }
    }
    out.sort();
    out
}
```

- [ ] **Step 3: Run tests to verify they fail**

Run: `cargo test -p harness-guard-rules` → FAILS (modules don't exist yet).

- [ ] **Step 4: Implement `schema.rs`**

```rust
//! Serde mirror of schemas/rule.schema.json and schemas/source.schema.json.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RawRule {
    pub schema_version: String,
    pub id: String,
    pub tool: String,
    pub category: String,
    pub title: String,
    pub why_it_matters: String,
    pub os: Vec<String>,
    pub scopes: Vec<String>,
    pub auth_prerequisites: Option<String>,
    pub observation: Observation,
    pub outcomes: Vec<RawOutcome>,
    pub tested_versions: Vec<TestedVersion>,
    pub sources: Vec<Source>,
    pub limitations: Vec<String>,
    pub unknown_conditions: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    pub file: String,
    pub key: String,
    #[serde(rename = "type")]
    pub value_type: String,
    pub allowed_render: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RawOutcome {
    pub when: String,
    pub status: String, // "pass" | "finding" | "unknown" (schema-constrained)
    #[serde(default)]
    pub severity: Option<String>,
    #[serde(default)]
    pub confidence: Option<String>,
    pub message: String,
    #[serde(default)]
    pub remediation: Option<Remediation>,
    #[serde(default)]
    pub unknown_reason: Option<String>,
    #[serde(default)]
    pub verify_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Remediation {
    pub summary: String,
    pub command: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub schema_version: String,
    pub url: String,
    pub publisher: String,
    pub title: String,
    pub evidence_class: String,
    pub retrieved: String,
    pub content_hash: String,
    pub archived_url: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestedVersion {
    pub min: String,        // may carry the MDN "<=" prefix
    pub max: String,
    pub verified_on: String,
}
```

- [ ] **Step 5: Implement `report.rs`**

```rust
//! §5.4 sanitized report — simultaneously the --json contract and the ONLY
//! artifact shape. No raw config values ever enter these structs.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub schema_version: String,
    pub harness_guard_version: String,
    pub ruleset_version: String,
    pub scanned_at: String,
    pub network_requests_made: u32, // always 0
    pub platform: Platform,
    pub tools: Vec<ToolReport>,
    pub summary: Summary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Platform { pub os: String }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolReport {
    pub tool: String,
    pub detected_version: Option<String>,
    pub config_paths: Vec<String>,
    pub detection_confidence: Confidence,
    pub rules_last_verified_version: Option<String>,
    pub rules_verified_date: Option<String>,
    pub version_in_range: bool,
    pub findings: Vec<FindingRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindingRecord {
    pub rule_id: String,
    pub status: Status,
    pub severity: Option<Severity>,
    pub confidence: Option<Confidence>,
    pub evidence_class: Option<String>,
    pub message: String,
    pub observation: Option<String>, // allowlisted rendering ONLY, or None
    pub remediation: Option<crate::schema::Remediation>,
    pub source: Option<SourceCite>,
    pub valid_from: Option<String>,
    pub valid_until: Option<String>,
    pub limitations: Vec<String>,
    pub unknown_reason: Option<String>,
    pub verify_url: Option<String>,
    pub stale_reason: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Status { Pass, Finding, Unknown, StaleRuleset }

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity { Info, Warning } // Ord: Info < Warning (fail-on threshold relies on this)

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Confidence { Low, Medium, High }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceCite { pub url: String, pub retrieved: String }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Summary {
    pub tools_scanned: u32,
    pub warning: u32,
    pub info: u32,
    pub unknown: u32,
    pub stale: u32,
    pub passed: u32,
}

impl Summary {
    pub fn from_tools(tools: &[ToolReport]) -> Summary {
        let mut s = Summary { tools_scanned: tools.len() as u32,
            warning: 0, info: 0, unknown: 0, stale: 0, passed: 0 };
        for t in tools {
            for f in &t.findings {
                match f.status {
                    Status::Pass => s.passed += 1,
                    Status::Unknown => s.unknown += 1,
                    Status::StaleRuleset => s.stale += 1,
                    Status::Finding => match f.severity {
                        Some(Severity::Warning) => s.warning += 1,
                        _ => s.info += 1,
                    },
                }
            }
        }
        s
    }
}
```

- [ ] **Step 6: Implement `loader.rs`**

```rust
//! Compile-time embedded rules (§4: rules ship inside the binary for now)
//! + the type-level citation guarantee (§5.2 point 1).
use crate::schema::{RawRule, Source};

const RULESET_JSON: &str = include_str!("../../../rules/ruleset.json");
const RULE_HISTORY_PERSIST: &str =
    include_str!("../../../rules/codex/history-persist-01.json");

/// A rule that passed structural validation. The `primary_source` field is
/// non-optional: a rule with any non-`unknown` outcome cannot become a
/// `ValidatedRule` without at least one Source — the type repeats the
/// schema's guarantee.
#[derive(Debug, Clone)]
pub struct ValidatedRule {
    pub raw: RawRule,
    pub primary_source: Source,
}

#[derive(Debug)]
pub struct RuleValidationError(pub String);

impl std::fmt::Display for RuleValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "rule validation failed: {}", self.0)
    }
}
impl std::error::Error for RuleValidationError {}

impl ValidatedRule {
    pub fn try_from_raw(raw: RawRule) -> Result<Self, RuleValidationError> {
        let has_cited_outcome = raw.outcomes.iter().any(|o| o.status != "unknown");
        let primary_source = match raw.sources.first() {
            Some(s) if !s.url.is_empty() && !s.retrieved.is_empty() => s.clone(),
            _ if has_cited_outcome => {
                return Err(RuleValidationError(format!(
                    "rule {} has a non-unknown outcome but no usable source", raw.id)));
            }
            Some(s) => s.clone(),
            None => return Err(RuleValidationError(format!(
                "rule {} has no sources at all", raw.id))),
        };
        if raw.limitations.is_empty() || raw.unknown_conditions.is_empty() {
            return Err(RuleValidationError(format!(
                "rule {} must declare limitations and unknown_conditions", raw.id)));
        }
        if raw.observation.allowed_render.is_empty() {
            return Err(RuleValidationError(format!(
                "rule {} has an empty allowed_render allowlist", raw.id)));
        }
        if raw.tested_versions.is_empty() {
            return Err(RuleValidationError(format!(
                "rule {} has no tested_versions", raw.id)));
        }
        Ok(ValidatedRule { raw, primary_source })
    }
}

/// All bundled rules. Panics only on a corrupt embed, which `cargo test`
/// catches before any release build ships.
pub fn load_rules() -> Vec<ValidatedRule> {
    let raw: RawRule = serde_json::from_str(RULE_HISTORY_PERSIST)
        .expect("embedded rule JSON is valid (checked in tests)");
    vec![ValidatedRule::try_from_raw(raw)
        .expect("embedded rule passes validation (checked in tests)")]
}

pub fn ruleset_version() -> String {
    let v: serde_json::Value = serde_json::from_str(RULESET_JSON)
        .expect("embedded ruleset.json is valid");
    v["ruleset_version"].as_str().expect("ruleset_version present").to_string()
}
```

`lib.rs`:

```rust
//! Rule/report types mirroring schemas/, plus embedded rule loading.
pub mod loader;
pub mod report;
pub mod schema;
```

- [ ] **Step 7: Run tests to verify they pass**

Run: `cargo test -p harness-guard-rules` → all PASS. Also `cargo clippy --workspace --all-targets -- -D warnings` and `cargo deny check` → green.

- [ ] **Step 8: Commit**

```bash
git add crates/harness-guard-rules Cargo.lock
git commit -m "feat: rules crate — schema types, embedded loading, ValidatedRule citation guarantee, schema validation tests"
```

---

### Task 7: Core — DiscoveryRoot, bounded refusing reads, safe TOML parse + extraction

**Files:**
- Create: `crates/harness-guard-core/src/discovery.rs`
- Create: `crates/harness-guard-core/src/readfs.rs`
- Create: `crates/harness-guard-core/src/parse.rs`
- Modify: `crates/harness-guard-core/src/lib.rs`

**Interfaces:**
- Produces (consumed by Tasks 8–11):
  - `harness_guard_core::discovery::DiscoveryRoot { pub codex_home: PathBuf, pub path_dirs: Vec<PathBuf> }`
  - `harness_guard_core::readfs::{read_config(root: &DiscoveryRoot) -> ConfigReadOutcome, ConfigReadOutcome::{NoConfig, Ok(String), Refused(RefusalReason)}, RefusalReason::{Symlink, NotRegularFile, Oversized, PermissionDenied, NotUtf8, Io}}`
  - `harness_guard_core::parse::{parse_config(text: &str) -> Result<toml::Value, ParseFailure>, ParseFailure { line: Option<usize>, col: Option<usize>, key_path: Option<String>, message: String }, extract_key(doc: &toml::Value, dotted_key: &str) -> ExtractedValue, ExtractedValue::{Unset, Str(String), NonString}, line_col(src: &str, byte: usize) -> (usize, usize)}`
- Everything takes explicit roots/text — no env, no home, no process (clippy-enforced from Task 3).

- [ ] **Step 1: Write failing unit tests** (bottom of each module, `#[cfg(test)] mod tests`, using `tempfile`)

In `readfs.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::DiscoveryRoot;
    use std::io::Write;

    fn root_with(config: Option<&[u8]>) -> (tempfile::TempDir, DiscoveryRoot) {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("codex-home");
        std::fs::create_dir_all(&home).unwrap();
        if let Some(bytes) = config {
            std::fs::File::create(home.join("config.toml")).unwrap()
                .write_all(bytes).unwrap();
        }
        let root = DiscoveryRoot { codex_home: home, path_dirs: vec![] };
        (dir, root)
    }

    #[test]
    fn missing_home_is_no_config() {
        let dir = tempfile::tempdir().unwrap();
        let root = DiscoveryRoot {
            codex_home: dir.path().join("nope"), path_dirs: vec![] };
        assert!(matches!(read_config(&root), ConfigReadOutcome::NoConfig));
    }

    #[test]
    fn missing_file_is_no_config() {
        let (_d, root) = root_with(None);
        assert!(matches!(read_config(&root), ConfigReadOutcome::NoConfig));
    }

    #[test]
    fn regular_file_within_bounds_reads_ok() {
        let (_d, root) = root_with(Some(b"[history]\npersistence = \"none\"\n"));
        match read_config(&root) {
            ConfigReadOutcome::Ok(s) => assert!(s.contains("persistence")),
            other => panic!("expected Ok, got {other:?}"),
        }
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_config_is_refused_not_followed() {
        let (_d, root) = root_with(None);
        let target = root.codex_home.join("real.toml");
        std::fs::write(&target, "[history]\npersistence = \"none\"\n").unwrap();
        std::os::unix::fs::symlink(&target, root.codex_home.join("config.toml")).unwrap();
        assert!(matches!(read_config(&root),
            ConfigReadOutcome::Refused(RefusalReason::Symlink)));
    }

    #[test]
    fn oversized_config_is_refused() {
        let big = vec![b'#'; MAX_CONFIG_BYTES as usize + 1];
        let (_d, root) = root_with(Some(&big));
        assert!(matches!(read_config(&root),
            ConfigReadOutcome::Refused(RefusalReason::Oversized)));
    }

    #[test]
    fn non_utf8_is_refused() {
        let (_d, root) = root_with(Some(&[0xff, 0xfe, 0x00, 0x41]));
        assert!(matches!(read_config(&root),
            ConfigReadOutcome::Refused(RefusalReason::NotUtf8)));
    }

    #[cfg(unix)]
    #[test]
    fn permission_denied_is_refused() {
        use std::os::unix::fs::PermissionsExt;
        let (_d, root) = root_with(Some(b"x = 1\n"));
        let p = root.codex_home.join("config.toml");
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o000)).unwrap();
        let out = read_config(&root);
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o644)).unwrap();
        assert!(matches!(out, ConfigReadOutcome::Refused(RefusalReason::PermissionDenied)));
    }
}
```

In `parse.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_toml_parses_and_extracts() {
        let doc = parse_config("[history]\npersistence = \"none\"\n").unwrap();
        assert!(matches!(extract_key(&doc, "history.persistence"),
            ExtractedValue::Str(ref s) if s == "none"));
    }

    #[test]
    fn missing_key_is_unset() {
        let doc = parse_config("model = \"gpt-5\"\n").unwrap();
        assert!(matches!(extract_key(&doc, "history.persistence"), ExtractedValue::Unset));
    }

    #[test]
    fn non_string_value_is_nonstring_and_never_carried() {
        let doc = parse_config("[history]\npersistence = 3\n").unwrap();
        assert!(matches!(extract_key(&doc, "history.persistence"), ExtractedValue::NonString));
    }

    #[test]
    fn malformed_toml_reports_line_col_without_raw_text() {
        let err = parse_config("[history\npersistence = \"none\"\n").unwrap_err();
        assert_eq!(err.line, Some(1));
        assert!(err.col.is_some());
        // The failure must never carry the file's content wholesale.
        assert!(!err.message.contains("persistence = "));
    }

    #[test]
    fn depth_over_32_is_rejected() {
        // 40 nested inline tables: a = {a = {a = ... {a = 1} ...}}
        let mut s = String::from("a = ");
        for _ in 0..40 { s.push_str("{a = "); }
        s.push('1');
        for _ in 0..40 { s.push('}'); }
        s.push('\n');
        let err = parse_config(&s).unwrap_err();
        assert!(err.message.contains("nesting depth"));
    }

    #[test]
    fn depth_at_32_is_accepted() {
        let mut s = String::from("a = ");
        for _ in 0..31 { s.push_str("{a = "); }
        s.push('1');
        for _ in 0..31 { s.push('}'); }
        s.push('\n');
        assert!(parse_config(&s).is_ok());
    }

    #[test]
    fn hostile_deep_nesting_never_panics() {
        // 20k opens historically overflowed toml parsers; the crate's
        // built-in recursion limit must turn this into an Err, not a crash.
        let mut s = String::from("a = ");
        for _ in 0..20_000 { s.push_str("[["); }
        assert!(parse_config(&s).is_err());
    }

    #[test]
    fn line_col_counts_from_one() {
        assert_eq!(line_col("ab\ncd", 0), (1, 1));
        assert_eq!(line_col("ab\ncd", 3), (2, 1));
        assert_eq!(line_col("ab\ncd", 4), (2, 2));
    }
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p harness-guard-core` → FAILS to compile (modules missing).

- [ ] **Step 3: Implement `discovery.rs`**

```rust
//! Injected roots — the ONLY way core learns about the filesystem (§9).
use std::path::PathBuf;

/// Explicit discovery scope. Only the CLI crate constructs this from the
/// real environment; tests always pass fixture paths. Core has no other
/// door to the filesystem's ambient state (clippy-enforced).
#[derive(Debug, Clone)]
pub struct DiscoveryRoot {
    pub codex_home: PathBuf,
    pub path_dirs: Vec<PathBuf>,
}

impl DiscoveryRoot {
    pub fn config_path(&self) -> PathBuf {
        self.codex_home.join("config.toml")
    }
    /// Tool detection: the codex home exists, or a codex entry sits on PATH.
    pub fn codex_home_exists(&self) -> bool {
        self.codex_home.is_dir()
    }
}
```

- [ ] **Step 4: Implement `readfs.rs`**

```rust
//! Bounded, refusing reads (§9): symlink_metadata before open, regular
//! files only, 1 MiB cap, UTF-8 only. Refusal is a value, not an error —
//! callers map it to `unknown` findings.
use crate::discovery::DiscoveryRoot;
use std::io::Read;

pub const MAX_CONFIG_BYTES: u64 = 1024 * 1024; // 1 MiB (§9)

#[derive(Debug)]
pub enum ConfigReadOutcome {
    /// Home dir or config file absent — tool undetected or unconfigured.
    NoConfig,
    Ok(String),
    Refused(RefusalReason),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefusalReason {
    Symlink,
    NotRegularFile,
    Oversized,
    PermissionDenied,
    NotUtf8,
    Io,
}

impl RefusalReason {
    /// Structural, value-free text used in unknown_reason and diagnostics.
    pub fn describe(&self) -> &'static str {
        match self {
            RefusalReason::Symlink => "config file is a symlink — not followed",
            RefusalReason::NotRegularFile => "config path is not a regular file",
            RefusalReason::Oversized => "config file exceeds the 1 MiB parse bound",
            RefusalReason::PermissionDenied => "config file is not readable (permission denied)",
            RefusalReason::NotUtf8 => "config file is not valid UTF-8",
            RefusalReason::Io => "config file could not be read (I/O error)",
        }
    }
}

pub fn read_config(root: &DiscoveryRoot) -> ConfigReadOutcome {
    let path = root.config_path();
    let meta = match std::fs::symlink_metadata(&path) {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return ConfigReadOutcome::NoConfig;
        }
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
            return ConfigReadOutcome::Refused(RefusalReason::PermissionDenied);
        }
        Err(_) => return ConfigReadOutcome::Refused(RefusalReason::Io),
    };
    if meta.file_type().is_symlink() {
        return ConfigReadOutcome::Refused(RefusalReason::Symlink);
    }
    if !meta.file_type().is_file() {
        return ConfigReadOutcome::Refused(RefusalReason::NotRegularFile);
    }
    if meta.len() > MAX_CONFIG_BYTES {
        return ConfigReadOutcome::Refused(RefusalReason::Oversized);
    }
    let file = match std::fs::File::open(&path) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
            return ConfigReadOutcome::Refused(RefusalReason::PermissionDenied);
        }
        Err(_) => return ConfigReadOutcome::Refused(RefusalReason::Io),
    };
    // Belt-and-suspenders: cap the read itself, not just the stat.
    let mut buf = Vec::with_capacity(meta.len() as usize);
    match file.take(MAX_CONFIG_BYTES + 1).read_to_end(&mut buf) {
        Ok(_) => {}
        Err(_) => return ConfigReadOutcome::Refused(RefusalReason::Io),
    }
    if buf.len() as u64 > MAX_CONFIG_BYTES {
        return ConfigReadOutcome::Refused(RefusalReason::Oversized);
    }
    match String::from_utf8(buf) {
        Ok(s) => ConfigReadOutcome::Ok(s),
        Err(_) => ConfigReadOutcome::Refused(RefusalReason::NotUtf8),
    }
}
```

- [ ] **Step 5: Implement `parse.rs`**

```rust
//! Safe TOML parsing (§9): crate recursion limit as backstop, our own
//! depth ≤ 32 bound, line/col computed locally, raw text dropped
//! immediately after extraction. NEVER enable toml's `unbounded` feature.

pub const MAX_NESTING_DEPTH: usize = 32; // §9

/// Value-free parse failure. `message` comes from toml::de::Error::message()
/// (never its Display, which can render source snippets); `key_path` is set
/// only for extraction-stage issues where WE know the key — the toml error
/// type does not expose one (research finding 3).
#[derive(Debug, Clone)]
pub struct ParseFailure {
    pub line: Option<usize>,
    pub col: Option<usize>,
    pub key_path: Option<String>,
    pub message: String,
}

pub fn parse_config(text: &str) -> Result<toml::Value, ParseFailure> {
    let doc: toml::Value = toml::from_str(text).map_err(|e| {
        let (line, col) = match e.span() {
            Some(span) => {
                let (l, c) = line_col(text, span.start.min(text.len()));
                (Some(l), Some(c))
            }
            None => (None, None),
        };
        ParseFailure { line, col, key_path: None, message: e.message().to_string() }
    })?;
    let depth = value_depth(&doc);
    if depth > MAX_NESTING_DEPTH {
        return Err(ParseFailure {
            line: None,
            col: None,
            key_path: None,
            message: format!(
                "nesting depth {depth} exceeds the safety bound of {MAX_NESTING_DEPTH}"),
        });
    }
    Ok(doc)
}

fn value_depth(v: &toml::Value) -> usize {
    match v {
        toml::Value::Table(t) =>
            1 + t.values().map(value_depth).max().unwrap_or(0),
        toml::Value::Array(a) =>
            1 + a.iter().map(value_depth).max().unwrap_or(0),
        _ => 0,
    }
}

/// 1-based line/column for a byte offset (miette diagnostics carry these
/// as plain numbers — never the source text, §7.3).
pub fn line_col(src: &str, byte: usize) -> (usize, usize) {
    let clamped = byte.min(src.len());
    let before = &src[..clamped];
    let line = before.bytes().filter(|&b| b == b'\n').count() + 1;
    let col = before.rfind('\n').map(|i| clamped - i).unwrap_or(clamped + 1);
    (line, col)
}

/// Rule-relevant key extraction (§9). Only the requested dotted key is
/// pulled out; the caller drops the parsed document (and thus every
/// unrelated key and raw value) immediately after.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtractedValue {
    Unset,
    /// A string value. Held in memory only until the allowed_render check
    /// in evaluation; it never enters a report struct un-allowlisted.
    Str(String),
    /// Present but not a string — never rendered, treated as unrecognized.
    NonString,
}

pub fn extract_key(doc: &toml::Value, dotted_key: &str) -> ExtractedValue {
    let mut cur = doc;
    for part in dotted_key.split('.') {
        match cur.get(part) {
            Some(next) => cur = next,
            None => return ExtractedValue::Unset,
        }
    }
    match cur.as_str() {
        Some(s) => ExtractedValue::Str(s.to_string()),
        None => ExtractedValue::NonString,
    }
}
```

`lib.rs`:

```rust
//! Discovery, bounded reads, safe parsing, and rule evaluation.
//! Everything takes an explicit DiscoveryRoot — no env, no network,
//! no process spawning (clippy + cargo-deny enforced).
pub mod discovery;
pub mod parse;
pub mod readfs;
```

(`version`, `evaluate`, `scan` modules are added in Tasks 8–9.)

- [ ] **Step 6: Run tests to verify they pass**

Run: `cargo test -p harness-guard-core` → all PASS.
Run: `cargo clippy --workspace --all-targets -- -D warnings` → green (proves core's implementation used no disallowed APIs).

- [ ] **Step 7: Commit**

```bash
git add crates/harness-guard-core Cargo.lock
git commit -m "feat: core discovery root, bounded refusing reads, depth-bounded TOML parse + key extraction"
```

---

### Task 8: Core — execution-free version detection + range matching

**Files:**
- Create: `crates/harness-guard-core/src/version.rs`
- Modify: `crates/harness-guard-core/src/lib.rs` (add `pub mod version;`)

**Interfaces:**
- Produces:
  - `harness_guard_core::version::{detect_codex_version(root: &DiscoveryRoot) -> Option<String>, version_in_range(detected: &str, ranges: &[TestedVersion]) -> bool, binary_on_path(root: &DiscoveryRoot) -> bool}`
- Consumes: `DiscoveryRoot` (Task 7), `harness_guard_rules::schema::TestedVersion` (Task 6).
- **Hard invariant: never executes anything.** Reads `symlink_metadata`, resolves symlinks with bounded hops, reads `package.json` files. That's all.
- Version comparison is a hand-rolled `(u64, u64, u64)` triple compare on strict `X.Y.Z` strings — the fixed crate stack has no `semver`, and Codex versions are plain triples. Suffixed versions (e.g. `0.144.4-darwin-arm64`, the npm platform sub-package trap) fail the parse and yield `None`/no-match — conservative by construction.

- [ ] **Step 1: Write failing tests** (in `version.rs`)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::DiscoveryRoot;
    use harness_guard_rules::schema::TestedVersion;

    fn tv(min: &str, max: &str) -> TestedVersion {
        TestedVersion { min: min.into(), max: max.into(), verified_on: "2026-07-14".into() }
    }

    #[test]
    fn parse_strict_triples_only() {
        assert_eq!(parse_version("0.144.4"), Some((0, 144, 4)));
        assert_eq!(parse_version("0.144.4-darwin-arm64"), None); // npm platform-pkg trap
        assert_eq!(parse_version("v0.144.4"), None);
        assert_eq!(parse_version(""), None);
    }

    #[test]
    fn le_prefixed_min_is_unbounded_below() {
        let r = [tv("<=0.144.4", "0.144.4")];
        assert!(version_in_range("0.1.0", &r));
        assert!(version_in_range("0.144.4", &r));
        assert!(!version_in_range("0.144.5", &r));
        assert!(!version_in_range("9.9.9", &r));
    }

    #[test]
    fn plain_min_is_a_real_lower_bound() {
        let r = [tv("0.100.0", "0.144.4")];
        assert!(!version_in_range("0.99.9", &r));
        assert!(version_in_range("0.100.0", &r));
    }

    #[test]
    fn unparseable_detected_version_never_matches() {
        let r = [tv("<=0.144.4", "0.144.4")];
        assert!(!version_in_range("0.144.4-darwin-arm64", &r));
    }

    fn npm_layout(version_json: &str) -> (tempfile::TempDir, DiscoveryRoot) {
        // Models research finding 4, case A, flattened to committed-file form:
        // path/codex is the shim; nearest package.json up the tree owns it.
        let dir = tempfile::tempdir().unwrap();
        let pkg = dir.path().join("node_modules/@openai/codex");
        std::fs::create_dir_all(pkg.join("bin")).unwrap();
        std::fs::write(pkg.join("bin/codex"), "#!/usr/bin/env node\n").unwrap();
        std::fs::write(pkg.join("package.json"), version_json).unwrap();
        let root = DiscoveryRoot {
            codex_home: dir.path().join("codex-home"),
            path_dirs: vec![pkg.join("bin")],
        };
        (dir, root)
    }

    #[test]
    fn npm_layout_detects_clean_version() {
        let (_d, root) = npm_layout(r#"{"name": "@openai/codex", "version": "0.144.4"}"#);
        assert_eq!(detect_codex_version(&root), Some("0.144.4".to_string()));
    }

    #[test]
    fn wrong_package_name_is_ignored() {
        let (_d, root) = npm_layout(r#"{"name": "something-else", "version": "0.144.4"}"#);
        assert_eq!(detect_codex_version(&root), None);
    }

    #[test]
    fn suffixed_version_is_rejected() {
        let (_d, root) = npm_layout(
            r#"{"name": "@openai/codex", "version": "0.144.4-darwin-arm64"}"#);
        assert_eq!(detect_codex_version(&root), None);
    }

    #[test]
    fn no_package_json_is_none() {
        // Homebrew / standalone-installer / manual layouts (finding 4, cases B & C).
        let dir = tempfile::tempdir().unwrap();
        let bin = dir.path().join("bin");
        std::fs::create_dir_all(&bin).unwrap();
        std::fs::write(bin.join("codex"), "binary").unwrap();
        let root = DiscoveryRoot { codex_home: dir.path().join("x"), path_dirs: vec![bin] };
        assert_eq!(detect_codex_version(&root), None);
    }

    #[cfg(unix)]
    #[test]
    fn symlink_chain_is_resolved_with_bounded_hops() {
        // bin/codex -> pkg/bin/codex (1 hop), like a real npm prefix symlink.
        let dir = tempfile::tempdir().unwrap();
        let pkg = dir.path().join("lib/node_modules/@openai/codex");
        std::fs::create_dir_all(pkg.join("bin")).unwrap();
        std::fs::write(pkg.join("bin/codex"), "shim").unwrap();
        std::fs::write(pkg.join("package.json"),
            r#"{"name": "@openai/codex", "version": "0.144.4"}"#).unwrap();
        let bin = dir.path().join("bin");
        std::fs::create_dir_all(&bin).unwrap();
        std::os::unix::fs::symlink(pkg.join("bin/codex"), bin.join("codex")).unwrap();
        let root = DiscoveryRoot { codex_home: dir.path().join("x"), path_dirs: vec![bin] };
        assert_eq!(detect_codex_version(&root), Some("0.144.4".to_string()));
    }

    #[cfg(unix)]
    #[test]
    fn symlink_loop_terminates_with_none() {
        let dir = tempfile::tempdir().unwrap();
        let bin = dir.path().join("bin");
        std::fs::create_dir_all(&bin).unwrap();
        std::os::unix::fs::symlink(bin.join("b"), bin.join("codex")).unwrap();
        std::os::unix::fs::symlink(bin.join("codex"), bin.join("b")).unwrap();
        let root = DiscoveryRoot { codex_home: dir.path().join("x"), path_dirs: vec![bin] };
        assert_eq!(detect_codex_version(&root), None); // no panic, no hang
    }
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p harness-guard-core version` → compile FAIL.

- [ ] **Step 3: Implement `version.rs`**

```rust
//! Execution-free Codex version detection (§9 + research finding 4).
//! NEVER runs the tool. npm layouts yield a version; standalone/Homebrew
//! layouts legitimately yield None → stale-ruleset ("version not detected").
use crate::discovery::DiscoveryRoot;
use harness_guard_rules::schema::TestedVersion;
use std::path::{Path, PathBuf};

const MAX_SYMLINK_HOPS: usize = 5;   // standalone chain needs 3; headroom, still bounded
const MAX_PARENT_WALK: usize = 5;    // resolved binary → owning package.json
const EXPECTED_PACKAGE: &str = "@openai/codex";

pub fn detect_codex_version(root: &DiscoveryRoot) -> Option<String> {
    let bin = find_codex_entry(root)?;
    let resolved = resolve_bounded(&bin)?;
    let pkg_json = nearest_package_json(&resolved)?;
    let text = std::fs::read_to_string(&pkg_json).ok()?;
    let v: serde_json::Value = serde_json::from_str(&text).ok()?;
    if v.get("name").and_then(|n| n.as_str()) != Some(EXPECTED_PACKAGE) {
        return None; // unrelated package.json — do not trust it
    }
    let version = v.get("version")?.as_str()?;
    parse_version(version)?; // reject suffixed/unparseable versions
    Some(version.to_string())
}

/// Tool-on-PATH check used for detection confidence and the `list` command.
pub fn binary_on_path(root: &DiscoveryRoot) -> bool {
    find_codex_entry(root).is_some()
}

fn find_codex_entry(root: &DiscoveryRoot) -> Option<PathBuf> {
    for dir in &root.path_dirs {
        let candidate = dir.join("codex");
        if std::fs::symlink_metadata(&candidate).is_ok() {
            return Some(candidate);
        }
    }
    None
}

fn resolve_bounded(start: &Path) -> Option<PathBuf> {
    let mut cur = start.to_path_buf();
    for _ in 0..=MAX_SYMLINK_HOPS {
        let meta = std::fs::symlink_metadata(&cur).ok()?;
        if !meta.file_type().is_symlink() {
            return Some(cur);
        }
        let target = std::fs::read_link(&cur).ok()?;
        cur = if target.is_absolute() {
            target
        } else {
            cur.parent()?.join(target)
        };
    }
    None // too many hops — refuse rather than chase
}

fn nearest_package_json(resolved_bin: &Path) -> Option<PathBuf> {
    let mut dir = resolved_bin.parent()?;
    for _ in 0..MAX_PARENT_WALK {
        let candidate = dir.join("package.json");
        if let Ok(meta) = std::fs::symlink_metadata(&candidate) {
            if meta.file_type().is_file() {
                return Some(candidate);
            }
        }
        dir = dir.parent()?;
    }
    None
}

/// Strict X.Y.Z only. Suffixed npm platform-package versions
/// ("0.144.4-darwin-arm64") deliberately fail.
pub fn parse_version(s: &str) -> Option<(u64, u64, u64)> {
    let mut parts = s.split('.');
    let maj = parts.next()?.parse().ok()?;
    let min = parts.next()?.parse().ok()?;
    let pat = parts.next()?.parse().ok()?;
    if parts.next().is_some() { return None; }
    Some((maj, min, pat))
}

/// A detected version is "verified" iff SOME entry matches (§5.2 point 3).
/// No match ⇒ caller degrades to stale-ruleset. `min` may carry the MDN
/// "<=" prefix meaning "confirmed at max, possibly applicable earlier"
/// ⇒ unbounded below.
pub fn version_in_range(detected: &str, ranges: &[TestedVersion]) -> bool {
    let Some(d) = parse_version(detected) else { return false };
    ranges.iter().any(|r| {
        let Some(max) = parse_version(&r.max) else { return false };
        if d > max { return false; }
        match r.min.strip_prefix("<=") {
            Some(_) => true, // unbounded below
            None => match parse_version(&r.min) {
                Some(min) => d >= min,
                None => false,
            },
        }
    })
}
```

Add `pub mod version;` to `lib.rs`.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p harness-guard-core` → all PASS. `cargo clippy --workspace --all-targets -- -D warnings` → green (no `Command`, no env).

- [ ] **Step 5: Commit**

```bash
git add crates/harness-guard-core/src crates/harness-guard-core/Cargo.toml Cargo.lock
git commit -m "feat: execution-free codex version detection + explicit tested-version range matching"
```

---

### Task 9: Core — rule evaluation with conservative degradation + scan orchestrator

**Files:**
- Create: `crates/harness-guard-core/src/evaluate.rs`
- Create: `crates/harness-guard-core/src/scan.rs`
- Modify: `crates/harness-guard-core/src/lib.rs` (add `pub mod evaluate; pub mod scan;`)

**Interfaces:**
- Produces (consumed by the CLI in Task 11):
  - `harness_guard_core::scan::{scan_codex(root: &DiscoveryRoot, rules: &[ValidatedRule]) -> Option<ScanResult>, ScanResult { pub tool_report: ToolReport, pub degraded: bool, pub parse_failure: Option<ParseFailure> }}` — `None` = tool not detected (no codex home AND no binary on PATH). `degraded == true` ⇔ exit code 2 territory (unreadable/unparseable config). `tool_report.config_paths` holds UNREDACTED absolute paths — the CLI redacts before rendering (core cannot know the home dir).
  - `harness_guard_core::evaluate::{evaluate_rule(rule: &ValidatedRule, config: &ConfigState, detected_version: Option<&str>) -> FindingRecord, ConfigState::{Missing, Unreadable(RefusalReason), Unparseable(ParseFailure), Parsed(BTreeMap<String, ExtractedValue>)}}`
- Consumes: Tasks 6–8 interfaces exactly as named there.

**Evaluation precedence (fixture-pinned in Task 10):**
1. `Unreadable`/`Unparseable` ⇒ `unknown` (declared `unknown_conditions` fired) — regardless of version.
2. Version undetected or outside every `tested_versions` entry ⇒ `stale-ruleset`; the indicative outcome is still computed and embedded IN THE MESSAGE, phrased as unverified; `stale_reason` required; source kept (shown as unverified). Never silently pass, never drop — the anti-next.js#92091 pin.
3. In range: value dispatch — unset ⇒ finding; `"none"` ⇒ pass; `"save-all"` ⇒ finding; anything else (incl. non-string) ⇒ `unknown` with the value never echoed.
4. `Missing` config file with tool detected = unset (documented default applies) ⇒ same as unset.

- [ ] **Step 1: Write failing tests** (in `evaluate.rs`)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::ExtractedValue;
    use harness_guard_rules::loader::load_rules;
    use harness_guard_rules::report::{Severity, Status};
    use std::collections::BTreeMap;

    fn rule() -> harness_guard_rules::loader::ValidatedRule {
        load_rules().into_iter().next().unwrap()
    }
    fn parsed(v: Option<ExtractedValue>) -> ConfigState {
        let mut m = BTreeMap::new();
        if let Some(v) = v { m.insert("history.persistence".to_string(), v); }
        else { m.insert("history.persistence".to_string(), ExtractedValue::Unset); }
        ConfigState::Parsed(m)
    }

    #[test]
    fn none_in_range_passes_with_citation() {
        let f = evaluate_rule(&rule(),
            &parsed(Some(ExtractedValue::Str("none".into()))), Some("0.144.4"));
        assert_eq!(f.status, Status::Pass);
        assert!(f.source.is_some(), "pass requires a citation (§5.4)");
        assert_eq!(f.observation.as_deref(), Some("history.persistence = \"none\""));
        assert_eq!(f.valid_until.as_deref(), Some("0.144.4"));
    }

    #[test]
    fn unset_in_range_is_warning_finding() {
        let f = evaluate_rule(&rule(), &parsed(None), Some("0.144.4"));
        assert_eq!(f.status, Status::Finding);
        assert_eq!(f.severity, Some(Severity::Warning));
        assert_eq!(f.observation.as_deref(),
            Some("history.persistence unset (documented default \"save-all\" applies)"));
        assert!(f.remediation.is_some());
        assert!(f.source.is_some());
    }

    #[test]
    fn explicit_save_all_is_warning_finding() {
        let f = evaluate_rule(&rule(),
            &parsed(Some(ExtractedValue::Str("save-all".into()))), Some("0.144.4"));
        assert_eq!(f.status, Status::Finding);
        assert_eq!(f.observation.as_deref(), Some("history.persistence = \"save-all\""));
    }

    #[test]
    fn unrecognized_value_is_unknown_and_never_echoed() {
        let f = evaluate_rule(&rule(),
            &parsed(Some(ExtractedValue::Str("archive".into()))), Some("0.144.4"));
        assert_eq!(f.status, Status::Unknown);
        assert!(f.severity.is_none() && f.confidence.is_none());
        assert!(f.unknown_reason.is_some());
        assert!(f.observation.is_none());
        let json = serde_json::to_string(&f).unwrap();
        assert!(!json.contains("archive"), "raw value leaked into the record");
    }

    #[test]
    fn non_string_value_is_unknown() {
        let f = evaluate_rule(&rule(),
            &parsed(Some(ExtractedValue::NonString)), Some("0.144.4"));
        assert_eq!(f.status, Status::Unknown);
    }

    #[test]
    fn missing_config_with_tool_detected_is_unset_finding() {
        let f = evaluate_rule(&rule(), &ConfigState::Missing, Some("0.144.4"));
        assert_eq!(f.status, Status::Finding);
    }

    #[test]
    fn unreadable_config_is_unknown() {
        let f = evaluate_rule(&rule(),
            &ConfigState::Unreadable(crate::readfs::RefusalReason::PermissionDenied),
            Some("0.144.4"));
        assert_eq!(f.status, Status::Unknown);
        assert!(f.unknown_reason.as_deref().unwrap().contains("permission"));
    }

    #[test]
    fn undetected_version_is_stale_never_pass() {
        // Conservative-degradation pin (anti-next.js#92091).
        let f = evaluate_rule(&rule(),
            &parsed(Some(ExtractedValue::Str("none".into()))), None);
        assert_eq!(f.status, Status::StaleRuleset);
        assert!(f.stale_reason.as_deref().unwrap().contains("not detected"));
        assert!(f.message.to_lowercase().contains("unverified"));
        assert!(f.message.contains("history.persistence = \"none\""),
            "indicative outcome must surface in the message");
        assert!(f.source.is_some(), "stale keeps the last-known source (§5.4)");
        assert!(f.severity.is_none() && f.confidence.is_none());
    }

    #[test]
    fn out_of_range_version_is_stale_never_pass() {
        let f = evaluate_rule(&rule(), &parsed(None), Some("9.9.9"));
        assert_eq!(f.status, Status::StaleRuleset);
        assert!(f.stale_reason.as_deref().unwrap().contains("9.9.9"));
    }

    #[test]
    fn unknown_beats_stale_when_config_unreadable() {
        let f = evaluate_rule(&rule(),
            &ConfigState::Unreadable(crate::readfs::RefusalReason::Symlink), None);
        assert_eq!(f.status, Status::Unknown);
    }
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p harness-guard-core evaluate` → compile FAIL.

- [ ] **Step 3: Implement `evaluate.rs`**

```rust
//! Rule evaluation (§5.4 status model). Degradation is conservative and
//! total: every path yields a FindingRecord; nothing is dropped or
//! silently passed.
use crate::parse::{ExtractedValue, ParseFailure};
use crate::readfs::RefusalReason;
use crate::version::version_in_range;
use harness_guard_rules::loader::ValidatedRule;
use harness_guard_rules::report::{Confidence, FindingRecord, Severity, SourceCite, Status};
use std::collections::BTreeMap;

#[derive(Debug)]
pub enum ConfigState {
    /// Tool detected but no config file — documented defaults apply.
    Missing,
    Unreadable(RefusalReason),
    Unparseable(ParseFailure),
    Parsed(BTreeMap<String, ExtractedValue>),
}

pub fn evaluate_rule(
    rule: &ValidatedRule,
    config: &ConfigState,
    detected_version: Option<&str>,
) -> FindingRecord {
    let base = base_record(rule);

    // 1. Declared unknown_conditions fire first — a file we refused to read
    //    is unknowable regardless of version bookkeeping.
    match config {
        ConfigState::Unreadable(reason) => {
            return unknown(base, reason.describe().to_string(), rule);
        }
        ConfigState::Unparseable(pf) => {
            return unknown(base,
                format!("config not safely parseable: {}", pf.message), rule);
        }
        _ => {}
    }

    // 2. Version gate — no matching tested_versions entry ⇒ stale-ruleset,
    //    indicative outcome only in the message, phrased as unverified.
    let in_range = detected_version
        .map(|v| version_in_range(v, &rule.raw.tested_versions))
        .unwrap_or(false);
    let (obs, indicative) = observe(rule, config);
    if !in_range {
        let stale_reason = match detected_version {
            None => "tool version not detected — no version marker found on PATH".to_string(),
            Some(v) => format!(
                "detected version {v} is outside every tested range (max tested {})",
                rule.raw.tested_versions.iter().map(|t| t.max.as_str())
                    .max().unwrap_or("?")),
        };
        return FindingRecord {
            status: Status::StaleRuleset,
            severity: None,
            confidence: None,
            // No "UNVERIFIED (stale ruleset)" prefix here — the terminal
            // label supplies it; duplicating would double-print.
            message: format!(
                "Unverified — last-known rule indicates: {} Observed: {}.",
                indicative.message, obs.clone().unwrap_or_else(|| "n/a".into())),
            observation: obs,
            remediation: None,
            stale_reason: Some(stale_reason),
            ..base
        };
    }

    // 3. In range: dispatch on the observed value.
    match indicative.kind {
        IndicativeKind::Pass => FindingRecord {
            status: Status::Pass,
            severity: None,
            confidence: Some(Confidence::High),
            message: indicative.message,
            observation: obs,
            remediation: None,
            ..base
        },
        IndicativeKind::Finding => FindingRecord {
            status: Status::Finding,
            severity: Some(Severity::Warning),
            confidence: Some(Confidence::High),
            message: indicative.message,
            observation: obs,
            remediation: rule.raw.outcomes.iter()
                .find(|o| o.status == "finding")
                .and_then(|o| o.remediation.clone()),
            ..base
        },
        IndicativeKind::Unrecognized => unknown(base,
            "history.persistence is set to an unrecognized value — raw values are never displayed".to_string(),
            rule),
    }
}

enum IndicativeKind { Pass, Finding, Unrecognized }
struct Indicative { kind: IndicativeKind, message: String }

/// Allowlisted observation rendering + which outcome the value indicates.
/// The raw value is inspected here and DROPPED unless it is in
/// allowed_render — it never escapes this function otherwise.
fn observe(rule: &ValidatedRule, config: &ConfigState) -> (Option<String>, Indicative) {
    let key = &rule.raw.observation.key;
    let value = match config {
        ConfigState::Parsed(map) => map.get(key).cloned().unwrap_or(ExtractedValue::Unset),
        _ => ExtractedValue::Unset, // Missing config: documented default applies
    };
    let msg = |status: &str| rule.raw.outcomes.iter()
        .find(|o| o.status == status)
        .map(|o| o.message.clone())
        .unwrap_or_default();
    match value {
        ExtractedValue::Str(ref s) if s == "none" => (
            Some(format!("{key} = \"none\"")),
            Indicative { kind: IndicativeKind::Pass, message: msg("pass") },
        ),
        ExtractedValue::Str(ref s) if s == "save-all" => (
            Some(format!("{key} = \"save-all\"")),
            Indicative { kind: IndicativeKind::Finding, message: msg("finding") },
        ),
        ExtractedValue::Unset => (
            Some(format!("{key} unset (documented default \"save-all\" applies)")),
            Indicative { kind: IndicativeKind::Finding, message: msg("finding") },
        ),
        // Str(other) or NonString: NOT in allowed_render — never rendered.
        _ => (
            None,
            Indicative { kind: IndicativeKind::Unrecognized, message: String::new() },
        ),
    }
}

fn base_record(rule: &ValidatedRule) -> FindingRecord {
    let tv = &rule.raw.tested_versions[0];
    FindingRecord {
        rule_id: rule.raw.id.clone(),
        status: Status::Unknown, // always overwritten
        severity: None,
        confidence: None,
        evidence_class: Some(rule.primary_source.evidence_class.clone()),
        message: String::new(),
        observation: None,
        remediation: None,
        source: Some(SourceCite {
            url: rule.primary_source.url.clone(),
            retrieved: rule.primary_source.retrieved.clone(),
        }),
        valid_from: Some(tv.min.clone()),
        valid_until: Some(tv.max.clone()),
        limitations: rule.raw.limitations.clone(),
        unknown_reason: None,
        verify_url: None,
        stale_reason: None,
    }
}

fn unknown(base: FindingRecord, reason: String, rule: &ValidatedRule) -> FindingRecord {
    let verify_url = rule.raw.outcomes.iter()
        .find(|o| o.status == "unknown")
        .and_then(|o| o.verify_url.clone());
    FindingRecord {
        status: Status::Unknown,
        severity: None,
        confidence: None,
        evidence_class: None,
        message: format!("Cannot determine history persistence posture: {reason}"),
        observation: None,
        remediation: None,
        source: None,          // §5.4: unknown requires no source
        unknown_reason: Some(reason),
        verify_url,
        ..base
    }
}
```

Note the hard-coded `"none"`/`"save-all"` literals sit ONLY in `observe()` and match `rules/codex/history-persist-01.json`'s `allowed_render` — a one-rule slice accepts this coupling; a Task 6 test already pins `allowed_render == ["save-all","none","unset"]`, so drift breaks the build. (Generalizing the value→outcome mapping into rule JSON is deliberate later work, not this slice.)

- [ ] **Step 4: Implement `scan.rs`**

```rust
//! The scan orchestrator: discovery → bounded read → parse → extract →
//! evaluate each rule → ToolReport. Raw config text and the parsed
//! document are dropped before this function returns.
use crate::discovery::DiscoveryRoot;
use crate::evaluate::{evaluate_rule, ConfigState};
use crate::parse::{extract_key, parse_config, ParseFailure};
use crate::readfs::{read_config, ConfigReadOutcome};
use crate::version::{binary_on_path, detect_codex_version};
use harness_guard_rules::loader::ValidatedRule;
use harness_guard_rules::report::{Confidence, ToolReport};
use std::collections::BTreeMap;

pub struct ScanResult {
    pub tool_report: ToolReport,
    /// true ⇔ the scan degraded (unreadable/unparseable config) ⇒ exit 2.
    pub degraded: bool,
    /// Set when degradation came from a parse failure — the CLI renders a
    /// miette diagnostic (line/col only, never source text) from this.
    pub parse_failure: Option<ParseFailure>,
}

/// None ⇔ tool not detected (no codex home and no codex on PATH).
pub fn scan_codex(root: &DiscoveryRoot, rules: &[ValidatedRule]) -> Option<ScanResult> {
    let home_exists = root.codex_home_exists();
    let on_path = binary_on_path(root);
    if !home_exists && !on_path {
        return None;
    }

    let detected_version = detect_codex_version(root);
    let mut parse_failure = None;
    let mut config_paths = vec![];

    let config_state = match read_config(root) {
        ConfigReadOutcome::NoConfig => ConfigState::Missing,
        ConfigReadOutcome::Refused(reason) => {
            config_paths.push(root.config_path().to_string_lossy().into_owned());
            ConfigState::Unreadable(reason)
        }
        ConfigReadOutcome::Ok(text) => {
            config_paths.push(root.config_path().to_string_lossy().into_owned());
            match parse_config(&text) {
                Err(pf) => {
                    parse_failure = Some(pf.clone());
                    ConfigState::Unparseable(pf)
                }
                Ok(doc) => {
                    let mut extracted = BTreeMap::new();
                    for rule in rules {
                        let key = rule.raw.observation.key.clone();
                        let value = extract_key(&doc, &key);
                        extracted.insert(key, value);
                    }
                    // `doc` (all raw values) drops here; only rule-relevant
                    // extracted values survive, and those are allowlist-
                    // checked before any rendering.
                    ConfigState::Parsed(extracted)
                }
            }
        }
    };

    let degraded = matches!(config_state,
        ConfigState::Unreadable(_) | ConfigState::Unparseable(_));

    let mut findings: Vec<_> = rules.iter()
        .map(|r| evaluate_rule(r, &config_state, detected_version.as_deref()))
        .collect();
    findings.sort_by(|a, b| a.rule_id.cmp(&b.rule_id)); // deterministic (§7.2)

    let version_in_range = detected_version.as_deref()
        .map(|v| rules.iter().all(|r|
            crate::version::version_in_range(v, &r.raw.tested_versions)))
        .unwrap_or(false);

    let (last_verified_version, verified_date) = rules.first()
        .map(|r| {
            let tv = &r.raw.tested_versions[0];
            (Some(tv.max.clone()), Some(tv.verified_on.clone()))
        })
        .unwrap_or((None, None));

    let detection_confidence = match (&detected_version, home_exists) {
        (Some(_), true) => Confidence::High,
        (Some(_), false) | (None, true) => Confidence::Medium,
        (None, false) => Confidence::Low,
    };

    Some(ScanResult {
        tool_report: ToolReport {
            tool: "codex".to_string(),
            detected_version,
            config_paths,
            detection_confidence,
            rules_last_verified_version: last_verified_version,
            rules_verified_date: verified_date,
            version_in_range,
            findings,
        },
        degraded,
        parse_failure,
    })
}
```

Add to `lib.rs`: `pub mod evaluate;` and `pub mod scan;`. Add `#[derive(Clone)]` to `ParseFailure` if not already present (it is, from Task 7).

- [ ] **Step 5: Add scan-level tests** (bottom of `scan.rs`)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use harness_guard_rules::loader::load_rules;
    use harness_guard_rules::report::Status;

    #[test]
    fn undetected_tool_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let root = DiscoveryRoot {
            codex_home: dir.path().join("absent"), path_dirs: vec![] };
        assert!(scan_codex(&root, &load_rules()).is_none());
    }

    #[test]
    fn malformed_config_degrades_with_unknown_findings() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("codex-home");
        std::fs::create_dir_all(&home).unwrap();
        std::fs::write(home.join("config.toml"), "[history\n").unwrap();
        let root = DiscoveryRoot { codex_home: home, path_dirs: vec![] };
        let r = scan_codex(&root, &load_rules()).unwrap();
        assert!(r.degraded);
        assert!(r.parse_failure.is_some());
        assert!(r.tool_report.findings.iter()
            .all(|f| f.status == Status::Unknown));
    }

    #[test]
    fn findings_are_sorted_by_rule_id() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("codex-home");
        std::fs::create_dir_all(&home).unwrap();
        std::fs::write(home.join("config.toml"), "").unwrap();
        let root = DiscoveryRoot { codex_home: home, path_dirs: vec![] };
        let r = scan_codex(&root, &load_rules()).unwrap();
        let ids: Vec<_> = r.tool_report.findings.iter()
            .map(|f| f.rule_id.clone()).collect();
        let mut sorted = ids.clone();
        sorted.sort();
        assert_eq!(ids, sorted);
    }
}
```

- [ ] **Step 6: Run tests, then commit**

Run: `cargo test -p harness-guard-core` → all PASS. `cargo clippy --workspace --all-targets -- -D warnings` → green.

```bash
git add crates/harness-guard-core Cargo.lock
git commit -m "feat: rule evaluation with conservative degradation + codex scan orchestrator"
```

---

### Task 10: Synthetic fixture matrix (13 cases) + safety tripwires

**Files:**
- Create: `fixtures/codex/<case>/files/...` and `fixtures/codex/<case>/expected.json` for the 13 cases below
- Test: `crates/harness-guard-rules/tests/fixture_validation.rs`

**Interfaces:**
- Produces: the fixture trees every later test consumes. Convention (used by ALL tests): `DiscoveryRoot { codex_home: fixtures/codex/<case>/files/codex-home, path_dirs: vec![fixtures/codex/<case>/files/path] }`; for CLI integration tests: `CODEX_HOME=<case>/files/codex-home`, `PATH=<case>/files/path`.
- ALL content is synthetic. No symlinks, no >64 KiB files, no absolute paths, no usernames are ever committed (tripwire-tested). Runtime-mutated cases (`symlink-config`, `oversized`, `permission-denied`) commit only base files + `expected.json`; tests copy to a tempdir and mutate there (Task 14).

- [ ] **Step 1: Create the shared version-marker tree** (used by every case that must be IN version range)

For each case marked `[npm@0.144.4]` below, create `files/path/` containing:

`files/path/codex` (regular file — models the npm shim; the walk-up finds the sibling package.json; committing a plain file instead of a symlink keeps Windows checkouts sane):

```text
#!/usr/bin/env node
// synthetic fixture shim — never executed by harness-guard
```

`files/path/package.json`:

```json
{ "name": "@openai/codex", "version": "0.144.4" }
```

- [ ] **Step 2: Create the 13 cases**

| case | files/codex-home/config.toml | version tree | expected status | exit (pinned in Task 11/14) |
|---|---|---|---|---|
| `missing` | no codex-home dir at all (`files/.gitkeep` only) | none | tool not detected | 0 |
| `minimal` | empty file (0 bytes) | [npm@0.144.4] | finding/warning (unset default) | 1 |
| `hardened` | `[history]`, `persistence = "none"` | [npm@0.144.4] | pass | 0 |
| `risky-unset` | `model = "gpt-5-codex"` (no `[history]`) | [npm@0.144.4] | finding/warning | 1 |
| `risky-explicit` | `[history]`, `persistence = "save-all"` | [npm@0.144.4] | finding/warning | 1 |
| `malformed-toml` | `[history` (unclosed) + second line `persistence = "none"` | [npm@0.144.4] | unknown | 2 |
| `unrecognized-value` | `[history]`, `persistence = "archive"` | [npm@0.144.4] | unknown, value NEVER echoed | 0 |
| `symlink-config` | commit `real-config.toml` (`persistence = "none"`); test symlinks `config.toml` → it at runtime | [npm@0.144.4] | unknown | 2 |
| `oversized` | committed empty `codex-home/.gitkeep`; test writes >1 MiB config at runtime | [npm@0.144.4] | unknown | 2 |
| `deep-nesting` | generated 40-deep inline-table file (below) | [npm@0.144.4] | unknown | 2 |
| `permission-denied` | `[history]`, `persistence = "none"`; test chmods 000 at runtime | [npm@0.144.4] | unknown | 2 |
| `unknown-version` | `[history]`, `persistence = "none"` | `files/path/codex` only, NO package.json | stale-ruleset ("version not detected") | 0 |
| `version-out-of-range` | `[history]`, `persistence = "none"` | npm tree but `"version": "9.9.9"` | stale-ruleset | 0 |

The last two plus `unrecognized-value` are the conservative-degradation pins (§5.4 / §10.2): a "safe-looking" config must still surface as stale/unknown when the ruleset can't vouch for it.

Generate `deep-nesting`'s config (commit the output, it is ~400 bytes):

```bash
python3 - <<'EOF' > fixtures/codex/deep-nesting/files/codex-home/config.toml
print("a = " + "{a = " * 40 + "1" + "}" * 40)
EOF
```

- [ ] **Step 3: Write each `expected.json`**

Full example — `fixtures/codex/risky-unset/expected.json` (the others follow the same shape; subset-matching means only the asserted keys appear):

```json
{
  "schema_version": "1.0",
  "case": "risky-unset",
  "description": "Config exists, [history] absent; documented default save-all applies => warning finding.",
  "expected_report": {
    "network_requests_made": 0,
    "tools": [
      {
        "tool": "codex",
        "detected_version": "0.144.4",
        "detection_confidence": "high",
        "version_in_range": true,
        "findings": [
          {
            "rule_id": "codex-history-persist-01",
            "status": "finding",
            "severity": "warning",
            "confidence": "high",
            "evidence_class": "official-documentation",
            "observation": "history.persistence unset (documented default \"save-all\" applies)"
          }
        ]
      }
    ],
    "summary": { "tools_scanned": 1, "warning": 1, "info": 0, "unknown": 0, "stale": 0, "passed": 0 }
  }
}
```

The remaining 12 `expected_report` values (write each into its `expected.json` with matching `case`/`description`; keys not listed here match `risky-unset`'s pattern):

- `missing`: `{ "tools": [], "summary": { "tools_scanned": 0, "warning": 0, "info": 0, "unknown": 0, "stale": 0, "passed": 0 } }`
- `minimal`: finding/warning, `observation` = the unset rendering, summary `warning: 1`.
- `hardened`: `status: "pass"`, `severity: null`, `confidence: "high"`, `observation: "history.persistence = \"none\""`, summary `passed: 1`.
- `risky-explicit`: finding/warning, `observation: "history.persistence = \"save-all\""`, summary `warning: 1`.
- `malformed-toml`: finding `status: "unknown"`, `severity: null`, `confidence: null`, `source: null`, `observation: null`, summary `unknown: 1`.
- `unrecognized-value`: `status: "unknown"`, `observation: null`, `unknown_reason` present (assert non-null via the literal string from evaluate.rs), summary `unknown: 1`.
- `symlink-config` / `oversized` / `permission-denied`: `status: "unknown"`, summary `unknown: 1` (their `unknown_reason` strings: symlink / 1 MiB bound / permission denied phrasing from `RefusalReason::describe`).
- `deep-nesting`: `status: "unknown"`, summary `unknown: 1`.
- `unknown-version`: `detected_version: null`, `version_in_range: false`, finding `status: "stale-ruleset"`, `stale_reason` containing `"not detected"`, `source` non-null, summary `stale: 1`.
- `version-out-of-range`: `detected_version: "9.9.9"`, `version_in_range: false`, `status: "stale-ruleset"`, summary `stale: 1`.

- [ ] **Step 4: Write the tripwire + validation tests** (`crates/harness-guard-rules/tests/fixture_validation.rs`)

```rust
use std::path::{Path, PathBuf};

fn fixtures_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures").canonicalize().unwrap()
}

#[test]
fn every_expected_json_validates_against_fixture_schema() {
    let schema_raw = std::fs::read_to_string(
        fixtures_root().join("../schemas/fixture.schema.json")).unwrap();
    let schema: serde_json::Value = serde_json::from_str(&schema_raw).unwrap();
    let v = jsonschema::validator_for(&schema).unwrap();
    let mut count = 0;
    for case in std::fs::read_dir(fixtures_root().join("codex")).unwrap() {
        let dir = case.unwrap().path();
        if !dir.is_dir() { continue; }
        let exp = dir.join("expected.json");
        let json: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(&exp).unwrap()).unwrap();
        assert!(v.validate(&json).is_ok(), "invalid {exp:?}");
        assert_eq!(json["case"], dir.file_name().unwrap().to_str().unwrap(),
            "case field must equal directory name");
        count += 1;
    }
    assert_eq!(count, 13, "the §10.2 matrix has exactly 13 cases");
}

#[test]
fn fixtures_contain_no_real_machine_leakage() {
    // §10.3: no absolute paths escaping the fixture dir, no usernames,
    // no symlinks committed, nothing oversized.
    let mut stack = vec![fixtures_root()];
    while let Some(dir) = stack.pop() {
        for e in std::fs::read_dir(&dir).unwrap() {
            let p = e.unwrap().path();
            let meta = std::fs::symlink_metadata(&p).unwrap();
            assert!(!meta.file_type().is_symlink(),
                "committed symlink at {p:?} — symlinks are created at test runtime only");
            if meta.is_dir() { stack.push(p); continue; }
            assert!(meta.len() < 64 * 1024, "oversized committed fixture {p:?}");
            let bytes = std::fs::read(&p).unwrap();
            if let Ok(text) = String::from_utf8(bytes) {
                for needle in ["/Users/", "/home/", "C:\\Users", "CODEX_HOME="] {
                    assert!(!text.contains(needle),
                        "fixture {p:?} contains forbidden fragment {needle:?}");
                }
            }
        }
    }
}
```

- [ ] **Step 5: Run, then commit**

Run: `cargo test -p harness-guard-rules` → all PASS (13 cases found, tripwires green).

```bash
git add fixtures/ crates/harness-guard-rules/tests/fixture_validation.rs
git commit -m "feat: 13-case synthetic fixture matrix with goldens + real-config leakage tripwires"
```

---

### Task 11: CLI — `scan`, JSON view, exit codes, env/root resolution

**Files:**
- Create: `crates/harness-guard-cli/src/redact.rs`
- Create: `crates/harness-guard-cli/src/render_json.rs`
- Modify: `crates/harness-guard-cli/src/main.rs`
- Test: `crates/harness-guard-cli/tests/common/mod.rs`, `crates/harness-guard-cli/tests/scan_fixtures.rs`
- Modify: `crates/harness-guard-cli/Cargo.toml` (dev-dep `jsonschema = { version = "0.30", default-features = false }`)

**Interfaces:**
- Consumes: `scan_codex`/`ScanResult` (Task 9), `load_rules`/`ruleset_version` (Task 6), report structs (Task 6), fixtures (Task 10).
- Produces: binary `harness-guard`; `scan` flags `--tool <id>... --json --min-severity <info|warning> --fail-on <never|info|warning> --color <auto|always|never> --quiet --verbose`; exit codes 0/1/2; `build_report(results, home) -> Report` and `redact::redact_home(&str, Option<&Path>) -> String` reused by Task 12/13.
- Decisions pinned here: `--min-severity` filters TERMINAL finding-blocks only — `--json` always emits the full report (the report schema IS the contract; filtering it would create a second shape). `--tool` accepts only `codex` (clap `value_parser` — anything else is a usage error, exit 2). Exit 2 takes precedence over 1; a degraded scan still prints the full report first.

- [ ] **Step 1: Write the failing integration tests**

`tests/common/mod.rs`:

```rust
use std::path::{Path, PathBuf};
use std::process::Output;

pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap()
}

pub fn fixture(case: &str) -> PathBuf {
    repo_root().join("fixtures/codex").join(case).join("files")
}

/// Runs the built binary against a fixture root. CODEX_HOME/PATH point ONLY
/// at fixture paths — the real ~/.codex is unreachable by construction (§10.3).
pub fn run_in(files_root: &Path, args: &[&str]) -> Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_harness-guard"))
        .args(args)
        .env_clear()
        .env("CODEX_HOME", files_root.join("codex-home"))
        .env("PATH", files_root.join("path"))
        .env("NO_COLOR", "1")
        .env("HOME", files_root) // any $HOME-derived rendering stays inside the fixture
        .output()
        .expect("binary runs")
}

pub fn run_case(case: &str, args: &[&str]) -> Output {
    run_in(&fixture(case), args)
}

/// Recursive subset assertion: every key present in `expected` must exist
/// and match in `actual`. Arrays match element-by-element as subsets.
pub fn assert_json_subset(expected: &serde_json::Value, actual: &serde_json::Value, path: &str) {
    use serde_json::Value;
    match (expected, actual) {
        (Value::Object(e), Value::Object(a)) => {
            for (k, ev) in e {
                let av = a.get(k).unwrap_or_else(||
                    panic!("missing key {path}.{k}"));
                assert_json_subset(ev, av, &format!("{path}.{k}"));
            }
        }
        (Value::Array(e), Value::Array(a)) => {
            assert_eq!(e.len(), a.len(), "array length mismatch at {path}");
            for (i, (ev, av)) in e.iter().zip(a).enumerate() {
                assert_json_subset(ev, av, &format!("{path}[{i}]"));
            }
        }
        _ => assert_eq!(expected, actual, "value mismatch at {path}"),
    }
}
```

`tests/scan_fixtures.rs`:

```rust
mod common;
use common::*;

/// Committed-fixture cases: (case, expected exit code). Runtime-mutated
/// cases (symlink/oversized/permission) are covered in hostile.rs (Task 14).
const CASES: &[(&str, i32)] = &[
    ("missing", 0),
    ("minimal", 1),
    ("hardened", 0),
    ("risky-unset", 1),
    ("risky-explicit", 1),
    ("malformed-toml", 2),
    ("unrecognized-value", 0),
    ("deep-nesting", 2),
    ("unknown-version", 0),
    ("version-out-of-range", 0),
];

#[test]
fn fixture_exit_codes_and_json_goldens() {
    for (case, want_exit) in CASES {
        let out = run_case(case, &["scan", "--json"]);
        assert_eq!(out.status.code(), Some(*want_exit), "exit code for {case}");
        let report: serde_json::Value = serde_json::from_slice(&out.stdout)
            .unwrap_or_else(|e| panic!("{case}: --json must emit valid JSON even degraded: {e}"));
        let expected: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(
                repo_root().join("fixtures/codex").join(case).join("expected.json")).unwrap()).unwrap();
        assert_json_subset(&expected["expected_report"], &report, case);
    }
}

#[test]
fn json_report_validates_against_report_schema() {
    let out = run_case("risky-unset", &["scan", "--json"]);
    let report: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let schema: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(repo_root().join("schemas/report.schema.json")).unwrap()).unwrap();
    let v = jsonschema::validator_for(&schema).unwrap();
    assert!(v.validate(&report).is_ok(), "{:?}",
        v.iter_errors(&report).map(|e| e.to_string()).collect::<Vec<_>>());
}

#[test]
fn fail_on_semantics() {
    // warning findings, --fail-on never => 0
    assert_eq!(run_case("risky-unset", &["scan", "--fail-on", "never"]).status.code(), Some(0));
    // unknown/stale never fail by default
    assert_eq!(run_case("unrecognized-value", &["scan"]).status.code(), Some(0));
    assert_eq!(run_case("unknown-version", &["scan"]).status.code(), Some(0));
    // degraded scan: exit 2 wins even with --fail-on never
    assert_eq!(run_case("malformed-toml", &["scan", "--fail-on", "never"]).status.code(), Some(2));
}

#[test]
fn unknown_tool_flag_is_usage_error() {
    let out = run_case("hardened", &["scan", "--tool", "cursor"]);
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn raw_values_never_echo_anywhere() {
    for args in [vec!["scan"], vec!["scan", "--json"], vec!["scan", "--verbose"]] {
        let out = run_case("unrecognized-value", &args);
        let all = format!("{}{}",
            String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
        assert!(!all.contains("archive"), "raw config value leaked via {args:?}");
    }
}

#[test]
fn output_paths_are_redacted() {
    let out = run_case("risky-unset", &["scan", "--json"]);
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(!text.contains("/Users/"), "absolute home path leaked");
    assert!(!text.contains("\"HOME\""));
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p harness-guard-cli --test scan_fixtures` → FAILS (no subcommands yet).

- [ ] **Step 3: Implement `redact.rs`**

```rust
//! §7.3: home directories render as `~`; usernames never appear.
use std::path::Path;

pub fn redact_home(path: &str, home: Option<&Path>) -> String {
    if let Some(home) = home {
        let home_str = home.to_string_lossy();
        if let Some(rest) = path.strip_prefix(home_str.as_ref()) {
            return format!("~{rest}");
        }
    }
    path.to_string()
}
```

- [ ] **Step 4: Implement `main.rs` (scan path + JSON view)**

```rust
mod redact;
mod render_json;

use clap::{Parser, Subcommand, ValueEnum};
use harness_guard_core::discovery::DiscoveryRoot;
use harness_guard_core::scan::{scan_codex, ScanResult};
use harness_guard_rules::loader::{load_rules, ruleset_version};
use harness_guard_rules::report::{Platform, Report, Severity, Status, Summary};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// local, execution-free, per-finding-cited config auditor for
/// privacy/retention/telemetry posture
#[derive(Parser)]
#[command(name = "harness-guard", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
    #[command(flatten)]
    color: colorchoice_clap::Color,
}

#[derive(Subcommand)]
enum Cmd {
    /// Scan detected tools' local config (reads files only; never executes tools)
    Scan(ScanArgs),
    /// Show detected tools, versions, and config paths — no rule evaluation
    List,
    /// Show a rule's full evidence record (works offline)
    Explain { rule_id: String },
    /// Show binary version and ruleset version separately
    Version,
    /// Generate shell completions
    Completions { shell: clap_complete::Shell },
}

#[derive(clap::Args)]
struct ScanArgs {
    /// Restrict to specific tools (v1: only `codex` is implemented)
    #[arg(long, value_parser = ["codex"])]
    tool: Vec<String>,
    /// Emit the sanitized report as JSON (the schemas/report contract)
    #[arg(long)]
    json: bool,
    /// Lowest finding severity to display (never hides unknown/stale blocks)
    #[arg(long, value_enum, default_value_t = MinSeverity::Info)]
    min_severity: MinSeverity,
    /// Findings at or above this severity set exit code 1
    #[arg(long, value_enum, default_value_t = FailOn::Warning)]
    fail_on: FailOn,
    /// Suppress header, detection block, and banners
    #[arg(long)]
    quiet: bool,
    /// Additionally itemize pass results
    #[arg(long)]
    verbose: bool,
}

#[derive(Clone, Copy, ValueEnum, PartialEq)]
enum MinSeverity { Info, Warning }

#[derive(Clone, Copy, ValueEnum, PartialEq)]
enum FailOn { Never, Info, Warning }

fn main() -> ExitCode {
    let cli = Cli::parse();
    cli.color.write_global();
    match cli.cmd {
        Cmd::Scan(args) => cmd_scan(args),
        Cmd::List => todo!("Task 13"),
        Cmd::Explain { rule_id } => todo!("Task 13"),
        Cmd::Version => todo!("Task 13"),
        Cmd::Completions { shell } => todo!("Task 13"),
    }
}

fn discovery_root_from_env() -> (DiscoveryRoot, Option<PathBuf>) {
    // The ONLY place ambient environment is read (§9). Core never can.
    let home = directories::BaseDirs::new().map(|b| b.home_dir().to_path_buf());
    let codex_home = std::env::var_os("CODEX_HOME")
        .map(PathBuf::from)
        .or_else(|| home.as_ref().map(|h| h.join(".codex")))
        .unwrap_or_else(|| PathBuf::from(".codex"));
    let path_dirs = std::env::var_os("PATH")
        .map(|p| std::env::split_paths(&p).collect())
        .unwrap_or_default();
    (DiscoveryRoot { codex_home, path_dirs }, home)
}

fn cmd_scan(args: ScanArgs) -> ExitCode {
    let (root, home) = discovery_root_from_env();
    let rules = load_rules();
    let results: Vec<ScanResult> = scan_codex(&root, &rules).into_iter().collect();
    let degraded = results.iter().any(|r| r.degraded);
    let parse_failures: Vec<_> = results.iter()
        .filter_map(|r| r.parse_failure.clone()).collect();

    let report = build_report(&results, home.as_deref());

    if args.json {
        println!("{}", render_json::render(&report));
    } else {
        // Task 12 replaces this stub with the §7.1 renderer.
        render_terminal_stub(&report, &args);
    }
    for pf in &parse_failures {
        // Task 14 renders these via miette (line/col only, no source text).
        eprintln!("config parse failure at line {:?} col {:?}: {}",
            pf.line, pf.col, pf.message);
    }

    let threshold = match args.fail_on {
        FailOn::Never => None,
        FailOn::Info => Some(Severity::Info),
        FailOn::Warning => Some(Severity::Warning),
    };
    let failing = threshold.is_some_and(|t| report.tools.iter().any(|tool|
        tool.findings.iter().any(|f|
            f.status == Status::Finding && f.severity.is_some_and(|s| s >= t))));

    if degraded { ExitCode::from(2) }
    else if failing { ExitCode::from(1) }
    else { ExitCode::SUCCESS }
}

fn build_report(results: &[ScanResult], home: Option<&Path>) -> Report {
    let mut tools: Vec<_> = results.iter().map(|r| {
        let mut t = r.tool_report.clone();
        t.config_paths = t.config_paths.iter()
            .map(|p| redact::redact_home(p, home)).collect();
        t
    }).collect();
    tools.sort_by(|a, b| a.tool.cmp(&b.tool)); // deterministic (§7.2)
    let summary = Summary::from_tools(&tools);
    let scanned_at = time::OffsetDateTime::now_local()
        .unwrap_or_else(|_| time::OffsetDateTime::now_utc())
        .format(&time::format_description::well_known::Rfc3339)
        .expect("RFC3339 formatting");
    Report {
        schema_version: "1.0".to_string(),
        harness_guard_version: env!("CARGO_PKG_VERSION").to_string(),
        ruleset_version: ruleset_version(),
        scanned_at,
        network_requests_made: 0,
        platform: Platform { os: current_os() },
        tools,
        summary,
    }
}

fn current_os() -> String {
    if cfg!(target_os = "macos") { "macos" }
    else if cfg!(target_os = "windows") { "windows" }
    else { "linux" }.to_string()
}

fn render_terminal_stub(report: &Report, _args: &ScanArgs) {
    // Minimal placeholder so scan_fixtures tests pass before Task 12.
    println!("{} warning · {} info · {} unknown · {} stale · {} passed — 0 network requests made",
        report.summary.warning, report.summary.info, report.summary.unknown,
        report.summary.stale, report.summary.passed);
}
```

`render_json.rs`:

```rust
//! §7.2: --json serializes the same Report struct the terminal view reads —
//! the two views cannot drift.
use harness_guard_rules::report::Report;

pub fn render(report: &Report) -> String {
    serde_json::to_string_pretty(report).expect("Report is always serializable")
}
```

Note: `ScanArgs.tool` is accepted and validated (only `codex`) but with one tool implemented it does not change behavior yet; the detection loop is already tool-keyed via `scan_codex`. Add `#[derive(Clone)]` to `ToolReport` and `FindingRecord` in Task 6's `report.rs` if the compiler asks (they carry it already via the derive list above).

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p harness-guard-cli --test scan_fixtures` → all PASS.
Run: `cargo clippy --workspace --all-targets -- -D warnings` (the `todo!` arms are allowed — clippy's `todo` lint is not denied by default) and `cargo deny check` → green.

- [ ] **Step 6: Commit**

```bash
git add crates/harness-guard-cli Cargo.lock
git commit -m "feat: scan subcommand — env root resolution, JSON view, redaction, ruff-style exit codes"
```

---

### Task 12: Terminal rendering (§7.1) + snapshot goldens

**Files:**
- Create: `crates/harness-guard-cli/src/render_term.rs`
- Modify: `crates/harness-guard-cli/src/main.rs` (replace `render_terminal_stub`)
- Test: `crates/harness-guard-cli/tests/scan_snapshots.rs`

**Interfaces:**
- Consumes: `Report`/`FindingRecord` (Task 6), `ScanArgs` fields `min_severity`/`quiet`/`verbose` (Task 11).
- Produces: `render_term::render(report: &Report, opts: &TermOpts) -> String` where `TermOpts { min_severity: Option<Severity>, quiet: bool, verbose: bool }`; `main.rs` writes it via `anstream::println!` so `--color`/`NO_COLOR`/TTY handling is library-driven (never hand-rolled).

**Exact layout (§7.1, all eight elements; glyphs fixed here):**

```text
harness-guard 0.1.0 · ruleset 2026.07.14 · scanned 2026-07-14T10:00:00+02:00 · no network requests made

detected tools
  ● codex 0.144.4 · config ~/.codex/config.toml · confidence high

codex 0.144.4 — rules verified ≤0.144.4 · 2026-07-14
1 warning · 0 info · 0 unknown · 0 stale · 0 passed

!! WARNING: Codex CLI persists full session history to disk in plaintext with no expiry.
   rule codex-history-persist-01 · official-documentation
   observed: history.persistence unset (documented default "save-all" applies)
   fix: Add to ~/.codex/config.toml:
        [history]
        persistence = "none"
   = source: https://developers.openai.com/codex/config-reference (2026-07-14)
   = harness-guard explain codex-history-persist-01

1 warning · 0 info · 0 unknown · 0 stale · 0 passed — 0 network requests made
No numeric score is produced — read findings individually.
```

Variants:
- Not-detected tool line: `○ codex — not detected`.
- Version undetected/out-of-range banner (hint-styled, dim — never an error), directly under the tool header: `rules verified ≤0.144.4 — you have 9.9.9, showing last-known rules as unverified` (or `— version not detected, showing last-known rules as unverified`).
- `unknown` block: `?? UNKNOWN: <message>` then `   reason: <unknown_reason>`, then `   verify: <verify_url>` when present, then the `= harness-guard explain …` line. (The account/server-state reframe sentence ships with the first rule that has account-state unknowns — no such rule exists in this slice; the `unknown_reason` text carries the why.)
- `stale-ruleset` block: `~ UNVERIFIED (stale ruleset): <message>` then `   reason: <stale_reason>`, then the source line (it renders as unverified because the label says so), then the explain line.
- `pass` (only with `--verbose`): `ok PASS: <message>` + rule/source lines.
- `--quiet`: header + detection block + banners suppressed; finding/unknown/stale blocks + final one-line summary kept.
- `--min-severity warning`: `info`-severity finding blocks hidden; `unknown`/`stale` blocks NEVER hidden (silent omission must not read as "verified safe").

**Color discipline (§7.1):** exactly one strong color — `!! WARNING` label red+bold via `owo_colors`; `??` cyan; `~` + `UNVERIFIED (stale ruleset)` yellow+dimmed; green only on the `passed` count and `ok PASS` label; everything else default. Implement with `owo_colors::OwoColorize` on the label substrings only.

- [ ] **Step 1: Write failing snapshot tests** (`tests/scan_snapshots.rs`)

```rust
mod common;
use common::*;

fn snap(case: &str, args: &[&str], name: &str) {
    let out = run_case(case, args);
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    insta::with_settings!({filters => vec![
        // timestamps vary per run
        (r"\d{4}-\d{2}-\d{2}T[0-9:.+\-Z]+", "[TIMESTAMP]"),
        // fixture path (under the test $HOME) varies per checkout
        (r"~[^\s]*codex-home[^\s]*", "[CONFIG_PATH]"),
    ]}, {
        insta::assert_snapshot!(name, text);
    });
}

#[test] fn term_risky_unset() { snap("risky-unset", &["scan"], "risky_unset"); }
#[test] fn term_hardened_verbose() { snap("hardened", &["scan", "--verbose"], "hardened_verbose"); }
#[test] fn term_hardened_default_hides_pass_blocks() {
    let out = run_case("hardened", &["scan"]);
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(!text.contains("PASS:"), "default output shows passes only as a count");
    assert!(text.contains("1 passed"));
}
#[test] fn term_unrecognized_value() { snap("unrecognized-value", &["scan"], "unknown_value"); }
#[test] fn term_unknown_version_banner() { snap("unknown-version", &["scan"], "stale_banner"); }
#[test] fn term_version_out_of_range() { snap("version-out-of-range", &["scan"], "stale_out_of_range"); }
#[test] fn term_missing() { snap("missing", &["scan"], "missing"); }
#[test] fn term_quiet() { snap("risky-unset", &["scan", "--quiet"], "risky_unset_quiet"); }

#[test]
fn min_severity_never_hides_unknown_or_stale() {
    let out = run_case("unrecognized-value", &["scan", "--min-severity", "warning"]);
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("?? UNKNOWN"), "--min-severity must not hide unknown blocks");
    let out = run_case("unknown-version", &["scan", "--min-severity", "warning"]);
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("UNVERIFIED (stale ruleset)"));
}

#[test]
fn citations_appear_in_default_output() {
    let out = run_case("risky-unset", &["scan"]);
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("= source: https://"), "citation must be in DEFAULT output");
    assert!(text.contains("("), "retrieved date shown with the citation");
    assert!(text.contains("= harness-guard explain codex-history-persist-01"));
    assert!(text.contains("No numeric score is produced"));
    assert!(text.contains("no network requests made"));
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p harness-guard-cli --test scan_snapshots` → FAILS (stub output).

- [ ] **Step 3: Implement `render_term.rs`**

```rust
//! §7.1 terminal view. Same Report struct as --json — no drift possible.
use harness_guard_rules::report::{FindingRecord, Report, Severity, Status, ToolReport};
use owo_colors::OwoColorize;
use std::fmt::Write;

pub struct TermOpts {
    pub min_severity: Option<Severity>, // None = show all finding blocks
    pub quiet: bool,
    pub verbose: bool,
}

pub fn render(report: &Report, opts: &TermOpts) -> String {
    let mut o = String::new();
    if !opts.quiet {
        let _ = writeln!(o,
            "harness-guard {} · ruleset {} · scanned {} · no network requests made\n",
            report.harness_guard_version, report.ruleset_version, report.scanned_at);
        let _ = writeln!(o, "detected tools");
        if report.tools.is_empty() {
            let _ = writeln!(o, "  ○ codex — not detected");
        }
        for t in &report.tools {
            let ver = t.detected_version.as_deref().unwrap_or("version not detected");
            let path = t.config_paths.first().map(String::as_str).unwrap_or("no config file");
            let _ = writeln!(o, "  ● {} {} · config {} · confidence {:?}",
                t.tool, ver, path, t.detection_confidence);
        }
        let _ = writeln!(o);
    }
    for t in &report.tools {
        render_tool(&mut o, t, opts);
    }
    let s = &report.summary;
    let _ = writeln!(o,
        "{} warning · {} info · {} unknown · {} stale · {} passed — 0 network requests made",
        s.warning, s.info, s.unknown, s.stale, s.passed.green());
    let _ = writeln!(o, "No numeric score is produced — read findings individually.");
    o
}

fn render_tool(o: &mut String, t: &ToolReport, opts: &TermOpts) {
    if !opts.quiet {
        let ver = t.detected_version.as_deref().unwrap_or("version not detected");
        let _ = writeln!(o, "{} {} — rules verified ≤{} · {}",
            t.tool, ver,
            t.rules_last_verified_version.as_deref().unwrap_or("?"),
            t.rules_verified_date.as_deref().unwrap_or("?"));
        if !t.version_in_range {
            // hint-styled banner, never an error (§7.1 point 3)
            let have = t.detected_version.as_deref()
                .map(|v| format!("you have {v}"))
                .unwrap_or_else(|| "version not detected".to_string());
            let _ = writeln!(o, "{}", format!(
                "rules verified ≤{} — {have}, showing last-known rules as unverified",
                t.rules_last_verified_version.as_deref().unwrap_or("?")).dimmed());
        }
        let counts = harness_guard_rules::report::Summary::from_tools(
            std::slice::from_ref(t));
        let _ = writeln!(o, "{} warning · {} info · {} unknown · {} stale · {} passed\n",
            counts.warning, counts.info, counts.unknown, counts.stale, counts.passed);
    }
    for f in &t.findings {
        render_finding(o, f, opts);
    }
}

fn render_finding(o: &mut String, f: &FindingRecord, opts: &TermOpts) {
    match f.status {
        Status::Finding => {
            if let (Some(min), Some(sev)) = (opts.min_severity, f.severity) {
                if sev < min { return; } // filters FINDING blocks only
            }
            let label = match f.severity {
                Some(Severity::Warning) => format!("{}", "!! WARNING:".red().bold()),
                _ => "-- INFO:".to_string(),
            };
            let _ = writeln!(o, "{label} {}", f.message);
            common_lines(o, f);
        }
        Status::Unknown => {
            let _ = writeln!(o, "{} {}", "?? UNKNOWN:".cyan(), f.message);
            if let Some(r) = &f.unknown_reason {
                let _ = writeln!(o, "   reason: {r}");
            }
            if let Some(v) = &f.verify_url {
                let _ = writeln!(o, "   verify: {v}");
            }
            let _ = writeln!(o, "   = harness-guard explain {}\n", f.rule_id);
        }
        Status::StaleRuleset => {
            let _ = writeln!(o, "{} {}",
                "~ UNVERIFIED (stale ruleset):".yellow().dimmed(), f.message);
            if let Some(r) = &f.stale_reason {
                let _ = writeln!(o, "   reason: {r}");
            }
            common_lines(o, f);
        }
        Status::Pass => {
            if !opts.verbose { return; } // default: passes are a count only
            let _ = writeln!(o, "{} {}", "ok PASS:".green(), f.message);
            common_lines(o, f);
        }
    }
}

fn common_lines(o: &mut String, f: &FindingRecord) {
    let _ = writeln!(o, "   rule {} · {}", f.rule_id,
        f.evidence_class.as_deref().unwrap_or("unverified"));
    if let Some(obs) = &f.observation {
        let _ = writeln!(o, "   observed: {obs}");
    }
    if let Some(rem) = &f.remediation {
        let mut lines = rem.command.lines();
        if let Some(first) = lines.next() {
            let _ = writeln!(o, "   fix: {first}");
            for l in lines { let _ = writeln!(o, "        {l}"); }
        }
    }
    if let Some(src) = &f.source {
        let _ = writeln!(o, "   = source: {} ({})", src.url, src.retrieved);
    }
    let _ = writeln!(o, "   = harness-guard explain {}\n", f.rule_id);
}
```

In `main.rs`, delete `render_terminal_stub` and replace its call site:

```rust
        let opts = render_term::TermOpts {
            min_severity: match args.min_severity {
                MinSeverity::Info => None,
                MinSeverity::Warning => Some(Severity::Warning),
            },
            quiet: args.quiet,
            verbose: args.verbose,
        };
        anstream::print!("{}", render_term::render(&report, &opts));
```

Add `mod render_term;` to `main.rs`.

- [ ] **Step 4: Run, review snapshots, verify tests pass**

Run: `cargo test -p harness-guard-cli` → snapshot tests create `.snap.new` files on first run. Run `cargo insta review` (or `INSTA_UPDATE=auto cargo test -p harness-guard-cli`) — READ each snapshot against the §7.1 checklist above (all 8 elements; no raw values; no absolute paths; citation present) before accepting. Then `cargo test -p harness-guard-cli` → all PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/harness-guard-cli
git commit -m "feat: §7.1 terminal renderer with color discipline, banners, and snapshot goldens"
```

---

### Task 13: `list`, `explain`, `version`, `completions`

**Files:**
- Create: `crates/harness-guard-cli/src/explain.rs`
- Modify: `crates/harness-guard-cli/src/main.rs` (fill the three `todo!` arms + `Completions`)
- Test: `crates/harness-guard-cli/tests/cli_surface.rs`

**Interfaces:**
- Consumes: `load_rules`/`ruleset_version` (Task 6), `scan_codex` detection fields + `binary_on_path`/`detect_codex_version` via core (Tasks 8–9), `redact_home` (Task 11).
- Produces: `explain::render_rule(rule: &ValidatedRule) -> String`, `explain::nearest<'a>(needle: &str, ids: &[&'a str]) -> Option<&'a str>` (hand-rolled Levenshtein — the fixed crate stack has no strsim).

- [ ] **Step 1: Write failing tests** (`tests/cli_surface.rs`)

```rust
mod common;
use common::*;

#[test]
fn list_shows_detection_only() {
    let out = run_case("hardened", &["list"]);
    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("codex"));
    assert!(text.contains("0.144.4"));
    assert!(!text.contains("codex-history-persist-01"), "list never evaluates rules");
    assert!(!text.contains("/Users/"), "paths must be redacted");
}

#[test]
fn list_reports_version_not_detected() {
    let out = run_case("unknown-version", &["list"]);
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("version not detected"));
}

#[test]
fn explain_shows_full_evidence_record() {
    let out = run_case("hardened", &["explain", "codex-history-persist-01"]);
    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8_lossy(&out.stdout);
    for needle in [
        "codex-history-persist-01",
        "official-documentation",
        "content_hash", "sha256:",
        "retrieved",
        "archived", "web.archive.org",
        "tested versions", "<=0.144.4", "verified",
        "limitations",
        "unknown conditions",
        "why it matters",
    ] {
        assert!(text.to_lowercase().contains(&needle.to_lowercase()),
            "explain output missing {needle:?}");
    }
}

#[test]
fn explain_unknown_rule_suggests_nearest_and_exits_2() {
    let out = run_case("hardened", &["explain", "codex-history-persist-02"]);
    assert_eq!(out.status.code(), Some(2));
    let text = String::from_utf8_lossy(&out.stderr);
    assert!(text.contains("codex-history-persist-01"), "nearest-match suggestion expected");
}

#[test]
fn version_reports_binary_and_ruleset_separately() {
    let out = run_case("hardened", &["version"]);
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("harness-guard 0.1.0"));
    assert!(text.contains("ruleset"));
}

#[test]
fn completions_emit_something() {
    let out = run_case("hardened", &["completions", "bash"]);
    assert_eq!(out.status.code(), Some(0));
    assert!(!out.stdout.is_empty());
}

#[test]
fn help_uses_positioning_never_forbidden_phrase() {
    for args in [vec!["--help"], vec!["scan", "--help"], vec!["list", "--help"],
                 vec!["explain", "--help"], vec!["version", "--help"]] {
        let out = run_case("hardened", &args);
        let text = format!("{}{}",
            String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
        assert!(!text.contains("AI agent security scanner"),
            "forbidden positioning phrase in {args:?}");
    }
    let out = run_case("hardened", &["--help"]);
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("local, execution-free, per-finding-cited config auditor"),
        "binding positioning phrase must appear in top-level help");
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p harness-guard-cli --test cli_surface` → FAILS (`todo!` panics → non-zero exits and missing text).

- [ ] **Step 3: Implement `explain.rs`**

```rust
//! `explain <rule-id>`: the full evidence record from bundled data (§6),
//! rustc --explain pattern. Works offline by construction.
use harness_guard_rules::loader::ValidatedRule;
use std::fmt::Write;

pub fn render_rule(rule: &ValidatedRule) -> String {
    let r = &rule.raw;
    let mut o = String::new();
    let _ = writeln!(o, "{} — {}\n", r.id, r.title);
    let _ = writeln!(o, "why it matters\n  {}\n", r.why_it_matters);
    let _ = writeln!(o, "observes\n  {} · key {} · rendered values: {}\n",
        r.observation.file, r.observation.key, r.observation.allowed_render.join(", "));
    let _ = writeln!(o, "outcomes");
    for oc in &r.outcomes {
        let sev = oc.severity.as_deref().unwrap_or("-");
        let _ = writeln!(o, "  [{}/{}] when {}\n      {}", oc.status, sev, oc.when, oc.message);
        if let Some(rem) = &oc.remediation {
            let _ = writeln!(o, "      fix: {}", rem.summary);
        }
    }
    let _ = writeln!(o, "\ntested versions");
    for tv in &r.tested_versions {
        let _ = writeln!(o, "  {} → {} (verified on {})", tv.min, tv.max, tv.verified_on);
    }
    let _ = writeln!(o, "\nsources");
    for s in &r.sources {
        let _ = writeln!(o, "  {} — {} ({})", s.publisher, s.title, s.evidence_class);
        let _ = writeln!(o, "    url: {}", s.url);
        let _ = writeln!(o, "    retrieved: {} · content_hash: {}", s.retrieved, s.content_hash);
        if let Some(a) = &s.archived_url {
            let _ = writeln!(o, "    archived: {a}");
        }
        if let Some(n) = &s.notes {
            let _ = writeln!(o, "    notes: {n}");
        }
    }
    let _ = writeln!(o, "\nlimitations");
    for l in &r.limitations { let _ = writeln!(o, "  - {l}"); }
    let _ = writeln!(o, "\nunknown conditions");
    for u in &r.unknown_conditions { let _ = writeln!(o, "  - {u}"); }
    o
}

pub fn nearest<'a>(needle: &str, ids: &[&'a str]) -> Option<&'a str> {
    ids.iter().copied().min_by_key(|id| levenshtein(needle, id))
}

fn levenshtein(a: &str, b: &str) -> usize {
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.iter().enumerate() {
        let mut cur = vec![i + 1];
        for (j, cb) in b.iter().enumerate() {
            let cost = usize::from(ca != cb);
            cur.push((prev[j + 1] + 1).min(cur[j] + 1).min(prev[j] + cost));
        }
        prev = cur;
    }
    prev[b.len()]
}
```

- [ ] **Step 4: Fill the `main.rs` arms**

```rust
        Cmd::List => cmd_list(),
        Cmd::Explain { rule_id } => cmd_explain(&rule_id),
        Cmd::Version => cmd_version(),
        Cmd::Completions { shell } => {
            let mut cmd = <Cli as clap::CommandFactory>::command();
            clap_complete::generate(shell, &mut cmd, "harness-guard", &mut std::io::stdout());
            ExitCode::SUCCESS
        }
```

```rust
fn cmd_list() -> ExitCode {
    // Detection only — no rule evaluation (§6).
    let (root, home) = discovery_root_from_env();
    let mut table = comfy_table::Table::new();
    table.set_header(["tool", "version", "config", "confidence"]);
    let home_exists = root.codex_home_exists();
    let on_path = harness_guard_core::version::binary_on_path(&root);
    if home_exists || on_path {
        let ver = harness_guard_core::version::detect_codex_version(&root)
            .unwrap_or_else(|| "version not detected".to_string());
        let cfg = root.config_path();
        let cfg = if std::fs::symlink_metadata(&cfg).is_ok() {
            redact::redact_home(&cfg.to_string_lossy(), home.as_deref())
        } else { "no config file".to_string() };
        let confidence = if ver == "version not detected" { "medium" } else { "high" };
        table.add_row([
            "codex",
            ver.as_str(),
            cfg.as_str(),
            confidence,
        ]);
    } else {
        table.add_row(["codex", "not detected", "-", "-"]);
    }
    anstream::println!("{table}");
    ExitCode::SUCCESS
}

fn cmd_explain(rule_id: &str) -> ExitCode {
    let rules = load_rules();
    match rules.iter().find(|r| r.raw.id == rule_id) {
        Some(rule) => {
            anstream::print!("{}", explain::render_rule(rule));
            ExitCode::SUCCESS
        }
        None => {
            let ids: Vec<&str> = rules.iter().map(|r| r.raw.id.as_str()).collect();
            match explain::nearest(rule_id, &ids) {
                Some(s) => eprintln!("unknown rule id {rule_id:?} — did you mean {s:?}?"),
                None => eprintln!("unknown rule id {rule_id:?}"),
            }
            ExitCode::from(2)
        }
    }
}

fn cmd_version() -> ExitCode {
    // Binary and ruleset versions reported separately (§6) — they diverge
    // once rules update independently of the binary.
    println!("harness-guard {}", env!("CARGO_PKG_VERSION"));
    println!("ruleset {}", ruleset_version());
    ExitCode::SUCCESS
}
```

Add `mod explain;` to `main.rs`.

- [ ] **Step 5: Run tests, then commit**

Run: `cargo test -p harness-guard-cli` → all PASS. `cargo clippy --workspace --all-targets -- -D warnings` → green (no `todo!` left anywhere: `grep -rn "todo!" crates/` → empty).

```bash
git add crates/harness-guard-cli
git commit -m "feat: list/explain/version/completions with nearest-match and positioning-phrase tests"
```

---

### Task 14: Hostile-input runtime cases + miette parse diagnostics

**Files:**
- Create: `crates/harness-guard-cli/src/diagnostics.rs`
- Modify: `crates/harness-guard-cli/src/main.rs` (replace the `eprintln!` parse-failure stub)
- Test: `crates/harness-guard-cli/tests/hostile.rs`

**Interfaces:**
- Consumes: `ParseFailure` (Task 7) via `ScanResult.parse_failure` (Task 9); fixtures `symlink-config`, `oversized`, `permission-denied` (Task 10).
- Produces: `diagnostics::report_parse_failure(pf: &ParseFailure, redacted_path: &str) -> String` — a miette diagnostic that carries line/col + optional key path **as plain message text** and never attaches file content as `source_code` (§7.3: we deliberately forgo miette's snippet rendering; a span label without source renders nothing, so all facts go in message/help).

- [ ] **Step 1: Write failing tests** (`tests/hostile.rs`)

```rust
mod common;
use common::*;
use std::path::{Path, PathBuf};

/// Copy a fixture's files/ into a tempdir so runtime mutation never touches
/// the committed tree (and absolutely never the real ~/.codex).
fn temp_copy(case: &str) -> (tempfile::TempDir, PathBuf) {
    let td = tempfile::tempdir().unwrap();
    let dst = td.path().join("files");
    copy_dir(&fixture(case), &dst);
    (td, dst)
}

fn copy_dir(src: &Path, dst: &Path) {
    std::fs::create_dir_all(dst).unwrap();
    for e in std::fs::read_dir(src).unwrap() {
        let e = e.unwrap();
        let to = dst.join(e.file_name());
        if e.path().is_dir() { copy_dir(&e.path(), &to); }
        else { std::fs::copy(e.path(), &to).unwrap(); }
    }
}

#[cfg(unix)]
#[test]
fn symlink_config_is_not_followed() {
    let (_td, files) = temp_copy("symlink-config");
    let home = files.join("codex-home");
    std::os::unix::fs::symlink(home.join("real-config.toml"), home.join("config.toml")).unwrap();
    let out = run_in(&files, &["scan", "--json"]);
    assert_eq!(out.status.code(), Some(2), "refused read degrades the scan");
    let report: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(report["tools"][0]["findings"][0]["status"], "unknown");
    let reason = report["tools"][0]["findings"][0]["unknown_reason"].as_str().unwrap();
    assert!(reason.contains("symlink"));
}

#[test]
fn oversized_config_is_refused() {
    let (_td, files) = temp_copy("oversized");
    let cfg = files.join("codex-home/config.toml");
    let mut big = String::with_capacity(1_100_000);
    while big.len() <= 1024 * 1024 { big.push_str("# padding line\n"); }
    std::fs::write(&cfg, big).unwrap();
    let out = run_in(&files, &["scan", "--json"]);
    assert_eq!(out.status.code(), Some(2));
    let report: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(report["tools"][0]["findings"][0]["status"], "unknown");
}

#[cfg(unix)]
#[test]
fn permission_denied_degrades_to_unknown() {
    use std::os::unix::fs::PermissionsExt;
    let (_td, files) = temp_copy("permission-denied");
    let cfg = files.join("codex-home/config.toml");
    std::fs::set_permissions(&cfg, std::fs::Permissions::from_mode(0o000)).unwrap();
    let out = run_in(&files, &["scan", "--json"]);
    std::fs::set_permissions(&cfg, std::fs::Permissions::from_mode(0o644)).unwrap();
    assert_eq!(out.status.code(), Some(2));
    let report: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let reason = report["tools"][0]["findings"][0]["unknown_reason"].as_str().unwrap();
    assert!(reason.contains("permission"));
}

#[test]
fn malformed_toml_diagnostic_has_line_col_but_never_content() {
    let out = run_case("malformed-toml", &["scan"]);
    assert_eq!(out.status.code(), Some(2));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("line 1"), "diagnostic must carry line/col: {err}");
    // §7.3: no snippet — the file's second line must not leak to stderr.
    assert!(!err.contains("persistence"), "config content leaked into diagnostic: {err}");
    // stdout still carries the full degraded report (§6 exit-code contract)
    assert!(!out.stdout.is_empty(), "degraded scan must still render a report");
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p harness-guard-cli --test hostile` → `malformed_toml_diagnostic...` FAILS (stub prints `Some(1)` not `line 1`); runtime cases may pass already — that is fine, they pin behavior.

- [ ] **Step 3: Implement `diagnostics.rs`**

The fixed crate stack has no `thiserror` — implement `Display`/`Error`/`Diagnostic` by hand:

```rust
//! miette is used ONLY for config-parse failures (§12) and NEVER attaches
//! source text (§7.3) — line/col/key-path travel as plain strings.
use harness_guard_core::parse::ParseFailure;

#[derive(Debug)]
struct ConfigParseDiagnostic {
    message: String,
    location: String,
    path: String,
}

impl std::fmt::Display for ConfigParseDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "config not safely parseable: {}", self.message)
    }
}
impl std::error::Error for ConfigParseDiagnostic {}

impl miette::Diagnostic for ConfigParseDiagnostic {
    fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        Some(Box::new("harness_guard::config_parse"))
    }
    fn help<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        Some(Box::new(format!(
            "{} in {} — fix the file and re-run; raw file content is never shown",
            self.location, self.path)))
    }
    // No source_code() implementation — deliberate (§7.3): a snippet could
    // leak config values. Line/col are plain text in `help`.
}

pub fn report_parse_failure(pf: &ParseFailure, redacted_path: &str) -> String {
    let location = match (pf.line, pf.col, &pf.key_path) {
        (Some(l), Some(c), Some(k)) => format!("line {l}, column {c}, key {k}"),
        (Some(l), Some(c), None) => format!("line {l}, column {c}"),
        (_, _, Some(k)) => format!("key {k}"),
        _ => "unknown location".to_string(),
    };
    let diag = ConfigParseDiagnostic {
        message: pf.message.clone(),
        location,
        path: redacted_path.to_string(),
    };
    format!("{:?}", miette::Report::new(diag))
}
```

In `main.rs`, replace the parse-failure `eprintln!` stub:

```rust
    for pf in &parse_failures {
        let path = report.tools.first()
            .and_then(|t| t.config_paths.first())
            .cloned()
            .unwrap_or_else(|| "config.toml".to_string());
        eprint!("{}", diagnostics::report_parse_failure(pf, &path));
    }
```

Add `mod diagnostics;` to `main.rs`.

- [ ] **Step 4: Run tests, then commit**

Run: `cargo test -p harness-guard-cli` → all PASS. Confirm the whole suite: `cargo test --workspace` → PASS; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo deny check` → green.

```bash
git add crates/harness-guard-cli
git commit -m "feat: runtime hostile-input coverage + no-snippet miette parse diagnostics"
```

---

### Task 15: Instrumented no-egress runtime proof (macOS, run now)

**Files:**
- Create: `scripts/no-egress/scan.sb`
- Create: `scripts/no-egress/run-macos.sh`

**Interfaces:**
- Consumes: the built binary + fixtures. Produces: a locally runnable proof script; `ci.yml` (Task 16) reuses it for the macOS job and authors the strace equivalent for Linux.

- [ ] **Step 1: Write `scripts/no-egress/scan.sb`**

```scheme
;; Layer 3 of the no-egress proof (§10.1): deny ALL network under sandbox-exec.
;; Any egress attempt errors visibly instead of silently succeeding.
(version 1)
(allow default)
(deny network*)
```

- [ ] **Step 2: Write `scripts/no-egress/run-macos.sh`**

```bash
#!/bin/sh
# Instrumented no-egress proof, macOS. Runs real scans over synthetic
# fixtures inside a deny-all-network sandbox and asserts the exact §6 exit
# codes — a blocked network call would surface as exit 2 / error output.
set -eu
cd "$(dirname "$0")/../.."

cargo build -p harness-guard-cli
BIN=target/debug/harness-guard
SB=scripts/no-egress/scan.sb

# NB: sandbox-exec/curl are invoked by absolute path — the per-command PATH
# override (needed so version detection sees only the fixture path dir)
# would otherwise break command lookup.
run_case() {
    case_dir="fixtures/codex/$1/files"
    want="$2"
    set +e
    CODEX_HOME="$PWD/$case_dir/codex-home" PATH="$PWD/$case_dir/path" NO_COLOR=1 \
        /usr/bin/sandbox-exec -f "$SB" "$PWD/$BIN" scan --json \
        > /tmp/harness-guard-noegress.json 2>&1
    got=$?
    set -e
    if [ "$got" -ne "$want" ]; then
        echo "FAIL: case $1 exited $got, expected $want" >&2
        cat /tmp/harness-guard-noegress.json >&2
        exit 1
    fi
    echo "ok: $1 (exit $got under deny-all-network sandbox)"
}

run_case hardened 0
run_case risky-unset 1
run_case malformed-toml 2
run_case unknown-version 0

# Sanity check that the sandbox profile actually blocks network: curl must fail.
if /usr/bin/sandbox-exec -f "$SB" /usr/bin/curl -s --max-time 5 https://example.com >/dev/null 2>&1; then
    echo "FAIL: sandbox profile did not block network — proof is void" >&2
    exit 1
fi
echo "ok: sandbox profile verified to block egress"
echo "NO-EGRESS PROOF PASSED"
```

`chmod +x scripts/no-egress/run-macos.sh`.

- [ ] **Step 3: Run it**

Run: `scripts/no-egress/run-macos.sh` → prints four `ok:` case lines + sandbox verification + `NO-EGRESS PROOF PASSED`, exit 0. This satisfies acceptance criterion 5's "runs locally now" half; the strace half is authored (not run) in Task 16.

- [ ] **Step 4: Commit**

```bash
git add scripts/no-egress/
git commit -m "feat: instrumented no-egress proof — sandbox-exec deny-all-network scan over fixtures"
```

---

### Task 16: CI + freshness pipeline (authored, NOT enabled) + runbook

**Files:**
- Create: `.github/workflows/ci.yml`
- Create: `.github/workflows/release-watch.yml`
- Create: `.github/workflows/doc-drift.yml`
- Create: `freshness/last-seen.json`
- Create: `docs/maintenance/runbook.md`

**Interfaces:**
- Consumes: gates (Task 3), test suite (Tasks 6–14), no-egress script (Task 15), `scripts/freshness/*` (Task 5), `freshness/url-hashes.json` (Task 5).
- Produces: workflow files that are inert until the repo is pushed to GitHub AND the user enables them — both explicitly deferred, user-triggered steps. Nothing here assumes a public repo. Bots only ever open triage issues; they never set verdicts or edit rules.

- [ ] **Step 1: `.github/workflows/ci.yml`**

```yaml
# Authored in the thin slice; runs only after the user pushes the repo to
# GitHub (a deferred, user-triggered decision). Until then the same steps
# run locally: cargo fmt/clippy/deny/test + scripts/no-egress/run-macos.sh.
name: ci
on:
  push: {}
  pull_request: {}
jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo fmt --all -- --check
      - run: cargo clippy --workspace --all-targets -- -D warnings
  deny:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: taiki-e/install-action@v2
        with:
          tool: cargo-deny
      - run: cargo deny check
  test:
    strategy:
      matrix:
        os: [macos-latest, ubuntu-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --workspace
  no-egress-macos:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: scripts/no-egress/run-macos.sh
  no-egress-linux-strace:
    # Layer 3, Linux flavor (§10.1): scan under strace network tracing and
    # assert zero socket-family syscalls reach the wire. Authored now; runs
    # when CI is enabled.
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo build -p harness-guard-cli
      - name: scan under strace, assert no network syscalls
        run: |
          set -eu
          sudo apt-get update && sudo apt-get install -y strace
          case_dir="fixtures/codex/risky-unset/files"
          set +e
          # PATH includes the fixture path dir so version detection sees the
          # synthetic npm marker (=> in-range => warning finding => exit 1);
          # strace by absolute path since PATH is overridden per-command.
          CODEX_HOME="$PWD/$case_dir/codex-home" NO_COLOR=1 \
            PATH="$PWD/$case_dir/path:/usr/bin:/bin" \
            /usr/bin/strace -f -e trace=network -o /tmp/net.trace \
            target/debug/harness-guard scan --json > /dev/null
          code=$?
          set -e
          [ "$code" -eq 1 ] || { echo "expected exit 1, got $code"; exit 1; }
          # Allow only harmless non-wire lines (strace headers/exit lines);
          # fail on any socket/connect/send/recv of an inet family.
          if grep -E 'socket\(AF_(INET|INET6)|connect\(|send(to|msg)?\(|recv(from|msg)?\(' /tmp/net.trace; then
            echo "network syscalls observed — no-egress violated"; exit 1
          fi
          echo "strace: zero inet socket syscalls"
```

- [ ] **Step 2: `.github/workflows/release-watch.yml`**

```yaml
# AUTHORED, NOT ENABLED (§11): schedules run only after the repo is published
# and the user re-enables workflows — GitHub also auto-disables cron after 60
# days of repo inactivity (see docs/maintenance/runbook.md).
# Automation only opens triage issues; bots never set verdicts or edit rules.
name: release-watch
on:
  workflow_dispatch: {}
  schedule:
    - cron: "17 6 * * 1"   # weekly, Monday 06:17 UTC
permissions:
  issues: write
  contents: read
jobs:
  watch:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: diff npm dist-tags against freshness/last-seen.json
        env:
          GH_TOKEN: ${{ github.token }}
        run: |
          set -eu
          check() {
            pkg="$1"; tag="$2"
            latest=$(curl -s -H 'Accept: application/vnd.npm.install-v1+json' \
              "https://registry.npmjs.org/$(echo "$pkg" | sed 's|/|%2F|')" \
              | jq -r ".\"dist-tags\".\"$tag\" // empty")
            [ -n "$latest" ] || { echo "WARN: no $tag tag for $pkg"; return 0; }
            seen=$(jq -r ".packages[\"$pkg\"].version // empty" freshness/last-seen.json)
            if [ "$latest" != "$seen" ]; then
              # Codex release notes live in GitHub release bodies (its
              # CHANGELOG.md is a stub) — link the releases page.
              gh issue create \
                --title "release-watch: $pkg $tag moved $seen -> $latest" \
                --body "Detected by release-watch. Re-verify affected rules against the new version, then update tested_versions + freshness/last-seen.json. Release notes: check the package's GitHub releases page (for @openai/codex use the release body, not CHANGELOG.md). No rule is edited by automation."
            fi
          }
          # Watch exactly these tags (§11.1): stable for claude-code (NOT
          # latest/next), latest for codex (ignore its ~16 other dist-tags;
          # filter -alpha. if ever reading releases.atom), latest for copilot
          # (NOT prerelease).
          check "@anthropic-ai/claude-code" "stable"
          check "@openai/codex" "latest"
          check "@github/copilot" "latest"
```

- [ ] **Step 3: `.github/workflows/doc-drift.yml`**

```yaml
# AUTHORED, NOT ENABLED (§11). Weekly semantic-drift tripwire over every URL
# cited by rules/. Uses the SAME normalization as rule content_hash
# (scripts/freshness/normalize.sh) so citation anchors and drift detection
# share one definition.
# Known pitfalls: JS-rendered vendor docs may need a per-page Playwright
# fallback later (not default); CDN edge-caching can stagger detection.
name: doc-drift
on:
  workflow_dispatch: {}
  schedule:
    - cron: "43 6 * * 1"   # weekly, Monday 06:43 UTC
permissions:
  issues: write
  contents: read
jobs:
  drift:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: taiki-e/install-action@v2
        with:
          tool: lychee
      - name: dead-link pass
        # lychee reads the URL list from stdin ("-" input) and extracts/checks
        # each; dead links surface in the drift issue rather than failing CI.
        continue-on-error: true
        run: scripts/freshness/extract-urls.sh | lychee --no-progress --max-retries 3 -
      - name: semantic-text hash pass
        env:
          GH_TOKEN: ${{ github.token }}
        run: |
          set -eu
          scripts/freshness/extract-urls.sh | while read -r url; do
            new="sha256:$(curl -sL "$url" | scripts/freshness/normalize.sh)"
            old=$(jq -r ".hashes[\"$url\"] // empty" freshness/url-hashes.json)
            if [ "$new" != "$old" ]; then
              # Evidence anchors: the rule's archived_url (old) + a fresh
              # Wayback SPN2 snapshot (new).
              snap=$(curl -s "https://web.archive.org/save/$url" -o /dev/null -w '%{redirect_url}' || true)
              gh issue create \
                --title "doc-drift: cited page changed: $url" \
                --body "Semantic-text hash drifted (old $old, new $new). Old snapshot: see the citing rule's archived_url. New snapshot: $snap. A human re-verifies the rule, updates retrieved/content_hash and freshness/url-hashes.json, and bumps the ruleset version. Automation never edits rules."
            fi
          done
```

- [ ] **Step 4: Seed `freshness/last-seen.json`** (manual dev-time retrieval, record the real values you get)

```bash
for p in @anthropic-ai/claude-code @openai/codex @github/copilot; do
  curl -s -H 'Accept: application/vnd.npm.install-v1+json' \
    "https://registry.npmjs.org/$(echo $p | sed 's|/|%2F|')" | jq '."dist-tags"'
done
```

```json
{
  "schema_version": "1.0",
  "checked": "<TODAY>",
  "packages": {
    "@anthropic-ai/claude-code": { "dist_tag": "stable", "version": "<value from stable tag>" },
    "@openai/codex": { "dist_tag": "latest", "version": "0.144.4" },
    "@github/copilot": { "dist_tag": "latest", "version": "<value from latest tag>" }
  }
}
```

(If `@openai/codex` latest has moved past 0.144.4, record the real value here — `last-seen` tracks reality; the rule's `tested_versions` moves only per Task 5 Step 3's criteria.)

- [ ] **Step 5: Write `docs/maintenance/runbook.md`**

```markdown
# Maintenance runbook

## Scheduled workflows: authored, not enabled

`release-watch.yml` and `doc-drift.yml` exist in-tree but do not run:
the repository is local-only. Publishing the repo and enabling workflow
schedules are separate, user-triggered decisions.

**GitHub auto-disable:** scheduled workflows are automatically disabled
after 60 days without repository activity. Re-enable: repo → Actions →
select workflow → "Enable workflow". Check this whenever activity has
lapsed; a calendar reminder at 45-day cadence is the cheapest guard.

## Triage flow (drift or release detected)

1. Automation opens a triage issue (release-watch or doc-drift). Bots
   never set verdicts and never edit rules.
2. A human re-verifies the claim against the live official page and the
   linked Wayback snapshots (old = rule's archived_url, new = issue link).
3. If the rule needs changing: edit the rule JSON with a new `retrieved`
   date + `content_hash` (via `scripts/freshness/normalize.sh`), refresh
   `archived_url`, update `tested_versions` with the re-verified range.
4. Update `freshness/url-hashes.json` / `freshness/last-seen.json`.
5. Bump `rules/ruleset.json` `ruleset_version` (CalVer, date of change).
6. Run the full test suite; fixture goldens are the second staleness
   signal — a rule silently failing to match config shape is stronger
   drift evidence than a doc hash.

## Cadence claims

No public verification-cadence claim ("verified monthly", badges, etc.)
is made anywhere until the freshness pipeline has actually run on a
schedule. This is a hard rule from the product decision record.
```

- [ ] **Step 6: Verify and commit**

Run: `jq -e . freshness/last-seen.json >/dev/null && echo OK` → OK. YAML sanity: `python3 -c "import yaml,glob; [yaml.safe_load(open(f)) for f in glob.glob('.github/workflows/*.yml')]" && echo YAML_OK` (or `yq` if available) → YAML_OK. Confirm no workflow claims a cadence in user-facing wording (workflow comments are fine): `grep -rn "verified monthly\|verified weekly" . --include='*.md' --include='*.rs' --include='*.json' | grep -v runbook.md` → empty.

```bash
git add .github/ freshness/last-seen.json docs/maintenance/runbook.md
git commit -m "feat: CI + freshness pipeline authored (not enabled) with triage-only automation and runbook"
```

---

### Task 17: Acceptance sweep, session log, review gate

**Files:**
- Modify: `./notes/session-history.md`
- Create: `./README.md` — do NOT create or rewrite; only touch if the existing README's positioning conflicts with the binding phrase (check, and if it does, fix just that wording).

**Interfaces:** Consumes everything. Produces: the slice, verified against spec §13, stopped at the human review gate.

- [ ] **Step 1: Run the full §13 acceptance checklist mechanically**

```bash
cd .
# 1. git history: docs first, implementation after
git log --oneline | tail -1   # → the docs commit from Task 1
# 2. full suite
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings \
  && cargo deny check && cargo test --workspace
# 3. §7.1 shape on risky-unset + JSON contract
CODEX_HOME=$PWD/fixtures/codex/risky-unset/files/codex-home \
  PATH=$PWD/fixtures/codex/risky-unset/files/path:/usr/bin:/bin \
  target/debug/harness-guard scan          # eyeball against §7.1: citation + retrieved date visible
CODEX_HOME=$PWD/fixtures/codex/risky-unset/files/codex-home \
  PATH=$PWD/fixtures/codex/risky-unset/files/path:/usr/bin:/bin \
  target/debug/harness-guard scan --json | jq .network_requests_made   # → 0
# 4. explain shows retrieved date, content_hash, archived_url
target/debug/harness-guard explain codex-history-persist-01 | grep -E "retrieved|sha256:|web.archive.org"
# 5. no-egress
scripts/no-egress/run-macos.sh
# 9. positioning phrase discipline, repo-wide (excludes quarantined legacy artifacts)
grep -rn "AI agent security scanner" crates/ rules/ schemas/ scripts/ .github/ docs/maintenance/ \
  && echo VIOLATION || echo CLEAN   # → CLEAN
grep -rn "local, execution-free, per-finding-cited config auditor" crates/harness-guard-cli/src/ \
  || echo MISSING   # → at least the clap `about` line
```

Criteria 6, 7, 8 are already pinned by tests (snapshot goldens for unknown/stale rendering; workflow files + committed freshness state; `rules/` LICENSE+README + schema-contract loading). Re-confirm each has a green test rather than re-arguing it.

- [ ] **Step 2: Check the existing README's positioning**

`grep -n "security scanner" README.md CONTEXT.md docs/product/*.md` — the binding phrase rule applies to user-facing text of the product. If `README.md` (the repo's face) uses the forbidden phrase, replace just that phrasing with the binding positioning sentence; do not otherwise rewrite pre-existing docs.

- [ ] **Step 3: Update `notes/session-history.md`**

Append (adjust the date to the actual completion date):

```markdown
## 2026-07-XX — v1 thin slice implemented

Implemented per docs/superpowers/plans/2026-07-14-harness-guard-v1-thin-slice.md
(spec: docs/superpowers/specs/2026-07-14-harness-guard-v1-thin-slice-design.md):
git repo initialized (docs-first commit); Phase 0 schemas; 3-crate workspace
with cargo-deny + core-scoped clippy no-egress gates; codex-history-persist-01
end-to-end with fresh evidence (retrieved <date>, hashed, archived); 13-case
synthetic fixture matrix with goldens; scan/list/explain/version CLI with
terminal + JSON views from shared structs; sandbox-exec no-egress proof run
locally (strace job authored in CI); freshness workflows + runbook authored,
NOT enabled. Slice is now at the human review gate — no second rule or tool
until review.
```

- [ ] **Step 4: Final commit and STOP**

```bash
git add notes/session-history.md README.md
git commit -m "chore: acceptance sweep for v1 thin slice; session history updated"
```

**STOP HERE.** Acceptance criterion 10: the slice is presented for human review BEFORE any second rule or tool. Do not start any follow-on work.

---

## Self-review notes (already applied)

- Spec coverage: §2 items 1–9 map to Tasks 4 / 2 / 7 / 5+6+9 / 11+13 / 11+12 / 10+14+15 / 16 / 1. §13 criteria 1–10 map to Tasks 1 / 6+10+11+14 / 11+12 / 13 / 3+15+16 / 12 / 16 / 5 / 13+17 / 17.
- Deliberate decisions recorded inline: `--min-severity` filters terminal finding-blocks only (JSON stays the full contract, Task 11); hand-rolled version triple + Levenshtein instead of new crates (fixed stack); fixture version markers use committed regular files instead of symlinks (Windows-safe), with symlink behavior unit-tested at runtime (Task 8) and the symlink-config case runtime-constructed (Task 14); `observe()`'s value literals are coupled to the single rule's `allowed_render` with a test pinning the coupling (Task 9).
- Known follow-ups explicitly out of slice: generalizing value→outcome dispatch into rule JSON, `codex.exe`/`codex.cmd` Windows PATH lookup (documented limitation — Windows fixtures still work since detection is file-name based on `codex`... note: Windows npm installs use `codex.cmd`; on Windows real machines this yields "version not detected" ⇒ stale-ruleset, which is conservative and acceptable for the slice), account/server-state reframe sentence (ships with the first rule that needs it).
