# AI Coding Harnesses / Agents — Data Handling, Logging, Telemetry, Uploads, Privacy & Configuration Research Report

> **Verification warning (2026-07-13):** This is the original broad research synthesis, not canonical application data. A second-pass audit found material stale, unsupported, or overly broad claims. Read [`verification-audit-2026-07-13.md`](./verification-audit-2026-07-13.md) before relying on any table, command, default, privacy control, or product recommendation below. In particular, do not use the documented Grok telemetry keys as mitigations without fresh official support.

**Date**: July 13, 2026  
**Status**: Legacy research artifact awaiting rule-by-rule migration and verification.  
**Standards**: Maximally truth-seeking. Highlights discrepancies between marketing/docs and observed behavior. Cites sources. Notes uncertainties.

---

## Executive Summary Table

Key risk dimensions (consumer/default setups as of mid-2026; "default" = out-of-box without hardening). 

| Tool                        | Architecture          | Full Repo Upload (default)?                  | Secrets Risk | Telemetry Default      | Effective Disable Telemetry?          | Code for Training (default) | Strong Privacy Toggle      | Main Config(s)                          | Primary Logs (macOS example)                     | Key Notes |
|-----------------------------|-----------------------|----------------------------------------------|--------------|------------------------|---------------------------------------|-----------------------------|----------------------------|-----------------------------------------|--------------------------------------------------|-----------|
| **Grok Build (xAI)**        | Hybrid TUI + cloud   | **Current default unknown**; git bundle captured in v0.2.93 | Conditional/high if upload applies | Not verified | Not verified | Depends on product/account terms | Not verified locally | `~/.grok/config.toml`, managed/requirements layers | Use documented logging settings for exact version | Reproducible v0.2.93 evidence; current behavior needs retest. |
| **Cursor**                  | Hybrid (VS Code fork)| No (chunks for indexing + on-demand)        | Medium-High | Configurable          | Yes (Privacy Mode + separate telemetry off) | Off with Privacy Mode      | **Strong** (Privacy Mode) | Settings, `~/.cursor/`, `.cursorignore`, `mcp.json` | `~/Library/Application Support/Cursor/logs/`    | Backend proxy even with own keys. |
| **Claude (Anthropic agents)** | Hybrid/API agentic  | Selective (tool on-demand reads)            | Medium      | Per client + usage    | Yes (account opt-out + commercial contracts) | Off if opted out (safety exceptions) | Good                      | Account settings; client configs       | Client-side + Anthropic Privacy Center          | Agentic sessions send Inputs/Outputs. |
| **GitHub Copilot (Agent/CLI/Workspace)** | Hybrid + cloud agents | Selective/summarized (not full in many modes) | Medium     | Usage metadata (prompts optional) | Yes (`COPILOT_OFFLINE`, settings)    | Claims limited / no training | Partial                   | VS Code settings, org policies, env    | VS Code logs + GitHub session data              | Content exclusion limited for agents. |
| **OpenAI Codex (CLI/agent)** | Hybrid CLI          | Context to chosen provider                  | Medium-High | Optional analytics/OTEL | Yes (flags + config)                 | Per OpenAI terms (opt-out available) | Config-driven             | `~/.codex/config.toml`, managed configs | `~/.codex/` (history.jsonl, logs)               | Strong sandbox/approval features. |
| **Aider**                   | Local client → LLM   | Selective (explicit reads + git-aware)      | Low (local LLM) | **Opt-in only** (no code) | Yes (strong; permanent disable)     | Never (explicit)           | **Excellent**             | CLI flags, env, PostHog custom         | `--analytics-log` (local JSONL)                 | One of the lowest-risk options. |
| **Continue.dev**            | Local/OSS            | Selective/relevant context                  | Low (BYO/local) | Removed (v2.0.0 final) | N/A                                  | N/A (BYO)                  | Strong (local models)     | Extension/CLI settings                 | Minimal (local)                                 | Repo read-only after acquisition. |
| **Cline**                   | OSS VS Code agent    | Selective + approval-gated                  | Low         | Opt-out anonymous (no code) | Yes                                 | No                         | Strong                    | Extension config                       | Local                                           | Transparent reasoning; privacy-focused. |
| **Windsurf**                | IDE/agent            | Likely hybrid/selective                     | Medium      | Varies                    | Varies (enterprise noted)            | Varies                     | Some self-host            | IDE config                             | Varies                                          | Enterprise positioning. |
| **Devin (Cognition)**       | Full cloud           | High (cloud execution)                      | **High**    | Yes                       | Limited                              | Cloud retention likely     | Limited                   | Platform settings                      | Cloud                                           | Parallel cloud agents. |
| **Replit Agent**            | Cloud IDE + agent    | High (platform)                             | **High**    | Yes                       | Limited                              | Platform                   | Limited                   | Platform                               | Cloud/platform                                  | Data lives on Replit. |

