# High-Level Production Implementation Plan

**Date:** 2026-07-13  
**Status:** Approved direction; implementation remains gated by evidence and user validation.  
**Product definition:** Local-only, read-only, source-cited posture auditor for AI coding tools.

## Production principles

1. **Bound every claim.** A scan reports local evidence, documented vendor behavior, or an unknown state—not a guess about live traffic.
2. **The scanner must be safer than the tools it audits.** No harness/plugin execution, no transcript ingestion, no secrets, and no network during scans.
3. **Rules are product code.** They require schemas, versions, review, fixtures, signatures, and freshness policies.
4. **One reliable vertical slice beats nominal support for eleven tools.** A tool is “supported” only when its important config layers and limitations are tested.
5. **CLI and library first.** Add a desktop shell after validating utility and UX needs.

## Target architecture

Use one Rust workspace with deliberately small boundaries:

```text
harness-guard/
├── crates/
│   ├── harness-guard-core/      # discovery, safe reads, parsing, evaluation
│   ├── harness-guard-rules/     # schemas, bundled rules, signature verification
│   └── harness-guard-cli/       # terminal and JSON presentation
├── rules/                       # source-cited versioned rule definitions
├── fixtures/                    # synthetic per-tool/per-OS configs and expected findings
├── docs/                        # methodology, threat model, support matrix
└── xtask/                       # deterministic maintainer/release tasks, if needed
```

Keep network/update code outside the scan core. A later Tauri application should call the same library and consume the same normalized findings. Do not add SQLite until a validated feature needs history or queries; a one-shot scan can remain stateless.

### Core domain types

- `ToolInstallation`: tool ID, detected version, platform, paths, and detection confidence.
- `ConfigLayer`: scope, path alias, precedence, readable state, and safe parsed values.
- `Rule`: applicability, observation, evidence, interpretation, remediation, and freshness.
- `Finding`: stable ID, status (`pass`, `action`, `review`, `unknown`, `unsupported`), severity, confidence, explanation, evidence references, and redacted observations.
- `Source`: URL, publisher, evidence class, retrieved date, content hash, and validity metadata.
- `ScanReport`: scanner/rules version, platform, supported tools, findings, limitations, and no raw configuration.

Avoid a single numeric risk score. Provide an ordered action list and posture counts. If an aggregate is added later, it must remain decomposable and must not imply mathematical certainty.

## Phase 0 — Evidence foundation

**Goal:** Make research safe for code consumption.

Work:

- Define JSON Schemas (or equivalent Rust/schema pair) for sources, rules, fixtures, and reports.
- Define evidence classes, confidence rubric, staleness policy, and source-review checklist.
- Replace the current flat comparison JSON and shell-command YAML with versioned rules; keep the legacy files quarantined until migrated.
- Build primary-source rule inventories for Claude Code, Codex CLI, and Copilot CLI.
- Record config precedence, platform paths, version range, plan/auth differences, and locally unknowable settings for each tool.
- Create only synthetic fixtures—never commit real user configuration, sessions, usernames, tokens, or paths.
- Decide license, contribution policy, security policy, and responsible-disclosure address.

Exit criteria:

- Every material rule has a primary official source and retrieval date.
- Independent behavior claims are labelled and reproducible; they never override current local observation without version applicability.
- Fixtures cover missing, minimal, hardened, risky, malformed, unknown-version, and managed-override cases.
- A review can trace every output string to a rule and every rule to evidence.

## Phase 1 — Three-tool CLI vertical slice

**Goal:** Give one developer a useful, safe result in under two minutes.

Suggested commands:

```text
harness-guard scan [--tool <id>] [--path <path>] [--format terminal|json]
harness-guard explain <finding-id>
harness-guard doctor
harness-guard rules status
```

Work:

- Implement platform-aware path expansion without leaking absolute paths into output.
- Detect versions without executing the harness where possible: package metadata, known manifests, or explicit user input. Mark version unknown when necessary.
- Parse JSON/JSONC and TOML with bounded file sizes and graceful error findings.
- Resolve documented config precedence separately per tool; do not create a universal precedence abstraction that erases differences.
- Implement the first high-value checks:
  - Claude Code: local transcript retention, telemetry/error-reporting/feedback controls, nonessential traffic, permission policy, managed/project/user layer awareness.
  - Codex CLI: history persistence, analytics, feedback, OTEL prompt logging, sandbox/approval/network settings, trusted-project limits, and auth-method-dependent policy reminder.
  - Copilot CLI: remote session export, offline/telemetry mode, stored tool permissions, and unknown organization/account policy.
- Render concise actions, explanations, exact limitations, and clickable official sources.
- Emit stable, sanitized JSON for integrations. Do not expose parsed config values outside a small allowlist of booleans/enums.

Exit criteria:

- Clean install and first scan are documented for macOS, Linux, and Windows.
- Scan performs no DNS/network operation in an instrumented test.
- The scanner never runs a discovered executable.
- Golden fixture output is stable and contains no home directory, username, source text, token-like value, or raw config fragment.
- Supported malformed/hostile inputs cannot crash, hang, traverse unintended paths, or consume unbounded memory.

## Phase 2 — Security hardening and user validation

**Goal:** Decide whether the product deserves expansion.

Engineering work:

