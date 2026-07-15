# CLI UX Research — 2026-07-14

**Status:** Research artifact (retrieved 2026-07-14). Not application ground truth; mockups are design proposals, not committed spec.
**Scope:** Audit/doctor CLI patterns, output mockups (human + JSON + unknown/citation rendering), v1 command surface, exit codes, SARIF decision, Rust crate stack, anti-patterns.

## Patterns worth stealing

**flutter doctor** — Three-symbol status vocabulary (✓ pass / ! possible issue / ✗ failure) plus a final "Doctor summary" rollup line. Steal: the tri-state (not binary) status glyph set, and running each check independently so one failure doesn't abort the rest — maps directly onto "known / unknown / needs attention" instead of forcing pass/fail.

**brew doctor** — Binary framing: either "Your system is ready to brew" or a flat list of `Warning:` blocks, each with a one-paragraph explanation of *why* it matters, no severity levels at all. Steal: the plain-English "why this matters" paragraph under each warning, and the satisfying single-line all-clear message for the zero-findings case (don't make "nothing wrong" look like an empty/broken screen).

**npm audit** — Cautionary tale, not a pattern to copy. ~80% false-positive rate on typical projects because it uses worst-case CVSS with no reachability/exploitability context, which trained a generation of developers to reflexively ignore audit output (65% of teams admit bypassing/delaying fixes due to noise). Directly relevant: Harness Guard's "unknown" and "confidence" fields exist specifically to avoid this trap — never assert a finding with more confidence than the evidence class supports.

**Trivy** — Two-part output: one-line summary of counts by severity, then an itemized table (CVE, severity, installed/fixed version, title). Explicitly does not truncate to 80 cols. Its own docs/community acknowledge the default is "overwhelming" and that `--severity` filtering is load-bearing for usability, not optional polish. Steal: summary-then-detail structure. Anti-pattern to avoid: shipping the noisy default as the *only* mode — filtering/threshold flags must be first-class, not an afterthought.

**Grype** — Same two-part shape as Trivy (artifact summary + CVE table), but adds an EPSS exploitability score/percentile alongside raw severity, which is a real answer to severity-inflation criticism (severity ≠ priority). Steal: pairing "how bad in theory" with "how likely to matter" as two distinct columns rather than collapsing to one severity/score.

**Semgrep** — Groups findings by rule by default (not by file), sorted by severity, with an option to regroup by count. Also segments by product/exposure type (blocking vs reachable vs informational). Steal: grouping findings by *rule* (i.e., by root cause) rather than by raw occurrence list — this is exactly right for Harness Guard, which should group by tool then by rule-category (retention/telemetry/permissions/sync/etc.), not dump a flat finding list.

**gitleaks** — Per-finding record is a clean citation model: Finding, RuleID, redacted Secret preview, File, Line, Fingerprint. `--redact[=N]` redacts a *configurable percentage*, leaving enough visible for triage without exposing the full secret. Steal directly: the redact-to-a-percentage pattern is a strong precedent for Harness Guard's "never print raw config values" rule — show a normalized/truncated representation, not the raw value, by default.

**OpenSSF Scorecard** — Uses a 0–10 numeric score per check plus an aggregate — i.e., exactly the false-precision pattern the product strategy explicitly rejects. No letter-grade system found in the docs (that's a misconception, not a real Scorecard feature). Useful as a **negative** reference: Scorecard's aggregate score is the most-criticized part of the tool in practice (it invites "chase the number" behavior disconnected from real risk). Do not imitate the aggregate-score piece; the per-check breakdown (before aggregation) is the useful part.

**gh CLI** — Minimal-flag, context-aware design (infers repo/branch from cwd) and a predictable `noun verb` command grammar (`gh pr create`, `gh issue list`). Standard 0-success/non-zero-failure exit codes, documented per-command. Steal: the `noun verb` grammar and leading-with-context-not-flags philosophy for the command surface (`harness-guard scan`, `harness-guard explain <rule-id>`, not `harness-guard --explain`).

**ruff / ESLint / RuboCop convention family** — Consistent 3-way exit code convention: 0 = clean, 1 = findings present (only when a fail-mode flag like `--exit-non-zero-on-fix` is engaged), 2 = tool itself errored (bad config/internal error). This 3-way split (success / findings / tool-error) is the dominant convention across the modern lint/audit tool generation — worth matching exactly since it's what CI authors already expect.

**cargo-audit** — Best precedent found for source citation. Each finding renders as a compact record: `Crate / Version / Title / Date / ID (RUSTSEC-YYYY-NNNN) / URL / Solution`, i.e., a stable rule ID, a resolvable canonical URL, a date, and a one-line remediation, all in ~6 lines — no wall of prose. This is close to a template for a Harness Guard finding block.

**clig.dev core principles directly applicable**: "humans first, machines second" (terminal output optimized for reading, `--json` for machines, never the reverse); color used "with intention" and disabled on non-TTY / `NO_COLOR` / `TERM=dumb` / `--no-color`; catch-and-rewrite errors instead of leaking raw stack/parse errors; "responsive is more important than fast" (stream findings as discovered rather than a single spinner-then-dump); exit codes must be documented and stable since scripts depend on them. Full guide: https://clig.dev/

## Output design recommendation — 3 mockups

**1. Default human view** (`harness-guard scan`)

```
Harness Guard v0.3.0 · ruleset 2026.07.01 · scanned 2026-07-14 09:41 local · no network requests made

Detected 2 of 3 supported tools

● Claude Code   v2.4.1   installed
● Codex CLI     v1.8.0   installed
○ Copilot CLI   —        not detected

──────────────────────────────────────────────────────────────────────
Claude Code  v2.4.1                              rules verified ≤2.5.x · 2026-07-01
──────────────────────────────────────────────────────────────────────
  Needs attention · 1 warning · 2 info · 1 unknown · 4 passed

  WARN  session history retained locally, no expiry
        rule cc-history-persist-01 · local-observation

        ~/.claude/history.jsonl exists, no `historyTTL` configured

        Set a retention window:
          $ claude config set historyTTL 30d

        = source: docs.claude.com/en/docs/claude-code/settings (2026-07-01)
        = harness-guard explain cc-history-persist-01   [full evidence]

  ??    UNKNOWN  whether this session syncs to a team workspace
        rule cc-team-sync-02 · requires account/server state (not locally observable)

        Local config doesn't indicate workspace membership either way.
        Verify at: https://claude.com/settings/data-controls  (sign-in required)

  INFO  MCP servers configured: 2 (filesystem, github)
        rule cc-mcp-inventory-01 · local-observation
        = source: docs.claude.com/en/docs/claude-code/mcp (2026-07-01)

──────────────────────────────────────────────────────────────────────
Codex CLI  v1.8.0     ⚠ rules verified ≤1.6.x — you have 1.8.0, showing last-known rules as unverified
──────────────────────────────────────────────────────────────────────
  [... same structure ...]

──────────────────────────────────────────────────────────────────────
2 tools scanned · 1 warning · 2 info · 1 unknown · 0 network requests made
No numeric score is produced — read findings individually. See harness-guard docs/why-no-score
```

Design notes: findings are grouped **by tool, then by rule/category** (Semgrep's grouping pattern), each block opens with a plain-word status line (brew doctor's tone) plus counts by kind (Trivy's summary-then-detail shape), never a score. The unknown finding uses a distinct `??` glyph and explicitly names *why* it can't be known (account/server state) rather than looking like a broken or skipped check. The citation line is compact (source + date, cargo-audit style) with an `explain` pointer for the full evidence record — this solves "citation without drowning the output." Version-boundedness is a one-line banner per tool section, not a blocking error, styled like a `git hint:`/`gh` update-nag line — informative, not alarming.

**2. `--json` sketch** (schema mirrors the reliability-model fields already specified in decision-and-strategy.md):

```json
{
  "schema_version": "1.0",
  "harness_guard_version": "0.3.0",
  "ruleset_version": "2026.07.01",
  "scanned_at": "2026-07-14T09:41:00-07:00",
  "network_requests_made": 0,
  "tools": [
    {
      "tool": "claude-code",
      "detected_version": "2.4.1",
      "rules_last_verified_version": "2.5.x",
      "rules_verified_date": "2026-07-01",
      "version_in_range": true,
      "findings": [
        {
          "rule_id": "cc-history-persist-01",
          "severity": "warning",
          "confidence": "high",
          "evidence_class": "local-observation",
          "status": "finding",
          "observation": "history.jsonl present, historyTTL unset",
          "message": "Session history retained locally with no expiry",
          "remediation": {
            "summary": "Set a retention window",
            "command": "claude config set historyTTL 30d"
          },
          "source": {
            "url": "https://docs.claude.com/en/docs/claude-code/settings",
            "retrieved": "2026-07-01"
          },
          "valid_from": "2.0.0",
          "valid_until": null,
          "limitations": "Cannot confirm remote copy retention."
        },
        {
          "rule_id": "cc-team-sync-02",
          "severity": null,
          "confidence": null,
          "evidence_class": "inference",
          "status": "unknown",
          "message": "Cannot determine whether sessions sync to a team workspace",
          "unknown_reason": "account/server state not observable from local files",
          "verify_url": "https://claude.com/settings/data-controls"
        }
      ]
    }
  ],
  "summary": { "tools_scanned": 2, "warning": 1, "info": 2, "unknown": 1, "passed": 4 }
}
```
Key design choice: `status` is a first-class enum (`finding | unknown | pass | stale-ruleset`) so `severity`/`confidence` are legitimately nullable for unknowns instead of forcing a fake severity onto something unverifiable — this is the JSON-level guarantee that backs the terminal `??` glyph.

**3. Unknown + citation close-up** (what makes "unknown" read as honest, not broken):

```
  ??  UNKNOWN   whether this session syncs to a team workspace
      rule cc-team-sync-02

      This is account/server state. No local file can confirm it either way —
      that's not a limitation of this scan, it's what "local-only" means.

      → Verify at: claude.com/settings/data-controls (sign-in required)
```
The explanatory sentence ("that's not a limitation of this scan, it's what local-only means") is the load-bearing line — it reframes "unknown" as an honest boundary of the tool's design rather than a bug or missing feature, which is the single biggest trust risk called out in the product strategy doc.

## Flags/commands surface — minimal v1

- `harness-guard scan [--tool <name>]... [--json] [--no-color|--color=auto|always|never] [--min-severity <level>] [--fail-on <level>] [--quiet] [--verbose]` — primary command, safe default (no args = scan everything detected)
- `harness-guard list` — detection-only pass (which supported tools are installed + version), no rule evaluation; cheap and fast, useful for `harness-guard list && harness-guard scan` scripting
- `harness-guard explain <rule-id>` — full evidence record for one rule: source URL, retrieved date, evidence class, valid version range, limitations, unknown-condition (compiler/clippy `--explain`/"for further information visit ..." pattern)
- `harness-guard version` / `--version` — reports both binary version and bundled ruleset version/date separately (these should be allowed to diverge once rules are independently updatable, per the decision doc)
- `--help` on every subcommand, examples-first (clig.dev guidance)

Deliberately excluded from v1: no `--fix`/auto-remediate flag (read-only invariant), no `rules update` network command yet (would violate "no network requests" default — could exist later as an explicit opt-in subcommand, never implied by `scan`).

## Exit codes + machine formats

Exit codes — adopt the ruff/ESLint/RuboCop 3-way convention exactly, since it's what CI authors already assume:
- `0` — scan completed, no findings at/above `--fail-on` threshold (default threshold: warning; info/unknown never fail by default)
- `1` — findings at/above threshold present
- `2` — the tool itself failed (permission error, malformed config it couldn't safely parse, internal error) — distinct from "found problems," matching Ruff's split

Recommend **against** a separate exit code for "ruleset stale/unverified version" — fold it into the findings/summary counts instead (as a `stale-ruleset` status, visible in output and JSON) rather than growing exit-code surface. clig.dev and the ruff precedent both favor a small, stable, documented exit-code set over one that grows a special code per condition.

SARIF — **defer past v1**, not "no forever." Reasoning: SARIF's `result` model is built around `physicalLocation` (file+region) tied to `tool.driver.rules`, which fits config-file findings reasonably well, but has no real vocabulary for "unknown, requires manual account-state verification" — the closest mappings (`level: note`, `kind: review`) risk exactly the evidence-class conflation the product strategy explicitly prohibits (treating local-observation, vendor-doc, and inference as equivalent). SARIF's primary value is CI/GitHub-code-scanning integration, and Harness Guard's v1 target user is an individual developer's terminal, not a CI pipeline — so the ROI is currently low and the risk of a sloppy mapping is real. Ship a stable, versioned native JSON schema first (the sketch above); reconsider `--sarif` once/if code-scanning integration is a validated user request, with a deliberate mapping design (stuff evidence_class/confidence into SARIF's `properties` bag; use `kind: review` for unknowns) rather than a rushed exporter.

## Rust crate stack recommendation

- **clap** (derive API, v4) — de facto standard, used by ripgrep/bat/fd; `noun verb` subcommands map cleanly to its subcommand derive.
- **anstream + owo-colors** — the maintained, TTY/`NO_COLOR`/CI-aware color pairing recommended by clap's own maintainer's CLI guide (rust-cli-recommendations.sunshowers.io); anstream handles stream detection, owo-colors handles styling. Don't hand-roll TTY detection.
- **colorchoice-clap** — small crate that wires `--color=auto|always|never` straight into clap + the anstream/owo-colors stack; matches the flag convention clig.dev recommends.
- **comfy-table** — dynamic-width table rendering for the (rare) tabular sub-views; minimalistic and safety-focused, good fit for a tool that must never crash on hostile/weird terminal widths.
- **serde + serde_json** — `--json` output; findings/report structs should derive `Serialize` directly from the same structs used for terminal rendering to guarantee the two views never drift.
- **miette** — *not* for the main findings list (it's built for single compiler-style diagnostics with a source snippet, wrong shape for a grouped findings report) — but a strong fit specifically for **config-parse failures**: when a discovered `settings.json`/TOML file is malformed, miette's `#[source_code]` span-highlighting is exactly the right way to show "here's the bad key, here's why we can't safely read this file" without dumping the raw file contents (respects the "never print config values" invariant if the snippet rendering is limited to structural context, not values).
- **directories** (or `etcetera`) — cross-platform config/cache path resolution (macOS/Linux/Windows), needed since the product must work identically across all three from day one per the strategy doc.
- **time** (not chrono, lighter and increasingly the modern default) — for `retrieved_date`/`valid_from`/`valid_until` handling with strict RFC3339 output for JSON.
- **clap_complete** — shell completion generation, cheap addition that meaningfully raises perceived polish for a dev-facing CLI.

## Anti-patterns to avoid

1. **npm audit alert fatigue** — asserting findings with more confidence than the evidence supports trains users to ignore all output. Every finding's displayed confidence must be honest per evidence_class, never inflated for impact.
2. **Wall-of-red / undifferentiated severity** (Trivy/Grype's acknowledged default-noise problem) — never let "everything is a warning" become the default; grouping + a real info/unknown tier that doesn't visually scream is required.
3. **Severity without exploitability/relevance context** — CVSS-style worst-case-only severity is the root cause of npm audit's trust collapse; Harness Guard's rules should note *why* a setting matters in context, not just tag it a color.
4. **False-precision aggregate scores** (OpenSSF Scorecard's 0–10 + aggregate, or any 0–100 "safety score") — explicitly already ruled out in decision-and-strategy.md; the research confirms this is also a live criticism of the closest real-world analog, so it's a well-founded exclusion, not just caution.
5. **Silently dropping what can't be verified** — omission reads as "we checked, it's fine," which is worse than a visible, well-explained `unknown`. Never let "we don't check this" and "verified safe" look the same.
6. **Exit-code sprawl** — adding a new exit code per condition (seen as a complaint pattern around Trivy/Grype's scan-error-vs-vulnerabilities-found ambiguity) — keep the 3-way convention and push edge conditions into the findings/summary payload instead.
7. **Emoji/color overuse diluting signal** — clig.dev's own warning ("if everything is a different color, color means nothing") — reserve red/bold exclusively for the highest severity tier actually used.
8. **Forcing `--verbose` to get essential trust info** — citations, version-bounds, and unknown-reasons must appear in default output (compact form), not be hidden behind a flag; only the *full* evidence record (`explain`) should require an extra step.

## Sources (retrieved 2026-07-14)

- [Command Line Interface Guidelines](https://clig.dev/) — core UX principles, color/NO_COLOR/exit-code conventions
- [cli-guidelines GitHub](https://github.com/cli-guidelines/cli-guidelines)
- [Trivy Reporting docs](https://trivy.dev/docs/latest/configuration/reporting/) and [Trivy Filtering docs](https://trivy.dev/docs/latest/configuration/filtering/)
- [Trivy Is Noise by Default — Medium](https://medium.com/@DynamoDevOps/trivy-is-noise-by-default-heres-the-seven-rule-filter-that-catches-real-risk-05c4c3249c26)
- [Understanding Grype results — Anchore](https://oss.anchore.com/docs/guides/vulnerability/interpreting-results/)
- [Grype Table Output — DeepWiki](https://deepwiki.com/anchore/grype/4.1-table-output)
- [npm audit: Broken by Design — overreacted.io](https://overreacted.io/npm-audit-broken-by-design/)
- [Why npm Audit Is Broken — PkgPulse](https://www.pkgpulse.com/guides/why-npm-audit-is-broken)
- [gitleaks GitHub](https://github.com/gitleaks/gitleaks) — `--redact` behavior, finding fields
- [Semgrep Output Formatting — DeepWiki](https://deepwiki.com/semgrep/semgrep/4.3-output-formatting)
- [Semgrep Managing findings docs](https://semgrep.dev/docs/managing-findings/)
- [OpenSSF Scorecard GitHub](https://github.com/ossf/scorecard) and [scorecard.dev](https://scorecard.dev/)
- [gh exit-codes manual](https://cli.github.com/manual/gh_help_exit-codes)
- [Ruff Linter docs — exit codes](https://docs.astral.sh/ruff/linter/)
- [rustsec/cargo-audit GitHub](https://github.com/rustsec/rustsec/tree/main/cargo-audit) and [RustSec advisory example](https://rustsec.org/advisories/RUSTSEC-2022-0051)
- [SARIF v2.1.0 OASIS spec](https://docs.oasis-open.org/sarif/sarif/v2.1.0/sarif-v2.1.0.html)
- [The complete guide to SARIF — Sonar](https://www.sonarsource.com/resources/library/sarif/)
- [Managing colors in Rust — rust-cli-recommendations (sunshowers/epage)](https://rust-cli-recommendations.sunshowers.io/managing-colors-in-rust.html)
- [anstream: simplifying terminal styling](https://epage.github.io/blog/2023/03/anstream-simplifying-terminal-styling/)
- [comfy-table — lib.rs](https://lib.rs/crates/comfy-table)
- [miette GitHub](https://github.com/zkat/miette) — `#[source_code]` snippet rendering
- [Homebrew brew doctor warnings — Homebrew Discourse](https://discourse.brew.sh/t/brew-doctor-warnings/581)
- Flutter Doctor status-symbol behavior — general knowledge, corroborated by [Flutter Doctor command guide](https://flutterfever.com/flutter-doctor-command/) and [flutter/flutter#20086](https://github.com/flutter/flutter/issues/20086)

Two things I could not verify from search and flagged as general-knowledge-only in the writeup above rather than citing a specific source: (1) Scorecard letter-grade badges — I found no evidence this feature exists; don't build on that assumption. (2) Exact current cargo-audit `--deny`/`--json` flag syntax — confirmed the JSON output mode and per-finding field set exist, but didn't pull the literal current flag spelling; worth a 2-minute `cargo audit --help` check before finalizing the flag surface rather than relying on this research.