**Patterns observed**:
- "Do not use for training" toggles frequently do **not** prevent transmission or storage.
- Agents that can run terminal commands or read arbitrary files dramatically increase blast radius.
- Local models + explicit scoping + ignore files = dramatically lower risk for most tools.
- Secrets in `.env`, shell history, or recently opened files are the most common real-world exfiltration vector.

---

## Grok Build (xAI)

### 1. Architecture Overview
Terminal TUI coding agent. Hybrid: heavy local TUI + cloud backend (primarily Grok models). Supports agents, subagents, MCP servers, plan mode, worktrees, skills.

Context handling: On-demand reads plus a **version-specific repository-upload finding**. The reproducible `grok-build-exfil-repro` test captured a git bundle in v0.2.93 that included tracked files the model had not explicitly read and repository history. Public follow-up reporting indicates a later remote disable flag; the current universal behavior is unknown.

### 2. Logging & Telemetry
- Current official settings document `~/.grok/config.toml`, managed and requirements layers, `GROK_HOME`, configurable logging, and `grok inspect`.
- The older draft's default `~/.grok/logs/` and `~/.grok/debug/` claims have not been revalidated and must not be used as discovery rules.
- Log content depends on settings and version; do not read or copy logs in a normal Harness Guard scan.

### 3. Data Uploads & Exfiltration Risks
**Potentially high, version-specific risk**. The reproducible v0.2.93 capture shows a repository git bundle uploaded to an xAI `/v1/storage` endpoint regardless of the tested session's model-training choice.

- Trigger/default behavior outside the reproduced setup is not established here.
- "Improve the model" is a training choice and did not stop the tested v0.2.93 upload; that does not establish current behavior.
- Destinations: xAI infrastructure.
- Current default and a documented local upload-disable control require fresh verification.

### 4. Configuration & Privacy Options
Current documentation describes system/user managed configuration, user configuration, requirements files, and project configuration. Exact precedence must be taken from the current xAI documentation and tested before becoming a rule.

The earlier draft listed `[telemetry]` and `GROK_TELEMETRY_*` controls. They are absent from the current official settings reference and have been retired from this report. Use `grok inspect` and the settings documentation for the exact installed version. A training choice must not be treated as a codebase-upload control unless xAI documents that relationship.

### 5. Security Features
MCP permission prompts, plan mode, always-approve, worktree isolation, folder trust. Recent changelog entries mention fixes to telemetry exporter crashes.

No strong public evidence of automatic secret scanning or redaction for bundle uploads.

### 6. Privacy Policy & Data Use
xAI Privacy Policy (effective ~April 2026): Broad language for "User Content" (prompts, files, inputs) used to "provide, analyze, maintain, develop, improve, and conduct research."

Consumer, API, and enterprise terms differ. Enterprise documentation describes team-level Zero Data Retention behavior; do not apply that behavior to a consumer login or infer it from a local config.

Analyses emphasize: transmission/storage ≠ proven training use.

### 7. Versioning & Change Tracking
Very frequent releases (v0.2.7x–0.2.9x in July 2026). 
- Changelog: https://x.ai/build/changelog
- Install: `curl -fsSL https://x.ai/cli/install.sh | bash`
- Docs: https://docs.x.ai/build/overview

Rapid addition of agent features, MCP, telemetry controls.

### 8. Audit / Fix Commands
```bash
# Documented inspection starting point; interpret against the exact installed version
grok inspect
```

