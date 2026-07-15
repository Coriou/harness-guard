# Research Synthesis — 2026-07-14 (competition, maintainability, UX)

**Status:** Decision-grade synthesis of three parallel research streams run 2026-07-14. Full reports:

- [competitive-landscape-2026-07-14.md](./competitive-landscape-2026-07-14.md)
- [maintainability-strategy-2026-07-14.md](./maintainability-strategy-2026-07-14.md)
- [cli-ux-research-2026-07-14.md](./cli-ux-research-2026-07-14.md)

This session asked three go/no-go questions before committing to the v1 spec/plan work. All three came back **conditional-positive** with concrete constraints. Nothing found reverses the 2026-07-13 decision to proceed.

## 1. Competition — the niche is real, narrow, and time-boxed

- **The exact niche is unoccupied but not a blue ocean.** No tool combines: local-only, read-only, zero execution, zero scan-time network, explicit `unknown` labeling, per-finding primary-source citation, privacy/retention/telemetry/training/sync coverage, across Claude Code + Codex CLI + Copilot CLI.
- **Closest analog: `claudit-sec`** (HarmonicSecurity, Apache-2.0, ~290 stars) — same safety invariants, but Claude-only, no citations, no unknown state, no telemetry/retention/training checks, and no pushes for ~2 months as of 2026-07-14. The window is open, not closed.
- **Positioning constraint (binding):** never present as an "AI agent security scanner." That category is crowded and funded (Snyk Agent Scan 2.8k stars/daily commits, Cisco, Armor1, Lakera→Check Point, Prompt Security→SentinelOne). The credible claim is: *"the only local, execution-free, per-finding-cited config auditor spanning these three harnesses' privacy/retention/telemetry surface."*
- **Top threat is incumbent feature creep**, not new entrants — Snyk or Cisco adding privacy-toggle checks is a small lift for them. Second threat: claudit-sec (or a fork) generalizing. Third: the content layer (Voibe AI Privacy Tracker — 34 tools, well-sourced, monthly-reviewed) winning top-of-funnel search before users ever consider a CLI.
- **Wedge vs. content competitors:** Voibe answers "should I trust this tool" at vendor-policy level. Harness Guard answers "what does *my machine's actual config* currently do — did I really turn that off." The benjsmin.com content must lead with that distinction.
- Vendor built-ins (`claude /doctor`, `codex doctor`, GitHub's 2026-05-18 Copilot cloud-agent config-audit API) are setup-health/org-fleet tools today, but vendors are visibly inching toward first-party posture surfaces — a long-run commoditization signal, mostly for the fleet tier.

## 2. Maintainability — solo is viable only with the freshness pipeline built first

- **Verdict: monthly verification of 3 tools solo is realistic only as "review automation-flagged diffs," never as "manually re-derive claims."** Maintainer-burnout data (Tidelift: 58% quit/considered quitting; hobbyist ceiling ≈ 9 hrs/week) rules out the manual version.
- **Binding constraint: do not publicly claim any verification cadence ("verified monthly") before the automated freshness pipeline ships.** Making that claim without the pipeline would repeat the overclaim failure the 2026-07-13 verification audit already caught once.
- **Pipeline shape** (all free-tier on public-repo GitHub Actions; automation surfaces candidates, humans certify — the bot never sets a verdict, per endoflife.date/MDN-BCD practice):
  1. Release-watch: daily/weekly cron on npm registry dist-tags — `@anthropic-ai/claude-code` → `stable`; `@openai/codex` → `latest` (ignore its ~16 other dist-tags); `@github/copilot` → `latest`. Diff against a committed last-seen JSON; on change, open a triage issue linking the changelog diff. GitHub `releases.atom` feeds as secondary signal (filter `-alpha.` for Codex).
  2. Doc-drift: weekly cron over the ~30–50 cited vendor URLs — `lychee` for dead links, then semantic-text content-hash (strip nav/timestamps before hashing); on drift, fire a Wayback Machine SPN2 snapshot and open a triage issue linking old + new snapshots as evidence anchors.
  3. Fixture/golden tests as the second staleness tripwire (OSSF Scorecard pattern): a rule silently failing to match real config shape is stronger drift evidence than a doc-hash change.
  4. Runbook item: GitHub scheduled workflows auto-disable after 60 days without repo activity — needs routine commit cadence or a documented re-enable step.
- **Rule schema constraints to bake into Phase 0:**
  - `source_url` + `retrieved_date` **structurally required** whenever a rule's status is anything other than `unknown` (MDN's mandatory-`spec_url` pattern) — the schema, not review discipline, blocks the next Grok-keys-style stale claim.
  - Explicit tested version ranges per rule; MDN's `≤`-prefix convention for "confirmed by this version, possibly earlier."
  - `unknown` = absence of a matching version-range entry, not an in-band sentinel; the degradation direction must be an explicit, fixture-tested, conservative default. Precedent for getting this wrong: vercel/next.js#92091, where "unknown version" silently resolved to "assume everything supported" and inverted the safety property.
