# Harness Guard 0.0.1 Multi-Harness Generalization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Spec:** `docs/superpowers/specs/2026-07-16-harness-guard-0.0.1-multi-harness-design.md` (read it before starting any task; section references below are to that spec).

**Goal:** Generalize the reviewed Codex thin slice into the 0.0.1 release: Claude Code, Codex CLI, and Grok Build as co-equal audited harnesses on a declarative rule engine, with the same evidence rigor, safety invariants, output contract, and conservative degradation.

**Architecture:** A closed `HarnessId` enum + static descriptor table drives multi-harness discovery, JSON/TOML parsing at equal hostile rigor, parameterized execution-free version detection, and a generalized `scan_harness` dispatch. Rule evaluation moves from hardcoded Rust (`evaluate.rs`) to a declarative engine (`engine.rs`) driven by a closed set of typed match primitives whose totality is proven at load time. Rules are pure data + fixtures; per-harness evidence work packages author them from fresh primary-source retrieval.

**Tech Stack:** Rust 2024 edition, MSRV 1.85. Workspace crates `harness-guard-core`, `harness-guard-rules`, `harness-guard-cli`. `toml =1.1.2` (never enable `unbounded`), `serde_json`, `rustix`, `clap 4`, `insta`, `jsonschema 0.30` (default-features off). **No new dependencies anywhere in this plan** (`cargo deny check` enforces the graph; the `build.rs` is std-only).

## Global Constraints

Copied from the project guardrails and spec; every task's requirements implicitly include all of these.

- No network requests during scans; never execute anything discovered (no harness binaries, no hooks). The three-layer no-egress proof (deny bans / core clippy `disallowed-methods` / `scripts/no-egress/run-macos.sh`) must stay green.
- Core takes only an explicit `DiscoveryRoot`; never ambient env/homes; only the CLI crate touches the real environment (`crates/harness-guard-core/clippy.toml` enforces).
- Synthetic fixtures only — never a real `~/.codex`, `~/.claude`, or `~/.grok`. Tests always inject fixture roots.
- Never read source code, transcripts, history contents, shell history, `.env` files, or credentials.
- Filesystem reads bounded, regular-file-only, symlink/reparse-refusing, depth-bounded, race-resistant — all reads go through the existing `readfs` layer; no new read path.
- Reports contain only normalized allowlisted observations; redact usernames, home paths, raw config values.
- Every non-unknown finding is version-bounded, source-cited, dated, fixture-tested, limitation-explicit; locally unknowable state reports as `unknown`, never inferred.
- Rule authoring requires fresh primary-source retrieval (actual dates, semantic hashes via `scripts/freshness/normalize.sh`, Wayback anchors) plus `freshness/` and `rules/ruleset.json` CalVer updates. `data/` and `docs/research/` are quarantined leads, never evidence.
- Positioning (binding, verbatim, test-pinned): "local, execution-free, per-finding-cited config auditor". "AI agent security scanner" appears nowhere in user-facing text.
- No public verification-cadence claims; freshness workflows stay gated off (`ENABLE_FRESHNESS_WORKFLOWS`) through 0.0.1.
- No package publishing, release/tag creation, branch pushing, or repo-settings changes without exact explicit owner authorization at execution time.
- Validation gates (run after every task; all must pass before commit): `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo deny check`, `cargo test --workspace`; on macOS `scripts/no-egress/run-macos.sh`; `actionlint` after any `.github/workflows/*.yml` edit.
- Retired Grok mitigation keys `GROK_TELEMETRY_ENABLED`, `GROK_TELEMETRY_TRACE_UPLOAD`, `[telemetry]` table + `trace_upload` must never reappear in any rule, remediation, or user-facing string (Task 1 pins this).
- `rules/` stays a standalone Apache-2.0 data package consumed only through the schema contract.
- Status enum `pass | finding | unknown | stale-ruleset`; exit codes 0/1/2 with degraded-scan semantics; unset ⇒ `unknown` while only user scope is inspected (validator-enforced).
- Tool ids (closed set): `codex`, `claude-code`, `grok-build`. Schema/CLI field name stays `tool`; "harness" is prose-only.
- Owner decisions locked 2026-07-16: Grok Build full rule parity (clean-room reproduction is release-gating); declarative rule engine; version 0.1.0 → 0.0.1 with tag `0.0.1`; Copilot CLI freshness tracking stays.

## Explicit assumptions (spec-gap resolutions — read before Task 3)

The spec left four small mechanical gaps; this plan resolves them as follows. Each is flagged at the task that implements it.

1. **`unknown_subject` rule field (new, beyond the §5.6 delta list).** The shipped goldens pin unknown messages of the form `"Cannot determine history persistence posture: {reason}"` for **all four** unknown paths (unset, unrecognized, unreadable, unparseable — see `fixtures/codex/permission-denied/expected.json` and `fixtures/codex/malformed-toml/expected.json`). The prefix `history persistence posture` is rule-specific data that today lives hardcoded in `evaluate.rs` and appears nowhere in the rule JSON. A data-driven engine that must reproduce goldens byte-identically (§6.6, §13.1) therefore needs one additional required rule field, `unknown_subject` (non-empty string). The engine's fixed template for every unknown-status record is `"Cannot determine {unknown_subject}: {reason}"` — the same interpolation class as the fixed stale template §6.4 already allows. No other message interpolation exists.
2. **`allowed_render` for integer observations is exactly `["unset"]`.** §5.7 renders integers from the parsed `i64` within `integer_bounds`, never from a string allowlist, but the schema requires `allowed_render` with ≥1 item. The validator pins integer rules to `["unset"]` so no stringly-typed integer domain can drift in.
3. **Engine-level unknowns (unreadable/unparseable) take `verify_url` from the rule's single `unset` outcome** (deterministic by the cardinality check §6.3.3; matches current behavior, which takes the first unknown outcome).
4. **New-harness fixture layout uses a committed synthetic `files/home/` directory** (containing `.claude/settings.json` or `.grok/config.toml`) so the CLI's default `HOME`-relative resolution is exercised without any override env var, and config paths render as `~/.claude/settings.json` per §5.7. Codex fixtures keep their existing `codex-home/` + `CODEX_HOME` layout untouched (byte-identity).

Open research items the spec itself defers to implementation-time fresh retrieval (NOT plan gaps): Claude Code home-override env var (lead: `CLAUDE_CONFIG_DIR`), Grok Build PATH binary name / version detection / distribution channel (§7.3 protocol outputs), exact rule sets for all three harnesses.

## File structure

```
rules/{codex,claude-code,grok-build}/*.json        # data package, schema 1.1
rules/ruleset.json                                 # CalVer, bumped by each rule WP
fixtures/{codex,claude-code,grok-build}/<case>/    # per-harness matrices
fixtures/mixed/codex-pass-claude-degraded/         # two-store aggregation case (Task 18)
schemas/rule.schema.json                           # 1.1 (Task 3)
schemas/report.schema.json                         # 1.1 (Task 3)
schemas/capabilities.schema.json                   # new 1.0 (Task 20)
crates/harness-guard-rules/build.rs                # new: rule embedding (Task 8)
crates/harness-guard-rules/src/{schema,loader,report}.rs
crates/harness-guard-rules/tests/{schema,fixture,report}_validation.rs
crates/harness-guard-rules/tests/tripwires.rs      # new (Task 1)
crates/harness-guard-core/src/harness.rs           # new (Task 10)
crates/harness-guard-core/src/engine.rs            # new (Task 6); evaluate.rs deleted (Task 7)
crates/harness-guard-core/src/parse_json.rs        # new (Task 12)
crates/harness-guard-core/src/{discovery,parse,readfs,scan,version}.rs
crates/harness-guard-cli/src/{main,redact,render_term,render_json,explain,diagnostics}.rs
crates/harness-guard-cli/src/capabilities.rs       # new (Task 20)
crates/harness-guard-cli/tests/…                   # extended throughout
docs/research/protocols/grok-build-cleanroom.md    # new (Task 2)
docs/research/evidence/grok-build/<date>/          # lab-run artifacts (owner)
docs/agent-guide.md                                # new (Task 21)
CHANGELOG.md                                       # new (Task 23)
```

## Sequencing

Tasks 1–2 (WP0 docs/tests) first — the owner lab run they enable is the longest pole and gates Task 19 and the tag. Tasks 3–9 (WP1 engine) strictly before Tasks 10–16 (WP2 abstraction) so engine regressions and abstraction regressions can never mask each other. Tasks 17, 18, and 19 are independent after Task 16 (19 is additionally gated on Owner Checkpoints A/B and folds in the WP5 freshness extension). Tasks 20–21 (WP4) run after Task 16, parallel to 17–19; Task 20's subcommand code lands in that window, but its rule-count golden/snapshot (Step 3) is authored last, after Task 19 stabilizes rule counts. Tasks 22–25 last.

---

## Phase 0 — WP0: Grok clean-room groundwork (starts immediately)

### Task 1: Retired-Grok-keys tripwire test

**Files:**
- Create: `crates/harness-guard-rules/tests/tripwires.rs`
- Modify: `crates/harness-guard-cli/tests/cli_surface.rs` (append one test)

**Interfaces:**
- Produces: workspace tests that fail if any retired Grok mitigation key reappears in `rules/**` or in CLI help output. Later tasks (19) rely on this staying green.

- [ ] **Step 1: Write the rules-tree tripwire test**

Create `crates/harness-guard-rules/tests/tripwires.rs`:

```rust
//! Retired-mitigation tripwires (spec §7.3.7). These strings were mitigation
//! keys for an old Grok Build version and must never reappear in any rule,
//! remediation, or user-facing string. Same mechanism as the forbidden
//! positioning-phrase test.
use std::path::{Path, PathBuf};

const RETIRED_GROK_KEYS: [&str; 4] = [
    "GROK_TELEMETRY_ENABLED",
    "GROK_TELEMETRY_TRACE_UPLOAD",
    "trace_upload",
    "[telemetry]",
];

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn walk_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            walk_files(&path, out);
        } else {
            out.push(path);
        }
    }
}

#[test]
fn retired_grok_keys_never_reappear_in_rules() {
    let mut files = Vec::new();
    walk_files(&repo_root().join("rules"), &mut files);
    assert!(!files.is_empty(), "rules tree must exist");
    for file in files {
        let text = std::fs::read_to_string(&file)
            .unwrap_or_else(|_| panic!("rule file {file:?} is readable UTF-8"));
        for key in RETIRED_GROK_KEYS {
            assert!(
                !text.contains(key),
                "retired Grok mitigation key {key:?} reappeared in {file:?}"
            );
        }
    }
}
```

- [ ] **Step 2: Run it to confirm it passes on the current tree**

Run: `cargo test -p harness-guard-rules --test tripwires`
Expected: PASS (1 test).

- [ ] **Step 3: Extend the CLI output corpus tripwire**

Append to `crates/harness-guard-cli/tests/cli_surface.rs`:

```rust
#[test]
fn retired_grok_keys_never_appear_in_cli_output() {
    // Spec §7.3.7: the tripwire covers user-facing output corpora, not just
    // rule files. Help text and a fixture scan are the output corpus.
    let retired = [
        "GROK_TELEMETRY_ENABLED",
        "GROK_TELEMETRY_TRACE_UPLOAD",
        "trace_upload",
        "[telemetry]",
    ];
    for args in [
        vec!["--help"],
        vec!["scan", "--help"],
        vec!["scan", "--json"],
        vec!["scan", "--verbose"],
        vec!["explain", "codex-history-persist-01"],
    ] {
        let output = run_case("hardened", &args);
        let text = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        for key in retired {
            assert!(!text.contains(key), "retired key {key:?} in output of {args:?}");
        }
    }
}
```

- [ ] **Step 4: Run the CLI test**

Run: `cargo test -p harness-guard-cli --test cli_surface retired_grok`
Expected: PASS.

- [ ] **Step 5: Gates and commit**

Run: `cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
Expected: all green.

```bash
git add crates/harness-guard-rules/tests/tripwires.rs crates/harness-guard-cli/tests/cli_surface.rs
git commit -m "test: pin retired Grok mitigation keys out of rules and CLI output"
```

### Task 2: Grok Build clean-room reproduction protocol document

**Files:**
- Create: `docs/research/protocols/grok-build-cleanroom.md`
- Create: `docs/research/evidence/grok-build/.gitkeep`

**Interfaces:**
- Produces: the release-gating protocol (§7.3) the owner executes outside the product. Task 19 consumes its artifacts under `docs/research/evidence/grok-build/<date>/`.

- [ ] **Step 1: Write the protocol document**

Create `docs/research/protocols/grok-build-cleanroom.md` with exactly this content (§7.3 items 1–7 made operational):

```markdown
# Grok Build clean-room reproduction protocol

**Status:** Release-gating for 0.0.1 (spec §7.3). This is maintainer lab work
performed OUTSIDE the product. Harness Guard itself never captures traffic,
never executes `grok`, and never phones home; this protocol produces the
dated evidence artifacts that Grok Build rules cite.

**Prior work is quarantined.** The v0.2.93 reproduction
(github.com/cereblab/grok-build-exfil-repro) and everything under
`docs/research/per-tool/grok-build.md` and `data/` are leads only. No risk
claim ships without a fresh run against the then-current version.

## 1. Environment

- A fresh, disposable VM or container per run; macOS and Linux runs both
  recorded. Destroy the environment after artifact extraction.
- No personal or client data anywhere inside the environment.
- An owner-provisioned **disposable** xAI account — never a personal or
  company account.
- A purpose-built canary repository: unique, never-published canary tokens
  embedded in files the model is NOT asked to read, plus files it IS asked
  to read. Token uniqueness is what makes payload search conclusive.

## 2. Version pinning

Before any run, record:
- the exact Grok Build version string as the product reports it;
- the install channel (npm package name + dist-tag, installer URL, or other);
- the SHA-256 of the installed binary/package artifact.

This run's findings apply to exactly that version. Rules cite it with
`tested_versions` min == max (no `<=` prefix) unless a separate written
justification widens the range.

## 3. Capture

- System-level egress observation scoped to the VM: mitmproxy with a locally
  installed CA **plus** raw packet capture (tcpdump/pcap) as corroboration.
- Record request targets, sizes, and payload structure for xAI endpoints.
- The transmission test is a canary-token search over all captured payloads
  (decoded/decompressed where applicable).

## 4. Matrix

Run each row in a fresh environment state; record config file + account UI
state alongside each capture:
1. Documented-default configuration (no config file edits).
2. Each currently documented mitigation key from docs.x.ai/build/settings and
   docs.x.ai/build/settings/reference toggled independently (retrieve those
   pages fresh at run time; do NOT reuse key names from prior research — the
   old GROK_TELEMETRY_* / [telemetry] trace_upload keys are retired and
   test-banned).
3. Account/server-side flag states (e.g. any remote codebase-upload disable
   feature) recorded as user-confirmed observations of the account UI where
   visible; otherwise recorded as unknown. Never inferred.

## 5. Artifacts

Store under `docs/research/evidence/grok-build/<YYYY-MM-DD>/`:
- dated, sanitized capture summaries (endpoints, sizes, payload structure,
  canary hit/no-hit per matrix row) — canary tokens are fine to include; no
  credentials, no personal data, no full payload dumps;
- the exact configuration file used per matrix row;
- the version-pinning record from §2;
- semantic hashes (`scripts/freshness/normalize.sh <url>`) and Wayback
  anchors for every docs page cited.

## 6. Rule authoring consequences

- Locally observable `~/.grok/config.toml` posture keys cite
  `official-documentation` sources.
- Upload/telemetry *behavior* claims cite `independent-reproduction` with the
  pinned version and are never generalized beyond it.
- Server-side/account state is `unknown` with a `verify_url`.
- Version drift after the run ⇒ `stale-ruleset` by construction — the honest
  state, not a defect.

## 7. Retired-keys tripwire

`GROK_TELEMETRY_ENABLED`, `GROK_TELEMETRY_TRACE_UPLOAD`, and the
`[telemetry]`/`trace_upload` key pair must never reappear in any rule,
remediation, or user-facing string. Enforced by
`crates/harness-guard-rules/tests/tripwires.rs` and the cli_surface corpus
test.

## Release gate

0.0.1 does not tag until this protocol has been executed against the
then-current Grok Build version and the shipped Grok rules cite that run. If
upstream releases between the run and the tag, follow the
`docs/maintenance/runbook.md` triage flow to decide re-run vs. shipping with
the pinned version honestly reflected (newer detected versions then yield
`stale-ruleset`, which is correct behavior).
```

- [ ] **Step 2: Create the evidence directory placeholder**

```bash
mkdir -p docs/research/evidence/grok-build
touch docs/research/evidence/grok-build/.gitkeep
```

- [ ] **Step 3: Commit**

```bash
git add docs/research/protocols/grok-build-cleanroom.md docs/research/evidence/grok-build/.gitkeep
git commit -m "docs: add release-gating Grok Build clean-room reproduction protocol"
```

### OWNER CHECKPOINT A (external, runs in parallel with Phases 1–2)

The owner executes the protocol from Task 2 in a lab environment and lands the dated artifacts under `docs/research/evidence/grok-build/<date>/`. **Task 19 must not start until these artifacts exist.** Nothing in this plan authorizes the agent to execute Grok Build, provision accounts, or capture traffic — this checkpoint is human work.

---

## Phase 1 — WP1: Declarative rule engine (single-harness, goldens byte-identical)

### Task 3: Schema 1.1 — JSON schemas, Rust mirror types, rule migration

This task changes schemas, the serde mirror, the rule JSON, and two version-string literals **together** so the workspace stays green with the old `evaluate.rs` still in place. Behavior must not change: all fixture goldens and snapshots stay byte-identical (nothing in them pins the report `schema_version`, which is the only emitted value that changes).

**Files:**
- Modify: `schemas/rule.schema.json`
- Modify: `schemas/report.schema.json`
- Modify: `crates/harness-guard-rules/src/schema.rs`
- Modify: `crates/harness-guard-rules/src/loader.rs` (version-string + tool check only; §6.3 validation comes in Task 4)
- Modify: `rules/codex/history-persist-01.json`
- Modify: `crates/harness-guard-cli/src/main.rs:312` (`schema_version: "1.0"` → `"1.1"` in `build_report`)
- Modify: `crates/harness-guard-rules/tests/schema_validation.rs` (fix tests that pin `1.0`)

**Interfaces:**
- Produces: `MatchSpec`, `MatchValue`, `IntegerBounds` types and `RawRule.unknown_subject: String`, `RawOutcome.match_spec: MatchSpec`, `Observation.integer_bounds: Option<IntegerBounds>` — Tasks 4 and 6 consume these exact names.

- [ ] **Step 1: Update `schemas/rule.schema.json` to 1.1**

Apply these exact deltas (keep everything else):
- `"$id": "harness-guard:rule:1.1"`, `"schema_version": { "const": "1.1" }`.
- `"tool": { "enum": ["codex", "claude-code", "grok-build"] }`.
- `"scopes"` items enum: `["user", "project", "local", "managed"]`.
- Add `"unknown_subject": { "type": "string", "minLength": 1 }` to `properties` and to the top-level `required` array (after `"why_it_matters"`).
- `observation.type` enum: `["enum", "bool", "integer"]`; add to observation `properties`:
  `"integer_bounds": { "type": ["object", "null"], "required": ["min", "max"], "additionalProperties": false, "properties": { "min": { "type": "integer" }, "max": { "type": "integer" } } }`
  and append to the observation object an `allOf`:
  ```json
  "allOf": [
    { "if": { "properties": { "type": { "const": "integer" } } },
      "then": { "required": ["integer_bounds"], "properties": { "integer_bounds": { "type": "object" } } } },
    { "if": { "properties": { "type": { "enum": ["enum", "bool"] } } },
      "then": { "properties": { "integer_bounds": { "const": null } } } }
  ]
  ```
- In each outcome: add `"match"` to the outcome `required` list and to `properties`:
  ```json
  "match": {
    "oneOf": [
      { "type": "object", "required": ["equals"], "additionalProperties": false,
        "properties": { "equals": { "type": "object", "required": ["value"], "additionalProperties": false,
          "properties": { "value": { "type": ["string", "boolean", "integer"] } } } } },
      { "type": "object", "required": ["any_of"], "additionalProperties": false,
        "properties": { "any_of": { "type": "object", "required": ["values"], "additionalProperties": false,
          "properties": { "values": { "type": "array", "minItems": 1,
            "items": { "type": ["string", "boolean", "integer"] } } } } } },
      { "type": "object", "required": ["int_range"], "additionalProperties": false,
        "properties": { "int_range": { "type": "object", "required": ["min", "max"], "additionalProperties": false,
          "properties": { "min": { "type": ["integer", "null"] }, "max": { "type": ["integer", "null"] } } } } },
      { "type": "object", "required": ["unset"], "additionalProperties": false,
        "properties": { "unset": { "const": true } } },
      { "type": "object", "required": ["unrecognized"], "additionalProperties": false,
        "properties": { "unrecognized": { "const": true } } }
    ]
  }
  ```
- Append two entries to the outcome `allOf` (status legality where the schema can express it, §6.2):
  ```json
  { "if": { "properties": { "match": { "type": "object", "anyOf": [ { "required": ["unset"] }, { "required": ["unrecognized"] } ] } } },
    "then": { "properties": { "status": { "const": "unknown" } } } },
  { "if": { "properties": { "match": { "type": "object", "anyOf": [ { "required": ["equals"] }, { "required": ["any_of"] }, { "required": ["int_range"] } ] } } },
    "then": { "properties": { "status": { "enum": ["pass", "finding"] } } } }
  ```

- [ ] **Step 2: Update `schemas/report.schema.json` to 1.1**

Deltas: `"$id": "harness-guard:report:1.1"`, `"schema_version": { "const": "1.1" }`, and `tools[].tool` enum → `["claude-code", "codex", "grok-build"]`. Nothing else changes (finding shape, status matrix, summary are unchanged per §5.6).

- [ ] **Step 3: Extend `crates/harness-guard-rules/src/schema.rs`**

Add after `Observation` (and add `integer_bounds` to `Observation` plus `unknown_subject` to `RawRule` after `why_it_matters`, `match_spec` to `RawOutcome` after `status`):

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct IntegerBounds {
    pub min: i64,
    pub max: i64,
}

/// The closed match-primitive set (spec §6.2). Externally tagged: exactly one
/// primitive key per outcome; serde rejects multiple keys, the JSON schema
/// oneOf pins the shapes, and loader validation (§6.3) proves totality.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum MatchSpec {
    Equals { value: MatchValue },
    AnyOf { values: Vec<MatchValue> },
    IntRange { min: Option<i64>, max: Option<i64> },
    Unset(bool),
    Unrecognized(bool),
}

/// Untagged and ordered: JSON true/false → Bool, integers → Int (floats and
/// out-of-i64 numbers fail deserialization), strings → Str.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum MatchValue {
    Bool(bool),
    Int(i64),
    Str(String),
}
```