**Strong recommendation**: Do not infer current upload behavior or a mitigation from the v0.2.93 reproduction alone. Use high caution with sensitive codebases until the exact version/account behavior has been verified.

---

## Cursor

### 1. Architecture Overview
AI-powered code editor (VS Code fork). Hybrid: rich local IDE + cloud for model inference and indexing.

Context: active editor files, Composer/agent context, semantic codebase indexing (uploads small chunks for embeddings).

Even when using personal API keys, prompt construction often goes through Cursor backend.

### 2. Logging & Telemetry
- Logs: `~/Library/Application Support/Cursor/logs/` (dated session folders containing `exthost`, renderer, window logs).
- Other: `~/.cursor/`, `~/Library/Application Support/Cursor/User/globalStorage/`
- Enterprise: Admin API audit logs (Privacy Mode changes, MCP config, etc.).
- Inspect:
  ```bash
  open ~/Library/Application\ Support/Cursor/logs/
  ls ~/.cursor/
  ```

### 3. Data Uploads & Exfiltration Risks
- Indexing: small plaintext chunks uploaded for embeddings. Plaintext deleted after request; embeddings + metadata (file names, hashes) may be retained.
- Temporary encrypted file content cache during requests (client-generated keys).
- On-demand context sent for chats/agents.
- Full repo bundles not the default pattern.

### 4. Configuration & Privacy Options
**Critical control**: Privacy Mode (Settings → General → Privacy).

- **On**: Zero Data Retention (ZDR) agreements with providers. Code not used for training by Cursor. In-memory processing. Abuse classifiers may still retain.
- **Off**: May store and use codebase data, prompts, editor actions for improvement and training.
- Separate telemetry toggle.
- `.cursorignore` (gitignore syntax) for indexing exclusions.
- Enterprise: SSO, SCIM, central policy enforcement, audit logs.

Note: "Even if you use your own API key, requests still go through our backend."

### 5. Security Features
Workspace trust, `.cursorrules`, MCP server controls. Multiple CVEs in 2025 related to MCP prompt injection, sandbox escapes, etc. (documented in hardening guides).

### 6. Privacy Policy & Data Use
cursor.com/data-use (updated June 2026):
- Privacy Mode = no training + ZDR (with noted exceptions).
- Without it = data may be used for training.
- Some inference providers may temporarily access data even in certain modes.

### 7. Versioning & Change Tracking
Active (3.x series in 2026). Heavy investment in agents, Composer, cloud agents, MCP. Docs and hardening guides available publicly.

### 8. Audit / Fix Commands
```bash
# Quick audit
ls ~/Library/Application\ Support/Cursor/logs/
ls ~/.cursor/

# Recommended first actions
# 1. Enable Privacy Mode in UI
# 2. Disable telemetry in Settings
# 3. Create .cursorignore for noisy dirs
```

**Recommendation**: Privacy Mode + telemetry off is table stakes for commercial code. Use enterprise features for audit logs.

---

## Claude Code / Claude Artifacts / Anthropic Terminal & Agent Tools

Primarily API-driven agentic usage of Claude (tools, computer use, multi-step agents, terminal clients, or extensions). "Claude Code" generally means using Claude with agent capabilities.

### 1–3. Architecture, Logs, Uploads
- Hybrid: client decides what to send; Anthropic receives Inputs (including code files read by tools) and produces Outputs + actions.
- Selective/on-demand rather than blanket full-repo bundles.
- Agentic sessions can read files, execute commands, and call external services.

Risk scales with what the agent is allowed to read or instructed to do.

### 4–5. Configuration & Security
- Account-level "use my data for training" opt-out.
- Commercial/Enterprise plans have stronger contractual ZDR and data processing agreements.
- Client-side tool permissioning and sandboxing depend on the specific harness or SDK being used.

### 6. Privacy Policy & Data Use
Anthropic Privacy Policy (effective July 2026):
- Inputs and Outputs (explicitly including "agentic sessions") are collected.
- May be used for model training **unless** user opts out.
- Even with opt-out: safety review, flagged content, and explicit feedback are still used.
- Technical/usage data always collected.
- Third-party services the agent calls receive data directly.

Separate consumer vs commercial policies. See privacy.claude.com.

