# Research & Design Decisions Log

Record important choices here so future sessions understand the rationale.

## 2026-07-13

- Decided to keep research as a set of high-quality Markdown + machine-readable JSON/YAML rather than a database initially. This makes it easy for both humans and AI sessions to consume.
- Chose Tauri + Rust as the recommended architecture in the recommendations (strong security and native FS advantages for an auditing tool).
- Structured data (`data/tools-comparison.json`) is intentionally simple and flat enough to be directly useful for an early scanner.
- Explicitly separated "vendor claims" from "observed behavior" (especially important for Grok Build).
- Will maintain a living report rather than treating it as frozen.

## 2026-07-13 — Verification and product decision

- Reversed the instruction to treat the original report and flat JSON/YAML as ground truth. They are now explicitly legacy research artifacts pending rule-by-rule verification.
- Decided to proceed conditionally with a read-only, cross-platform CLI/core for Claude Code, Codex CLI, and Copilot CLI. The GUI and broad coverage are validation-gated.
- Defined the product's differentiator as source-cited privacy/configuration posture with explicit unknowns, no detected-tool execution, no session/code ingestion, and no scan-time network.
- Separated inference transfer, retention, training, telemetry, feedback, remote synchronization/sharing, permissions, sandbox, and network access in the future evidence model.
- Rejected a false-precision numeric risk score for the initial product; use concrete findings, severities, confidence, evidence class, limitations, and ordered actions.
- Rejected $0.99 Mac App Store distribution as the initial model because sandboxed filesystem access conflicts with frictionless discovery, it excludes Linux/Windows, and it cannot fund continuous evidence maintenance.
- Recommended a free Apache-2.0 core, direct signed/notarized macOS distribution if a GUI is later justified, and native/package-manager distribution across platforms.
- Recommended using the project as transparent proof of work on `benjsmin.com`, with a fixed-scope team AI-coding posture review as the first commercial offer. Team/fleet product work requires repeated demand or paid pilots.
- Safety invariants: no network in scans, no arbitrary execution, no secrets/source/transcript reads in the initial product, sanitized reports, synthetic fixtures, and signed/versioned rules.
- Established phased decision gates and an immediate thin vertical slice rather than authorization for a full application.

## 2026-07-14 — Competition, maintainability, and UX constraints

- Locked positioning: "local, execution-free, per-finding-cited config auditor for the privacy/retention/telemetry surface of Claude Code, Codex CLI, and Copilot CLI" — never "AI agent security scanner" (crowded, funded category we lose on resources).
- No public verification-cadence claim ("verified monthly") until the automated freshness pipeline (release-watch, doc-drift detection with Wayback snapshot anchoring, fixture tripwires) is operational. Automation surfaces change candidates; only humans certify verdicts.
- Rule schema: `source_url` + `retrieved_date` structurally required for any non-`unknown` status; explicit tested version ranges (MDN `≤` convention allowed); `unknown` implemented as absence of a matching version-range entry with an explicit, fixture-tested, conservative degradation default (precedent for the failure mode: vercel/next.js#92091).
- Data/core separation (separate, permissively-licensed `rules/`) is a hard invariant — forkability insurance (Wappalyzer vs. CRXcavator precedent).
- Contribution intake starts as structured issue templates, not PRs; schema-validated data PRs only once fixtures allow contributor self-service.
- UX: `scan`/`list`/`explain`/`version` command surface; 0/1/2 exit codes (ruff convention); first-class `status` enum incl. `unknown` and `stale-ruleset`; no numeric aggregate score (reconfirmed — OpenSSF Scorecard's aggregate is its most-criticized feature); SARIF deferred past v1; Rust stack: clap v4 + anstream/owo-colors + serde + directories + time, miette for parse errors only.
- Adopted differentiator candidate: user-visible per-claim "verified as of DATE" staleness badges (unclaimed territory across all studied analogs).
- Full rationale: `docs/research/synthesis-2026-07-14.md`.

## Open Questions / Areas for Future Refresh

- Reproduce current Grok Build behavior per exact version/account; the legacy telemetry/upload keys are absent from current official settings documentation.
- Monitor whether Cursor adds more client-side-only options or better local indexing.
- Track status of Continue forks post-acquisition.
- Watch for new entrants (new agents/tools) in the space.
- Validate non-executing version detection and exact configuration precedence on clean macOS, Linux, and Windows installations for the first three tools.
- Test whether individuals find enough recurring value and whether teams will pay for a posture review or centralized policy layer.