Field additions:

```rust
// in RawRule, after why_it_matters:
    /// Subject of the engine's fixed unknown-message template:
    /// "Cannot determine {unknown_subject}: {reason}".
    pub unknown_subject: String,

// in Observation, after allowed_render:
    #[serde(default)]
    pub integer_bounds: Option<IntegerBounds>,

// in RawOutcome, after status:
    #[serde(rename = "match")]
    pub match_spec: MatchSpec,
```

- [ ] **Step 4: Migrate `rules/codex/history-persist-01.json`**

Exact edits (everything else byte-for-byte unchanged):
- `"schema_version": "1.1"`.
- After `"why_it_matters"`: add `"unknown_subject": "history persistence posture",`.
- Outcome 1 (pass): add `"match": { "equals": { "value": "none" } },` before `"when"`.
- Outcome 2 (finding): add `"match": { "equals": { "value": "save-all" } },`.
- Outcome 3 (unset-unknown): add `"match": { "unset": true },`.
- Outcome 4 (unrecognized-unknown): add `"match": { "unrecognized": true },` **and** replace its `unknown_reason` value with exactly:
  `"history.persistence is set to an unrecognized value — raw values are never displayed"`
  (this is the string `evaluate.rs` hardcodes today and the golden `fixtures/codex/unrecognized-value/expected.json` pins; moving it into data is what lets the engine reproduce the golden byte-identically. The old reason text "Value is outside the documented enum…" is dropped.)

Source `schema_version` values stay `"1.0"` (`source.schema.json` is unversioned by this change; the loader keeps checking `source.schema_version == "1.0"`).

- [ ] **Step 5: Minimal loader updates so the workspace compiles and rules load**

In `crates/harness-guard-rules/src/loader.rs`:
- `validate_rule`: `if raw.schema_version != "1.1" { return invalid(raw, "schema_version must be 1.1"); }`
- Replace the `raw.tool != "codex"` check with the closed set:
  ```rust
  if !matches!(raw.tool.as_str(), "codex" | "claude-code" | "grok-build") {
      return invalid(raw, "tool must be codex, claude-code, or grok-build");
  }
  ```
- Widen the scopes check items to `"user" | "project" | "local" | "managed"`.
- Widen the observation `value_type` check to `"enum" | "bool" | "integer"`.
- Add non-empty check: `if raw.unknown_subject.is_empty() { return invalid(raw, "unknown_subject must be non-empty"); }`

- [ ] **Step 6: Bump the emitted report schema version**

In `crates/harness-guard-cli/src/main.rs` `build_report`: `schema_version: "1.1".to_string(),`.

- [ ] **Step 7: Fix schema-validation tests that pin 1.0 and run everything**

In `crates/harness-guard-rules/tests/schema_validation.rs`, update any literal expectations tied to schema 1.0 (e.g. mutation helpers that rebuild the rule JSON) so the suite validates the migrated rule against the 1.1 schema. Keep the negative tests (missing source, bad URL) — they must still fail validation.

Run: `cargo test --workspace`
Expected: PASS, including `scan_fixtures::fixture_exit_codes_and_json_goldens` and all `insta` snapshots **unchanged** (byte-identity: old `evaluate.rs` still drives evaluation and ignores `match_spec`).

- [ ] **Step 8: Add schema negative tests for the new 1.1 surface**

Append to `crates/harness-guard-rules/tests/schema_validation.rs` (using its existing `compiled`/`rule_json` helpers):

```rust
#[test]
fn rule_schema_rejects_missing_match() {
    let v = compiled("rule.schema.json");
    let mut json = rule_json();
    json["outcomes"][0].as_object_mut().unwrap().remove("match");
    assert!(v.validate(&json).is_err(), "match is required on every outcome");
}

#[test]
fn rule_schema_rejects_multiple_primitives_in_one_match() {
    let v = compiled("rule.schema.json");
    let mut json = rule_json();
    json["outcomes"][0]["match"] =
        serde_json::json!({ "equals": { "value": "none" }, "unset": true });
    assert!(v.validate(&json).is_err());
}

#[test]
fn rule_schema_rejects_unset_match_with_pass_status() {
    let v = compiled("rule.schema.json");
    let mut json = rule_json();
    json["outcomes"][0]["match"] = serde_json::json!({ "unset": true });
    // outcome 0 has status "pass" — unset must force status unknown
    assert!(v.validate(&json).is_err());
}

#[test]
fn rule_schema_rejects_empty_any_of_and_inverted_int_range_shape() {
    let v = compiled("rule.schema.json");
    let mut json = rule_json();
    json["outcomes"][0]["match"] = serde_json::json!({ "any_of": { "values": [] } });
    assert!(v.validate(&json).is_err(), "empty any_of must fail");
    let mut json = rule_json();
    json["outcomes"][0]["match"] = serde_json::json!({ "int_range": { "min": 1 } });
    assert!(v.validate(&json).is_err(), "int_range requires both min and max keys");
}

#[test]
fn rule_schema_requires_integer_bounds_for_integer_observations() {
    let v = compiled("rule.schema.json");
    let mut json = rule_json();
    json["observation"]["type"] = serde_json::json!("integer");
    // no integer_bounds supplied
    assert!(v.validate(&json).is_err());
}
```

(If `rule_json()` does not exist as a helper yet, add it: read + parse `rules/codex/history-persist-01.json` exactly like the existing tests do.)

Run: `cargo test -p harness-guard-rules --test schema_validation`
Expected: PASS.

- [ ] **Step 9: Gates and commit**

Run: `cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo deny check && cargo test --workspace`
Expected: all green, snapshots untouched (`git diff --stat` shows no `fixtures/` or `tests/snapshots/` changes).

```bash
git add schemas/ crates/harness-guard-rules/ crates/harness-guard-cli/src/main.rs rules/codex/history-persist-01.json
git commit -m "feat: bump rule and report schemas to 1.1 with declarative match primitives"
```

### Task 4: Load-time totality validation (§6.3) + malformed-rule corpus

**Files:**
- Modify: `crates/harness-guard-rules/src/loader.rs`
- Test: same file (`#[cfg(test)]` module) — the corpus lives next to the validator.

**Interfaces:**
- Consumes: `MatchSpec`, `MatchValue`, `IntegerBounds` from Task 3.
- Produces: `validate_rule` rejecting every §6.3 violation with a specific error; Task 6's engine may then assume totality. Also `pub fn load_rules_from_sources(sources: &[(&str, &str)]) -> Result<Vec<ValidatedRule>, RuleValidationError>` is NOT added here — keep `load_rules()` as-is until Task 8.

- [ ] **Step 1: Write the failing malformed-rule corpus tests**

Append a test module to `loader.rs`. Build each corpus entry by deserializing a mutated copy of the bundled rule JSON (`include_str!` already in scope) so tests stay in sync with the real shape:

```rust
#[cfg(test)]
mod validation_tests {
    use super::*;

    fn raw_rule() -> RawRule {
        serde_json::from_str(RULE_HISTORY_PERSIST).unwrap()
    }

    fn raw_with(mutate: impl FnOnce(&mut serde_json::Value)) -> RawRule {
        let mut json: serde_json::Value = serde_json::from_str(RULE_HISTORY_PERSIST).unwrap();
        mutate(&mut json);
        serde_json::from_value(json).expect("corpus mutation still deserializes")
    }

    fn assert_rejected(rule: RawRule, needle: &str) {
        let error = ValidatedRule::try_from_raw(rule).expect_err("must be rejected");
        assert!(
            error.0.contains(needle),
            "error {:?} should mention {needle:?}",
            error.0
        );
    }

    #[test]
    fn bundled_rule_passes_validation() {
        assert!(ValidatedRule::try_from_raw(raw_rule()).is_ok());
    }

    // §6.3.1 type agreement
    #[test]
    fn equals_bool_on_enum_observation_is_rejected() {
        assert_rejected(
            raw_with(|j| j["outcomes"][0]["match"] = serde_json::json!({"equals": {"value": true}})),
            "match value type",
        );
    }
    #[test]
    fn int_range_on_enum_observation_is_rejected() {
        assert_rejected(
            raw_with(|j| {
                j["outcomes"][0]["match"] =
                    serde_json::json!({"int_range": {"min": 0, "max": 1}})
            }),
            "int_range",
        );
    }

    // §6.3.2 domain membership
    #[test]
    fn equals_value_outside_allowed_render_is_rejected() {
        assert_rejected(
            raw_with(|j| {
                j["outcomes"][0]["match"] =
                    serde_json::json!({"equals": {"value": "archive"}})
            }),
            "allowed_render",
        );
    }
    #[test]
    fn equals_value_unset_string_is_rejected() {
        // "unset" is the unset-rendering token, not a matchable domain value.
        assert_rejected(
            raw_with(|j| {
                j["outcomes"][0]["match"] = serde_json::json!({"equals": {"value": "unset"}})
            }),
            "allowed_render",
        );
    }

    // §6.3.3 cardinality
    #[test]
    fn missing_unrecognized_catch_all_is_rejected() {
        assert_rejected(
            raw_with(|j| {
                let outcomes = j["outcomes"].as_array_mut().unwrap();
                outcomes.retain(|o| o["match"].get("unrecognized").is_none());
            }),
            "unrecognized",
        );
    }
    #[test]
    fn duplicate_unset_outcome_is_rejected() {
        assert_rejected(
            raw_with(|j| {
                let unset = j["outcomes"][2].clone();
                j["outcomes"].as_array_mut().unwrap().push(unset);
            }),
            "exactly one",
        );
    }

    // §6.3.4 exhaustiveness
    #[test]
    fn uncovered_enum_domain_value_is_rejected() {
        assert_rejected(
            raw_with(|j| {
                // Domain becomes {save-all, none, keep-latest} but no outcome
                // matches keep-latest.
                j["observation"]["allowed_render"] =
                    serde_json::json!(["save-all", "none", "keep-latest", "unset"]);
            }),
            "exhaustive",
        );
    }

    // §6.3.5 overlap freedom
    #[test]
    fn overlapping_value_outcomes_are_rejected() {
        assert_rejected(
            raw_with(|j| {
                j["outcomes"][1]["match"] =
                    serde_json::json!({"any_of": {"values": ["save-all", "none"]}})
            }),
            "overlap",
        );
    }

    // §6.3.6 status legality
    #[test]
    fn unset_with_pass_status_is_rejected() {
        assert_rejected(
            raw_with(|j| {
                j["outcomes"][2]["status"] = serde_json::json!("pass");
                j["outcomes"][2]["unknown_reason"] = serde_json::Value::Null;
                j["outcomes"][2]["verify_url"] = serde_json::Value::Null;
                j["outcomes"][2]["confidence"] = serde_json::json!("high");
            }),
            "unset",
        );
    }
    #[test]
    fn value_match_with_unknown_status_is_rejected() {
        assert_rejected(
            raw_with(|j| {
                j["outcomes"][0]["status"] = serde_json::json!("unknown");
                j["outcomes"][0]["confidence"] = serde_json::Value::Null;
                j["outcomes"][0]["unknown_reason"] = serde_json::json!("x");
            }),
            "equals",
        );
    }
}
```

Also add integer-rule corpus tests (build a synthetic integer rule by mutating observation type to `integer`, `allowed_render` to `["unset"]`, `integer_bounds` `{min: 0, max: 90}`, and outcomes to `int_range` blocks) covering: bounds-escaping `int_range` (`{"min": -5, "max": 10}` → rejected, "integer_bounds"), inverted range (`min > max` → rejected), gap in coverage (`[0,29]` + `[40,90]` → rejected, "cover"), overlapping ranges (`[0,40]` + `[30,90]` → rejected, "overlap"), non-`["unset"]` `allowed_render` for integer observations (rejected, "allowed_render"), and full coverage `[0,29]` + `[30,90]` + unset + unrecognized → accepted.

- [ ] **Step 2: Run to verify the new tests fail**

Run: `cargo test -p harness-guard-rules validation_tests`
Expected: FAIL (validator does not yet reject these).

- [ ] **Step 3: Implement the §6.3 validator**

Add to `loader.rs` (called from `validate_rule` after the existing per-outcome loop):

```rust
use crate::schema::{IntegerBounds, MatchSpec, MatchValue, Observation};

fn validate_match_semantics(raw: &RawRule) -> Result<(), RuleValidationError> {
    let observation = &raw.observation;
    let value_type = observation.value_type.as_str();

    // Integer observations render from the parsed i64, never a string
    // allowlist (§5.7); pin allowed_render to exactly ["unset"].
    if value_type == "integer" {
        if observation.integer_bounds.is_none() {
            return invalid(raw, "integer observations require integer_bounds");
        }
        if observation.allowed_render != ["unset".to_string()] {
            return invalid(raw, "integer observations must set allowed_render to [\"unset\"]");
        }
        let bounds = observation.integer_bounds.expect("checked above");
        if bounds.min > bounds.max {
            return invalid(raw, "integer_bounds min must be <= max");
        }
    } else {
        if observation.integer_bounds.is_some() {
            return invalid(raw, "integer_bounds is only valid for integer observations");
        }
        if !observation.allowed_render.iter().any(|r| r == "unset") {
            return invalid(raw, "allowed_render must include the \"unset\" rendering token");
        }
        if value_type == "bool" {
            let mut expected: Vec<&str> = vec!["true", "false", "unset"];
            expected.sort_unstable();
            let mut actual: Vec<&str> =
                observation.allowed_render.iter().map(String::as_str).collect();
            actual.sort_unstable();
            if actual != expected {
                return invalid(raw, "bool observations must set allowed_render to [\"true\", \"false\", \"unset\"]");
            }
        }
    }

    // §6.3.3 cardinality + §6.3.6 status legality.
    let mut unset_count = 0usize;
    let mut unrecognized_count = 0usize;
    for outcome in &raw.outcomes {
        match &outcome.match_spec {
            MatchSpec::Unset(flag) => {
                unset_count += 1;
                if !flag || outcome.status != "unknown" {
                    return invalid(raw, "unset outcomes must be `\"unset\": true` with status unknown");
                }
            }
            MatchSpec::Unrecognized(flag) => {
                unrecognized_count += 1;
                if !flag || outcome.status != "unknown" {
                    return invalid(raw, "unrecognized outcomes must be `\"unrecognized\": true` with status unknown");
                }
            }
            MatchSpec::Equals { value } => {
                if !matches!(outcome.status.as_str(), "pass" | "finding") {
                    return invalid(raw, "equals outcomes allow only pass or finding status");
                }
                validate_match_value(raw, observation, value)?;
            }
            MatchSpec::AnyOf { values } => {
                if !matches!(outcome.status.as_str(), "pass" | "finding") {
                    return invalid(raw, "any_of outcomes allow only pass or finding status");
                }
                if values.is_empty() {
                    return invalid(raw, "any_of must list at least one value");
                }
                for value in values {
                    validate_match_value(raw, observation, value)?;
                }
            }
            MatchSpec::IntRange { min, max } => {
                if !matches!(outcome.status.as_str(), "pass" | "finding") {
                    return invalid(raw, "int_range outcomes allow only pass or finding status");
                }
                if value_type != "integer" {
                    return invalid(raw, "int_range applies only to integer observations");
                }
                let bounds = observation.integer_bounds.expect("validated above");
                let low = min.unwrap_or(bounds.min);
                let high = max.unwrap_or(bounds.max);
                if low > high {
                    return invalid(raw, "int_range min must be <= max");
                }
                if low < bounds.min || high > bounds.max {
                    return invalid(raw, "int_range must lie within integer_bounds");
                }
            }
        }
    }
    if unset_count != 1 || unrecognized_count != 1 {
        return invalid(raw, "exactly one unset and exactly one unrecognized outcome are required");
    }

    // §6.3.4 exhaustiveness + §6.3.5 overlap freedom.
    match value_type {
        "enum" => validate_enum_partition(raw),
        "bool" => validate_bool_partition(raw),
        _ => validate_integer_partition(raw),
    }
}

fn validate_match_value(
    raw: &RawRule,
    observation: &Observation,
    value: &MatchValue,
) -> Result<(), RuleValidationError> {
    match (observation.value_type.as_str(), value) {
        ("enum", MatchValue::Str(text)) => {
            let in_domain =
                text != "unset" && observation.allowed_render.iter().any(|r| r == text);
            if !in_domain {
                return invalid(raw, "match string values must be in allowed_render (excluding \"unset\")");
            }
        }
        ("bool", MatchValue::Bool(_)) => {}
        ("integer", MatchValue::Int(number)) => {
            let bounds = observation.integer_bounds.expect("validated above");
            if *number < bounds.min || *number > bounds.max {
                return invalid(raw, "match integer values must lie within integer_bounds");
            }
        }
        _ => return invalid(raw, "match value type must agree with observation.type"),
    }
    Ok(())
}
```

Partition checks (same file):

```rust
fn value_sets(raw: &RawRule) -> Vec<Vec<MatchValue>> {
    raw.outcomes
        .iter()
        .filter_map(|outcome| match &outcome.match_spec {
            MatchSpec::Equals { value } => Some(vec![value.clone()]),
            MatchSpec::AnyOf { values } => Some(values.clone()),
            _ => None,
        })
        .collect()
}

fn validate_enum_partition(raw: &RawRule) -> Result<(), RuleValidationError> {
    let domain: Vec<&str> = raw
        .observation
        .allowed_render
        .iter()
        .map(String::as_str)
        .filter(|render| *render != "unset")
        .collect();
    let sets = value_sets(raw);
    let mut seen: Vec<&str> = Vec::new();
    for set in &sets {
        for value in set {
            let MatchValue::Str(text) = value else {
                return invalid(raw, "enum match values must be strings");
            };
            if seen.contains(&text.as_str()) {
                return invalid(raw, "value-match outcomes overlap; evaluation must be order-independent");
            }
            seen.push(text);
        }
    }
    for value in &domain {
        if !seen.contains(value) {
            return invalid(raw, "value-match outcomes are not exhaustive over the enum domain");
        }
    }
    Ok(())
}