### 7–8. Versioning & Audit
Frequent model and agent capability releases. Audit primarily by reviewing transcripts + client logs for what files/tools were used.

**Recommendation**: Use commercial plans + explicit scoping for sensitive work. Opt out of training. Review agent tool permissions carefully.

---

## GitHub Copilot (Workspace, Agent Mode, CLI)

### 1. Architecture Overview
Hybrid. Strong integration in VS Code + standalone CLI + cloud agents.

Agent mode frequently uses **summarized workspace structure** rather than full codebase for token efficiency.

### 2–3. Logging & Uploads
- Telemetry primarily usage metadata.
- Prompt/suggestion collection is controllable.
- CLI supports `COPILOT_OFFLINE=true` (stops GitHub telemetry; routes only to your model provider).
- Content exclusion supported in some modes (explicitly **not** for CLI, cloud agent, and certain agent chat modes).

### 4–5. Configuration & Security
- VS Code settings for telemetry and prompt retention.
- Organization/Enterprise policies.
- Sandboxing available in CLI (`/sandbox enable`).
- Terminal auto-approve setting.

### 6. Privacy Policy & Data Use
GitHub Copilot documentation and telemetry terms:
- Data used for diagnostics and improvement.
- Various statements that code is not used to train models for other users.
- Session data for cloud agents retained for service lifetime in some cases.
- Shared with Microsoft and historically OpenAI in some contexts.

### 7–8. Audit / Fix
```bash
# CLI offline mode
export COPILOT_OFFLINE=true

# Review settings in VS Code and organization admin
```

**Recommendation**: Prefer offline mode + own model provider when possible. Review workspace permissions granted to Copilot.

---

## OpenAI Codex (CLI / Agent)

### 1. Architecture Overview
Hybrid CLI and agent tooling (significant Rust rewrite by 2026). Sends context to chosen provider (primarily OpenAI models).

Features include sandboxing, approval policies, MCP, browser use, profiles.

### 2–3. Logging & Uploads
- Home: `~/.codex/` (or `$CODEX_HOME`)
- Files: `config.toml`, `history.jsonl`, caches, logs.
- Telemetry: Optional analytics (anonymous usage/health) + OpenTelemetry (OTEL) export support.
- Context sent according to the model provider used.

### 4–5. Configuration & Security
- `~/.codex/config.toml` + project-level `.codex/config.toml`
- Enterprise: `managed_config.toml`
- Controls for `sandbox_mode`, `approval_policy`, `features`, OTEL exporters.

### 6. Privacy Policy & Data Use
Follows the terms of the model provider being used. Analytics described as optional and (in planning) open-source visible.

### 7–8. Audit / Fix
```bash
ls ~/.codex/
cat ~/.codex/config.toml
# Configure OTEL export for your own observability
```

**Recommendation**: Use strict approval policies and sandboxing. Export telemetry to your own systems.

---

## Aider

### 1. Architecture Overview
Pure terminal pair-programming tool. Local client that reads files you (or it) explicitly reference and sends context to **your chosen LLM provider**.

No central Aider server receives your code.

### 2–3. Logging & Uploads
- Analytics: **opt-in only**, anonymous, **explicitly does not collect code, prompts, chat, or keys**.
- Can log locally to JSONL.
- Supports sending analytics to your own PostHog instance.
- Full local operation possible with Ollama and similar.

### 4–5. Configuration & Security
```bash
aider --analytics-disable          # permanent opt-out
aider --analytics-log /tmp/audit.jsonl --no-analytics
```
Local models = zero external transmission.

### 6. Privacy Policy & Data Use
Clear statements: never collects your code etc. in analytics. Strong respect for privacy.

### 7–8. Audit / Fix
See commands above. Source is fully open; analytics collection points are searchable in the repo.

**Strong recommendation**: Excellent baseline choice for privacy-conscious users, especially with local models.

---

## Continue.dev

Open-source (Apache 2.0) coding agent (VS Code extension, JetBrains, CLI).

- Final v2.0.0 removed anonymous telemetry and authentication.
- Repository became read-only after acquisition by Cursor (community forks exist).
- Context sent only to whatever provider you configure (local models work great).
- Strong `.continueignore` support.

