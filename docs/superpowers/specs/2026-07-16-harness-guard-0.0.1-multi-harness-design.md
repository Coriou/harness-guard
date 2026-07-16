# Harness Guard 0.0.1 Multi-Harness Generalization — Design Spec

**Date:** 2026-07-16
**Status:** Approved decision pack → spec. Inputs: `CONTEXT.md`, `AGENTS.md`, `docs/superpowers/specs/2026-07-14-harness-guard-v1-thin-slice-design.md` (extended, not replaced), `docs/superpowers/plans/2026-07-14-harness-guard-v1-thin-slice-review-findings.md` (adjudicated), `docs/research/verification-audit-2026-07-13.md`, `docs/research/per-tool/grok-build.md`, `docs/maintenance/runbook.md`, owner decisions of 2026-07-16 (harness set, Grok parity, declarative engine, 0.0.1 versioning, freshness gating).
**Positioning (binding, verbatim, test-pinned):** Harness Guard is a *local, execution-free, per-finding-cited config auditor*. "AI agent security scanner" appears nowhere in user-facing text (`crates/harness-guard-cli/tests/cli_surface.rs` pins this).

## 1. Goal

Generalize the reviewed Codex thin slice into the 0.0.1 release: **Claude Code, Codex CLI, and Grok Build as co-equal audited harnesses**, with the same evidence rigor, safety invariants, output contract, and conservative degradation the slice proved. Rule evaluation becomes a declarative engine so that new rules are pure data + fixtures with no new Rust per rule — the sustainability mechanism for near-daily upstream releases. 0.0.1 is CLI-only, tagged `0.0.1`, and gated on a fresh clean-room reproduction of Grok Build's current behavior.

Owner decisions this spec implements (final, 2026-07-16):

1. The 0.0.1 harness set is **Claude Code + Codex CLI + Grok Build** (supersedes older docs naming GitHub Copilot CLI as the third tool; Copilot CLI remains a likely 0.x candidate and its freshness tracking stays).
2. Grok Build ships at **full rule parity** — not advisory-tier — which makes a clean-room, version-pinned reproduction protocol a release-gating work package (§7.3).
3. Rule evaluation becomes a **declarative engine** with a small, closed set of typed check primitives (§6).
4. Workspace Cargo version moves **0.1.0 → 0.0.1**; the release is tagged `0.0.1`. Ruleset CalVer stays independent.
5. Freshness workflows stay **gated off** through 0.0.1; Grok Build tracking is added so it is ready to switch on (§9).

## 2. Scope

**In scope (exactly, no more):**

1. Harness abstraction: multi-harness `DiscoveryRoot`, harness descriptors, JSON config parsing at TOML-equivalent hostile rigor, per-harness execution-free version detection, generalized scan dispatch, CLI `--tool` widening, rule/report schema evolution to 1.1 (§5).
2. Declarative rule engine: closed primitive set, load-time totality/exhaustiveness validation, hostile-input tests, behavior-preserving migration of `codex-history-persist-01` (§6).
3. Per-harness evidence work packages: Codex expansion beyond the single rule; Claude Code rules from fresh retrieval; Grok Build clean-room reproduction protocol + rules (§7).
4. Agent-facing polish: `capabilities` introspection subcommand + consumer-facing agent guide (§8).
5. Freshness extension for Grok Build, authored but kept off; `@github/copilot` tracking retained (§9).
6. Release mechanics: version → 0.0.1, CHANGELOG, doc corrections (including the stale CONTEXT.md private-visibility lines), tag/Release checklist gated on explicit owner authorization at execution time (§10).

**Explicitly not in 0.0.1** (deferred, not authorized here): GUI/Tauri; any fourth harness (Gemini CLI, Copilot CLI, Cursor, OpenCode are 0.x candidates per `docs/research/verification-audit-2026-07-13.md`); Windows support (the `compile_error!` gate stays); `--fix` or any write operation; any networking in the product; SQLite/history; SARIF; rule signing or out-of-band ruleset delivery (rules remain compiled in); crates.io or any package publishing; enabling scheduled freshness runs; any public verification-cadence claim; project-scope/multi-layer config inspection (§5.4); observation-less account-state rules (§7.4).

