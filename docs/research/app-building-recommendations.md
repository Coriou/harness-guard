# App Building Recommendations for harness-guard

> **Superseded in part (2026-07-13):** Use [`../product/decision-and-strategy.md`](../product/decision-and-strategy.md) and [`../product/implementation-plan.md`](../product/implementation-plan.md) for the current direction. In particular, do not start with broad log/secret scanning, SQLite, numeric risk scores, one-click fixes, or a Tauri shell. The current plan begins with a stateless, read-only CLI and versioned evidence rules.

Extracted and expanded from the main research report (July 2026).

## High-Level Architecture

**Preferred stack**:
- Frontend + shell: Tauri (Rust core + web UI)
- Language for core logic: Rust (strong preference) or Go
- Why: Smaller attack surface, excellent filesystem and process control, native performance for scanning, easy to sandbox helper operations.

**Alternative**: Electron if UI velocity is more important initially.

**Data model**:
- Local-first storage (SQLite preferred for structured queries over time).
- Cache of tool "fingerprints" (known paths, config schemas, observed behaviors per version).
- User overrides and previous audit results.

## Core Capabilities

1. **Discovery**
   - Walk known paths per tool (from `data/tools-comparison.json` and per-tool docs).
   - Inspect environment variables.
   - Detect running processes (safe read-only).
   - Check package managers and global installs.
   - Handle multiple versions and custom locations.

2. **Configuration Parsing**
   - TOML, JSON, VS Code settings JSON.
   - Layered configs (user vs managed).
   - Redaction of keys/secrets during parsing.

3. **Log & Telemetry Analysis**
   - Streaming JSONL parsers with size limits.
   - Heuristic detection of repo uploads, file reads, command execution.
   - Secret pattern scanning (with user confirmation step).

4. **Risk Scoring**
   - Composite score based on:
     - Upload behavior (full repo vs selective)
     - Telemetry status
     - Privacy mode / opt-out status
     - Presence of secrets in workspace
     - Agentic capabilities enabled
     - Version age / known issues

5. **Recommendations & Fixes**
   - Safe, previewable edits to config files (with automatic backup).
   - Generated shell snippets (`export ...`).
   - One-click "safe defaults" profiles.
   - Never run destructive commands without explicit user approval + dry-run.

6. **Reporting**
   - Rich self-contained HTML report.
   - Export to JSON (for CI / team dashboards).
   - PDF option.
   - Include citations and confidence levels from the research.

## UI / Experience Priorities

- "Audit All" as the hero action.
- Clear visual risk indicators (traffic-light + numeric score).
- Per-tool expandable detail panels.
- "What changed since last audit?" diff view.
- Warnings about version-specific behavior and the need to re-audit.
- Offline-first experience (core features work without internet).

## Maintenance & Freshness System

- GitHub release watchers + changelogs for each tracked tool.
- On-demand or scheduled lightweight web research refresh (store results with timestamps).
- Version pinning of observed behaviors.
- Simple mechanism for users to contribute new observations (with strong source requirements).

## Reliability & Trust

- Every important claim in the UI should be traceable back to a source in `docs/research/`.
- Confidence indicators: High / Medium / Low.
- Explicit "This is based on research from [date]. Behavior may have changed."
- Conservative heuristics to avoid scaring users with false positives.
- Clear distinction between "vendor claims" and "observed / reported behavior".

## Edge Cases & Robustness

- Multiple concurrent installations of the same tool.
- Enterprise managed configuration layers (they win).
- Containers / remote development environments.
- WSL, devcontainers, SSH remotes.
- Air-gapped machines (full offline mode + local research cache).
- Secrets in git history (especially relevant for bundle-upload tools).
- User has never run the tool before (graceful empty state).

## Data Files the App Should Consume

- `data/tools-comparison.json` (core facts)
- `data/audit-commands.yaml`
- Per-tool config examples in `data/config-examples/`
- Future: `data/known-issues.json`, `data/version-behaviors.json`

## Suggested Phased Approach (Pre-Implementation)

1. Solidify this spec and the research data.
2. Build a CLI-first "harness-guard scan" tool (great for validation and CI).
3. Add Tauri desktop shell with rich UI.
4. Add reporting, fix actions, and maintenance features.
5. Add update / freshness mechanisms.

This keeps risk low and allows the research to stay the source of truth.