- **Data/core separation is a hard invariant, not a folder convention** — Wappalyzer's separated MIT data files survived its 2023 paywalling via forks; CRXcavator died unforkable. `rules/` stays a separate, permissively-licensed package from day one.
- **Contribution model:** during the data-curation phase, prefer a structured issue template (URL + before/after claim) over PRs (EasyList's reasoning); move to schema-validated one-file-per-rule data PRs once fixtures let contributors self-serve a passing PR. Budget *reviewer* time as the real cost (Exodus/ETIP's lesson). Two-tier trust needs exactly one trusted second reviewer to bootstrap.
- **Succession/security notes:** domain, npm publish rights, and GitHub org need a documented second keyholder early (privacytools.io precedent); any future co-maintainer handoff is a security decision (XZ precedent). Honest public retirement (Ingress NGINX model) is an acceptable documented end-state and already fits the phase-gated plan.
- **Unclaimed differentiator:** none of the studied projects (including ToS;DR and Privacy Guides) show users a per-claim "verified as of DATE / unverified past N days" staleness badge. Cheap to add once the pipeline's last-checked timestamps exist, and it operationalizes the "explicit about limitations" invariant visibly.

## 3. UX — the trust rendering is the product

- **v1 command surface:** `scan` (default: everything detected; `--tool`, `--json`, `--min-severity`, `--fail-on`, color flags), `list` (detection only), `explain <rule-id>` (full evidence record, rustc `--explain` pattern), `version` (binary and ruleset versions reported separately). Deliberately excluded: `--fix` (read-only invariant) and any implicit-network `rules update`.
- **Exit codes:** ruff/ESLint 3-way convention — 0 clean / 1 findings at-or-above `--fail-on` / 2 tool error. No extra exit code for stale ruleset; that's a `stale-ruleset` status in output. `info` and `unknown` never fail a scan by default.
- **Output shape:** group by tool → rule category (Semgrep pattern); tri-state-plus glyphs (pass / info / warn / `??` unknown, flutter-doctor style); cargo-audit-style compact finding block (rule ID, plain-English why-it-matters, copy-pasteable remediation, source URL + retrieved date in ~6 lines); per-tool version-bound banner ("rules verified ≤2.5.x on 2026-07-01 — you have 2.6.0, shown unverified") as a hint-style line, not an error.
- **The load-bearing trust move:** `unknown` rendered as a first-class status with the reason spelled out ("this is account/server state; no local file can confirm it — that's what local-only means") plus an official verify-here link. In JSON, `status` is a first-class enum (`finding | unknown | pass | stale-ruleset`) so severity/confidence are legitimately null for unknowns.
- **SARIF: deferred past v1** — it has no honest vocabulary for "unknown, verify manually" and v1's user is a terminal, not CI. Ship a versioned native JSON schema first.
- **Rust stack:** clap v4 (derive) + anstream/owo-colors (+ colorchoice-clap) + serde/serde_json (same structs render terminal and JSON so views can't drift) + comfy-table + `directories` + `time`; miette only for config-parse failures; clap_complete for polish. Verify current cargo-audit flag spelling before finalizing flags (flagged unverified by the researcher).
- **Anti-patterns with real-world evidence:** npm audit's trust collapse came from confidence inflation — never display more confidence than the evidence class supports; OpenSSF Scorecard's aggregate score is its most-criticized feature — validates the no-numeric-score invariant; never let "we don't check this" and "verified safe" look the same (silent omission reads as a pass).

## Implications for the implementation plan

1. **Phase 0 additions:** rule schema gains structurally-required `source_url`/`retrieved_date` (non-unknown statuses), explicit version ranges with `≤` convention, first-class `status` enum, and a fixture-tested conservative unknown-degradation default.
2. **Freshness pipeline becomes a gated deliverable** (release-watch + doc-drift + fixture tripwires), scope-blocking for any public verification-cadence claim — not a post-launch enhancement.
3. **Output/UX layer adopts the mockups** in the UX report as the starting spec for the thin vertical slice's terminal + JSON rendering.
4. **Positioning language** ("config auditor for privacy posture," never "agent security scanner") flows into README, docs, and the benjsmin.com content plan; the Voibe distinction (policy-level vs. your-actual-config) anchors the launch article.
5. **Timing:** claudit-sec's stall and incumbents' daily shipping cadence argue for moving to the v1 spec/plan promptly rather than extending research.

## Recommendation

Proceed to the brainstorm → spec → plan pipeline for the v1 thin slice (Phase 0 + one Codex rule end-to-end, per CONTEXT.md), with the three reports above as required inputs to the Decision Pack.