Lowest risk when configured with local models.

---

## Cline (and similar OSS agents)

Open-source VS Code (and other editors) autonomous agent.

- Apache 2.0.
- BYO API key / local models.
- Explicit approval gating on actions.
- Transparent step-by-step visualization.
- Telemetry: opt-out, anonymous, does not include code (per reports).
- Can be run fully offline.

Frequently recommended for privacy-sensitive developers.

---

## Windsurf, Devin, Replit Agent (Other Notable Tools)

- **Windsurf**: IDE/agent with enterprise features and some self-hosting mentions. SOC 2 Type 2 referenced in comparisons. Less public detail on exact upload mechanics than peers.
- **Devin (Cognition)**: Cloud-first AI software engineer. Parallel agents that deeply ingest codebases in the cloud. High data exposure by design.
- **Replit Agent**: Runs inside Replit's cloud platform. Code and execution are platform-hosted.

These are generally higher risk for sensitive or proprietary work.

---

## App Building Recommendations (for harness-guard)

### Suggested Architecture
- **UI**: Tauri (Rust backend + web frontend) preferred for security, size, and native filesystem access. Electron acceptable for faster UI iteration.
- **Core**: Rust (or Go) for safe path scanning, TOML/JSON parsing, redaction, and controlled command execution.
- **Storage**: Local-first (SQLite or files). Cache research facts + user configuration.
- **Detection**: Walk well-known paths for each tool + check running processes (safely), environment variables, and package managers.
- **Log parsing**: Streaming JSONL readers with strict size limits and secret redaction. Never execute contents.
- **Reporting**: Self-contained HTML reports (with embedded charts) + PDF export option.

### UI / UX Ideas
- "Audit All" button.
- Per-tool risk score + status (Privacy Mode on? Telemetry enabled? Recent uploads detected?).
- One-click "Apply Recommended Fixes" with preview + backup of config files.
- Secrets scanner (heuristic) with review step.
- Version fingerprinting and "behavior may have changed" warnings.
- Exportable machine-readable report (JSON) for teams.

### Maintenance System (keeping data fresh)
- Watch GitHub releases and changelogs for each tool.
- Scheduled or on-demand web searches / RSS for policy updates.
- Version pinning of observed behaviors.
- Community signal ingestion (with source vetting and confidence levels).

### Reliability Best Practices
- Every claim should be citable (official doc + date, analysis link, changelog).
- Confidence annotations ("High – multiple wire captures", "Medium – docs only", "Low – anecdotal").
- Prominent user warnings about version drift and false positives.
- Strong offline capability.
- Clear separation between "what we observed" and "what the vendor claims".

### Edge Cases to Handle
- Multiple versions of the same tool installed.
- Custom / non-standard install locations.
- Containers, devcontainers, remote SSH, WSL.
- Enterprise managed config layers (that override user settings).
- Air-gapped environments.
- Secrets in git history (relevant for bundle-uploading tools).

---

## Sources, Citations & Notes on Uncertainties

**Primary sources used** (non-exhaustive):
- Official privacy policies and data-use pages (xAI, Anthropic, Cursor, GitHub, Aider).
- Product documentation and changelogs (x.ai/build/changelog, docs.github.com/copilot, cursor.com, aider.chat/docs, developers.openai.com/codex, etc.).
- Independent technical analyses and wire captures (2026 reports on Grok Build uploads).
- Hardening guides and security research (howtoharden.com/guides/cursor, various blogs).
- GitHub repositories and issues (continuedev/continue, cline/cline, aider-ai/aider, etc.).
- Community discussions (HN, Reddit, forums).

**Uncertainties & Evolving Nature**:
- Many behaviors are version-specific and can change with updates.
- Enterprise vs consumer experiences differ significantly.
- Wire captures are powerful but snapshots.
- "No training" claims are common; actual transmission and retention are separate questions.
- Local model usage changes the risk profile completely for almost every tool.

**Recommendation for harness-guard users**: Treat the report as a strong starting map, not gospel. The app should make it easy for users to re-audit their own environment regularly.

---

*End of main research report. This document should be treated as the canonical reference until refreshed.*
