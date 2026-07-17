# Product Decision and Strategy

**Decision date:** 2026-07-13  
**Status:** Proceed with a deliberately narrow, validation-gated implementation.  
**Distribution recommendation:** Free and open source first; do not launch as a $0.99 App Store product.

> **Correction (2026-07-16):** the third-tool selection below is superseded by
> the 2026-07-16 owner decision — the 0.0.1 harness set is Claude Code + Codex
> CLI + Grok Build. GitHub Copilot CLI remains a likely 0.x candidate and its
> freshness tracking is retained. (Grok Build later shipped four local-posture
> rules for 0.2.102 from 2026-07-17 evidence; see `CONTEXT.md` and
> `rules/grok-build/` for current scope.)

## Decision in one sentence

Build Harness Guard as a local-only, cross-platform, source-cited auditor of AI coding-tool privacy, permissions, local retention, and synchronization posture—starting with a read-only CLI and three well-supported tools, then earn the right to add a GUI or commercial layer through user validation.

This is a **conditional yes**, not approval of the original broad desktop concept. The problem is real, but the product is only credible if it is conservative about what a local scan can prove and if maintaining current evidence is treated as core engineering rather than documentation work.

## The product promise

> Know which supported AI coding tools are installed, which locally observable settings increase exposure, which account settings still need manual verification, and why each recommendation is being made.

Harness Guard should answer concrete questions:

- Is local session history being retained, and for how long?
- Is a CLI configured to synchronize session data remotely?
- Are broad tool permissions or unsafe execution modes enabled?
- Are telemetry and feedback controls configured as intended?
- Which effective configuration layer wins: managed, project, user, CLI, or environment?
- Which material controls cannot be verified from this machine?
- What official source and product version support each finding?

It should **not** claim that a static scan proves what bytes have left a machine, whether a vendor trained on a specific session, or what an opaque server-side account toggle currently contains.

## Why this is useful

AI coding tools are now widely used, agent permissions are expanding, and developers commonly mix personal and work accounts. Current tools expose privacy, retention, telemetry, synchronization, sandbox, and permission settings in different locations and with different precedence rules. The concepts are routinely conflated: disabling model training does not necessarily disable inference transfer, retention, telemetry, feedback, or session synchronization.

There is also credible market validation:

- The [2025 Stack Overflow Developer Survey](https://survey.stackoverflow.co/2025/ai) reports broad use or planned use of AI tools, while Stack Overflow's 2026 reporting describes a persistent developer trust gap.
- [Snyk Agent Scan](https://github.com/snyk/agent-scan), [Cisco's IDE AI Security Scanner](https://cisco-ai-defense.github.io/docs/ai-security-scanner), [Armor1](https://armor1.ai), and [Skarn](https://getskarn.com) demonstrate demand for AI-agent security and local inspection.
- These products also mean the category is no longer empty. A generic “AI agent security scanner” would be weakly differentiated.

Harness Guard's credible niche is narrower: **transparent configuration and data-practice posture, with evidence the user can inspect**. It should never execute an MCP server, skill, plugin, or detected harness during a normal scan. That is both a safety property and a useful differentiator.

## Who benefits

| User | Primary value | Limitation |
|---|---|---|
| Individual developer | Finds risky defaults and forgotten local/sync settings without learning every tool's config model | May not pay for a utility used occasionally |
| Freelancer/consultant | Produces a defensible client-work posture and repeatable pre-project check | Still needs client policy and contractual context |
| Engineering lead/security champion | Standardizes an explainable baseline across several tools | Fleet visibility needs a later team product |
| Security/compliance team | Receives machine-readable evidence and known/unknown status rather than screenshots | Static local evidence is not DLP or network proof |
| New AI-tool user | Gets a guided checklist in plain language | Advanced configuration can remain tool-specific |

The individual developer is the best design target for the first release. Teams are the better eventual payer.

## Product boundaries required for trust

### What the first product does

- Discovers supported installations and configuration files using documented paths.
- Parses configuration directly; it does not scrape arbitrary logs or run vendor commands by default.
- Resolves configuration precedence where it can do so reliably.
- Emits concrete findings with severity, confidence, observation type, affected versions, last verification date, and citations.
- Marks remote/account state as `unknown` when it cannot be observed locally and links to the exact verification step.
- Produces terminal and sanitized JSON reports without raw config values, file contents, tokens, usernames, or full home paths.
- Works without a network connection and makes no network request during a scan.

### What it does not do initially

- Read source code, prompt transcripts, shell history, `.env` files, or secret values.
- Execute an installed AI tool, MCP server, extension, hook, skill, or plugin.
- Intercept TLS traffic or install certificates.
- Assign a false-precision 0–100 “safety score.”
- Automatically edit configuration.
- Promise comprehensive detection of environment variables from a desktop process.
- Treat vendor privacy claims, local settings, independent observations, and live network evidence as equivalent.

These boundaries avoid turning a privacy auditor into another sensitive-data collector.

## Initial tool scope

Start with **Claude Code, OpenAI Codex CLI, and GitHub Copilot CLI**.

They provide a valuable first slice because they are widely used, expose meaningful local configuration, and collectively exercise JSON/TOML parsing, precedence, local transcript/history settings, telemetry/feedback controls, permissions, and remote session synchronization.

Next candidates:

1. **Gemini CLI** — large and growing user base, rich local security settings, cross-platform.
2. **OpenCode** — substantial adoption, local provider credentials, permission policy, and potentially public session sharing.
3. **Cursor** — strategically important, but several decisive privacy modes are account/server state. The tool must display `unknown—verify in Cursor` rather than infer the setting from incomplete local traces.
4. **Grok Build** — include only as a versioned advisory/experimental ruleset until current behavior is reproduced. The July 2026 upload evidence is important, but a remote feature flag appears to have changed behavior after disclosure.

Popularity numbers, stars, and package downloads can help order work, but they are not user counts and must not be presented as such.

## Reliability model

Reliability comes from traceability and bounded claims, not from scanning more files.

Every rule should contain at least:

- Stable rule ID and schema version.
- Tool, operating system, configuration scope, and applicable version range.
- Authentication/plan prerequisites when those change the interpretation.
- Exact local observation and expected value type.
- Finding, remediation, severity, and confidence.
- Evidence class: `local-observation`, `official-documentation`, `official-policy`, `independent-reproduction`, or `inference`.
- Primary source URL, retrieval date, and optional archived content hash.
- `valid_from`, `valid_until`, and last-tested product version.
- Explicit limitations and the condition that should produce `unknown`.

Rules must be signed, versioned, schema-validated, fixture-tested, and independently updatable from application releases. If a rule is stale or outside its tested version range, the app should say so instead of silently applying it.

## Safety model

The core should be small, auditable, and usable as a library by both CLI and a later GUI. Rust remains a reasonable choice, but the language is less important than the invariants:

- No egress in scan code; automated tests should fail on attempted network access.
- Read-only operation in the first release.
- Bounded file size, depth, and parsing work.
- Refuse or safely handle symlinks, hostile configs, malformed JSON/TOML, permission errors, and concurrent file changes.
- Never print config values unless a rule explicitly defines a safe enum/boolean rendering.
- Store only normalized findings, not copies of configs.
- Signed releases, checksums, SBOM, dependency review, and reproducible-build work before calling the product production-ready.

Automatic fixes can come later. Each fix must preview a minimal diff, preserve formatting where practical, create a permission-preserving backup, compare the file before commit, write atomically, and offer rollback. Copyable vendor-specific instructions are safer for early releases.

## Distribution and pricing

### Recommendation: free/open source core

Use a permissive license such as Apache-2.0 for the core and rules. Trust, reviewability, community-maintained tool coverage, package-manager distribution, and adoption are more valuable here than low-priced consumer revenue.

Do **not** begin with a $0.99 Mac App Store app:

- Apple's [App Sandbox](https://developer.apple.com/documentation/security/protecting-user-data-with-app-sandbox) is required for Mac App Store apps and limits arbitrary filesystem access. Users would need to select protected directories and persist security-scoped access, which weakens the “audit my tools” experience.
- The product should be useful on macOS, Linux, and Windows; an App Store-first model narrows the audience and architecture.
- A one-dollar purchase does not fund continuously verifying fast-changing vendor behavior, user support, signing, and release operations.
- A closed privacy scanner has a harder trust story than inspectable source and rules.

For macOS, distribute a signed and [notarized](https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution) app directly if/when a GUI exists. The App Store can be reconsidered later as an optional companion, not the canonical product.

### How the project can create economic value

The strongest near-term return is authority and qualified leads, not microtransactions:

1. Publish the open-source tool and evidence-driven findings.
2. Create canonical per-tool privacy/posture pages and a transparent methodology on `benjsmin.com`.
3. Publish a case study showing the research system, secure local architecture, cross-platform engineering, and product judgment.
4. Offer a fixed-scope “AI coding posture review” for small engineering teams: inventory, policy baseline, rollout guidance, and a sanitized report.
5. Only build paid team functionality after real requests: fleet policy, signed organizational baselines, CI enforcement, private rules, evidence retention, and support.

The website already positions Benjamin as a senior builder of custom tools and AI workflow systems. Harness Guard fits that story unusually well. Developer traffic will not automatically become buyers, so content should connect the tool to business outcomes and include a specific team-audit or custom-tool call to action—not merely a generic portfolio link.

Keep the lead-generation relationship honest: no telemetry, no required account, no advertising inside findings, and a plainly labeled maintainer/project link.

## Worldwide adoption and localization

The underlying problem is global, but a Mac-only GUI would not be. The core should support macOS, Linux, and Windows paths and configuration semantics from the start, even if the first alpha is validated on macOS.

Design localization into the finding schema:

- Stable machine-readable IDs; no English strings used as program logic.
- ICU-style parameters and pluralization.
- Separate short finding, explanation, remediation, limitations, and source titles.
- Locale-independent JSON output.
- Dates, paths, links, and keyboard instructions rendered per platform/locale.

Launch English first, then French because the website already supports both and the maintainer can verify the language. Community translations become practical once wording and rule IDs stabilize. Sources can remain in their original language with localized summaries.

The name and promise should avoid legal/compliance guarantees. “Posture auditor” and “configuration check” translate more safely than “protection” or “certification.”

## Decision gates

Implementation is authorized only through the next validation gate, not as an unlimited desktop build.

| Gate | Evidence required | Stop/rethink condition |
|---|---|---|
| Evidence foundation | Three tools; primary-source rules; applicable versions; cross-platform fixtures; explicit unknown states | Material rules still depend on unsourced prose or guessed config keys |
| CLI alpha | 15–20 target-user tests; zero raw-data egress; no critical false positives in supported fixtures; most users understand known vs unknown | Users do not discover an actionable or previously unclear fact, or cannot trust the result |
| GUI | Repeated evidence that the CLI blocks target users or that history/comparison materially increases value | GUI is primarily aesthetic and does not improve task completion |
| Commercial team layer | At least three teams request the same centralized capability or a paid pilot | Monetization depends only on individual one-time purchases |

Proposed alpha success target: at least 60% of test users discover one actionable or previously unknown posture fact, and at least 80% correctly understand that the scan does not prove network behavior. These are decision thresholds, not market facts.

## Final recommendation

Proceed. Build the evidence system and a three-tool read-only CLI vertical slice. Release it free and open source if the validation gate succeeds. Use it as a high-quality public proof of work and an entry point to a paid team posture review. Defer the GUI, App Store, automated fixes, broad tool coverage, and paid product until users demonstrate which of those actually solve a problem.