fn validate_bool_partition(raw: &RawRule) -> Result<(), RuleValidationError> {
    let sets = value_sets(raw);
    let mut seen: Vec<bool> = Vec::new();
    for set in &sets {
        for value in set {
            let MatchValue::Bool(flag) = value else {
                return invalid(raw, "bool match values must be booleans");
            };
            if seen.contains(flag) {
                return invalid(raw, "value-match outcomes overlap; evaluation must be order-independent");
            }
            seen.push(*flag);
        }
    }
    if !(seen.contains(&true) && seen.contains(&false)) {
        return invalid(raw, "bool outcomes must be exhaustive over true and false");
    }
    Ok(())
}

fn validate_integer_partition(raw: &RawRule) -> Result<(), RuleValidationError> {
    let bounds = raw.observation.integer_bounds.expect("validated above");
    // Every value-matching outcome becomes one or more closed intervals.
    let mut intervals: Vec<(i64, i64)> = Vec::new();
    for outcome in &raw.outcomes {
        match &outcome.match_spec {
            MatchSpec::Equals { value: MatchValue::Int(number) } => {
                intervals.push((*number, *number));
            }
            MatchSpec::AnyOf { values } => {
                for value in values {
                    let MatchValue::Int(number) = value else {
                        return invalid(raw, "integer match values must be integers");
                    };
                    intervals.push((*number, *number));
                }
            }
            MatchSpec::IntRange { min, max } => {
                intervals.push((min.unwrap_or(bounds.min), max.unwrap_or(bounds.max)));
            }
            _ => {}
        }
    }
    intervals.sort_unstable();
    let mut expected_next = bounds.min;
    for (low, high) in &intervals {
        if *low < expected_next {
            return invalid(raw, "integer outcomes overlap; evaluation must be order-independent");
        }
        if *low > expected_next {
            return invalid(raw, "integer outcomes do not cover integer_bounds exhaustively");
        }
        // i64 overflow-safe advance: high == i64::MAX only when bounds.max is.
        expected_next = high.saturating_add(1);
    }
    if intervals.is_empty() || expected_next <= bounds.max {
        return invalid(raw, "integer outcomes do not cover integer_bounds exhaustively");
    }
    Ok(())
}
```

Call `validate_match_semantics(raw)?;` inside `validate_rule` immediately after the existing `for outcome in &raw.outcomes { validate_outcome(...)?; }` loop.

- [ ] **Step 4: Run the corpus to green**

Run: `cargo test -p harness-guard-rules`
Expected: PASS (bundled rule accepted, every corpus mutant rejected with its named error).

- [ ] **Step 5: Gates and commit**

Run: `cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`

```bash
git add crates/harness-guard-rules/src/loader.rs
git commit -m "feat: prove rule totality at load time (type agreement, domains, cardinality, exhaustiveness, overlap freedom)"
```

### Task 5: Typed `ExtractedValue` for TOML

**Files:**
- Modify: `crates/harness-guard-core/src/parse.rs`
- Modify: `crates/harness-guard-core/src/evaluate.rs` (mechanical: `NonString` → `Other`)

**Interfaces:**
- Produces: `pub enum ExtractedValue { Unset, Str(String), Bool(bool), Int(i64), Other }` and object-only dotted-key traversal. Tasks 6 and 12 consume this exact shape. `MAX_NESTING_DEPTH` stays `pub` in `parse.rs` — the single shared definition both parsers use (§5.2 "shared module": one definition, no drift).

- [ ] **Step 1: Write the failing tests**

Replace/extend the extraction tests in `parse.rs`:

```rust
#[test]
fn typed_values_are_extracted() {
    let doc = parse_config("[history]\npersistence = \"none\"\nenabled = true\ndays = 30\nratio = 1.5\n").unwrap();
    assert!(matches!(extract_key(&doc, "history.persistence"), ExtractedValue::Str(ref s) if s == "none"));
    assert!(matches!(extract_key(&doc, "history.enabled"), ExtractedValue::Bool(true)));
    assert!(matches!(extract_key(&doc, "history.days"), ExtractedValue::Int(30)));
    assert!(matches!(extract_key(&doc, "history.ratio"), ExtractedValue::Other));
}

#[test]
fn key_path_through_an_array_or_scalar_is_other_not_unset() {
    // §5.2: traversal is tables-only; array indexing is unsupported in 0.0.1.
    let doc = parse_config("history = [1, 2]\n").unwrap();
    assert!(matches!(extract_key(&doc, "history.persistence"), ExtractedValue::Other));
    let doc = parse_config("history = \"flat\"\n").unwrap();
    assert!(matches!(extract_key(&doc, "history.persistence"), ExtractedValue::Other));
}
```

Run: `cargo test -p harness-guard-core parse` — Expected: FAIL (no `Bool`/`Int`/`Other` variants).

- [ ] **Step 2: Implement**

In `parse.rs` replace the enum and `extract_key`:

```rust
/// Rule-relevant key extraction, typed (§5.2). Only the requested dotted key
/// is retained; Str is held only until the engine checks the rule's domain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtractedValue {
    Unset,
    /// Held only until the engine checks the rule's rendering domain.
    Str(String),
    Bool(bool),
    /// Floats and out-of-i64 numbers become Other, never Int.
    Int(i64),
    /// Present but not representable — never rendered.
    Other,
}

pub fn extract_key(document: &toml::Value, dotted_key: &str) -> ExtractedValue {
    let mut current = document;
    for part in dotted_key.split('.') {
        match current {
            toml::Value::Table(table) => match table.get(part) {
                Some(next) => current = next,
                None => return ExtractedValue::Unset,
            },
            // Tables-only traversal: a path hitting an array or scalar is
            // present-but-not-representable (§5.2).
            _ => return ExtractedValue::Other,
        }
    }
    match current {
        toml::Value::String(text) => ExtractedValue::Str(text.clone()),
        toml::Value::Boolean(flag) => ExtractedValue::Bool(*flag),
        toml::Value::Integer(number) => ExtractedValue::Int(*number),
        _ => ExtractedValue::Other,
    }
}
```

In `evaluate.rs`, replace the two `ExtractedValue::NonString` mentions with `ExtractedValue::Other` and extend the final catch-all arm of `observe()` to `ExtractedValue::Str(_) | ExtractedValue::Bool(_) | ExtractedValue::Int(_) | ExtractedValue::Other => (...unrecognized...)` (Bool/Int on this enum rule were `NonString` before — identical unrecognized behavior, goldens unaffected).

- [ ] **Step 3: Run tests, gates, commit**

Run: `cargo test --workspace` — Expected: PASS, snapshots and goldens untouched.

```bash
git add crates/harness-guard-core/src/parse.rs crates/harness-guard-core/src/evaluate.rs
git commit -m "feat: typed ExtractedValue with tables-only dotted-key traversal"
```

### Task 6: The declarative engine (`engine.rs`)

`engine.rs` lands alongside `evaluate.rs` with the same public surface (`ConfigState`, `evaluate_rule`) in its own module; nothing switches over yet. Its unit tests replicate the full `evaluate.rs` matrix so Task 7's switchover is a pure rewiring.

**Files:**
- Create: `crates/harness-guard-core/src/engine.rs`
- Modify: `crates/harness-guard-core/src/lib.rs` (add `pub mod engine;`)

**Interfaces:**
- Consumes: `ExtractedValue` (Task 5), `MatchSpec`/`MatchValue` (Task 3), `ValidatedRule` totality guarantees (Task 4).
- Produces: `engine::ConfigState` (same variants as `evaluate::ConfigState`) and `engine::evaluate_rule(rule: &ValidatedRule, config: &ConfigState, detected_version: Option<&str>) -> FindingRecord`. Task 7 and Task 14 consume these.

- [ ] **Step 1: Write the engine with its embedded test matrix**

Create `crates/harness-guard-core/src/engine.rs`:

```rust
//! Declarative rule evaluation (spec §6). Rule JSON drives evaluation through
//! the closed match-primitive set; precedence, message templates, and
//! observation rendering are engine-fixed. Loader validation (§6.3) proves at
//! load time that evaluation is total and order-independent, so the lookups
//! here cannot fall through. Degradation is conservative: every path yields a
//! schema-valid FindingRecord.
use crate::parse::{ExtractedValue, ParseFailure};
use crate::readfs::RefusalReason;
use crate::version::version_in_range;
use harness_guard_rules::loader::ValidatedRule;
use harness_guard_rules::report::{Confidence, FindingRecord, Severity, SourceCite, Status};
use harness_guard_rules::schema::{MatchSpec, MatchValue, RawOutcome, RawRule};
use std::collections::BTreeMap;

#[derive(Debug)]
pub enum ConfigState {
    /// Tool detected but no user config file. Other layers may still supply
    /// the effective value, so this remains unknown.
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
    let raw = rule.raw();

    // §6.4.1 — declared unknown conditions beat version bookkeeping.
    match config {
        ConfigState::Unreadable(reason) => {
            return engine_unknown(base, rule, reason.describe().to_string());
        }
        ConfigState::Unparseable(failure) => {
            return engine_unknown(
                base,
                rule,
                format!("config not safely parseable: {}", failure.message),
            );
        }
        ConfigState::Missing | ConfigState::Parsed(_) => {}
    }

    // §6.4.2 — extract the typed value, select the unique matching outcome.
    let value = extracted_value(raw, config);
    let matched = select_outcome(raw, &value);
    let observation = render_observation(raw, &value, matched);

    // §6.4.3 — out-of-range or undetected version wraps the matched outcome.
    let in_range = detected_version
        .map(|version| version_in_range(version, &raw.tested_versions))
        .unwrap_or(false);
    if !in_range {
        return stale(base, raw, matched, observation, detected_version);
    }

    // §6.4.4 — emit the matched outcome with the §5.7 rendering.
    emit(base, rule, matched, observation)
}

fn extracted_value(raw: &RawRule, config: &ConfigState) -> ExtractedValue {
    match config {
        ConfigState::Parsed(values) => values
            .get(&raw.observation.key)
            .cloned()
            .unwrap_or(ExtractedValue::Unset),
        // Missing config: other layers may supply the value — unset.
        // Unreadable/Unparseable are unreachable (early return) but total.
        _ => ExtractedValue::Unset,
    }
}

/// §6.2: present-but-outside-domain values (type mismatches, Other, strings
/// outside allowed_render, out-of-integer_bounds integers) are unrecognized
/// BEFORE any value matching, so an open-ended int_range can never claim an
/// out-of-domain integer.
fn in_domain(raw: &RawRule, value: &ExtractedValue) -> bool {
    match (raw.observation.value_type.as_str(), value) {
        (_, ExtractedValue::Unset) => true,
        ("enum", ExtractedValue::Str(text)) => {
            text != "unset" && raw.observation.allowed_render.iter().any(|r| r == text)
        }
        ("bool", ExtractedValue::Bool(_)) => true,
        ("integer", ExtractedValue::Int(number)) => raw
            .observation
            .integer_bounds
            .is_some_and(|bounds| *number >= bounds.min && *number <= bounds.max),
        _ => false,
    }
}

fn select_outcome<'r>(raw: &'r RawRule, value: &ExtractedValue) -> &'r RawOutcome {
    if !in_domain(raw, value) {
        return unrecognized_outcome(raw);
    }
    if matches!(value, ExtractedValue::Unset) {
        return unset_outcome(raw);
    }
    raw.outcomes
        .iter()
        .find(|outcome| match_fires(&outcome.match_spec, value))
        // Unreachable for validated rules (§6.3 exhaustiveness); stay total
        // and conservative rather than panicking on a hostile forked ruleset.
        .unwrap_or_else(|| unrecognized_outcome(raw))
}

fn match_fires(spec: &MatchSpec, value: &ExtractedValue) -> bool {
    match (spec, value) {
        (MatchSpec::Equals { value: expected }, observed) => value_eq(expected, observed),
        (MatchSpec::AnyOf { values }, observed) => {
            values.iter().any(|expected| value_eq(expected, observed))
        }
        (MatchSpec::IntRange { min, max }, ExtractedValue::Int(number)) => {
            min.map_or(true, |low| *number >= low) && max.map_or(true, |high| *number <= high)
        }
        _ => false,
    }
}

fn value_eq(expected: &MatchValue, observed: &ExtractedValue) -> bool {
    match (expected, observed) {
        (MatchValue::Str(left), ExtractedValue::Str(right)) => left == right,
        (MatchValue::Bool(left), ExtractedValue::Bool(right)) => left == right,
        (MatchValue::Int(left), ExtractedValue::Int(right)) => left == right,
        _ => false,
    }
}

fn unset_outcome(raw: &RawRule) -> &RawOutcome {
    raw.outcomes
        .iter()
        .find(|outcome| matches!(outcome.match_spec, MatchSpec::Unset(_)))
        .expect("validated rules carry exactly one unset outcome (§6.3.3)")
}

fn unrecognized_outcome(raw: &RawRule) -> &RawOutcome {
    raw.outcomes
        .iter()
        .find(|outcome| matches!(outcome.match_spec, MatchSpec::Unrecognized(_)))
        .expect("validated rules carry exactly one unrecognized outcome (§6.3.3)")
}

/// §5.7: observations render ONLY from the parsed typed value, never source
/// text. Enum strings passed the domain check; integers passed the bounds
/// check — re-serializing them cannot leak arbitrary content.
fn render_observation(
    raw: &RawRule,
    value: &ExtractedValue,
    matched: &RawOutcome,
) -> Option<String> {
    let key = &raw.observation.key;
    match matched.match_spec {
        MatchSpec::Unrecognized(_) => None,
        MatchSpec::Unset(_) => Some(format!("{key} unset in user config")),
        _ => match value {
            ExtractedValue::Str(text) => Some(format!("{key} = \"{text}\"")),
            ExtractedValue::Bool(flag) => Some(format!("{key} = {flag}")),
            ExtractedValue::Int(number) => Some(format!("{key} = {number}")),
            ExtractedValue::Unset | ExtractedValue::Other => None,
        },
    }
}

fn emit(
    base: FindingRecord,
    rule: &ValidatedRule,
    outcome: &RawOutcome,
    observation: Option<String>,
) -> FindingRecord {
    match outcome.status.as_str() {
        "pass" => checked(FindingRecord {
            status: Status::Pass,
            severity: None,
            confidence: confidence_of(outcome),
            message: outcome.message.clone(),
            observation,
            remediation: None,
            ..base
        }),
        "finding" => checked(FindingRecord {
            status: Status::Finding,
            severity: severity_of(outcome),
            confidence: confidence_of(outcome),
            message: outcome.message.clone(),
            observation,
            remediation: outcome.remediation.clone(),
            ..base
        }),
        // unknown — the unset and unrecognized outcomes (§6.3.6).
        _ => {
            let reason = outcome.unknown_reason.clone().unwrap_or_default();
            unknown_record(base, rule, reason, observation, outcome.verify_url.clone())
        }
    }
}

/// Engine-level unknowns (unreadable/unparseable config). verify_url comes
/// from the rule's single unset outcome — deterministic by §6.3.3.
fn engine_unknown(base: FindingRecord, rule: &ValidatedRule, reason: String) -> FindingRecord {
    let verify_url = unset_outcome(rule.raw()).verify_url.clone();
    unknown_record(base, rule, reason, None, verify_url)
}

/// The engine's only unknown-message template (plan assumption 1):
/// "Cannot determine {unknown_subject}: {reason}". No other interpolation of
/// rule text exists anywhere in the engine.
fn unknown_record(
    base: FindingRecord,
    rule: &ValidatedRule,
    reason: String,
    observation: Option<String>,
    verify_url: Option<String>,
) -> FindingRecord {
    checked(FindingRecord {
        status: Status::Unknown,
        severity: None,
        confidence: None,
        evidence_class: None,
        message: format!("Cannot determine {}: {reason}", rule.raw().unknown_subject),
        observation,
        remediation: None,
        source: None,
        unknown_reason: Some(reason),
        verify_url,
        stale_reason: None,
        ..base
    })
}

fn stale(
    base: FindingRecord,
    raw: &RawRule,
    matched: &RawOutcome,
    observation: Option<String>,
    detected_version: Option<&str>,
) -> FindingRecord {
    let stale_reason = match detected_version {
        None => "tool version not detected — no version marker found on PATH".to_string(),
        Some(version) => format!(
            "detected version {version} is outside every tested range (max tested {})",
            raw.tested_versions
                .iter()
                .map(|tested| tested.max.as_str())
                .max()
                .unwrap_or("?")
        ),
    };
    // The unrecognized+stale safe fallback phrasing is preserved verbatim
    // (adjudicated review finding 9; spec §6.4.3).
    let message = if matches!(matched.match_spec, MatchSpec::Unrecognized(_)) {
        "Unverified — last-known rule indicates the configured value cannot be interpreted safely. Observed: unrecognized value (raw value withheld).".to_string()
    } else {
        format!(
            "Unverified — last-known rule indicates: {} Observed: {}.",
            matched.message,
            observation
                .as_deref()
                .unwrap_or("unrecognized value (raw value withheld)")
        )
    };
    checked(FindingRecord {
        status: Status::StaleRuleset,
        severity: None,
        confidence: None,
        message,
        observation,
        remediation: None,
        stale_reason: Some(stale_reason),
        ..base
    })
}

fn confidence_of(outcome: &RawOutcome) -> Option<Confidence> {
    outcome.confidence.as_deref().map(|confidence| match confidence {
        "low" => Confidence::Low,
        "medium" => Confidence::Medium,
        _ => Confidence::High,
    })
}

fn severity_of(outcome: &RawOutcome) -> Option<Severity> {
    outcome.severity.as_deref().map(|severity| {
        if severity == "info" { Severity::Info } else { Severity::Warning }
    })
}

fn base_record(rule: &ValidatedRule) -> FindingRecord {
    let raw = rule.raw();
    let primary_source = rule.primary_source();
    let tested_version = &raw.tested_versions[0];
    FindingRecord {
        rule_id: raw.id.clone(),
        status: Status::Unknown,
        severity: None,
        confidence: None,
        evidence_class: Some(primary_source.evidence_class.clone()),
        message: String::new(),
        observation: None,
        remediation: None,
        source: Some(SourceCite {
            url: primary_source.url.clone(),
            retrieved: primary_source.retrieved.clone(),
        }),
        valid_from: Some(tested_version.min.clone()),
        valid_until: Some(tested_version.max.clone()),
        limitations: raw.limitations.clone(),
        unknown_reason: None,
        verify_url: None,
        stale_reason: None,
    }
}

fn checked(finding: FindingRecord) -> FindingRecord {
    finding
        .validate()
        .expect("engine must construct a schema-valid finding record");
    finding
}
```

- [ ] **Step 2: Port the full `evaluate.rs` test matrix into `engine.rs`**

Copy the entire `#[cfg(test)] mod tests` block from `crates/harness-guard-core/src/evaluate.rs` (all 11 tests: `none_in_range_passes_with_citation`, `unset_in_range_is_unknown_because_other_layers_are_uninspected`, `explicit_save_all_is_warning_finding`, `unrecognized_value_is_unknown_and_never_echoed`, `non_string_value_is_unknown` — adapted to `ExtractedValue::Other`, `missing_config_with_tool_detected_is_unknown`, `unreadable_config_is_unknown`, `undetected_version_is_stale_never_pass`, `out_of_range_version_is_stale_never_pass`, `unknown_beats_stale_when_config_unreadable`, `stale_unrecognized_value_uses_safe_nonempty_fallback_without_raw_value`) into `engine.rs`'s test module, changing only the `use` paths (`super::*` now refers to the engine). Add one exact-string test that pins the migrated message contract:

```rust
#[test]
fn unknown_messages_reproduce_the_golden_template() {
    let finding = evaluate_rule(
        &rule(),
        &ConfigState::Unreadable(crate::readfs::RefusalReason::PermissionDenied),
        Some("0.144.5"),
    );
    assert_eq!(
        finding.message,
        "Cannot determine history persistence posture: config file is not readable (permission denied)"
    );
    let finding = evaluate_rule(&rule(), &parsed(None), Some("0.144.5"));
    assert_eq!(
        finding.message,
        "Cannot determine history persistence posture: history.persistence is unset in the user-level config; uninspected system, profile, trusted-project, or CLI layers may determine the effective value."
    );
    let finding = evaluate_rule(
        &rule(),
        &parsed(Some(ExtractedValue::Str("archive".into()))),
        Some("0.144.5"),
    );
    assert_eq!(
        finding.message,
        "Cannot determine history persistence posture: history.persistence is set to an unrecognized value — raw values are never displayed"
    );
}
```