- Write and review a threat model covering hostile configs, symlinks, race conditions, path traversal, oversized files, terminal escape injection, poisoned rules/updates, malicious source URLs, and report deanonymization.
- Add property/fuzz tests for config parsers, rule evaluation, redaction, path normalization, and report rendering.
- Test permissions errors, concurrent writes, network-mounted homes, WSL, containers, and unsupported future versions.
- Add dependency auditing, license checks, SBOM generation, secret scanning of the Harness Guard repository, and pinned release automation.
- Obtain an external security review before a “production-ready” claim.

Validation work:

- Observe 15–20 developers across individual, freelance, and team-lead profiles installing and using the alpha.
- Ask users to explain findings in their own words; specifically test whether `unknown` and “does not prove transmission” are understood.
- Measure time to first result, failed discovery, false-action findings, newly learned/actionable facts, and whether users would run it again after tool updates.
- Interview 5–8 engineering/security leads about policy and evidence needs without pitching a preselected paid dashboard.

Go criterion:

- At least 60% discover an actionable or previously unclear posture fact.
- At least 80% understand the static-scan limitation.
- No raw-data egress, secret disclosure, or critical false positive in the supported fixture/version matrix.
- A clear repeated-use trigger emerges: new tool install, tool update, client onboarding, policy check, or CI.

If those conditions fail, publish the research/checklist and stop expanding the application.

## Phase 3 — Coverage and evidence updates

**Goal:** Expand only after the first slice proves valuable.

Order:

1. Gemini CLI.
2. OpenCode.
3. Cursor with explicit local/remote knowledge boundaries.
4. Grok Build as version-specific advisory/experimental coverage.

Work:

- Add signed rules bundles with an offline bundled baseline and explicit user-initiated updates.
- Verify bundle signatures and schema compatibility before activation; keep the previous valid bundle for rollback.
- Add a human-readable support matrix: tool version, OS, checked settings, unchecked settings, last verified date.
- Run scheduled CI for link health and upstream version detection, but require human review for semantic rule changes.
- Add community contribution templates that demand source, version, fixture, expected finding, and limitations.
- Consider SARIF only if team/CI interviews show demand; JSON remains the stable base format.

Do not add automatic fixes in this phase unless read-only validation shows that instructions are the main remaining user failure.

## Phase 4 — UX shell and safe remediation

**Entry condition:** User research shows a GUI solves an observed accessibility, comprehension, comparison, or history problem.

Desktop work:

- Build a minimal Tauri shell over the same core.
- Make `Scan` and the ordered actions primary; avoid a dashboard full of decorative scores.
- Display source, date, applicable version, confidence, and limitation beside each finding.
- Make path access transparent and user-initiated where OS permissions require it.
- Internationalize from stable IDs; launch English, then French.
- Distribute a signed/notarized macOS build directly, plus appropriate Windows/Linux packages. Reassess App Store feasibility only after the filesystem UX is proven.

Remediation work, if justified:

- Preview minimal semantic and textual diff.
- Check file identity/content has not changed since scan.
- Preserve owner, permissions, line endings, and unrelated formatting where possible.
- Write to a sibling temporary file, fsync, atomically replace, and retain a clearly named rollback backup.
- Never change managed/org settings, execute shell snippets, or claim success without rereading and reevaluating.

## Phase 5 — Sustainable commercial layer

**Entry condition:** At least three teams request materially similar centralized capability or a paid pilot.

Potential paid work:

- Fixed-scope AI coding posture audits and rollout consulting.
- Organization-authored signed baselines and private rules.
- Fleet/CI policy evaluation using sanitized findings.
- Evidence retention, change history, exceptions, and export for internal reviews.
- Support for custom/internal harnesses.

Keep local single-user scanning, core rules, and report verification open. Do not require an account or silently add telemetry to the community product.

## Verification matrix

Minimum automated coverage before a stable release:

| Layer | Required checks |
|---|---|
| Schema/rules | Valid/invalid corpus, compatibility, signatures, source fields, expired versions |
| Discovery | Per-OS fixtures, custom home/config paths, missing access, multiple installs, symlinks |
| Parsing | JSON, JSONC, TOML, malformed/oversized/hostile input, unknown fields preserved safely |
| Evaluation | Precedence, conflicting layers, version bounds, plan/auth prerequisites, unknown states |
| Privacy | No-network test, redaction corpus, terminal escaping, sanitized JSON snapshots |
| Robustness | Fuzz/property tests, race simulation, interruption, resource caps |
| Release | Cross-platform CI, checksums, signed artifacts, SBOM, clean-machine installation |
| UX | Comprehension tests, accessibility, localization placeholders, remediation clarity |

## Definition of production-ready

Harness Guard is production-ready only when:

- Its supported-version matrix is explicit and current.
- Every material finding is source-cited and fixture-tested.
- Stale/unknown behavior degrades to `unknown`, never confident advice.
- Normal scans read no source code or transcript content, execute nothing discovered, and make no network requests.
- Reports are demonstrably sanitized.
- Threat model, security policy, release provenance, rollback process, and update-signing design are public.
- All three major desktop operating systems pass the declared support matrix.
- External review found no unresolved high-severity issue.
- Real users have completed the core task and understood its limitations.

## Immediate next work package

The next focused implementation effort should stop at Phase 0 plus one thin end-to-end rule:

1. Create the Rust workspace and rule/report schemas.
2. Implement safe discovery and parsing for one Codex CLI config fixture.
3. Evaluate one high-confidence finding such as history persistence.
4. Render terminal and sanitized JSON output with source and limitation.
5. Prove no egress and add hostile-input tests.

Review that slice before multiplying rules or tools. It will expose whether the evidence schema, output contract, and safety boundaries are sound while the cost of changing them is still low.
