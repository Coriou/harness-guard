# Verification Audit — 2026-07-13

## Status

The original July 2026 report is a useful research hypothesis and inventory, but it is **not safe as canonical application data**. This audit rechecked decision-critical claims against current official documentation and identifiable independent reproductions.

Until migrated into the versioned evidence model described in the implementation plan:

- Do not consume `data/tools-comparison.json` as product truth.
- Do not execute or recommend commands from `data/audit-commands.yaml`.
- Do not present the original executive table as a current, universal default-state comparison.

## Why the old model is unreliable

The flat table combines dimensions that must remain separate:

- Data sent for inference.
- Local and remote retention.
- Model training/data sharing.
- Product telemetry and error reporting.
- Explicit feedback.
- Session synchronization/sharing.
- Agent file/tool permissions.
- Sandbox and network access.
- Consumer, commercial, API, enterprise, and zero-data-retention plans.
- Local observation, vendor documentation, policy language, and independent traffic observation.

Each can have a different default, control, plan dependency, and confidence level. A single “telemetry disabled” or “strong privacy toggle” field obscures this.

## Material corrections

### Grok Build

**Original claim:** Full-repository git bundle upload is a stable default, with mitigations such as `GROK_TELEMETRY_ENABLED`, `GROK_TELEMETRY_TRACE_UPLOAD`, and `[telemetry] trace_upload = false`.

**Verified state:**