Register the module in `lib.rs`: `pub mod engine;` (keep `pub mod evaluate;` for now).

- [ ] **Step 3: Run engine tests**

Run: `cargo test -p harness-guard-core engine`
Expected: PASS — every ported test green against the engine.

- [ ] **Step 4: Gates and commit**

Run: `cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`

```bash
git add crates/harness-guard-core/src/engine.rs crates/harness-guard-core/src/lib.rs
git commit -m "feat: declarative rule engine evaluating from match primitives"
```

### Task 7: Switch scan to the engine, delete `evaluate.rs`, prove byte-identity

**Files:**
- Modify: `crates/harness-guard-core/src/scan.rs` (imports only: `crate::evaluate::` → `crate::engine::`)
- Delete: `crates/harness-guard-core/src/evaluate.rs`
- Modify: `crates/harness-guard-core/src/lib.rs` (remove `pub mod evaluate;`)

- [ ] **Step 1: Rewire scan.rs**

In `scan.rs` change `use crate::evaluate::{ConfigState, evaluate_rule};` to `use crate::engine::{ConfigState, evaluate_rule};`. No other change.

- [ ] **Step 2: Delete `evaluate.rs` and its module registration**

```bash
git rm crates/harness-guard-core/src/evaluate.rs
```
Remove `pub mod evaluate;` from `lib.rs`.

- [ ] **Step 3: Byte-identity gate (§6.6 acceptance)**

Run, in order:
```bash
cargo test --workspace
git status --porcelain fixtures/ crates/harness-guard-cli/tests/snapshots/
```
Expected: all tests PASS with **zero** modified files under `fixtures/` and `tests/snapshots/` — no `insta` pending snapshots (`cargo insta pending-snapshots` reports none if insta CLI is installed; otherwise absence of `*.snap.new` files under `crates/harness-guard-cli/tests/snapshots/` is the check). If any snapshot or golden differs, the engine is wrong — fix the engine, never the golden (superpowers:systematic-debugging).

- [ ] **Step 4: Gates and commit**

Run: `cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo deny check && cargo test --workspace`
On macOS also: `scripts/no-egress/run-macos.sh` — Expected: ok lines, no denials.

```bash
git add -A crates/harness-guard-core/
git commit -m "feat: evaluate rules through the declarative engine; delete bespoke evaluate.rs"
```

### Task 8: `build.rs` rule embedding + disk/embedded 1:1 + path-consistency tests

**Files:**
- Create: `crates/harness-guard-rules/build.rs`
- Modify: `crates/harness-guard-rules/src/loader.rs` (replace the hand-maintained `include_str!` vec at `loader.rs:319`)
- Create: `crates/harness-guard-rules/tests/embedding.rs`

**Interfaces:**
- Produces: `EMBEDDED_RULES: &[(&str, &str)]` (relative-path key like `"codex/history-persist-01.json"`, file contents) generated into `OUT_DIR`; `load_rules()` parses/validates/sorts all of them. WP3 tasks then add rules with zero Rust changes.

- [ ] **Step 1: Write the build script (std-only, no new dependencies)**

Create `crates/harness-guard-rules/build.rs`:

```rust
//! Embeds every rules/**/*.json (except ruleset.json) at compile time
//! (spec §6.5). Std-only; a forgotten or orphaned rule fails the 1:1 test in
//! tests/embedding.rs, not review attention.
use std::path::{Path, PathBuf};

fn main() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let rules_dir = manifest.join("../../rules");
    println!("cargo::rerun-if-changed={}", rules_dir.display());

    let mut rule_files = Vec::new();
    collect_json(&rules_dir, &rules_dir, &mut rule_files);
    rule_files.sort();

    let mut generated = String::from(
        "/// (relative path under rules/, file contents) — generated by build.rs.\n\
         pub const EMBEDDED_RULES: &[(&str, &str)] = &[\n",
    );
    for relative in &rule_files {
        let absolute = rules_dir.join(relative);
        generated.push_str(&format!(
            "    ({:?}, include_str!({:?})),\n",
            relative,
            absolute.display()
        ));
    }
    generated.push_str("];\n");

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    std::fs::write(out_dir.join("embedded_rules.rs"), generated).unwrap();
}

fn collect_json(root: &Path, dir: &Path, out: &mut Vec<String>) {
    for entry in std::fs::read_dir(dir).expect("rules directory is readable") {
        let path = entry.expect("rules entry is readable").path();
        if path.is_dir() {
            collect_json(root, &path, out);
        } else if path.extension().is_some_and(|ext| ext == "json")
            && path.file_name().is_some_and(|name| name != "ruleset.json")
        {
            let relative = path
                .strip_prefix(root)
                .unwrap()
                .components()
                .map(|component| component.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");
            out.push(relative);
        }
    }
}
```

- [ ] **Step 2: Replace the loader's hardcoded vec**

In `loader.rs`, delete `const RULE_HISTORY_PERSIST: &str = include_str!(...)` and replace `load_rules()`:

```rust
include!(concat!(env!("OUT_DIR"), "/embedded_rules.rs"));

/// All bundled rules, sorted by id. Panics only on a corrupt embed, which
/// `cargo test` catches before any release build ships.
pub fn load_rules() -> Vec<ValidatedRule> {
    let mut rules: Vec<ValidatedRule> = EMBEDDED_RULES
        .iter()
        .map(|(path, text)| {
            let raw: RawRule = serde_json::from_str(text)
                .unwrap_or_else(|error| panic!("embedded rule {path} is invalid JSON: {error}"));
            ValidatedRule::try_from_raw(raw)
                .unwrap_or_else(|error| panic!("embedded rule {path} failed validation: {error}"))
        })
        .collect();
    rules.sort_by(|left, right| left.raw().id.cmp(&right.raw().id));
    rules
}
```

The validation-test module's `RULE_HISTORY_PERSIST` uses: replace with a helper reading from `EMBEDDED_RULES`:
```rust
fn bundled_rule_text() -> &'static str {
    EMBEDDED_RULES
        .iter()
        .find(|(path, _)| *path == "codex/history-persist-01.json")
        .expect("codex rule is embedded")
        .1
}
```

- [ ] **Step 3: Write the 1:1 and path-consistency tests**

Create `crates/harness-guard-rules/tests/embedding.rs`:

```rust
//! §6.5: embedded rules correspond 1:1 with the on-disk rules/ tree, every
//! embedded rule validates, and a rule in rules/<tool>/ declares that tool.
use harness_guard_rules::loader::load_rules;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap()
}

fn on_disk_rule_ids() -> BTreeSet<(String, String)> {
    // (tool-dir, rule id) pairs discovered by walking rules/.
    let mut ids = BTreeSet::new();
    let rules_dir = repo_root().join("rules");
    for tool_entry in std::fs::read_dir(&rules_dir).unwrap() {
        let tool_path = tool_entry.unwrap().path();
        if !tool_path.is_dir() {
            continue;
        }
        let tool_dir = tool_path.file_name().unwrap().to_string_lossy().into_owned();
        for rule_entry in std::fs::read_dir(&tool_path).unwrap() {
            let rule_path = rule_entry.unwrap().path();
            if rule_path.extension().is_some_and(|ext| ext == "json") {
                let json: serde_json::Value =
                    serde_json::from_str(&std::fs::read_to_string(&rule_path).unwrap()).unwrap();
                ids.insert((tool_dir.clone(), json["id"].as_str().unwrap().to_string()));
            }
        }
    }
    ids
}

#[test]
fn embedded_rules_match_the_on_disk_tree_one_to_one() {
    let disk = on_disk_rule_ids();
    let embedded: BTreeSet<(String, String)> = load_rules()
        .iter()
        .map(|rule| (rule.raw().tool.clone(), rule.raw().id.clone()))
        .collect();
    // Path consistency: the tool directory IS the declared tool, so comparing
    // (dir, id) with (tool, id) proves both 1:1 embedding and §5.6 path
    // consistency in one assertion.
    assert_eq!(disk, embedded);
}

#[test]
fn every_embedded_rule_id_is_prefixed_with_its_tool_id() {
    for rule in load_rules() {
        let raw = rule.raw();
        assert!(
            raw.id.starts_with(&format!("{}-", raw.tool)),
            "rule id {} must be prefixed with its tool id {}",
            raw.id,
            raw.tool
        );
    }
}
```

Also update `crates/harness-guard-rules/tests/schema_validation.rs`: replace the `assert_eq!(seen, 1, "slice ships exactly one rule")` line with `assert_eq!(seen, harness_guard_rules::loader::load_rules().len(), "every on-disk rule must be embedded and validated");`.

- [ ] **Step 4: Run and commit**

Run: `cargo test --workspace`
Expected: PASS; `list`/`scan` behavior unchanged (still exactly one rule embedded).

```bash
git add crates/harness-guard-rules/
git commit -m "feat: embed rules via build.rs glob with 1:1 disk/embedded and path-consistency tests"
```

### Task 9: Engine hostility property tests (§6.7)

**Files:**
- Create: `crates/harness-guard-core/tests/engine_hostility.rs`

- [ ] **Step 1: Write the hand-rolled property test (no new dependencies)**

```rust
//! §6.7: for arbitrary ExtractedValue × every bundled rule × every
//! ConfigState × version state, the engine returns a schema-valid record and
//! never renders a string outside the rule's derivable renderings — the
//! hostile-archive-value test generalized.
use harness_guard_core::engine::{ConfigState, evaluate_rule};
use harness_guard_core::parse::{ExtractedValue, ParseFailure};
use harness_guard_core::readfs::RefusalReason;
use harness_guard_rules::loader::load_rules;
use std::collections::BTreeMap;

const HOSTILE_STRINGS: [&str; 8] = [
    "hostile-archive-value",
    "",
    "unset",
    "none\" } { \"injected",
    "{unknown_subject}",
    "$(curl evil)",
    "línea-ünicode-💥",
    "very-long-aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
];

fn hostile_values() -> Vec<ExtractedValue> {
    let mut values = vec![
        ExtractedValue::Unset,
        ExtractedValue::Other,
        ExtractedValue::Bool(true),
        ExtractedValue::Bool(false),
        ExtractedValue::Int(i64::MIN),
        ExtractedValue::Int(-1),
        ExtractedValue::Int(0),
        ExtractedValue::Int(30),
        ExtractedValue::Int(i64::MAX),
    ];
    values.extend(HOSTILE_STRINGS.iter().map(|s| ExtractedValue::Str(s.to_string())));
    values
}

fn config_states(rule_key: &str, value: &ExtractedValue) -> Vec<ConfigState> {
    let mut parsed = BTreeMap::new();
    parsed.insert(rule_key.to_string(), value.clone());
    vec![
        ConfigState::Missing,
        ConfigState::Unreadable(RefusalReason::PermissionDenied),
        ConfigState::Unreadable(RefusalReason::Symlink),
        ConfigState::Unreadable(RefusalReason::Oversized),
        ConfigState::Unreadable(RefusalReason::NotUtf8),
        ConfigState::Unparseable(ParseFailure {
            line: Some(1),
            col: Some(1),
            key_path: None,
            message: "invalid JSON syntax".to_string(),
        }),
        ConfigState::Parsed(parsed),
    ]
}

#[test]
fn engine_is_total_schema_valid_and_never_leaks() {
    for rule in load_rules() {
        let key = rule.raw().observation.key.clone();
        // Renderings derivable from rule data: domain values, bools, and any
        // in-bounds integer render from the parsed value; hostile strings
        // outside the domain must never appear anywhere in the record.
        for value in hostile_values() {
            for config in config_states(&key, &value) {
                for version in [None, Some("0.144.5"), Some("9.9.9"), Some("not-a-version")] {
                    let finding = evaluate_rule(&rule, &config, version);
                    finding.validate().unwrap_or_else(|error| {
                        panic!("invalid record from rule {} ({error})", rule.raw().id)
                    });
                    let serialized = serde_json::to_string(&finding).unwrap();
                    // Leak-check only when the hostile string actually flowed
                    // through as the observed value (ConfigState::Parsed) —
                    // for Missing/Unreadable/Unparseable states `value` never
                    // reaches the engine, so the record's content is
                    // unrelated to it. "unset" is excluded from the
                    // leak-checked set entirely: it is the engine's own
                    // legitimate unset-outcome rendering token (e.g. "history
                    // .persistence is unset in the user-level config"), not a
                    // leaked raw value, so asserting its absence would be a
                    // false failure rather than a real leak check.
                    if let (ExtractedValue::Str(text), ConfigState::Parsed(_)) = (&value, &config) {
                        let in_domain =
                            rule.raw().observation.allowed_render.iter().any(|r| r == text);
                        if !in_domain && !text.is_empty() && text != "unset" {
                            assert!(
                                !serialized.contains(text.as_str()),
                                "rule {} leaked hostile value {text:?}",
                                rule.raw().id
                            );
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn conservative_direction_never_inverts() {
    // §6.3: nothing falls through to pass. Any non-domain value must yield
    // unknown (in range) or stale-ruleset (out of range) — never pass/finding.
    use harness_guard_rules::report::Status;
    for rule in load_rules() {
        let key = rule.raw().observation.key.clone();
        for value in [ExtractedValue::Other, ExtractedValue::Str("hostile-archive-value".into())] {
            let mut parsed = BTreeMap::new();
            parsed.insert(key.clone(), value);
            let finding = evaluate_rule(&rule, &ConfigState::Parsed(parsed), Some("0.144.5"));
            assert!(
                matches!(finding.status, Status::Unknown | Status::StaleRuleset),
                "rule {} let a non-domain value reach {:?}",
                rule.raw().id,
                finding.status
            );
        }
    }
}
```

Note: `evaluate_rule` with `Some("not-a-version")` exercises the unparseable-detected-version path (`version_in_range` returns false → stale). `harness_guard_core::parse::ParseFailure` fields are `pub`, and `ExtractedValue: Clone`.

- [ ] **Step 2: Run, gates, commit**

Run: `cargo test -p harness-guard-core --test engine_hostility` — Expected: PASS.
Run: `cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`

```bash
git add crates/harness-guard-core/tests/engine_hostility.rs
git commit -m "test: engine hostility matrix — totality, schema validity, no leakage, no inversion"
```

**Phase 1 exit criteria (§13.1):** `codex-history-persist-01` evaluated purely from rule data; goldens/snapshots byte-identical; `evaluate.rs` deleted; every §6.3 check has a negative test; §6.7 corpus green.

---

## Phase 2 — WP2: Harness abstraction

### Task 10: `HarnessId` and the descriptor table

**Files:**
- Create: `crates/harness-guard-core/src/harness.rs`
- Modify: `crates/harness-guard-core/src/lib.rs` (add `pub mod harness;`)

**Interfaces:**
- Produces: `HarnessId` (`Codex | ClaudeCode | GrokBuild`), `HarnessId::ALL` (alphabetical by id), `as_str()`, `parse()`, `ConfigFormat`, `HarnessDescriptor`, `descriptor(id)`. Tasks 11–16 and 20 consume these exact names.

- [ ] **Step 1: Write failing tests + module**

Create `crates/harness-guard-core/src/harness.rs`:

```rust
//! The closed harness set (§3) and per-harness descriptor facts (§5.1).
//! Descriptors are code, not config — adding a harness is a deliberate,
//! compile-visible act; every match on HarnessId is exhaustive. Descriptor
//! facts must be traceable to the evidence recorded with that harness's
//! rules; entries still awaiting fresh retrieval are None and say so.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HarnessId {
    ClaudeCode,
    Codex,
    GrokBuild,
}

impl HarnessId {
    /// Alphabetical by tool id — the contractual report/list ordering (§5.5).
    pub const ALL: [HarnessId; 3] = [HarnessId::ClaudeCode, HarnessId::Codex, HarnessId::GrokBuild];

    pub fn as_str(self) -> &'static str {
        match self {
            HarnessId::ClaudeCode => "claude-code",
            HarnessId::Codex => "codex",
            HarnessId::GrokBuild => "grok-build",
        }
    }

    pub fn parse(text: &str) -> Option<HarnessId> {
        match text {
            "claude-code" => Some(HarnessId::ClaudeCode),
            "codex" => Some(HarnessId::Codex),
            "grok-build" => Some(HarnessId::GrokBuild),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFormat {
    Toml,
    Json,
}

pub struct HarnessDescriptor {
    pub id: HarnessId,
    /// User-scope config file name inside the harness home (§5.1 table).
    pub config_file: &'static str,
    pub config_format: ConfigFormat,
    /// PATH entry name used for detection. None until evidence establishes
    /// one (grok-build: §7.3 protocol output).
    pub path_binary: Option<&'static str>,
    /// npm package the version walk expects. None disables detection
    /// entirely — findings then degrade to stale-ruleset, never a guess.
    pub npm_package: Option<&'static str>,
    /// Symbolic token for a redacted config path that is not under the user
    /// home (reachable only via a home-override env var in the CLI crate).
    pub home_token: &'static str,
}

static CLAUDE_CODE: HarnessDescriptor = HarnessDescriptor {
    id: HarnessId::ClaudeCode,
    config_file: "settings.json",
    config_format: ConfigFormat::Json,
    path_binary: Some("claude"),
    npm_package: Some("@anthropic-ai/claude-code"),
    // Token only; whether a home-override env var exists is a CLI-crate
    // fresh-retrieval item (§5.1, lead: CLAUDE_CONFIG_DIR) — see Task 15.
    home_token: "$CLAUDE_HOME",
};

static CODEX: HarnessDescriptor = HarnessDescriptor {
    id: HarnessId::Codex,
    config_file: "config.toml",
    config_format: ConfigFormat::Toml,
    path_binary: Some("codex"),
    npm_package: Some("@openai/codex"),
    home_token: "$CODEX_HOME",
};

static GROK_BUILD: HarnessDescriptor = HarnessDescriptor {
    id: HarnessId::GrokBuild,
    config_file: "config.toml",
    config_format: ConfigFormat::Toml,
    // §5.3: detection strategy is an output of the §7.3 protocol. Until
    // packaging is established from evidence, detection returns None and
    // every Grok finding degrades to stale-ruleset. Do NOT assume npm.
    path_binary: None,
    npm_package: None,
    home_token: "$GROK_HOME",
};

pub fn descriptor(id: HarnessId) -> &'static HarnessDescriptor {
    match id {
        HarnessId::ClaudeCode => &CLAUDE_CODE,
        HarnessId::Codex => &CODEX,
        HarnessId::GrokBuild => &GROK_BUILD,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_round_trip_and_all_is_alphabetical() {
        for id in HarnessId::ALL {
            assert_eq!(HarnessId::parse(id.as_str()), Some(id));
        }
        let mut names: Vec<&str> = HarnessId::ALL.iter().map(|id| id.as_str()).collect();
        let sorted = { let mut s = names.clone(); s.sort_unstable(); s };
        assert_eq!(names, sorted, "ALL must stay alphabetical by tool id");
        names.dedup();
        assert_eq!(names.len(), 3);
        assert_eq!(HarnessId::parse("cursor"), None);
    }

    #[test]
    fn descriptors_carry_the_spec_table_facts() {
        assert_eq!(descriptor(HarnessId::Codex).config_file, "config.toml");
        assert_eq!(descriptor(HarnessId::Codex).npm_package, Some("@openai/codex"));
        assert_eq!(descriptor(HarnessId::ClaudeCode).config_file, "settings.json");
        assert_eq!(descriptor(HarnessId::ClaudeCode).config_format, ConfigFormat::Json);
        assert_eq!(
            descriptor(HarnessId::ClaudeCode).npm_package,
            Some("@anthropic-ai/claude-code")
        );
        // Grok detection stays off until §7.3 evidence exists.
        assert_eq!(descriptor(HarnessId::GrokBuild).path_binary, None);
        assert_eq!(descriptor(HarnessId::GrokBuild).npm_package, None);
    }
}
```

Add `pub mod harness;` to `lib.rs`.

- [ ] **Step 2: Run, gates, commit**

Run: `cargo test -p harness-guard-core harness` — Expected: PASS.