**Unchanged and binding (the thin slice's contracts are extended, never replaced):** status enum `pass | finding | unknown | stale-ruleset`; exit codes 0/1/2 with degraded-scan semantics; redaction rules (§5.4 of the thin-slice spec, plus §5.7 here); the three-layer no-egress proof; synthetic-fixtures-only testing; explicit-`DiscoveryRoot`-only core; the evidence model (≥1 source with `url`+`retrieved` structurally required for non-unknown outcomes, explicit `tested_versions` with the MDN `<=` convention, conservative degradation to `stale-ruleset`/`unknown`); `rules/` as a standalone Apache-2.0 data package consumed only through the schema contract; auth/account state user-confirmed-or-unknown, never inferred.

## 3. Naming and identifiers (resolved)

- The schema/report/CLI field stays **`tool`** (contract continuity with rule schema, report schema, and `--tool`). "Harness" remains the product-prose word.
- Tool ids (closed set, kebab-case): **`codex`**, **`claude-code`**, **`grok-build`**. Rule ids prefix their tool id (`claude-code-transcript-retention-01`); rule files live in `rules/<tool-id>/`; fixtures in `fixtures/<tool-id>/<case>/`.
- Rust: `enum HarnessId { Codex, ClaudeCode, GrokBuild }` in `harness-guard-core`, serialized to/parsed from the kebab-case ids. Every match on it is exhaustive — adding a harness is a deliberate compile-visible act.

## 4. Layout deltas

```text
harness-guard/
├── rules/
│   ├── codex/…                    # existing + expansion rules
│   ├── claude-code/…              # new
│   └── grok-build/…               # new
├── fixtures/
│   ├── codex/…                    # existing matrix
│   ├── claude-code/…              # new matrix (JSON hostile cases, §11.2)
│   └── grok-build/…               # new matrix
├── schemas/                       # rule + report bumped to 1.1; capabilities.schema.json added (1.0)
├── crates/harness-guard-core/src/
│   ├── discovery.rs               # multi-harness DiscoveryRoot
│   ├── harness.rs                 # NEW: HarnessId + descriptor table
│   ├── parse.rs                   # TOML (existing)
│   ├── parse_json.rs              # NEW: JSON at equal rigor
│   ├── engine.rs                  # NEW: declarative evaluation (replaces evaluate.rs)
│   └── version.rs                 # parameterized npm-walk detection
├── docs/
│   ├── agent-guide.md             # NEW: consumer-facing agent documentation
│   └── research/protocols/grok-build-cleanroom.md   # NEW: §7.3 protocol
└── CHANGELOG.md                   # NEW
```

## 5. Harness abstraction

### 5.1 DiscoveryRoot and descriptors

`DiscoveryRoot` generalizes from Codex-shaped to one explicit home per harness. Core still never touches ambient state; only the CLI crate constructs the root from the environment:

```rust
pub struct DiscoveryRoot {
    pub codex_home: PathBuf,
    pub claude_home: PathBuf,
    pub grok_home: PathBuf,
    pub path_dirs: Vec<PathBuf>,
}
impl DiscoveryRoot {
    pub fn home(&self, harness: HarnessId) -> &Path { … }
    pub fn config_path(&self, harness: HarnessId) -> PathBuf { … }
}
```

A static descriptor table (code, not config — the harness set is closed) carries per-harness facts:

| field | codex | claude-code | grok-build |
|---|---|---|---|
| default home | `~/.codex` | `~/.claude` | `~/.grok` |
| home override env var (CLI crate only) | `CODEX_HOME` (documented) | fresh-retrieval item; lead: `CLAUDE_CONFIG_DIR` | fresh-retrieval item; none assumed |
| user-scope config file | `config.toml` | `settings.json` | `config.toml` |
| config format | TOML | JSON | TOML |
| PATH binary name | `codex` | `claude` | fresh-retrieval item (protocol §7.3) |
| version detection | npm walk, expects `@openai/codex` | npm walk, expects `@anthropic-ai/claude-code` | **open research question** — resolved by §7.3; `None` until then |

Descriptor entries marked "fresh-retrieval item" are filled at implementation time from dated primary sources per the runbook — this spec deliberately does not pre-assert them. Every descriptor fact used in code must be traceable to the evidence recorded with that harness's rules.

Tests always inject fixture roots; a fixture-tree test asserts no absolute path escapes the fixture dir for each harness (extends the existing real-config protection to `~/.claude` and `~/.grok`, which also exist on the dev machine — AGENTS.md's "never inspect real harness stores" applies to all three, and its wording is generalized in §10.4).

### 5.2 Config parsing: JSON at equal rigor

New `parse_json.rs` mirroring `parse.rs` invariant-for-invariant:

- Reads go through the same `readfs` layer: hardened no-follow open, opened-handle validation, regular-file-only, symlink/reparse refusal, 1 MiB cap, UTF-8 only. No new read path.
- `serde_json` parses to `serde_json::Value`; serde_json's default recursion limit (128) is the backstop, and our own shared `MAX_NESTING_DEPTH = 32` bound is enforced identically to TOML (`value_depth` over objects/arrays). The constant moves to a shared module so the two parsers cannot drift.
- Parse failures produce a value-free `ParseFailure` (line/col where the error exposes offsets, structural message only — never raw text). serde_json error messages can embed source fragments; the failure constructor must strip to the categorical message (test-pinned with a hostile fixture whose secret-looking content must not appear in diagnostics).
- Duplicate keys: serde_json's last-value-wins behavior is documented in the module header and pinned by a test; it matches how the harness itself would parse the file, so it is the correct observation.
- Raw text and the parsed `Value` are dropped inside the scan, exactly as `scan.rs` does for TOML today.

Extraction generalizes `ExtractedValue` into a typed value shared by both parsers:

```rust
pub enum ExtractedValue {
    Unset,
    Str(String),   // held only until the engine checks the rule's domain
    Bool(bool),
    Int(i64),      // rejects floats, out-of-i64 numbers → Other
    Other,         // present but not representable — never rendered
}
```

Dotted-key extraction traverses objects/tables only (array indexing unsupported in 0.0.1; a key path hitting an array yields `Other`). Only rule-relevant keys are retained, as today.

### 5.3 Version detection per harness

`version.rs` parameterizes the existing bounded npm walk (binary name + expected package name from the descriptor) — the bounded symlink resolution, parent walk, 64 KiB `package.json` cap, strict `X.Y.Z` parsing, and TOCTOU-stable-handle behavior are unchanged and now shared:

- **codex:** unchanged (`@openai/codex`).
- **claude-code:** same walk expecting `@anthropic-ai/claude-code`. Native-installer layouts may legitimately yield `None` → `stale-ruleset` ("version not detected"), exactly like Codex Homebrew/standalone today. If fresh retrieval at implementation time documents a read-only, execution-free version marker for native installs, it may be added behind the same interface with its own bounded probe and recorded evidence; nothing is assumed here.
- **grok-build:** detection strategy is an output of the §7.3 protocol. Until packaging is established from evidence, detection returns `None` and every Grok finding degrades to `stale-ruleset` — never a confident claim. Do **not** assume npm.

Never run any harness binary. `detection_confidence` keeps the existing matrix (version × home) per harness.

### 5.4 Scopes

0.0.1 inspects the **user scope only** for all three harnesses, preserving the reviewed thin-slice posture: uninspected layers (managed/system, project, local, CLI, profiles) make an unset user-level value `unknown`, never an inferred default. The rule schema's `scopes` enum widens to `["user", "project", "local", "managed"]` so rules can *declare* which layers exist for their key, but only `user` is *inspected*; every rule's `limitations` must name the uninspected layers (pattern established by `rules/codex/history-persist-01.json`). Multi-layer inspection is a post-0.0.1 feature requiring its own design (project-scope scanning needs a project-directory argument and consent story).

### 5.5 Scan dispatch and CLI widening

- `scan.rs` gains `scan_harness(root, harness: HarnessId, rules: &[ValidatedRule]) -> Option<ScanResult>`; the current `scan_codex` body becomes its `Codex` arm with format/paths supplied by the descriptor. Per-harness rules are filtered by the rule's `tool` field.
- `--tool` accepts `codex | claude-code | grok-build` (repeatable). Default: scan every detected harness. `list` renders one row per harness (detected or not), alphabetical.
- Report `tools[]` stays alphabetically sorted (`claude-code`, `codex`, `grok-build`); findings sorted by rule id within each tool. `degraded` (exit 2) is true if any scanned harness degraded.
- Per-tool aggregate fix (latent single-rule shortcut in `scan.rs:94-100`): with multiple rules, `rules_last_verified_version` = the **minimum** of the rules' greatest tested `max` (the weakest guarantee), and `rules_verified_date` = the **earliest** `verified_on` among those rules — conservative in both dimensions, test-pinned. `version_in_range` keeps its existing all-rules semantics.

### 5.6 Schema evolution to 1.1

Rule and report schemas bump `schema_version` to `"1.1"`. Nothing has been released, so no external contract exists yet; the bump is still explicit because the repo is public. The loader and `FindingRecord::validate` accept exactly `1.1`; all bundled rules and fixture goldens migrate in the same change.

Rule schema 1.1 deltas (over `schemas/rule.schema.json`):

- `tool` enum: `["codex", "claude-code", "grok-build"]` (loader's `tool must be codex` check at `crates/harness-guard-rules/src/loader.rs:66` is replaced by the closed-set check plus a path consistency test: a rule in `rules/<tool>/` must declare that tool).
- `scopes` enum widened per §5.4.
- `observation.type` enum: `["enum", "bool", "integer"]`.
- `observation.integer_bounds` (`{ "min": int, "max": int }`, required iff `type` is `integer`): the closed domain for integer observations.
- `outcomes[].match` — the declarative condition, required on every outcome (§6.2). `when` remains required human prose.

Report schema 1.1 deltas (over `schemas/report.schema.json`): `tools[].tool` enum widened to the three ids. Finding shape, status matrix, and summary are unchanged.

### 5.7 Rendering and redaction deltas

- The allowlist invariant is restated for typed values: **an observation is rendered only from the parsed typed value, never echoed from source text.** Enum values render iff the string is in `allowed_render`; booleans render as `true`/`false` (so `allowed_render` for bool observations is `["true", "false", "unset"]`); integers render as the decimal re-serialization of the parsed `i64` iff it lies within `integer_bounds` — re-serializing a bounded parsed integer cannot leak arbitrary content. Anything else is `unrecognized` and never printed.
- `redact_config_path` generalizes: each harness home renders symbolically (`~/.claude/settings.json`, `$CODEX_HOME/config.toml` when the override var set it, etc.). Usernames and home paths never appear, as today.
- Terminal layout (§7.1 of the thin-slice spec) is unchanged; the detection block and per-tool sections simply enumerate three harnesses.

## 6. Declarative rule engine

### 6.1 Motivation and shape

`crates/harness-guard-core/src/evaluate.rs` today hardcodes `codex-history-persist-01`'s semantics (the `"none"`/`"save-all"` literals, unset-handling, message lookup); rule JSON's `when` is display-only. The engine replaces it: **rule JSON drives evaluation through a small, closed set of typed check primitives; adding a rule adds data + fixtures, zero Rust.** The engine and its rule-validation layer get the same hostile-input rigor as `readfs`/`parse` — bundled rules are trusted-at-review but the loader must survive a hostile forked ruleset, because `rules/` is an independently forkable data package.

### 6.2 Primitive set (closed, schema 1.1)

Each outcome carries exactly one `match` primitive (JSON Schema `oneOf`):

| primitive | shape | applies to | allowed statuses |
|---|---|---|---|
| `equals` | `{ "equals": { "value": <string\|bool\|int> } }` | enum/bool/integer | `pass`, `finding` |
| `any_of` | `{ "any_of": { "values": [ … ] } }` | enum/bool/integer | `pass`, `finding` |
| `int_range` | `{ "int_range": { "min": int\|null, "max": int\|null } }` | integer | `pass`, `finding` |
| `unset` | `{ "unset": true }` | all | `unknown` only |
| `unrecognized` | `{ "unrecognized": true }` | all | `unknown` only |

- `unset` fires when the key is absent from the inspected layer (or the config file is absent while the harness is detected). It **must** carry status `unknown` — a validator-enforced consequence of §5.4's user-scope-only inspection (uninspected layers may supply the effective value). Revisit only when multi-layer inspection lands.
- `unrecognized` is the mandatory catch-all: present-but-outside-domain values (including type mismatches, `Other`, out-of-`integer_bounds` integers). It **must** carry status `unknown`, with the fixed "raw values are never displayed" phrasing convention.
- The set is closed and versioned with the rule schema. A rule needing a new primitive is a schema 1.x proposal with its own validation and hostile tests — never an ad-hoc code path.

### 6.3 Load-time totality and determinism (validator, not runtime)

Extended `loader.rs` validation, all load-time errors (plus JSON Schema equivalents where expressible, mirroring the schema/Rust dual enforcement pattern):

1. **Type agreement:** every `match` value matches `observation.type`; `int_range` only on integer observations; `int_range` bounds lie within `integer_bounds` and `min ≤ max`.
2. **Domain membership:** every `equals`/`any_of` string value ∈ `allowed_render` (minus `"unset"`); this is what guarantees no un-allowlisted rendering can ever be produced by data alone.
3. **Cardinality:** exactly one `unset` outcome and exactly one `unrecognized` outcome per rule.
4. **Exhaustiveness:** enum domains (`allowed_render` minus `"unset"`) and bool domains (`true`+`false`) must be fully covered by the value-matching outcomes; integer rules' `int_range`s must jointly cover `integer_bounds` (interval-union check).
5. **No overlaps:** value sets and intervals must be pairwise disjoint, so evaluation is order-independent and deterministic — first-match-wins never matters.
6. **Status legality:** per the table above, plus the existing per-status field constraints (severity/confidence/remediation/unknown_reason/verify_url), unchanged from schema 1.0.

Together, 3–5 prove at load time that evaluation is **total**: every extracted value maps to exactly one outcome. Nothing falls through, nothing is order-sensitive, and the conservative-degradation direction cannot silently invert (the anti-precedent remains vercel/next.js#92091).

### 6.4 Evaluation semantics (engine-level, fixed)

Precedence is engine code, not rule data, and matches today's `evaluate.rs` exactly:

1. `ConfigState::Unreadable | Unparseable` ⇒ `unknown` with the refusal/parse reason (declared unknown conditions beat version bookkeeping).
2. Extract the typed value; select the unique matching outcome.
3. Version out of every tested range, or undetected ⇒ `stale-ruleset` wrapping the matched outcome: "Unverified — last-known rule indicates: {outcome.message} Observed: {rendering}." The unrecognized+stale safe fallback phrasing (adjudicated review finding 9) is preserved verbatim.
4. Otherwise emit the matched outcome's status/severity/confidence/message/remediation, with the §5.7 rendering.
5. Every constructed record passes `FindingRecord::validate()` before leaving the engine (the existing `checked` pattern).

Messages are rule-authored plain strings. The engine interpolates **only** the allowlisted observation rendering into the fixed stale template — no other interpolation, no format-string interpretation of rule text.

### 6.5 Loader embedding

The hardcoded `include_str!` vec (`loader.rs:319`) is replaced by a `build.rs` in `harness-guard-rules` (std-only, no new dependencies) that embeds every `rules/**/*.json` at compile time. A test enumerates the on-disk `rules/` tree and asserts 1:1 correspondence with the embedded set, and that every embedded rule validates — a forgotten or orphaned rule fails CI, not review attention.

### 6.6 Migration of `codex-history-persist-01`

The rule gains `match` blocks (`equals "none"` → pass; `equals "save-all"` → finding; `unset`; `unrecognized`) and `schema_version: "1.1"`. **Acceptance test: behavior-preserving** — every fixture golden and snapshot is byte-identical except schema-version fields. `evaluate.rs` is deleted only after the engine reproduces the full existing matrix, including all `stale-ruleset` phrasings.

### 6.7 Engine hostility tests

- Property-style tests: for arbitrary `ExtractedValue` (including hostile strings, `i64::MIN/MAX`, `Other`) × every bundled rule × every `ConfigState`, the engine returns a schema-valid record and never renders a string outside the rule's derivable renderings (assert the raw value is absent from the serialized record — the existing `hostile-archive-value` test generalized).
- Malformed-rule corpus: rules violating each §6.3 check (non-exhaustive, overlapping, type-mismatched, unset→pass, missing catch-all, hostile message content) must fail loading with a specific error; a negative schema-validation test per check where the schema can express it.
- The oneOf `match` parsing itself gets malformed-JSON tests (multiple primitives, empty `any_of`, inverted ranges).

## 7. Per-harness evidence work packages

All three follow the runbook protocol: fresh primary-source retrieval at rule-authoring time with actual `retrieved` dates, semantic-text `content_hash` via `scripts/freshness/normalize.sh`, Wayback `archived_url` anchors, `freshness/url-hashes.json` updates, and a `rules/ruleset.json` CalVer bump. `data/` and `docs/research/` remain quarantined leads, never evidence. This spec deliberately leaves documented values as placeholders — rule content is filled only from fresh retrieval.

**Parity definition (owner decision, made precise):** full parity means each harness gets the same rule *depth and rigor* — at minimum, cited coverage of the `retention` and `telemetry` categories where locally observable state exists, identical structural evidence requirements, and explicit `unknown` reporting for locally-unknowable dimensions. Parity is rigor + category coverage backed by fresh evidence, never claim-count symmetry: where evidence does not exist, the harness reports `unknown`, not a padded rule.

### 7.1 WP-Codex — expansion beyond the thin-slice rule

Leads from `docs/research/verification-audit-2026-07-13.md:85-93` (leads, not evidence): `analytics.enabled`, `feedback.enabled`, OTEL controls including prompt logging, plaintext TUI logging. Target 2–4 additional rules, exact set fixed after fresh retrieval of the config reference (`learn.chatgpt.com/docs/config-file/…`, the URLs already cited by the existing rule). Auth-method data-policy interpretation stays a limitation + `verify_url`, never inferred (existing pattern). Each rule: full fixture matrix rows, goldens, `explain` coverage.

### 7.2 WP-Claude-Code — rules from fresh retrieval

Starting evidence pointers (fresh-ish, dated, in-repo — still require fresh retrieval at authoring time): `verification-audit-2026-07-13.md:58-69` — local transcripts under `~/.claude/projects/` retained 30 days by default via `cleanupPeriodDays`; telemetry/error-reporting/feedback/nonessential-traffic are separate controls; managed/CLI/local/project/user settings scopes; sources `code.claude.com/docs/en/data-usage` and `/settings`.

Candidate rules (leads): transcript retention (`cleanupPeriodDays`, the first **integer** observation — exercises `int_range`/`integer_bounds`); telemetry/usage-statistics posture where a documented user-scope `settings.json` key exists. Controls that are env-var-only are **not** locally observable file state and must not be faked from the scanner's own environment; where the documented control lives in `settings.json`'s `env` block, that is observable and may be ruled on with the layer limitation stated. Consumer/commercial training split is auth-dependent ⇒ limitations + `verify_url` on concrete rules (§7.4).

### 7.3 WP-Grok-Build — clean-room reproduction protocol + rules (RELEASE-GATING)

No current-version evidence exists; the prior v0.2.93 reproduction (`github.com/cereblab/grok-build-exfil-repro`) and everything in `docs/research/per-tool/grok-build.md` are quarantined leads. Prior research demands fresh reproduction before any risk claim. The protocol is **maintainer lab work outside the product** — the product never captures traffic, never executes `grok`, never phones home; the protocol produces evidence artifacts that rules cite.

Protocol document: `docs/research/protocols/grok-build-cleanroom.md`, containing at minimum:

1. **Environment:** a fresh, disposable VM/container per run (macOS and Linux); no personal or client data anywhere in it; an owner-provisioned disposable xAI account (never a personal/company account); a purpose-built canary repository containing unique, never-published canary tokens and files the model is not asked to read.
2. **Version pinning:** the exact Grok Build version, install channel, and binary/package hash recorded before any run; this run's findings apply to exactly that version (`tested_versions` pins it precisely — the `<=` convention is not used for reproduction-derived claims unless separately justified).
3. **Capture:** system-level egress observation (e.g., mitmproxy with a locally installed CA, plus packet capture) scoped to the VM; record request targets, sizes, and payload structure for xAI endpoints; canary-token search over captured payloads is the transmission test.
4. **Matrix:** documented-default configuration; each currently documented mitigation key from `docs.x.ai/build/settings` and `/settings/reference` toggled independently; account/server-side flag states (e.g., the publicly reported remote `disable_codebase_upload` feature response) recorded as user-confirmed observations of the account UI where visible, otherwise `unknown`.
5. **Artifacts:** dated, sanitized capture summaries and configuration files under `docs/research/evidence/grok-build/<date>/` (canary tokens fine; no credentials, no personal data); semantic hashes and Wayback anchors for every cited docs page.
6. **Rule authoring:** locally observable `~/.grok/config.toml` posture keys cite `official-documentation`; upload/telemetry *behavior* claims cite `independent-reproduction` with the pinned version and are never generalized beyond it; server-side/account state is `unknown` with `verify_url`. Version drift after the run ⇒ `stale-ruleset` by construction, which is the honest state.
7. **Retired-keys tripwire (test-pinned):** `GROK_TELEMETRY_ENABLED`, `GROK_TELEMETRY_TRACE_UPLOAD`, and `[telemetry]`/`trace_upload` must never reappear in any rule, remediation, or user-facing string — a workspace test asserts their absence from `rules/**` and CLI output corpora (same mechanism as the forbidden-positioning-phrase test).

**Release gate:** 0.0.1 does not tag until this protocol has been executed against the then-current Grok Build version and its rules cite that run. If upstream releases between the run and the tag, the runbook triage flow decides re-run vs. shipping with the pinned version honestly reflected (detection of the newer version will yield `stale-ruleset`, which is correct behavior, not a blocker — but the *initial* evidence must exist).

### 7.4 Account-state rules — resolved

Rules whose only possible outcomes are `unknown` (pure account/server-state questions, e.g. "is this account commercial?") are **out of scope for 0.0.1**: `observation` stays required and file-backed. The auth/training/ZDR dimension is carried as limitations + `verify_url` on concrete rules (the pattern in `rules/codex/history-persist-01.json` limitations), and by the unknown-block reframe sentence in terminal output. Observation-less rule support is a schema proposal for later, if the GUI needs it.

## 8. Agent-facing surface

### 8.1 `capabilities` subcommand

`harness-guard capabilities [--json]` — offline, deterministic, zero-network introspection for agents deciding how to invoke the tool:

```json
{
  "schema_version": "1.0",
  "harness_guard_version": "0.0.1",
  "ruleset_version": "2026.MM.DD",
  "report_schema_version": "1.1",
  "tools": [
    { "tool": "claude-code", "rules": 2, "categories": ["retention", "telemetry"],
      "rules_last_verified_version": "…", "rules_verified_date": "…" },
    { "tool": "codex", "rules": 4, "…": "…" },
    { "tool": "grok-build", "rules": 3, "…": "…" }
  ],
  "commands": ["scan", "list", "explain", "version", "capabilities", "completions"],
  "exit_codes": { "0": "no findings at/above --fail-on", "1": "findings at/above --fail-on", "2": "degraded or internal/usage error" }
}
```

Contract: `schemas/capabilities.schema.json` (new, 1.0), golden-tested both views, alphabetical ordering, sourced from the same loaded-rules data as `scan` so it cannot drift. The human view is a compact table. Help text carries the binding positioning phrase; the `cli_surface.rs` forbidden-phrase/Examples-before-Usage tests extend to it.

### 8.2 Consumer-facing agent guide

`docs/agent-guide.md` (linked from README): how an agent should drive Harness Guard — `capabilities` first, `scan --json` contract and report schema pointer, exit-code handling, how to interpret `unknown` (locally unknowable ≠ unchecked ≠ safe) and `stale-ruleset` (unverified, not failing), `explain <rule-id>` for the full evidence record, and the hard nots (no network, no execution, no raw values — so agents must not expect config contents in output). Written against the same invariants; contains no cadence claims.

## 9. Freshness extension (authored, kept OFF)

Gating is unchanged: both workflows stay behind `ENABLE_FRESHNESS_WORKFLOWS`; enablement is a separate explicit post-release owner action; no public cadence claims until the pipeline has demonstrably run (runbook rule).

- **`doc-drift.yml`:** no structural change needed — it already derives its URL set from `rules/**/*.json`, so Claude Code and Grok Build citations are covered the moment their rules land. `freshness/url-hashes.json` gains their hashes as part of each rule-authoring WP.
- **`release-watch.yml` + `freshness/last-seen.json`:** today they track zero Grok packages. Once §7.3 establishes Grok Build's distribution channel: npm package ⇒ a `check` entry like the others; GitHub releases ⇒ an atom/`gh api` check with prerelease filtering; docs-page-only ⇒ rely on doc-drift and record that determination in the workflow comment and runbook. `last-seen.json` gains a `grok-build` entry naming the discovered channel. This lands in the same change as the Grok rules so switch-on readiness is complete.
- **`@github/copilot` tracking stays** (owner-resolved: working infrastructure; Copilot CLI is a likely 0.x candidate). `@anthropic-ai/claude-code` (`stable` tag) and `@openai/codex` (`latest`) entries are already correct for the 0.0.1 harness set.
- `actionlint` after any workflow edit (existing gate).

**Sustainability model for near-daily upstream releases** (how the pieces compose; document in the runbook, claim nowhere publicly): vendors release near-daily; `tested_versions` + conservative `stale-ruleset` degradation means an out-of-date ruleset *tells the truth* instead of guessing; release-watch (once on) turns version movement into triage issues; the declarative engine makes the human re-verification loop a data+fixtures edit with a CalVer bump — no Rust, no re-review of evaluation logic; ruleset version is reported separately from the binary version so users see rule freshness directly.

## 10. Release mechanics — 0.0.1

1. **Version:** workspace `Cargo.toml` `0.1.0` → `0.0.1` (owner decision; nothing published, so the semver-backwards move is safe — noted in CHANGELOG). Update the pinned `harness-guard 0.1.0` assertions in `cli_surface.rs` and any goldens embedding the binary version. Ruleset CalVer moves on its own axis with the new rules.
2. **CHANGELOG.md:** created (Keep a Changelog format); `0.0.1` is the first entry — thin slice + this generalization, with the schema-1.1 note.
3. **Doc corrections (this work package owns them):**
   - `CONTEXT.md:81-85` — the "preserve private visibility" language is stale (repo verified public 2026-07-16); correct it, and refresh "Current implemented scope"/"Current phase" to the three-harness state *as it becomes true*, never ahead of implementation.
   - `AGENTS.md` — generalize "real `~/.codex`" phrasing to all harness stores (`~/.codex`, `~/.claude`, `~/.grok`, and their override vars).
   - `README.md`, `rules/README.md` — three-harness coverage described only once shipped; positioning phrase everywhere; no cadence claims.
   - `docs/product/decision-and-strategy.md` gets a dated header note that the third-tool selection was superseded by the 2026-07-16 owner decision (append a correction, do not rewrite history).
   - `notes/session-history.md` — append (never contradict).
4. **Validation gates (unchanged, all green required):** `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo deny check`, `cargo test --workspace`, `scripts/no-egress/run-macos.sh` (extended to a multi-harness fixture scan), `actionlint`. MSRV stays 1.85 and remains release-enforced (commit 798a7f1's gate).
5. **Tag/Release checklist (execution gated on explicit owner authorization — repo-public was separately authorized; the tag and GitHub Release each need their own explicit go at execution time):** all gates green at the release commit → `git tag 0.0.1` → GitHub Release with CHANGELOG excerpt. **No crates.io publish, no npm, no other distribution in 0.0.1.** No step in this spec authorizes pushing, tagging, or releasing by itself.

## 11. Test plan additions

### 11.1 Carried invariants

The full thin-slice §10 plan carries forward and is replicated per harness: three-layer no-egress proof (deny bans / core clippy `disallowed-methods` / sandbox-exec + strace runtime scans, now over a three-harness fixture scan), schema validation of every `rules/` and `fixtures/` file plus negative citation tests, golden terminal+JSON outputs, exit-code tests, deterministic ordering, real-config protection for all three home dirs.

### 11.2 New matrices

- **Per-harness fixture matrix:** the existing 13-case Codex matrix pattern replicated for `claude-code` (JSON) and `grok-build` (TOML): missing-home, minimal, hardened, risky-explicit, per-rule value cases, malformed-config, unrecognized-value, symlink-config, oversized, deep-nesting, permission-denied, unknown-version, version-out-of-range. JSON-specific hostile cases: duplicate keys (last-wins pinned), float/huge-number where integer expected (→ unrecognized/unknown, never rendered), non-UTF-8, secret-shaped content that must not surface in parse diagnostics.
- **Engine tests:** §6.7 property + malformed-rule corpus; the migration byte-identity check (§6.6).
- **Aggregation tests:** multi-rule per-tool conservative aggregates (§5.5); multi-harness summary counts; mixed states (one harness degraded, another passing → exit 2 with full report).
- **Tripwires:** retired Grok keys (§7.3.7); forbidden positioning phrase extended to new commands; path-consistency (rule dir ↔ `tool` field); embedded-vs-on-disk rules (§6.5).

## 12. Sequencing and dependencies

```text
WP0  Grok clean-room protocol doc + lab run  ──────────────┐  (research; starts immediately,
                                                           │   longest pole, release-gating)
WP1  Declarative engine + migration (single-harness,       │
     goldens byte-identical)                               │
  └─► WP2  Harness abstraction (DiscoveryRoot, descriptors,│
           parse_json, version param., dispatch, CLI,      │
           schema 1.1 bump)                                │
        ├─► WP3a Codex rule expansion (fresh retrieval)    │
        ├─► WP3b Claude Code rules (fresh retrieval)       │
        └─► WP3c Grok Build rules  ◄───────────────────────┘
              └─► WP5 Freshness extension (last-seen + release-watch entry,
                      url-hashes; lands with WP3c, stays off)
WP4  Agent surface (capabilities + agent-guide)   — after WP2, parallel to WP3*
WP6  Release mechanics (version, CHANGELOG, doc fixes, checklist)
     — last; tag gated on WP0/WP3c completion and explicit owner authorization
```

WP1 before WP2: the engine migrates against the frozen single-harness goldens, so engine regressions and abstraction regressions can never mask each other. WP3a/3b/3c are independent once WP2 lands. WP0 is pure research and runs concurrently from day one — it gates only WP3c and the tag.

## 13. Acceptance criteria (release review gate)

1. Engine: `codex-history-persist-01` evaluated purely from rule data; migration byte-identical on all goldens; `evaluate.rs` bespoke logic deleted; §6.3 validator checks each proven by a negative test; §6.7 hostile corpus green.
2. Abstraction: `scan`, `list`, `explain`, `capabilities` operate over all three harnesses from synthetic fixtures; core still compiles with the env/process/net clippy bans; JSON parsing passes the §11.2 hostile matrix with value-free diagnostics.
3. Rules: every shipped rule schema-1.1-valid with fresh `retrieved` dates, semantic hashes, and archive anchors from authoring-time retrieval; Codex ≥3 rules total; Claude Code and Grok Build each meet the §7 parity definition; every non-unknown outcome fixture-tested per branch.
4. Grok gate: the clean-room protocol document exists, a dated run against the then-current version is recorded under `docs/research/evidence/grok-build/`, Grok rules cite it with a pinned version, and the retired-keys tripwire test is green.
5. Conservative degradation pinned per harness: version-out-of-range and unknown-version fixtures yield `stale-ruleset`; unrecognized/unreadable/unparseable yield `unknown`; nothing renders outside derivable renderings (property test).
6. Agent surface: `capabilities --json` validates against its schema and agrees with loaded rules; `docs/agent-guide.md` exists and makes no cadence claims.
7. Freshness: `last-seen.json` + `release-watch.yml` carry the Grok Build entry (channel per WP0), Copilot retained; workflows verifiably still gated off; `actionlint` green.
8. Release: workspace version `0.0.1` everywhere (tests updated); CHANGELOG present; CONTEXT.md visibility lines corrected; AGENTS.md store list generalized; all §10.4 gates green at the release commit; tag/Release executed **only** on explicit owner authorization.
9. Positioning: the binding phrase in all new user-facing text; "AI agent security scanner" nowhere (test-pinned, extended to new commands).
10. `notes/session-history.md` appended; unresolved risks recorded in the handoff.

## 14. Resolved decision log

- **(a) Field naming:** keep `tool` as the schema/CLI field; ids `codex | claude-code | grok-build`; "harness" stays prose-only. §3.
- **(b) DiscoveryRoot shape:** one flat struct with an explicit home per harness + `path_dirs`, not a map — closed set, exhaustive matches, CLI-only construction preserved. §5.1.
- **(c) Schema versioning:** rule + report bump to `1.1` in one migration (no release exists; silent redefinition of `1.0` rejected because the repo is public); loader accepts exactly `1.1`. §5.6.
- **(d) Integer rendering:** render only the decimal re-serialization of the parsed `i64` within schema-declared `integer_bounds`; `allowed_render` string-allowlisting stays authoritative for enums/bools. Echoing source text is never possible by construction. §5.7.
- **(e) Unset semantics:** `unset` ⇒ `unknown`, validator-enforced while only user scope is inspected — codifies the implemented (reviewed) position over the thin-slice spec's earlier unset⇒finding sketch. §6.2.
- **(f) Outcome dispatch:** overlap-free + exhaustive at load time rather than first-match-wins at runtime — determinism proven, not promised. §6.3.
- **(g) Rule embedding:** `build.rs` glob embedding + on-disk/embedded 1:1 test replaces the hand-maintained `include_str!` vec. §6.5.
- **(h) Account-state rules:** deferred; auth/plan dimensions carried as limitations + `verify_url` on concrete rules. §7.4.
- **(i) Grok parity:** parity = rigor + category coverage backed by fresh evidence with `unknown` for the locally unknowable, never claim-count symmetry; clean-room run release-gates the tag. §7, §7.3.
- **(j) Per-tool version aggregates:** minimum-of-maxes and earliest date (conservative both ways), fixing the current `rules.first()` shortcut before it can mislead with >1 rule. §5.5.
- **(k) Third-tool supersession:** Grok Build replaces GitHub Copilot CLI per the 2026-07-16 owner decision; Copilot freshness tracking retained; superseded strategy docs get a dated correction note, not a rewrite. §1, §10.3.