- The independent [grok-build-exfil-repro](https://github.com/cereblab/grok-build-exfil-repro) provides unusually strong, reproducible evidence for Grok Build v0.2.93: a captured `/v1/storage` upload reconstructed into a git bundle containing tracked files/history, including a canary file the model had not read. This proves transmission/storage in that tested setup, not training use.
- Current [xAI settings documentation](https://docs.x.ai/build/settings) and [settings reference](https://docs.x.ai/build/settings/reference) do not document the two `GROK_TELEMETRY_*` variables or the proposed `[telemetry]` keys. They must not be recommended as current fixes.
- Current official documentation instead describes `~/.grok/config.toml`, managed/requirements layers, `grok inspect`, logging/proxy controls, `respect_gitignore`, and enterprise sandbox/network settings.
- Public follow-up reporting indicates xAI later returned a remote `disable_codebase_upload` feature response. Current default behavior therefore requires a fresh reproduction per version/account rather than a timeless `true` field.

**Product treatment:** Version-specific advisory with evidence date and tested version. Never assert current upload behavior solely from the v0.2.93 reproduction. Remove unsupported mitigations.

### Cursor

**Original claim:** A binary Privacy Mode provides ZDR/no training; otherwise data may be used.

**Verified state:**

- Current [Cursor privacy documentation](https://docs.cursor.com/account/privacy) distinguishes Share Data, Privacy Mode with Storage, and Privacy Mode rather than one binary state.
- Requests still route through Cursor's backend, including when users bring an API key.
- Codebase indexing sends plaintext chunks for embedding generation; plaintext and retained embedding/metadata behavior need to be described separately.
- Decisive account privacy state may not be observable from local files.

**Product treatment:** Detect only supported local artifacts. Show account mode as `unknown` with an official UI verification step unless Cursor exposes an authenticated, stable local/admin source.

### Claude Code

**Original claim:** Consumer users can opt out of training; commercial/Enterprise has stronger or ZDR-like handling.

**Verified state:**

- [Claude Code data usage](https://code.claude.com/docs/en/data-usage) distinguishes consumer and commercial terms. Consumer training choice changes training use and retention; commercial products are not used for training by default and normally retain data for 30 days. Zero Data Retention is for qualified configurations, not an automatic property of every Enterprise account.
- Local transcripts are stored under `~/.claude/projects/` and are retained for 30 days by default, configurable through `cleanupPeriodDays`.
- Telemetry, error reporting, feedback, and nonessential traffic are separate controls. Feedback is an explicit path that can include conversation/code.
- [Claude Code settings](https://code.claude.com/docs/en/settings) documents managed, CLI, local, project, and user scopes with setting-specific merge/precedence behavior.

**Product treatment:** Separate local retention, telemetry, error reporting, feedback, training terms, and ZDR. Include plan/account as user-confirmed or unknown input; never infer ZDR from “Enterprise.”

### GitHub Copilot CLI

**Original claim:** Primarily selective context and usage telemetry; offline mode is the main privacy control.

**Verified state:**

- Copilot CLI's official [session chronicle documentation](https://docs.github.com/en/copilot/concepts/agents/copilot-cli/chronicle) says complete local sessions are stored under `~/.copilot/session-state/` and synchronized to GitHub by default. `remoteExport: false` opts out of remote export; remote deletion and local deletion are separate.
- Official [permission documentation](https://docs.github.com/en/copilot/how-tos/copilot-cli/use-copilot-cli/allowing-tools) describes saved approvals in `~/.copilot/permissions-config.json`; session deny rules take precedence over both allow rules and saved approvals.
- GitHub's [Copilot CLI authentication documentation](https://docs.github.com/en/copilot/how-tos/copilot-cli/set-up-copilot-cli/authenticate-copilot-cli) says `COPILOT_OFFLINE=true` disables GitHub telemetry, while non-offline BYOK still sends normal telemetry. Offline mode can still send prompts/code to a remote configured model provider.

**Product treatment:** Remote session export, local retention, permission policy, and telemetry/offline state are separate first-class checks.

### OpenAI Codex CLI

**Original claim:** Analytics and OTEL are optional; data use is simply “per terms, opt-out available.”

**Verified state:**

- The official [Codex configuration reference](https://learn.chatgpt.com/docs/config-file/config-reference) documents `analytics.enabled`, `feedback.enabled`, `history.persistence`, plaintext TUI logging when configured, OTEL controls including prompt logging, sandbox/approval/network settings, and trusted-project behavior.
- Project configuration cannot override certain provider, authentication, or telemetry-routing keys.
- [Codex authentication](https://learn.chatgpt.com/docs/auth#openai-authentication) explains that ChatGPT sign-in and API-key use attach different workspace/organization data policies.

**Product treatment:** Parse observable local controls individually and report auth/data-policy interpretation as user-confirmed or unknown. Do not reduce it to a training opt-out boolean.

### Gemini CLI (missing from original report)

Official Gemini CLI settings and sandbox documentation expose material local posture controls: folder trust, tool sandboxing, approval modes, environment-variable redaction, ignore-file behavior, usage statistics, and detailed telemetry/prompt logging. Gemini CLI should be an early expansion target after the first three-tool slice.

Sources: [settings](https://geminicli.com/docs/cli/settings/), [sandbox](https://geminicli.com/docs/cli/sandbox/).

### OpenCode (missing from original report)

OpenCode exposes allow/ask/deny tool policies, locally stored provider credentials, local-provider support, and manual/automatic/disabled session sharing. Automatic sharing and overly broad permissions are useful auditable posture dimensions.

Sources: [permissions](https://opencode.ai/docs/permissions/), [providers](https://opencode.ai/docs/providers/), [sharing](https://opencode.ai/docs/share/).

### Aider

Aider analytics are privacy-preserving relative to many tools but “opt-in only” needs nuance. [Aider's analytics documentation](https://aider.chat/docs/more/analytics.html) says a random subset of users is prompted and the prompt defaults to Yes; it also says analytics omit code, prompts, chats, filenames, and keys and can be permanently disabled.

**Product treatment:** Report the actual configured/consent state where observable; do not turn a favorable design into an absolute “never” claim beyond the documented analytics payload.

### Continue

The original status is broadly supported: the upstream repository describes itself as no longer actively maintained/read-only, and the final 2.0 release removes anonymous telemetry/authentication. This is lower priority than currently expanding tools, but forks and older installations make version detection necessary.

## Source hierarchy

Rules should prefer evidence in this order, while keeping the evidence class visible:

1. Direct local observation of a documented setting in the scanned version.
2. Official versioned product/configuration documentation.
3. Official contractual/privacy policy for the user's known plan/auth method.
4. Reproducible independent technical evidence with artifacts and precise version/setup.
5. Maintainer issue/comment or vendor support statement.
6. Reputable secondary analysis.
7. Community anecdote, used only to form a testable research question.

“Higher” does not mean vendor policy can disprove a wire capture; the classes answer different questions. Conflicts must be displayed, scoped, and dated.

## Competitive context

The market is validated but crowded:

- [Snyk Agent Scan](https://github.com/snyk/agent-scan) discovers many harnesses and scans agents, MCP, and skills. Its own documentation warns that scanning MCP configuration can start commands and that some metadata is sent to Snyk's API.
- [Cisco IDE AI Security Scanner](https://cisco-ai-defense.github.io/docs/ai-security-scanner) offers local/static MCP and skills scanning inside compatible editors.
- [Armor1](https://armor1.ai) advertises AI agent security posture, privacy controls, workspace trust, MCP, and remediation.
- [Skarn](https://getskarn.com) scans local session histories for secrets and attack chains.
- [Code Insights](https://code-insights.app/docs/faq) is an open-source local session analytics tool rather than a security auditor.

Harness Guard should not compete on the broadest scan. Its differentiation is: no execution, no content/session ingestion, no scan-time network, primary-source traceability, explicit unknowns, and a focused privacy/configuration posture model.

## Research work still required before coding broad coverage

- Freeze and archive exact official source content used by each initial rule, subject to source terms.
- Validate local paths/defaults on clean installations for each supported OS and version.
- Determine safe non-executing version-detection methods.
- Document every precedence/merge rule, including managed configuration.
- Verify which account states can and cannot be observed locally.
- Convert all claims into the rule schema; reject any claim without applicability and limitation fields.
- Run independent clean-room tests for Grok Build before presenting a current upload advisory.

This audit supersedes the old instruction to treat the main report or structured data as ground truth.