```bash
git add crates/harness-guard-core/src/harness.rs crates/harness-guard-core/src/lib.rs
git commit -m "feat: closed HarnessId set with static descriptor table"
```

### Task 11: Multi-harness `DiscoveryRoot` + parameterized `readfs::read_config`

**Files:**
- Modify: `crates/harness-guard-core/src/discovery.rs`
- Modify: `crates/harness-guard-core/src/readfs.rs`
- Modify (mechanical fixes): `crates/harness-guard-core/src/{scan.rs,version.rs}` tests, `crates/harness-guard-cli/src/main.rs` — every `DiscoveryRoot { ... }` literal gains the two new homes.

**Interfaces:**
- Consumes: `HarnessId`, `descriptor` (Task 10).
- Produces: `DiscoveryRoot { codex_home, claude_home, grok_home, path_dirs }`, `root.home(harness) -> &Path`, `root.config_path(harness) -> PathBuf`, `readfs::read_config(root, harness) -> ConfigReadOutcome`. Tasks 13–16 consume these signatures.

- [ ] **Step 1: Rewrite `discovery.rs`**

```rust
//! Injected roots — the ONLY way core learns about the filesystem (§9).
use crate::harness::{HarnessId, descriptor};
use std::path::{Path, PathBuf};

/// Explicit discovery scope: one explicit home per harness (§5.1). Only the
/// CLI crate constructs this from the real environment; tests always pass
/// fixture paths. Core has no other door to ambient state (clippy-enforced).
#[derive(Debug, Clone)]
pub struct DiscoveryRoot {
    pub codex_home: PathBuf,
    pub claude_home: PathBuf,
    pub grok_home: PathBuf,
    pub path_dirs: Vec<PathBuf>,
}

impl DiscoveryRoot {
    pub fn home(&self, harness: HarnessId) -> &Path {
        match harness {
            HarnessId::ClaudeCode => &self.claude_home,
            HarnessId::Codex => &self.codex_home,
            HarnessId::GrokBuild => &self.grok_home,
        }
    }

    pub fn config_path(&self, harness: HarnessId) -> PathBuf {
        self.home(harness).join(descriptor(harness).config_file)
    }
}
```

- [ ] **Step 2: Parameterize `readfs::read_config`**

```rust
pub fn read_config(root: &DiscoveryRoot, harness: crate::harness::HarnessId) -> ConfigReadOutcome {
    let path = root.config_path(harness);
    // body unchanged from today — same hardened bounded read
```

- [ ] **Step 3: Mechanically fix every construction/call site**

Compiler-driven: every test helper constructing `DiscoveryRoot` adds
```rust
claude_home: base.join("absent-claude-home"),
grok_home: base.join("absent-grok-home"),
```
(where `base` is the existing temp base; absent paths keep those harnesses undetected so existing single-harness assertions still hold). Every `root.config_path()` call becomes `root.config_path(HarnessId::Codex)`; every `read_config(&root)` becomes `read_config(&root, HarnessId::Codex)`. `main.rs` `discovery_root_from_env` temporarily sets `claude_home`/`grok_home` to `home.join(".claude")`/`home.join(".grok")` fallbacks (full CLI generalization is Task 15; scan behavior stays codex-only until Task 14–15 land).

- [ ] **Step 4: Run, gates, commit**

Run: `cargo test --workspace` — Expected: PASS, goldens/snapshots untouched (still codex-only scanning).

```bash
git add -A crates/
git commit -m "feat: multi-harness DiscoveryRoot with per-harness config paths"
```

### Task 12: `parse_json.rs` — JSON at TOML-equivalent hostile rigor

**Files:**
- Create: `crates/harness-guard-core/src/parse_json.rs`
- Modify: `crates/harness-guard-core/src/lib.rs` (add `pub mod parse_json;`)

**Interfaces:**
- Consumes: `ExtractedValue`, `MAX_NESTING_DEPTH`, `ParseFailure` from `crate::parse` (single shared definitions — the two parsers cannot drift, §5.2).
- Produces: `parse_config_json(text: &str) -> Result<serde_json::Value, ParseFailure>`, `extract_key_json(document: &serde_json::Value, dotted_key: &str) -> ExtractedValue`. Task 14 consumes both.

- [ ] **Step 1: Write the failing tests first**

The test module below is the §11.2 JSON-hostility contract; write it inside `parse_json.rs` and run before implementing:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_json_parses_and_extracts_typed_values() {
        let doc = parse_config_json(
            r#"{"cleanupPeriodDays": 30, "env": {"FOO": "bar"}, "flag": true}"#,
        )
        .unwrap();
        assert!(matches!(extract_key_json(&doc, "cleanupPeriodDays"), ExtractedValue::Int(30)));
        assert!(matches!(extract_key_json(&doc, "env.FOO"), ExtractedValue::Str(ref s) if s == "bar"));
        assert!(matches!(extract_key_json(&doc, "flag"), ExtractedValue::Bool(true)));
        assert!(matches!(extract_key_json(&doc, "absent"), ExtractedValue::Unset));
        assert!(matches!(extract_key_json(&doc, "flag.nested"), ExtractedValue::Other));
    }

    #[test]
    fn floats_and_out_of_i64_numbers_are_other_never_rendered() {
        let doc = parse_config_json(
            r#"{"days": 1.5, "huge": 99999999999999999999999999, "neg": -3}"#,
        )
        .unwrap();
        assert!(matches!(extract_key_json(&doc, "days"), ExtractedValue::Other));
        assert!(matches!(extract_key_json(&doc, "huge"), ExtractedValue::Other));
        assert!(matches!(extract_key_json(&doc, "neg"), ExtractedValue::Int(-3)));
    }

    #[test]
    fn duplicate_keys_are_last_value_wins_pinned() {
        // serde_json keeps the LAST value for a repeated key — the same
        // resolution the harness's own JSON parser applies, so it is the
        // correct observation (§5.2). This test pins the dependency behavior.
        let doc = parse_config_json(r#"{"key": "first", "key": "second"}"#).unwrap();
        assert!(matches!(extract_key_json(&doc, "key"), ExtractedValue::Str(ref s) if s == "second"));
    }

    #[test]
    fn key_path_hitting_an_array_is_other() {
        let doc = parse_config_json(r#"{"list": [1, 2, 3]}"#).unwrap();
        assert!(matches!(extract_key_json(&doc, "list.0"), ExtractedValue::Other));
    }

    #[test]
    fn parse_failures_are_categorical_and_never_quote_source() {
        // serde_json Display output can embed source fragments; our failure
        // must strip to the categorical message (§5.2, test-pinned).
        const SECRET: &str = "sk-hostile-secret-value";
        let text = format!(r#"{{"key": {SECRET}}}"#);
        let failure = parse_config_json(&text).unwrap_err();
        assert!(!failure.message.contains(SECRET), "secret leaked: {}", failure.message);
        assert!(!failure.message.contains("sk-"), "secret fragment leaked");
        assert!(failure.line.is_some() && failure.col.is_some());
    }

    #[test]
    fn truncated_json_is_categorical_eof() {
        let failure = parse_config_json(r#"{"key": "value"#).unwrap_err();
        assert_eq!(failure.message, "unexpected end of JSON input");
    }

    #[test]
    fn depth_over_shared_bound_is_rejected_and_at_bound_accepted() {
        // Identical bound to TOML — the shared MAX_NESTING_DEPTH constant.
        let over = format!("{}1{}", "{\"a\":".repeat(33), "}".repeat(33));
        let failure = parse_config_json(&over).unwrap_err();
        assert!(failure.message.contains("nesting depth"));
        let at = format!("{}1{}", "{\"a\":".repeat(32), "}".repeat(32));
        assert!(parse_config_json(&at).is_ok());
    }

    #[test]
    fn hostile_deep_nesting_never_panics() {
        // serde_json's default recursion limit (128) is the backstop.
        let hostile = "[".repeat(20_000);
        assert!(parse_config_json(&hostile).is_err());
    }
}
```

Run: `cargo test -p harness-guard-core parse_json` — Expected: FAIL (module missing).

- [ ] **Step 2: Implement**

```rust
//! Safe JSON parsing at TOML-equivalent hostile rigor (§5.2), mirroring
//! parse.rs invariant-for-invariant. Reads arrive through the same readfs
//! layer (no new read path); serde_json's default recursion limit (128) is
//! the overflow backstop; the shared MAX_NESTING_DEPTH = 32 bound is
//! enforced identically to TOML; diagnostics are categorical only because
//! serde_json error text can embed source fragments. Duplicate keys resolve
//! last-value-wins — matching the harness's own parser (test-pinned). Raw
//! text and the parsed Value are dropped inside the scan, as for TOML.
use crate::parse::{ExtractedValue, MAX_NESTING_DEPTH, ParseFailure};

pub fn parse_config_json(text: &str) -> Result<serde_json::Value, ParseFailure> {
    let document: serde_json::Value = serde_json::from_str(text).map_err(|error| {
        let line = error.line();
        let column = error.column();
        ParseFailure {
            line: (line > 0).then_some(line),
            col: (column > 0).then_some(column),
            key_path: None,
            message: categorical_message(&error),
        }
    })?;

    let depth = value_depth(&document);
    if depth > MAX_NESTING_DEPTH {
        return Err(ParseFailure {
            line: None,
            col: None,
            key_path: None,
            message: format!(
                "nesting depth {depth} exceeds the safety bound of {MAX_NESTING_DEPTH}"
            ),
        });
    }
    Ok(document)
}

/// Only the error CATEGORY reaches diagnostics — never serde_json's Display
/// text, which can quote source content (test-pinned).
fn categorical_message(error: &serde_json::Error) -> String {
    match error.classify() {
        serde_json::error::Category::Eof => "unexpected end of JSON input".to_string(),
        serde_json::error::Category::Syntax => "invalid JSON syntax".to_string(),
        serde_json::error::Category::Data => "JSON structure is not interpretable".to_string(),
        serde_json::error::Category::Io => "JSON could not be read".to_string(),
    }
}

fn value_depth(value: &serde_json::Value) -> usize {
    match value {
        serde_json::Value::Object(map) => {
            1 + map.values().map(value_depth).max().unwrap_or_default()
        }
        serde_json::Value::Array(array) => {
            1 + array.iter().map(value_depth).max().unwrap_or_default()
        }
        _ => 0,
    }
}

/// Objects-only dotted-key traversal (§5.2): array indexing is unsupported in
/// 0.0.1; a path hitting an array or scalar yields Other.
pub fn extract_key_json(document: &serde_json::Value, dotted_key: &str) -> ExtractedValue {
    let mut current = document;
    for part in dotted_key.split('.') {
        match current {
            serde_json::Value::Object(map) => match map.get(part) {
                Some(next) => current = next,
                None => return ExtractedValue::Unset,
            },
            _ => return ExtractedValue::Other,
        }
    }
    match current {
        serde_json::Value::String(text) => ExtractedValue::Str(text.clone()),
        serde_json::Value::Bool(flag) => ExtractedValue::Bool(*flag),
        serde_json::Value::Number(number) => number
            .as_i64()
            .map(ExtractedValue::Int)
            .unwrap_or(ExtractedValue::Other),
        _ => ExtractedValue::Other,
    }
}
```

Note: if the huge-number literal in the float test fails at the *parse* stage on this serde_json version (arbitrary-precision handling differs), adjust that one assertion to expect a parse error instead — either behavior is conservative; pin whichever this serde_json exhibits.

- [ ] **Step 3: Run, gates, commit**

Run: `cargo test -p harness-guard-core parse_json` then full gates.

```bash
git add crates/harness-guard-core/src/parse_json.rs crates/harness-guard-core/src/lib.rs
git commit -m "feat: JSON config parsing at TOML-equivalent hostile rigor"
```

### Task 13: Parameterized version detection

**Files:**
- Modify: `crates/harness-guard-core/src/version.rs`

**Interfaces:**
- Consumes: `HarnessId`, `descriptor` (Task 10), `DiscoveryRoot` (Task 11).
- Produces: `detect_version(root, harness) -> Option<String>`, `binary_on_path(root, harness) -> bool`. The bounded symlink resolution, parent walk, 64 KiB `package.json` cap, strict `X.Y.Z` parsing, and TOCTOU-stable-handle behavior are unchanged and shared (§5.3).

- [ ] **Step 1: Parameterize**

Replace the codex-specific entry points (keep `resolve_bounded`, `read_nearest_package_json_with_hook`, `parse_version`, `version_in_range`, and all constants exactly as they are):

```rust
use crate::harness::{HarnessId, descriptor};

pub fn detect_version(root: &DiscoveryRoot, harness: HarnessId) -> Option<String> {
    detect_version_with_hook(root, harness, || {})
}

fn detect_version_with_hook(
    root: &DiscoveryRoot,
    harness: HarnessId,
    after_package_open: impl FnOnce(),
) -> Option<String> {
    let facts = descriptor(harness);
    // §5.3: no established, evidence-backed marker ⇒ None ⇒ stale-ruleset.
    let binary_name = facts.path_binary?;
    let expected_package = facts.npm_package?;
    let binary = find_path_entry(root, binary_name)?;
    let resolved = resolve_bounded(&binary)?;
    let bytes = read_nearest_package_json_with_hook(&resolved, after_package_open)?;
    let text = String::from_utf8(bytes).ok()?;
    let package: serde_json::Value = serde_json::from_str(&text).ok()?;
    if package.get("name").and_then(|name| name.as_str()) != Some(expected_package) {
        return None;
    }
    let version = package.get("version")?.as_str()?;
    parse_version(version)?;
    Some(version.to_string())
}

/// Tool-on-PATH check used for detection confidence and the `list` command.
pub fn binary_on_path(root: &DiscoveryRoot, harness: HarnessId) -> bool {
    descriptor(harness)
        .path_binary
        .and_then(|name| find_path_entry(root, name))
        .and_then(|entry| resolve_bounded(&entry))
        .is_some()
}

fn find_path_entry(root: &DiscoveryRoot, binary_name: &str) -> Option<PathBuf> {
    root.path_dirs.iter().find_map(|directory| {
        let candidate = directory.join(binary_name);
        std::fs::symlink_metadata(&candidate).is_ok().then_some(candidate)
    })
}
```

Delete `find_codex_entry`, `detect_codex_version`, `detect_codex_version_with_hook`, and the `EXPECTED_PACKAGE` const. Update all callers (`scan.rs`, `main.rs` `cmd_list`) to the parameterized names with `HarnessId::Codex`, and every existing version test likewise. Add two new tests:

```rust
#[test]
fn claude_npm_layout_detects_version_with_the_shared_walk() {
    let dir = tempfile::tempdir().unwrap();
    let base = dir.path().canonicalize().unwrap();
    let package = base.join("node_modules/@anthropic-ai/claude-code");
    std::fs::create_dir_all(package.join("bin")).unwrap();
    std::fs::write(package.join("bin/claude"), "#!/usr/bin/env node\n").unwrap();
    std::fs::write(
        package.join("package.json"),
        r#"{"name": "@anthropic-ai/claude-code", "version": "2.1.202"}"#,
    )
    .unwrap();
    let root = DiscoveryRoot {
        codex_home: base.join("absent-codex"),
        claude_home: base.join("absent-claude"),
        grok_home: base.join("absent-grok"),
        path_dirs: vec![package.join("bin")],
    };
    assert_eq!(
        detect_version(&root, HarnessId::ClaudeCode),
        Some("2.1.202".to_string())
    );
    // The same layout must NOT satisfy codex detection (wrong package name).
    assert_eq!(detect_version(&root, HarnessId::Codex), None);
}

#[test]
fn grok_detection_is_none_until_evidence_establishes_a_marker() {
    let dir = tempfile::tempdir().unwrap();
    let base = dir.path().canonicalize().unwrap();
    let root = DiscoveryRoot {
        codex_home: base.join("a"),
        claude_home: base.join("b"),
        grok_home: base.join("c"),
        path_dirs: vec![base.clone()],
    };
    assert_eq!(detect_version(&root, HarnessId::GrokBuild), None);
    assert!(!binary_on_path(&root, HarnessId::GrokBuild));
}
```

- [ ] **Step 2: Run, gates, commit**

Run: `cargo test --workspace` — Expected: PASS, goldens untouched.

```bash
git add crates/harness-guard-core/src/version.rs crates/harness-guard-core/src/scan.rs crates/harness-guard-cli/src/main.rs
git commit -m "feat: parameterize execution-free version detection by harness descriptor"
```

### Task 14: `scan_harness` dispatch + conservative per-tool aggregates

**Files:**
- Modify: `crates/harness-guard-core/src/scan.rs`

**Interfaces:**
- Consumes: everything from Tasks 6, 10–13.
- Produces: `pub fn scan_harness(root: &DiscoveryRoot, harness: HarnessId, rules: &[ValidatedRule]) -> Option<ScanResult>` and `pub fn conservative_aggregates(rules: &[&ValidatedRule]) -> (Option<String>, Option<String>)`. Task 15 (CLI) and Task 20 (capabilities) consume both. `scan_codex` is deleted.

- [ ] **Step 1: Write the failing aggregate test (§5.5, decision j)**

```rust
#[test]
fn per_tool_aggregates_are_conservative_in_both_dimensions() {
    // Two synthetic rules: greatest maxes 0.144.5 (verified 2026-07-16) and
    // 0.150.0 (verified 2026-07-10). Weakest guarantee: min of maxes =
    // 0.144.5; earliest date = 2026-07-10.
    let rules = load_rules();
    let mut newer = rules[0].raw().clone();
    newer.id = "codex-synthetic-02".to_string();
    newer.tested_versions = vec![harness_guard_rules::schema::TestedVersion {
        min: "<=0.150.0".to_string(),
        max: "0.150.0".to_string(),
        verified_on: "2026-07-10".to_string(),
    }];
    let newer = harness_guard_rules::loader::ValidatedRule::try_from_raw(newer).unwrap();
    let pair = [&rules[0], &newer];
    let (version, date) = conservative_aggregates(&pair);
    assert_eq!(version.as_deref(), Some("0.144.5"));
    assert_eq!(date.as_deref(), Some("2026-07-10"));
    assert_eq!(conservative_aggregates(&[]), (None, None));
}
```

Run: `cargo test -p harness-guard-core aggregates` — Expected: FAIL (function missing).

- [ ] **Step 2: Implement `conservative_aggregates` and `scan_harness`**

```rust
use crate::harness::{ConfigFormat, HarnessId, descriptor};
use crate::parse_json::{extract_key_json, parse_config_json};
use crate::version::{binary_on_path, detect_version, parse_version};
use harness_guard_rules::schema::TestedVersion;

/// §5.5 (decision j): rules_last_verified_version is the MINIMUM of the
/// rules' greatest tested maxes (the weakest guarantee) and
/// rules_verified_date the EARLIEST verified_on among those greatest-max
/// entries — conservative in both dimensions. Fixes the former
/// rules.first() shortcut before >1 rule can mislead.
pub fn conservative_aggregates(rules: &[&ValidatedRule]) -> (Option<String>, Option<String>) {
    let mut greatest_per_rule: Vec<&TestedVersion> = Vec::new();
    for rule in rules {
        let greatest = rule
            .raw()
            .tested_versions
            .iter()
            .max_by_key(|tested| parse_version(&tested.max).unwrap_or((0, 0, 0)));
        match greatest {
            Some(tested) => greatest_per_rule.push(tested),
            None => return (None, None),
        }
    }
    let weakest_version = greatest_per_rule
        .iter()
        .min_by_key(|tested| parse_version(&tested.max).unwrap_or((0, 0, 0)))
        .map(|tested| tested.max.clone());
    // ISO dates: lexicographic order IS chronological order.
    let earliest_date = greatest_per_rule
        .iter()
        .map(|tested| tested.verified_on.as_str())
        .min()
        .map(str::to_string);
    (weakest_version, earliest_date)
}

enum ParsedDocument {
    Toml(toml::Value),
    Json(serde_json::Value),
}

/// Returns None iff neither the harness home nor a PATH marker is present
/// (§5.5). Rules are filtered to this harness's tool id.
pub fn scan_harness(
    root: &DiscoveryRoot,
    harness: HarnessId,
    rules: &[ValidatedRule],
) -> Option<ScanResult> {
    let facts = descriptor(harness);
    let home_detected = probe_directory(root.home(harness)) != PathProbe::Missing;
    let on_path = binary_on_path(root, harness);
    if !home_detected && !on_path {
        return None;
    }

    let harness_rules: Vec<&ValidatedRule> = rules
        .iter()
        .filter(|rule| rule.raw().tool == harness.as_str())
        .collect();

    let detected_version = detect_version(root, harness);
    let mut parse_failure = None;
    let mut config_paths = Vec::new();

    let config_state = match read_config(root, harness) {
        ConfigReadOutcome::NoConfig => ConfigState::Missing,
        ConfigReadOutcome::Refused(reason) => {
            config_paths.push(root.config_path(harness).to_string_lossy().into_owned());
            ConfigState::Unreadable(reason)
        }
        ConfigReadOutcome::Ok(text) => {
            config_paths.push(root.config_path(harness).to_string_lossy().into_owned());
            let parsed = match facts.config_format {
                ConfigFormat::Toml => parse_config(&text).map(ParsedDocument::Toml),
                ConfigFormat::Json => parse_config_json(&text).map(ParsedDocument::Json),
            };
            match parsed {
                Err(failure) => {
                    parse_failure = Some(failure.clone());
                    ConfigState::Unparseable(failure)
                }
                Ok(document) => {
                    let mut extracted = BTreeMap::new();
                    for rule in &harness_rules {
                        let key = rule.raw().observation.key.clone();
                        let value = match &document {
                            ParsedDocument::Toml(doc) => extract_key(doc, &key),
                            ParsedDocument::Json(doc) => extract_key_json(doc, &key),
                        };
                        extracted.insert(key, value);
                    }
                    // The parsed document and every unrelated value drop here.
                    ConfigState::Parsed(extracted)
                }
            }
        }
    };

    let degraded = matches!(
        &config_state,
        ConfigState::Unreadable(_) | ConfigState::Unparseable(_)
    );

    let mut findings: Vec<_> = harness_rules
        .iter()
        .map(|rule| evaluate_rule(rule, &config_state, detected_version.as_deref()))
        .collect();
    findings.sort_by(|left, right| left.rule_id.cmp(&right.rule_id));

    let version_in_range = detected_version
        .as_deref()
        .map(|version| {
            harness_rules.iter().all(|rule| {
                crate::version::version_in_range(version, &rule.raw().tested_versions)
            })
        })
        .unwrap_or(false);

    let (rules_last_verified_version, rules_verified_date) =
        conservative_aggregates(&harness_rules);
    let detection_confidence = detection_confidence(detected_version.as_deref(), home_detected);

    Some(ScanResult {
        tool_report: ToolReport {
            tool: harness.as_str().to_string(),
            detected_version,
            config_paths,
            detection_confidence,
            rules_last_verified_version,
            rules_verified_date,
            version_in_range,
            findings,
        },
        degraded,
        parse_failure,
    })
}
```

Delete `scan_codex`; update the module's existing tests to call `scan_harness(&root, HarnessId::Codex, &load_rules())` (and add the two absent homes to their `DiscoveryRoot` literals — done in Task 11). Update `main.rs` `cmd_scan` minimally (`scan_codex(&root, &rules)` → `scan_harness(&root, HarnessId::Codex, &rules)`) so the workspace compiles; the full CLI loop is Task 15.

Single-rule byte-identity check: for one rule with one tested_versions entry, `conservative_aggregates` returns exactly `(tested_versions[0].max, tested_versions[0].verified_on)` — identical to the old `rules.first()` shortcut, so goldens stay untouched.

- [ ] **Step 3: Run, gates, commit**

Run: `cargo test --workspace` — Expected: PASS, zero golden/snapshot drift.

```bash
git add crates/harness-guard-core/src/scan.rs crates/harness-guard-cli/src/main.rs
git commit -m "feat: scan_harness dispatch with per-tool conservative aggregates"
```

### Task 15: CLI generalization — env roots, `--tool`, scan loop, `list`, redaction, renderer

**Files:**
- Modify: `crates/harness-guard-cli/src/main.rs`
- Modify: `crates/harness-guard-cli/src/redact.rs`
- Modify: `crates/harness-guard-cli/src/render_term.rs`
- Modify: `crates/harness-guard-cli/tests/cli_surface.rs`, `crates/harness-guard-cli/tests/scan_fixtures.rs`
- Snapshots under `crates/harness-guard-cli/tests/snapshots/` (reviewed update — detection block now enumerates three harnesses)

**Interfaces:**
- Consumes: `scan_harness`, `HarnessId`, `descriptor` (Tasks 10, 14).
- Produces: `--tool codex|claude-code|grok-build` (repeatable, default all detected), three-row `list`, per-harness redaction `redact_config_path(path, home, harness_home, home_token, config_file)`, `TermOpts.requested: Vec<String>`.

- [ ] **Step 1: Fresh-retrieval sub-step — Claude Code home-override env var (§5.1)**

Retrieve the current official Claude Code settings documentation (`https://code.claude.com/docs/en/settings`) NOW, at implementation time. Record the retrieval date. Decision rule:
- If it documents an environment variable that relocates the `~/.claude` config directory (lead: `CLAUDE_CONFIG_DIR`), wire it in `discovery_root_from_env` exactly like `CODEX_HOME`, set the Task 10 descriptor `home_token` to `"$CLAUDE_CONFIG_DIR"` (or the documented name), and note URL + retrieved date in a comment at the wiring site; carry the citation into the Task 18 rule sources.
- If not documented: no env override for claude (home is always `HOME/.claude`), keep the defensive `"$CLAUDE_HOME"` token, and record that determination in the same comment.

No Grok override var is assumed either way (§5.1: "none assumed").

- [ ] **Step 2: Generalize `discovery_root_from_env`**

```rust
fn discovery_root_from_env() -> (DiscoveryRoot, Option<PathBuf>) {
    // This is the only ambient-environment boundary. Core always receives an
    // explicit root and therefore cannot fall through to the real home.
    let home = directories::BaseDirs::new().map(|dirs| dirs.home_dir().to_path_buf());
    let home_join = |dotdir: &str, fallback: &str| {
        home.as_ref()
            .map(|home| home.join(dotdir))
            .unwrap_or_else(|| PathBuf::from(fallback))
    };
    let codex_home = std::env::var_os("CODEX_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home_join(".codex", ".codex"));
    // Claude override var per Step 1 retrieval (documented ⇒ honored here,
    // with citation comment; undocumented ⇒ plain home join).
    let claude_home = std::env::var_os("CLAUDE_CONFIG_DIR") // ← keep/remove per Step 1
        .map(PathBuf::from)
        .unwrap_or_else(|| home_join(".claude", ".claude"));
    // §5.1: no Grok home-override env var is assumed.
    let grok_home = home_join(".grok", ".grok");
    let path_dirs = std::env::var_os("PATH")
        .map(|path| std::env::split_paths(&path).collect())
        .unwrap_or_default();

    (DiscoveryRoot { codex_home, claude_home, grok_home, path_dirs }, home)
}
```

- [ ] **Step 3: Widen `--tool` and loop the scan**

`ScanArgs`: `#[arg(long, value_parser = ["codex", "claude-code", "grok-build"])] tool: Vec<String>,` with doc comment `/// Restrict to specific tools (repeatable; default: every detected tool)`.

`cmd_scan` core loop:

```rust
let selected: Vec<HarnessId> = if args.tool.is_empty() {
    HarnessId::ALL.to_vec()
} else {
    let mut ids: Vec<HarnessId> = args
        .tool
        .iter()
        .filter_map(|tool| HarnessId::parse(tool))
        .collect();
    ids.sort_unstable();
    ids.dedup();
    ids
};
let results: Vec<ScanResult> = selected
    .iter()
    .filter_map(|&harness| scan_harness(&root, harness, &rules))
    .collect();
```

Parse-failure diagnostics must pair each failure with ITS tool's redacted config path (the old `tools.first()` shortcut is wrong with several tools):

```rust
let parse_failures: Vec<(String, ParseFailure)> = report
    .tools
    .iter()
    .zip(results.iter())  // both alphabetical: results scanned in ALL order, tools sorted
    .filter_map(|(tool, result)| {
        result.parse_failure.clone().map(|failure| {
            (tool.config_paths.first().cloned().unwrap_or_default(), failure)
        })
    })
    .collect();
...
for (path, failure) in &parse_failures {
    eprint!("{}", diagnostics::report_parse_failure(failure, path));
}
```

(`results` is produced by iterating `selected`, which is sorted `HarnessId` order = alphabetical tool ids, and `build_report` sorts tools the same way, so `zip` is positionally correct; add `debug_assert_eq!(tool.tool, result.tool_report.tool)` inside the closure.)

`build_report` signature becomes `build_report(results: &[ScanResult], home: Option<&Path>, root: &DiscoveryRoot) -> Report`; redaction per harness:

```rust
let mut tools: Vec<_> = results
    .iter()
    .map(|result| {
        let mut tool = result.tool_report.clone();
        let harness = HarnessId::parse(&tool.tool).expect("scan produces known tool ids");
        let facts = harness_guard_core::harness::descriptor(harness);
        tool.config_paths = tool
            .config_paths
            .iter()
            .map(|path| {
                redact::redact_config_path(
                    path,
                    home,
                    root.home(harness),
                    facts.home_token,
                    facts.config_file,
                )
            })
            .collect();
        tool
    })
    .collect();
tools.sort_by(|left, right| left.tool.cmp(&right.tool));
```

- [ ] **Step 4: Generalize `redact.rs`**

```rust
/// Config paths have two safe render roots. Prefer `~` when the harness home
/// is below HOME; otherwise use the harness's fixed symbolic token and never
/// emit an absolute custom home value.
pub fn redact_config_path(
    path: &str,
    home: Option<&Path>,
    harness_home: &Path,
    home_token: &str,
    config_file: &str,
) -> String {
    let home_redacted = redact_home(path, home);
    if home_redacted != path {
        return home_redacted;
    }
    redact_under(path, Some(harness_home), home_token)
        .unwrap_or_else(|| format!("{home_token}/{config_file}"))
}
```

Update its unit tests (same scenarios, explicit token/file arguments; add one asserting `redact_config_path("/x/grok-root/config.toml", Some(Path::new("/synthetic/home")), Path::new("/x/grok-root"), "$GROK_HOME", "config.toml") == "$GROK_HOME/config.toml"`).

- [ ] **Step 5: Three-harness detection block and `list`**

`TermOpts` gains `pub requested: Vec<String>` (alphabetical tool ids the scan covered). In `render()`, replace the empty-tools special case with a merge over requested ids:

```rust
let mut detected: std::collections::BTreeMap<&str, &ToolReport> =
    report.tools.iter().map(|tool| (tool.tool.as_str(), tool)).collect();
for requested in &opts.requested {
    match detected.remove(requested.as_str()) {
        Some(tool) => { /* existing ● line */ }
        None => {
            let _ = writeln!(output, "  ○ {requested} — not detected");
        }
    }
}
```

`cmd_list` iterates `HarnessId::ALL` (alphabetical), one row per harness, detected or not, using `detect_version(&root, harness)` / `probe_directory(root.home(harness))` / per-harness redaction — the codex row logic generalized verbatim.

- [ ] **Step 6: Update CLI tests and snapshots**

- `scan_fixtures.rs::unknown_tool_flag_is_usage_error` stays (cursor still rejected). Add:
  ```rust
  #[test]
  fn tool_flag_accepts_all_three_ids() {
      for tool in ["codex", "claude-code", "grok-build"] {
          let output = run_case("hardened", &["scan", "--tool", tool, "--json"]);
          assert!(matches!(output.status.code(), Some(0) | Some(1)), "{tool} must be a valid --tool");
      }
  }
  ```
- `cli_surface.rs::list_shows_detection_only`: add assertions that the output also names `claude-code` and `grok-build` rows.
- Snapshot review: run `cargo test -p harness-guard-cli`; the detection-block snapshots (`missing`, `risky_unset`, `hardened_verbose`, `stale_banner`, `stale_out_of_range`, `unknown_value`) gain two `○ … — not detected` lines. Review each `.snap.new` diff — ONLY detection-block lines may change; findings, summaries, and messages must be identical. Accept with `cargo insta accept` (or move `.snap.new` over `.snap`).
- Golden `expected.json` files: unchanged (tools arrays still contain exactly the one detected tool).

- [ ] **Step 7: Run, gates, commit**

Run: `cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo deny check && cargo test --workspace`
On macOS: `scripts/no-egress/run-macos.sh` — Expected: still green (codex fixtures scan all three harnesses; two undetected).

```bash
git add crates/harness-guard-cli/
git commit -m "feat: multi-harness CLI — --tool widening, three-row list, per-harness redaction"
```

### Task 16: Multi-harness test runner + real-config protection for all three stores

**Files:**
- Modify: `crates/harness-guard-cli/tests/common/mod.rs`
- Modify: `crates/harness-guard-cli/tests/scan_fixtures.rs`

**Interfaces:**
- Produces: `harness_fixture(tool, case) -> PathBuf` and `run_harness_case(tool, case, args) -> Output` for the new-harness fixture layout (plan assumption 4: committed `files/home/.claude/...`, `files/home/.grok/...`), plus `run_mixed_case(case, args) -> Output` for the two-store `fixtures/mixed/` layout. Tasks 18–19 consume these (Task 18 step 5 consumes `run_mixed_case`).

- [ ] **Step 1: Add the helpers**

Append to `common/mod.rs`:

```rust
pub fn harness_fixture(tool: &str, case: &str) -> PathBuf {
    repo_root().join("fixtures").join(tool).join(case).join("files")
}

/// New-harness runner (claude-code, grok-build): HOME points at the fixture's
/// committed synthetic home (containing .claude/ or .grok/), PATH at the
/// fixture's path dir, and CODEX_HOME at an absent dir so codex stays
/// undetected. env_clear() plus these roots make the developer's real
/// ~/.codex, ~/.claude, and ~/.grok unreachable by construction.
#[allow(dead_code)]
pub fn run_harness_case(tool: &str, case: &str, args: &[&str]) -> Output {
    let files_root = harness_fixture(tool, case);
    let home = files_root.join("home");
    run_with_roots(&home.join("absent-codex-home"), &files_root.join("path"), &home, args)
}

/// Mixed multi-harness runner (§11.2 aggregation): the fixture's committed
/// synthetic home contains TWO stores (.codex/ AND .claude/), and CODEX_HOME
/// points INTO the fixture home rather than at an absent dir, so one scan
/// detects two harnesses. Same env_clear() containment: the developer's real
/// ~/.codex, ~/.claude, and ~/.grok stay unreachable by construction.
/// Consumed by Task 18 step 5.
#[allow(dead_code)]
pub fn run_mixed_case(case: &str, args: &[&str]) -> Output {
    let files_root = repo_root().join("fixtures").join("mixed").join(case).join("files");
    let home = files_root.join("home");
    run_with_roots(&home.join(".codex"), &files_root.join("path"), &home, args)
}
```

- [ ] **Step 2: Extend real-config protection to all three homes**

Append to `scan_fixtures.rs`:

```rust
#[test]
fn no_absolute_path_escapes_the_fixture_tree_for_any_harness() {
    // §5.1: extends the existing real-config protection to ~/.claude and
    // ~/.grok, which also exist on dev machines. The scan runs with every
    // ambient variable cleared and all three homes inside the fixture; no
    // absolute path outside the fixture may appear in any output.
    //
    // This run only covers the codex `hardened` fixture — claude-code and
    // grok-build are not detected in it, so their config-path redaction is
    // not yet exercised here. Task 18 extends this test to also run over
    // `fixtures/mixed/codex-pass-claude-degraded` once that two-store
    // fixture lands, covering claude-code's redaction in the same test.
    let files_root = fixture("hardened");
    let output = run_in(&files_root, &["scan", "--json", "--verbose"]);
    let all = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!all.contains("/Users/"), "home-anchored absolute path leaked");
    assert!(!all.contains(&files_root.to_string_lossy().into_owned()), "fixture path leaked");
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    for tool in report["tools"].as_array().unwrap() {
        for path in tool["config_paths"].as_array().unwrap() {
            let rendered = path.as_str().unwrap();
            assert!(
                rendered.starts_with('~') || rendered.starts_with('$'),
                "config path {rendered:?} must have a symbolic root"
            );
        }
    }
}
```

- [ ] **Step 3: Run, gates, commit**

Run: `cargo test -p harness-guard-cli` — Expected: PASS.

```bash
git add crates/harness-guard-cli/tests/
git commit -m "test: multi-harness fixture runner and three-store real-config protection"
```

**Phase 2 exit criteria (§13.2):** `scan`, `list`, `explain` operate over all three harnesses from synthetic fixtures; core still compiles under the env/process/net clippy bans; JSON parsing passes the hostile unit matrix with value-free diagnostics.

---

## Phase 3 — WP3 (+WP5): Per-harness evidence work packages

**Shared protocol for Tasks 17–19 (the runbook protocol, §7 preamble — repeat for every rule):**
1. Retrieve every cited page fresh from the official primary source at authoring time. Record the ACTUAL retrieval date (the date you fetch, not today's plan date).
2. Compute the semantic hash: `scripts/freshness/normalize.sh <url>` → `sha256:<64 hex>`.
3. Capture a Wayback anchor (`https://web.archive.org/save/<url>`; record the resulting snapshot URL). `archived_url: null` only if archiving fails; note why in the source `notes`.
4. Add/refresh the URL's hash in `freshness/url-hashes.json`.
5. `data/` and `docs/research/` are quarantined leads — never copy a claim, key name, or default from them without re-verifying against the fresh retrieval.
6. Every rule: `schema_version "1.1"`, tool-prefixed kebab id, `unknown_subject`, complete match partition (validator will reject anything else), `tested_versions` naming the actually-verified version, limitations naming every uninspected layer (§5.4), auth-dependent policy as limitation + `verify_url`, never inferred.
7. Bump `rules/ruleset.json` `ruleset_version` to the authoring date (CalVer `YYYY.MM.DD`) — once per task, in the same commit as its rules. Update every golden that pins `ruleset_version` (all fixture `expected.json` files) and the `cli_surface.rs` `ruleset 2026.07.16` assertions in the same change.
8. Per rule: full fixture-matrix rows + goldens + `explain` smoke assertion + hostility coverage (the Task 9 property test automatically covers every embedded rule — run it).
9. Fixture `files/path/package.json` synthetic versions must equal the rule's verified version so in-range cases stay in range.

**Fixture matrix per harness (§11.2, 13 cases — replicate the codex naming):** `missing`, `minimal`, `hardened`, `risky-explicit`, per-rule value cases, the format-specific malformed case (naming matches the codex precedent `fixtures/codex/malformed-toml`: `malformed-json` for claude-code's JSON settings, `malformed-toml` for grok-build's TOML config — never the generic `malformed-config`), `unrecognized-value`, `symlink-config` (symlink created at runtime by `hostile.rs`-style mutation, never committed), `oversized`, `deep-nesting`, `permission-denied` (runtime mutation), `unknown-version`, `version-out-of-range`. JSON harnesses add: `duplicate-keys` (last-wins pinned end-to-end), `float-where-integer`, `huge-number`, `non-utf8` (runtime-written bytes), `secret-shaped` (a parse-error file containing a secret-looking token that must not appear in any output — assert in the test, echoing `raw_values_never_echo_anywhere`).

### Task 17: WP3a — Codex rule expansion (target 2–4 new rules; ≥3 total per §13.3)

**Files:**
- Create: `rules/codex/<new-rule>.json` (×2–4)
- Modify: `rules/ruleset.json`, `freshness/url-hashes.json`
- Create: `fixtures/codex/<new-case>/…` (per-rule value cases)
- Modify: existing `fixtures/codex/*/expected.json` (new rules add findings to every case's report — each case's config decides each rule's outcome; unset keys yield `unknown` findings everywhere)
- Modify: `fixtures/mixed/codex-pass-claude-degraded/{files/home/.codex/config.toml,expected.json}` — only if Task 18 has already landed it (Tasks 17/18 are order-independent): the mixed config gains the new rules' safe values, its codex tool entry gains their `pass` findings, and its summary counts shift accordingly
- Modify: `crates/harness-guard-rules/tests/fixture_validation.rs` (CASES arrays), `crates/harness-guard-cli/tests/{scan_fixtures.rs,cli_surface.rs,scan_snapshots.rs}` + snapshots

- [ ] **Step 1: Fresh retrieval.** Fetch `https://learn.chatgpt.com/docs/config-file/config-reference`, `config-advanced`, `config-basic` (the URLs the existing rule cites). Leads to evaluate against what the docs ACTUALLY say now (`docs/research/verification-audit-2026-07-13.md:85-93` — leads, not evidence): `analytics.enabled`, `feedback.enabled`, OTEL controls incl. prompt logging, plaintext TUI logging. Fix the exact rule set from what is documented as user-scope `config.toml` keys with locally observable state. Only enum/bool/integer-typed keys are rulable in 0.0.1.
- [ ] **Step 2: Author each rule** following the shared protocol and the `history-persist-01` structure verbatim (categories: `retention`/`telemetry` as applicable). Auth-method data-policy interpretation stays a limitation + `verify_url` (existing pattern).
- [ ] **Step 3: Extend fixtures.** For each new rule: extend `hardened` (safe value), `risky-explicit` (risky value), add a per-rule value case if the domain has >2 values; every existing case's `expected.json` gains the new rules' findings in rule-id order. Keep summaries consistent (`unknown` counts rise in unset cases). If `fixtures/mixed/codex-pass-claude-degraded/` already exists (Task 18 landed first), treat it exactly like `hardened`: add each new rule's safe value to its `files/home/.codex/config.toml` and regenerate its `expected.json` (codex entry stays all-`pass`; merged summary counts shift).
- [ ] **Step 4: Run everything; update snapshots deliberately** (new findings appear). `cargo insta accept` only after reviewing each diff.
- [ ] **Step 5: Aggregate check.** With >1 codex rule, the Task 14 conservative aggregates become observable: verify `rules_last_verified_version` equals the weakest max across the codex rules in one golden.
- [ ] **Step 6: Gates** (full set incl. `scripts/no-egress/run-macos.sh` on macOS) **and commit:**

```bash
git add rules/ fixtures/codex/ freshness/url-hashes.json crates/
git commit -m "feat(rules): expand Codex coverage from fresh retrieval (ruleset CalVer bump)"
```

### Task 18: WP3b — Claude Code rules + JSON fixture matrix

**Files:**
- Create: `rules/claude-code/<rule>.json` (transcript retention lead: `cleanupPeriodDays` — the first integer observation; telemetry-posture lead where a documented user-scope `settings.json` key exists)
- Create: `fixtures/claude-code/<case>/…` full matrix incl. JSON-specific hostile cases
- Create: `fixtures/mixed/codex-pass-claude-degraded/…` (the §11.2 two-store aggregation case — step 5)
- Modify: `rules/ruleset.json`, `freshness/url-hashes.json`, test case lists, `crates/harness-guard-cli/tests/hostile.rs` (mutation cases for claude-code)

- [ ] **Step 1: Fresh retrieval.** Fetch `https://code.claude.com/docs/en/data-usage` and `https://code.claude.com/docs/en/settings`. Leads (in-repo, still requiring fresh confirmation — `verification-audit-2026-07-13.md:58-69`): local transcripts under `~/.claude/projects/` with 30-day default retention via `cleanupPeriodDays`; separate telemetry/error-reporting/feedback/nonessential-traffic controls; managed/CLI/local/project/user scopes. Constraints: env-var-only controls are NOT locally observable file state — no rule may fake them from the scanner's environment; a control documented inside `settings.json`'s `env` block IS observable (key path e.g. `env.SOME_VAR`) and may be ruled on with the layer limitation stated. Consumer/commercial training split is auth-dependent ⇒ limitations + `verify_url` (§7.4).
- [ ] **Step 2: Author the integer rule** (retention). Shape (values from retrieval):
  - `observation`: `{"file": "settings.json", "key": "cleanupPeriodDays", "type": "integer", "allowed_render": ["unset"], "integer_bounds": {"min": <documented-min>, "max": <documented-max-or-conservative-cap>}}` — bounds come from the documentation; if the docs give no upper bound, choose a cap and state it in `limitations` (values above it render as unrecognized/unknown, never displayed — conservative).
  - Outcomes: `int_range` blocks partitioning the bounds (e.g. shorter-retention pass vs. longer-retention finding per what the docs justify), plus the mandatory `unset` (status unknown — uninspected managed/project/local/CLI layers may supply it) and `unrecognized` outcomes. The Task 4 validator forces the partition to be exact.
- [ ] **Step 3: Author the telemetry-posture rule(s)** if and only if retrieval confirms a documented user-scope `settings.json` key (bool → the `["true","false","unset"]` allowed_render form). If no such key exists, record that determination in the task's commit message and ship the retention rule only — parity is rigor + category coverage where locally observable state EXISTS, with `unknown` for the rest, never a padded rule (§7 parity definition).
- [ ] **Step 4: Build the fixture matrix** (13 cases + 5 JSON-hostile cases) under `fixtures/claude-code/`, layout per plan assumption 4:
  ```
  fixtures/claude-code/hardened/files/home/.claude/settings.json
  fixtures/claude-code/hardened/files/path/claude          # synthetic marker; never executed
  fixtures/claude-code/hardened/files/path/package.json    # {"name":"@anthropic-ai/claude-code","version":"<verified>"}
  fixtures/claude-code/hardened/expected.json
  ```
  Goldens: `tool: "claude-code"`, `config_paths: ["~/.claude/settings.json"]`, findings per rule per case. The shared matrix's format-specific malformed case is `fixtures/claude-code/malformed-json/` here (truncated/invalid JSON settings). `duplicate-keys` case pins last-value-wins end-to-end; `float-where-integer`/`huge-number` pin unrecognized→unknown with no rendering; `secret-shaped` (e.g. `{"cleanupPeriodDays": sk-secret-looking-token}` — malformed) pins categorical diagnostics with a `raw_values_never_echo` assertion; `non-utf8` and `permission-denied` and `symlink-config` are runtime mutations added to `hostile.rs` reusing its `temp_copy` machinery pointed at the claude fixture tree.
- [ ] **Step 5: Mixed multi-harness aggregation fixture (§11.2: multi-harness summary counts; mixed states — one harness degraded, another passing ⇒ exit 2 with full report).** Create `fixtures/mixed/codex-pass-claude-degraded/` — the one fixture whose committed synthetic home contains TWO harness stores, so a single scan detects two tools and the cross-tool aggregation semantics become observable:
  ```
  fixtures/mixed/codex-pass-claude-degraded/files/home/.codex/config.toml     # mirrors fixtures/codex/hardened's config: safe value for EVERY codex rule ⇒ all pass
  fixtures/mixed/codex-pass-claude-degraded/files/home/.claude/settings.json  # malformed JSON (e.g. `{"cleanupPeriodDays": 30,` — truncated) ⇒ unparseable ⇒ degraded
  fixtures/mixed/codex-pass-claude-degraded/files/path/codex                  # synthetic marker; never executed
  fixtures/mixed/codex-pass-claude-degraded/files/path/package.json           # {"name":"@openai/codex","version":"<codex verified version>"} — codex detected + in range
  fixtures/mixed/codex-pass-claude-degraded/expected.json
  ```
  Deliberately NO `claude` PATH marker: a single `files/path/package.json` can only name one npm package (the Task 13 nearest-package walk checks `name`), claude-code is detected via its home alone, and §6.4.1 precedence makes the malformed config yield `unknown` findings regardless of version bookkeeping — so the missing claude version costs nothing. The codex store is reached via `run_mixed_case` (Task 16), whose `CODEX_HOME` points INTO the fixture home — never at the developer's real store.

  Author `expected.json` from a reviewed run; the golden must pin:
  - **exit code 2** — claude-code degraded; §5.5: `degraded` is true if ANY scanned harness degraded, even though the codex side alone would exit 0 (this is the degraded-any ⇒ exit-2 semantics with one harness degraded and another passing);
  - **`tools` with exactly two entries**, alphabetical: `claude-code` first (unparseable ⇒ every claude rule `unknown` with message `"Cannot determine …: config not safely parseable: …"`, no observation, `detected_version: null`) then `codex` (every codex rule `pass`, version in range);
  - **merged summary counts** equal to the sum across both tool entries (claude rule-count `unknown`s + codex rule-count `pass`es) — the only golden where two tools contribute to `summary`;
  - redaction across both stores: no absolute path, no raw config value; `config_paths` symbolic for both tools (`~/.claude/settings.json`, `~/.codex/config.toml`).

  Wire it in: `scan_fixtures.rs` gains a mixed-case test using `run_mixed_case("codex-pass-claude-degraded", &["scan", "--json"])` asserting exit 2 + golden equality, plus a term-mode run pinned as one snapshot (e.g. `mixed_codex_pass_claude_degraded`) asserting both `●` detection lines and the `○ grok-build — not detected` line render (full report despite degradation); `fixture_validation.rs` validates `fixtures/mixed/codex-pass-claude-degraded/expected.json` against the report schema via its own `MIXED_CASES` array (the per-harness CASES arrays stay per-harness).

  Ordering note: Tasks 17 and 18 are order-independent. If Task 17 (codex expansion) lands AFTER this task, its Step 3 updates this fixture's config and golden; if it landed BEFORE, author this golden against the expanded codex ruleset directly.
- [ ] **Step 6: Extend Task 16's multi-store real-config protection test.** `no_absolute_path_escapes_the_fixture_tree_for_any_harness` (`crates/harness-guard-cli/tests/scan_fixtures.rs`) so far only runs the codex `hardened` fixture, where claude-code and grok-build are never detected and so their config-path redaction goes unexercised. Now that this fixture exists, extend the test to also run over `fixtures/mixed/codex-pass-claude-degraded` via `run_mixed_case("codex-pass-claude-degraded", &["scan", "--json", "--verbose"])`, asserting the same no-leaked-absolute-path and symbolic-`config_paths` checks against its output — this is what exercises claude-code's config-path redaction for the first time.
- [ ] **Step 7: Wire the test lists.** `scan_fixtures.rs` gains a claude case table using `run_harness_case("claude-code", case, &["scan", "--json"])`; `fixture_validation.rs` gains claude CASES arrays; add one term snapshot (e.g. `claude_risky_unset`).
- [ ] **Step 8: Gates and commit:**

```bash
git add rules/claude-code/ fixtures/claude-code/ fixtures/mixed/ rules/ruleset.json freshness/url-hashes.json crates/
git commit -m "feat(rules): Claude Code rules from fresh retrieval with JSON hostile fixture matrix and mixed-state aggregation golden"
```

### OWNER CHECKPOINT B — verify before Task 19

Confirm `docs/research/evidence/grok-build/<date>/` contains a dated run (version-pinning record, matrix capture summaries, doc hashes) produced via the Task 2 protocol against the then-current Grok Build version. If absent, STOP: Task 19 and the release tag are blocked (§7.3 release gate). If upstream released since the run, apply the runbook triage flow (re-run vs. ship-pinned decision belongs to the owner).

### Task 19: WP3c — Grok Build rules, descriptor completion, fixtures (RELEASE-GATING)

**Files:**
- Create: `rules/grok-build/<rule>.json`
- Modify: `crates/harness-guard-core/src/harness.rs` (GROK_BUILD descriptor: `path_binary`, `npm_package`/detection per evidence)
- Create: `fixtures/grok-build/<case>/…` (TOML matrix)
- Modify: `rules/ruleset.json`, `freshness/url-hashes.json`, test case lists

- [ ] **Step 1: Fresh retrieval + evidence intake.** Fetch `https://docs.x.ai/build/settings` and `https://docs.x.ai/build/settings/reference` fresh; hash + archive them. Read the lab-run artifacts. Establish from evidence: the config keys that exist TODAY, the pinned version, the install channel, and the PATH binary name.
- [ ] **Step 2: Complete the descriptor from evidence.** Update `GROK_BUILD` in `harness.rs`: `path_binary: Some("<evidenced name>")`; `npm_package: Some("<package>")` only if the channel is npm — otherwise leave `None` (detection stays off; every finding degrades to `stale-ruleset`, which is the honest state §5.3). Cite the evidence artifact path + retrieval date in a comment. Update the Task 10 descriptor test accordingly.
- [ ] **Step 3: Author the rules** at full parity (§7 definition: cited coverage of `retention` and `telemetry` categories where locally observable state exists; explicit `unknown` for locally-unknowable dimensions):
  - `~/.grok/config.toml` posture keys → `evidence_class: "official-documentation"` sources.
  - Upload/telemetry BEHAVIOR claims → `evidence_class: "independent-reproduction"` citing the lab run (source `url` = the xAI docs page it contradicts/confirms; `notes` names `docs/research/evidence/grok-build/<date>/`), `tested_versions` with `min == max ==` the pinned version, no `<=` prefix (§7.3.2).
  - Server-side/account state (e.g. remote `disable_codebase_upload`) → `unknown` outcomes with `verify_url`, and limitations naming it.
  - The retired keys must not appear — Task 1's tripwire enforces; do not weaken it.
- [ ] **Step 4: Fixture matrix** under `fixtures/grok-build/` (TOML; same 13-case pattern; layout `files/home/.grok/config.toml` + `files/path/<binary>` per evidenced name). The shared matrix's format-specific malformed case is `fixtures/grok-build/malformed-toml/` here, matching the codex precedent (`fixtures/codex/malformed-toml`) directly since both are TOML. If detection is `None` by evidence, the version-dependent cases collapse honestly: every case's findings are `stale-ruleset`-wrapped ("version not detected"), `unknown-version` is the norm and `version-out-of-range` is omitted — goldens say so explicitly in their `description` fields.
- [ ] **Step 5: WP5 freshness extension — lands in this same change (§9).**
  - `freshness/last-seen.json`: add a `grok-build` entry naming the discovered channel (npm package + dist-tag + version if npm; `{"channel": "github-releases", "repo": "...", "version": "..."}` shape if GitHub; `{"channel": "docs-only"}` if neither).
  - `.github/workflows/release-watch.yml`: npm channel ⇒ a `check "<package>" "<tag>" "<notes>"` line like the existing three; GitHub releases ⇒ a `gh api repos/<owner>/<repo>/releases/latest` step with prerelease filtering writing the same triage-issue flow; docs-only ⇒ NO new check — add a workflow comment recording that doc-drift.yml covers Grok (it already derives URLs from `rules/**/*.json`) and note the determination in `docs/maintenance/runbook.md`.
  - `@github/copilot`, `@anthropic-ai/claude-code`, `@openai/codex` entries stay untouched (owner-resolved).
  - The `if: vars.ENABLE_FRESHNESS_WORKFLOWS == 'true'` gate stays exactly as is — authored, not enabled.
  - Run `actionlint` — must pass.
- [ ] **Step 6: Gates and commit:**

```bash
git add rules/grok-build/ fixtures/grok-build/ crates/ freshness/ .github/workflows/release-watch.yml rules/ruleset.json docs/maintenance/runbook.md
git commit -m "feat(rules): Grok Build rules citing the clean-room run; freshness channel authored, still gated off"
```

---

## Phase 4 — WP4: Agent-facing surface (after Task 16; parallel to Phase 3)

### Task 20: `capabilities` subcommand + schema

**Files:**
- Create: `schemas/capabilities.schema.json`
- Create: `crates/harness-guard-cli/src/capabilities.rs`
- Modify: `crates/harness-guard-cli/src/main.rs` (subcommand + dispatch)
- Modify: `crates/harness-guard-cli/tests/cli_surface.rs`
- Create: `crates/harness-guard-cli/tests/capabilities.rs`
- Create: `crates/harness-guard-cli/tests/goldens/capabilities.expected.json` (authored after Task 19 lands — see Step 3)

**Interfaces:**
- Consumes: `load_rules()`, `ruleset_version()`, `conservative_aggregates` (Task 14) — the SAME loaded-rules data as `scan`, so the two cannot drift (§8.1).
- Produces: `harness-guard capabilities [--json]`, offline, deterministic, zero-network.

- [ ] **Step 1: Write `schemas/capabilities.schema.json`**

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "harness-guard:capabilities:1.0",
  "title": "Harness Guard capabilities introspection",
  "type": "object",
  "required": ["schema_version", "harness_guard_version", "ruleset_version", "report_schema_version", "tools", "commands", "exit_codes"],
  "additionalProperties": false,
  "properties": {
    "schema_version": { "const": "1.0" },
    "harness_guard_version": { "type": "string", "minLength": 1 },
    "ruleset_version": { "type": "string", "pattern": "^\\d{4}\\.\\d{2}\\.\\d{2}$" },
    "report_schema_version": { "const": "1.1" },
    "tools": {
      "type": "array",
      "minItems": 3,
      "maxItems": 3,
      "items": {
        "type": "object",
        "required": ["tool", "rules", "categories", "rules_last_verified_version", "rules_verified_date"],
        "additionalProperties": false,
        "properties": {
          "tool": { "enum": ["claude-code", "codex", "grok-build"] },
          "rules": { "type": "integer", "minimum": 0 },
          "categories": { "type": "array", "items": { "type": "string", "minLength": 1 } },
          "rules_last_verified_version": { "type": ["string", "null"] },
          "rules_verified_date": { "type": ["string", "null"] }
        }
      }
    },
    "commands": { "type": "array", "items": { "type": "string", "minLength": 1 } },
    "exit_codes": {
      "type": "object",
      "required": ["0", "1", "2"],
      "additionalProperties": false,
      "properties": {
        "0": { "type": "string", "minLength": 1 },
        "1": { "type": "string", "minLength": 1 },
        "2": { "type": "string", "minLength": 1 }
      }
    }
  }
}
```

- [ ] **Step 2: Implement `capabilities.rs`**

```rust
//! `capabilities [--json]` — offline, deterministic, zero-network
//! introspection for agents deciding how to invoke the tool (§8.1). Sourced
//! from the same loaded-rules data as `scan`, so the two cannot drift.
use harness_guard_core::harness::HarnessId;
use harness_guard_core::scan::conservative_aggregates;
use harness_guard_rules::loader::{ValidatedRule, load_rules, ruleset_version};
use serde::Serialize;
use std::collections::BTreeSet;

#[derive(Serialize)]
pub struct Capabilities {
    pub schema_version: String,
    pub harness_guard_version: String,
    pub ruleset_version: String,
    pub report_schema_version: String,
    pub tools: Vec<ToolCapabilities>,
    pub commands: Vec<String>,
    pub exit_codes: ExitCodes,
}

#[derive(Serialize)]
pub struct ToolCapabilities {
    pub tool: String,
    pub rules: u32,
    pub categories: Vec<String>,
    pub rules_last_verified_version: Option<String>,
    pub rules_verified_date: Option<String>,
}

#[derive(Serialize)]
pub struct ExitCodes {
    #[serde(rename = "0")]
    pub ok: String,
    #[serde(rename = "1")]
    pub findings: String,
    #[serde(rename = "2")]
    pub degraded: String,
}

pub fn gather() -> Capabilities {
    let rules = load_rules();
    let tools = HarnessId::ALL
        .iter()
        .map(|&harness| {
            let tool_rules: Vec<&ValidatedRule> = rules
                .iter()
                .filter(|rule| rule.raw().tool == harness.as_str())
                .collect();
            let categories: BTreeSet<String> = tool_rules
                .iter()
                .map(|rule| rule.raw().category.clone())
                .collect();
            let (rules_last_verified_version, rules_verified_date) =
                conservative_aggregates(&tool_rules);
            ToolCapabilities {
                tool: harness.as_str().to_string(),
                rules: tool_rules.len() as u32,
                categories: categories.into_iter().collect(),
                rules_last_verified_version,
                rules_verified_date,
            }
        })
        .collect();

    Capabilities {
        schema_version: "1.0".to_string(),
        harness_guard_version: env!("CARGO_PKG_VERSION").to_string(),
        ruleset_version: ruleset_version(),
        report_schema_version: "1.1".to_string(),
        tools,
        commands: ["scan", "list", "explain", "version", "capabilities", "completions"]
            .map(String::from)
            .to_vec(),
        exit_codes: ExitCodes {
            ok: "no findings at/above --fail-on".to_string(),
            findings: "findings at/above --fail-on".to_string(),
            degraded: "degraded or internal/usage error".to_string(),
        },
    }
}

pub fn render_json(capabilities: &Capabilities) -> String {
    serde_json::to_string_pretty(capabilities).expect("capabilities serialize")
}

pub fn render_table(capabilities: &Capabilities) -> String {
    let mut table = comfy_table::Table::new();
    table.set_header(["tool", "rules", "categories", "verified ≤", "verified on"]);
    for tool in &capabilities.tools {
        table.add_row([
            tool.tool.as_str(),
            &tool.rules.to_string(),
            &tool.categories.join(", "),
            tool.rules_last_verified_version.as_deref().unwrap_or("-"),
            tool.rules_verified_date.as_deref().unwrap_or("-"),
        ]);
    }
    format!(
        "harness-guard {} · ruleset {} · report schema {}\n{table}\n",
        capabilities.harness_guard_version,
        capabilities.ruleset_version,
        capabilities.report_schema_version
    )
}
```

Wire in `main.rs`: `mod capabilities;`, subcommand

```rust
/// Machine-readable summary of audited tools, rule counts, and contracts
#[command(before_help = "Examples:\n  harness-guard capabilities\n  harness-guard capabilities --json")]
Capabilities {
    /// Emit JSON (schemas/capabilities.schema.json contract)
    #[arg(long)]
    json: bool,
},
```

and dispatch: `Cmd::Capabilities { json } => { let caps = capabilities::gather(); if json { println!("{}", capabilities::render_json(&caps)); } else { anstream::print!("{}", capabilities::render_table(&caps)); } ExitCode::SUCCESS }`.

- [ ] **Step 3: Tests**

Create `crates/harness-guard-cli/tests/capabilities.rs`:

Sequencing: the schema-validation, determinism, and contains-assertion tests below have no dependency and land now. The two golden/snapshot tests at the end of this step (`capabilities_table_view_is_golden_tested`, `capabilities_json_view_matches_committed_golden`) pin exact rule counts and categories per tool, which are not stable until Task 19 (last of the rule-authoring tasks, release-gating) lands — author those two and their committed golden file only once Task 19 has landed, per the Sequencing note above.

```rust
mod common;
use common::*;

#[test]
fn capabilities_json_validates_against_its_schema_and_agrees_with_rules() {
    let output = run_case("hardened", &["capabilities", "--json"]);
    assert_eq!(output.status.code(), Some(0));
    let caps: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let schema: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(repo_root().join("schemas/capabilities.schema.json")).unwrap(),
    )
    .unwrap();
    let validator = jsonschema::validator_for(&schema).unwrap();
    assert!(
        validator.validate(&caps).is_ok(),
        "{:?}",
        validator.iter_errors(&caps).map(|e| e.to_string()).collect::<Vec<_>>()
    );
    // Alphabetical tool ordering, and rule counts agree with explain surface.
    let tools: Vec<&str> = caps["tools"].as_array().unwrap().iter()
        .map(|tool| tool["tool"].as_str().unwrap()).collect();
    assert_eq!(tools, ["claude-code", "codex", "grok-build"]);
    let total: u64 = caps["tools"].as_array().unwrap().iter()
        .map(|tool| tool["rules"].as_u64().unwrap()).sum();
    assert!(total >= 1);
}

#[test]
fn capabilities_is_identical_regardless_of_fixture_environment() {
    // Offline + deterministic: capabilities reads no filesystem state.
    let first = run_case("hardened", &["capabilities", "--json"]);
    let second = run_case("missing", &["capabilities", "--json"]);
    assert_eq!(first.stdout, second.stdout);
}

#[test]
fn capabilities_table_lists_all_three_tools() {
    let output = run_case("hardened", &["capabilities"]);
    let text = String::from_utf8_lossy(&output.stdout);
    for tool in ["claude-code", "codex", "grok-build"] {
        assert!(text.contains(tool));
    }
}

// §8.1 requires both views golden-tested. Author these two only after Task 19
// lands (see the Sequencing note above) — before then rule counts/categories
// per tool are still moving and any pinned golden would immediately drift.
#[test]
fn capabilities_table_view_is_golden_tested() {
    let output = run_case("hardened", &["capabilities"]);
    insta::assert_snapshot!("capabilities_table", String::from_utf8_lossy(&output.stdout));
}

#[test]
fn capabilities_json_view_matches_committed_golden() {
    // Companion to the table snapshot above: golden-tested via byte-for-byte
    // comparison against a committed fixture — the same convention
    // scan_fixtures.rs uses for expected.json — rather than a second insta
    // snapshot, so a rule-count regression is caught by two independent
    // mechanisms.
    let output = run_case("hardened", &["capabilities", "--json"]);
    let actual: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let golden: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(
            repo_root().join("crates/harness-guard-cli/tests/goldens/capabilities.expected.json"),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(actual, golden, "capabilities --json output drifted from the committed golden");
}
```

Generate `crates/harness-guard-cli/tests/goldens/capabilities.expected.json` by running `harness-guard capabilities --json` against the final, post-Task-19 ruleset and reviewing the output before committing it — the same reviewed-run convention Task 18 Step 5 uses for `expected.json`.

In `cli_surface.rs`, add `vec!["capabilities", "--help"]` to the help-loop (forbidden-phrase + Examples-before-Usage coverage extends automatically, §8.1).

- [ ] **Step 4: Run, gates, commit**

```bash
git add schemas/capabilities.schema.json crates/harness-guard-cli/
git commit -m "feat: capabilities introspection subcommand with schema contract"
```

### Task 21: Consumer-facing agent guide

**Files:**
- Create: `docs/agent-guide.md`
- Modify: `README.md` (one link line in an appropriate section)
- Modify: `crates/harness-guard-rules/tests/tripwires.rs` (extend the retired-keys scan to `docs/agent-guide.md`; add a no-cadence-claim assertion)

- [ ] **Step 1: Write `docs/agent-guide.md`**

Content requirements (§8.2) — write the full document, ~80 lines, containing: (1) what Harness Guard is, using the binding phrase "local, execution-free, per-finding-cited config auditor" verbatim; (2) drive `capabilities --json` first and how to read it (rule counts per tool, `ruleset_version` vs binary version); (3) `scan --json` contract — point to `schemas/report.schema.json` 1.1, note `tools[]`/findings ordering is contractual and `network_requests_made` is always 0; (4) exit codes: 0 = no findings at/above `--fail-on`, 1 = findings at/above threshold, 2 = degraded or internal/usage error — and that 2 means REDUCED visibility, not safety; (5) status semantics: `unknown` = locally unknowable ≠ unchecked ≠ safe; `stale-ruleset` = unverified for the detected version, not failing; (6) `explain <rule-id>` for the full evidence record (sources, hashes, tested versions, limitations); (7) the hard nots — no network, no execution of discovered tools, no raw config values in any output (agents must not expect config contents), no numeric score; (8) NO verification-cadence claims anywhere in the document. Link from `README.md`: `See [docs/agent-guide.md](docs/agent-guide.md) for driving Harness Guard from an agent.`

- [ ] **Step 2: Pin it.** In `tripwires.rs`, add:

```rust
#[test]
fn agent_guide_carries_positioning_and_no_cadence_claims() {
    let text = std::fs::read_to_string(repo_root().join("docs/agent-guide.md")).unwrap();
    assert!(text.contains("local, execution-free, per-finding-cited config auditor"));
    let forbidden_phrase = ["AI agent", "security scanner"].join(" ");
    assert!(!text.contains(&forbidden_phrase));
    for cadence in ["weekly", "daily re-verification", "continuously verified", "always up to date"] {
        assert!(!text.to_lowercase().contains(cadence), "cadence claim {cadence:?} found");
    }
}
```

- [ ] **Step 3: Run, gates, commit**

```bash
git add docs/agent-guide.md README.md crates/harness-guard-rules/tests/tripwires.rs
git commit -m "docs: consumer-facing agent guide, positioning-pinned"
```

---

## Phase 5 — WP6: Release mechanics (last)

### Task 22: No-egress proof over a multi-harness fixture scan

**Files:**
- Modify: `scripts/no-egress/run-macos.sh`

- [ ] **Step 1: Extend the sandboxed scan matrix.** The script currently loops codex cases setting `CODEX_HOME="$case_dir/codex-home"`. Add a second loop over representative claude-code and grok-build cases (at minimum `hardened` and the format-specific malformed case — `malformed-json` for claude-code, `malformed-toml` for grok-build — per harness) using the new fixture layout, plus the Task 18 mixed two-store case so at least one sandboxed scan detects TWO harnesses in the same run:

```sh
for tool_case in "claude-code hardened" "claude-code duplicate-keys" "claude-code malformed-json" "grok-build hardened" "grok-build malformed-toml"; do
    set -- $tool_case
    tool="$1"; case_name="$2"
    case_dir="$PWD/fixtures/$tool/$case_name/files"
    # HOME is the committed synthetic home; CODEX_HOME points at an absent
    # dir so only the fixture's harness is detected.
    /usr/bin/env -i \
        HOME="$case_dir/home" \
        CODEX_HOME="$case_dir/home/absent-codex-home" \
        PATH="$case_dir/path" \
        NO_COLOR=1 \
        /usr/bin/sandbox-exec -f "$SB" "$BIN" scan --json \
        > "$PROOF_DIR/scan-$tool-$case_name.json" || true
done

# Mixed two-store case (§11.2): codex AND claude-code detected in one scan;
# CODEX_HOME points INTO the fixture home, at the committed synthetic store.
mixed_dir="$PWD/fixtures/mixed/codex-pass-claude-degraded/files"
/usr/bin/env -i \
    HOME="$mixed_dir/home" \
    CODEX_HOME="$mixed_dir/home/.codex" \
    PATH="$mixed_dir/path" \
    NO_COLOR=1 \
    /usr/bin/sandbox-exec -f "$SB" "$BIN" scan --json \
    > "$PROOF_DIR/scan-mixed-codex-pass-claude-degraded.json" || true
```

Mirror the existing JSON-contract verification (`jq` checks: `network_requests_made == 0`, valid report shape) for the new outputs; for the mixed output additionally assert `jq '.tools | length' == 2` — the runtime proof must cover a genuinely multi-detected scan, not only single-harness rows. Keep the telemetry-denial sweep unchanged — it already covers the whole scan window.

- [ ] **Step 2: Run it.**

Run: `scripts/no-egress/run-macos.sh`
Expected: `ok: scan telemetry contains no denied network attempts` and per-case contract OKs.

- [ ] **Step 3: Commit**

```bash
git add scripts/no-egress/run-macos.sh
git commit -m "test: extend the no-egress runtime proof to a multi-harness fixture scan"
```

### Task 23: Version 0.0.1 + CHANGELOG

**Files:**
- Modify: `Cargo.toml` (workspace `version = "0.1.0"` → `"0.0.1"`), `Cargo.lock` (regenerates)
- Modify: `crates/harness-guard-cli/tests/cli_surface.rs:141,150` (`harness-guard 0.1.0` → `harness-guard 0.0.1`, two tests)
- Modify: `crates/harness-guard-cli/src/render_term.rs` unit test (`"0.1.0"` literal → `"0.0.1"`)
- Snapshots: every `.snap` embedding `harness-guard 0.1.0` in the header line
- Create: `CHANGELOG.md`

- [ ] **Step 1: Bump the version.** Edit `Cargo.toml` `[workspace.package] version = "0.0.1"`. Run `cargo build` to refresh `Cargo.lock`. Update the two `cli_surface.rs` assertions and the `render_term.rs` test literal. Run `cargo test -p harness-guard-cli`; review snapshot diffs — ONLY the version token in header lines may change; accept.

- [ ] **Step 2: Write `CHANGELOG.md`** (Keep a Changelog format):

```markdown
# Changelog

All notable changes to this project will be documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [0.0.1] - <tag date>

First release: the reviewed Codex CLI thin slice generalized to three
co-equal audited harnesses.

### Added
- Harnesses: Claude Code, Codex CLI, and Grok Build — user-scope config
  auditing with per-finding citations, execution-free version detection, and
  conservative degradation (`unknown` / `stale-ruleset`).
- Declarative rule engine: rules are pure data over a closed set of typed
  match primitives; totality (exhaustiveness, overlap freedom, status
  legality) is proven at rule load time.
- `capabilities` subcommand (`schemas/capabilities.schema.json` 1.0) and
  `docs/agent-guide.md` for agent consumers.
- Grok Build clean-room reproduction protocol and dated evidence artifacts;
  behavior claims are version-pinned to the reproduced release.
- JSON config parsing (Claude Code `settings.json`) at the same hostile-input
  rigor as TOML: bounded reads, depth limits, value-free diagnostics.

### Changed
- Rule and report schemas: 1.0 → 1.1 (`match` primitives, integer
  observations with `integer_bounds`, widened `tool`/`scopes` enums).
- Workspace version 0.1.0 → 0.0.1 (owner decision 2026-07-16; nothing was
  ever published, so the backwards move has no consumers).

### Notes
- No network requests are ever made by a scan; nothing discovered is
  executed. Freshness automation ships authored but disabled.
```

Replace `<tag date>` with the actual date at release execution.

- [ ] **Step 3: Gates and commit**

Run: full gates (fmt, clippy, deny, test; no-egress on macOS).

```bash
git add Cargo.toml Cargo.lock CHANGELOG.md crates/harness-guard-cli/
git commit -m "chore: set workspace version 0.0.1 and add CHANGELOG"
```

### Task 24: Documentation corrections (§10.3)

**Files:**
- Modify: `CONTEXT.md`, `AGENTS.md`, `README.md`, `rules/README.md`, `docs/product/decision-and-strategy.md`, `notes/session-history.md`

- [ ] **Step 1: `CONTEXT.md`.** Lines 81–85: delete the stale "Release work must preserve private visibility until local gates and private CI pass." sentence (repo verified public 2026-07-16) and replace with: "The repository is public. Freshness workflows remain triage-only and disabled. Do not publish packages, create a GitHub Release, or make other external changes without the exact authorization required by `AGENTS.md`." Refresh "Current implemented scope" and "Current phase" to the three-harness state exactly as shipped by Tasks 3–21 (never ahead of what is merged): three harnesses, the declarative engine, rule counts per tool, capabilities subcommand. Update the "not implemented" list (remove Claude Code and Grok; keep Copilot CLI, Gemini CLI, Cursor, OpenCode as unimplemented candidates).
- [ ] **Step 2: `AGENTS.md`.** Generalize the safety rule: "Never inspect or ingest the developer's real `~/.codex`, ambient `CODEX_HOME`, other harness stores…" → "Never inspect or ingest the developer's real harness stores — `~/.codex`, `~/.claude`, `~/.grok`, or any home-override environment variable pointing at them (`CODEX_HOME` and any documented equivalents) — nor source projects, prompt/session transcripts, shell history, `.env` files, credentials, or secrets."
- [ ] **Step 3: `README.md` and `rules/README.md`.** Describe three-harness coverage only as shipped; positioning phrase verbatim where the tool is described; zero cadence claims. `rules/README.md` documents schema 1.1 and the per-tool directory convention (`rules/<tool-id>/`, id prefix rule).
- [ ] **Step 4: `docs/product/decision-and-strategy.md`.** Append (do not rewrite) a dated correction note at the top: "**Correction (2026-07-16):** the third-tool selection below is superseded by the 2026-07-16 owner decision — the 0.0.1 harness set is Claude Code + Codex CLI + Grok Build. GitHub Copilot CLI remains a likely 0.x candidate and its freshness tracking is retained."
- [ ] **Step 5: `notes/session-history.md`.** Append (never contradict) a dated entry summarizing this release's scope and any unresolved risks.
- [ ] **Step 6: Commit**

```bash
git add CONTEXT.md AGENTS.md README.md rules/README.md docs/product/decision-and-strategy.md notes/session-history.md
git commit -m "docs: three-harness scope corrections; visibility language fixed; strategy supersession noted"
```

### Task 25: Release gate run + tag checklist (execution owner-gated)

**Files:** none new — this is verification + an owner conversation.

- [ ] **Step 1: Acceptance sweep against spec §13.** Verify each criterion mechanically:
  1. Engine: `git log --diff-filter=D --name-only | grep evaluate.rs` shows the deletion; Tasks 4/9 tests green.
  2. Abstraction: `cargo test --workspace` green; `run_harness_case` fixtures cover all three tools; `fixtures/mixed/codex-pass-claude-degraded/expected.json` pins exit 2 with a two-entry `tools[]` and merged summary counts (`jq '.tools | length'` → 2).
  3. Rules: `jq -r .schema_version rules/*/*.json` → all `1.1`; codex rule count ≥ 3; every rule's sources carry authoring-time `retrieved` dates + hashes + anchors.
  4. Grok gate: `ls docs/research/evidence/grok-build/` shows a dated run; grok rules cite it; tripwires green.
  5. Degradation: fixture matrices per harness include unknown-version / out-of-range / unrecognized / unreadable / unparseable cases; engine_hostility green.
  6. Agent surface: capabilities schema validation test green; agent-guide test green.
  7. Freshness: `grep ENABLE_FRESHNESS_WORKFLOWS .github/workflows/*.yml` shows both gates intact; `jq .packages freshness/last-seen.json` includes copilot + the grok entry; `actionlint` green.
  8. Release: `grep '^version' Cargo.toml` → `0.0.1`; CHANGELOG present; doc corrections merged.
  9. Positioning: cli_surface forbidden-phrase tests green over all commands.
  10. `notes/session-history.md` appended.
- [ ] **Step 2: Full gates at the release commit.**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo deny check
cargo test --workspace
scripts/no-egress/run-macos.sh   # macOS
actionlint                        # if workflows changed since last run
```
All must pass at the exact commit to be tagged. MSRV 1.85 stays release-enforced (commit 798a7f1's gate) — do not touch it.

- [ ] **Step 3: STOP — owner authorization checkpoint.** Present the checklist results to the owner. The tag (`git tag 0.0.1`) and the GitHub Release (with the CHANGELOG excerpt) EACH require their own explicit owner go at execution time. No crates.io publish, no npm, no other distribution, no branch push without the exact requested action. Nothing in this plan authorizes executing these steps.

---

## Self-review notes (performed while writing)

- **Spec coverage:** §3 naming → Tasks 10/14/15; §5.1 → 10/11/15/16; §5.2 → 5/12; §5.3 → 13/19; §5.4 → 3 (scopes enum) + validator unset⇒unknown (4); §5.5 → 14/15; §5.6 → 3; §5.7 → 6 (render_observation)/15 (redaction); §6 → 3–9; §7.1/7.2/7.3/7.4 → 17/18/2+19/covered by observation-required schema; §8 → 20/21; §9 → 19 step 5; §10 → 22–25; §11 → distributed per task, with the §11.2 aggregation bullet mapped explicitly: multi-rule per-tool conservative aggregates → Task 14 (unit) + Task 17 step 5 (golden); multi-harness summary counts and mixed states (one harness degraded, another passing ⇒ exit 2 with full report) → Task 18 step 5 (`fixtures/mixed/codex-pass-claude-degraded`, the only fixture where a scan detects two tools) + Task 22's mixed no-egress row; §12 ordering honored; §13 swept in Task 25; §14 decisions embedded at their tasks (j → 14, g → 8, e → 4, d → 6, f → 4, b → 11, a → 10, c → 3, h → schema keeps observation required, i → 19, k → 24).
- **Type consistency check:** `MatchSpec`/`MatchValue`/`IntegerBounds` (Task 3) are consumed by exactly those names in Tasks 4/6; `ExtractedValue::{Unset,Str,Bool,Int,Other}` (Task 5) in 6/12; `scan_harness`/`conservative_aggregates` (14) in 15/20; `detect_version`/`binary_on_path(root, harness)` (13) in 14/15; `run_harness_case` (16) in 18/19; `run_mixed_case` (16) in 18 (step 5) and Task 25's sweep.
- **Placeholder scan:** rule-content values in Tasks 17–19 are deliberately retrieval-time inputs mandated by the spec ("this spec deliberately leaves documented values as placeholders — rule content is filled only from fresh retrieval"); each such point names its exact source URL, decision rule, and structural skeleton. No other TBDs.

## Execution handoff

Plan complete and saved to `docs/superpowers/plans/2026-07-16-harness-guard-0.0.1-multi-harness.md`. Two execution options:

**1. Subagent-Driven (recommended)** — dispatch a fresh subagent per task with review between tasks (`superpowers:subagent-driven-development`). Tasks 17–19 need live web retrieval; give those subagents fetch access and the runbook.

**2. Inline Execution** — `superpowers:executing-plans`, batch execution with checkpoints at each phase boundary.

Hard gates either way: OWNER CHECKPOINT A/B before Task 19, and the Task 25 owner authorization before any tag/Release action.
