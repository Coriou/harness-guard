# Sources and References

> **Legacy source list.** Several entries below are too vague to support production findings. The current verified corrections and exact decision-critical links are in [`verification-audit-2026-07-13.md`](./verification-audit-2026-07-13.md). Future rules must cite exact pages, retrieval dates, versions, evidence classes, and limitations.

This document tracks the primary sources used for the July 2026 AI coding tools privacy research.

## Official Documentation & Policies

- xAI Privacy Policy: https://x.ai/legal/privacy-policy (effective ~April 2026)
- Cursor Data Use & Privacy Overview: https://cursor.com/data-use (updated June 2026)
- Anthropic Privacy Policy: https://www.anthropic.com/legal/privacy (effective July 2026)
- Anthropic Privacy Center: https://privacy.claude.com/
- GitHub Copilot documentation (telemetry, agents, CLI, responsible use): docs.github.com/copilot
- Aider Privacy Policy & Analytics docs: https://aider.chat/docs/legal/privacy.html and https://aider.chat/docs/more/analytics.html
- OpenAI Codex configuration docs: developers.openai.com/codex/...

## Changelogs & Release Notes

- Grok Build Changelog: https://x.ai/build/changelog (very active v0.2.x series)
- Various product blogs and release posts for Cursor, Copilot, etc.

## Independent Analyses & Security Research (2025–2026)

- Wire-level analyses of Grok Build CLI repository uploads (multiple sources including HN discussions, developer blogs, and gists — June/July 2026)
- Cursor Hardening Guide: https://howtoharden.com/guides/cursor/
- Various security and privacy blog posts on Cursor, Copilot, and agent risks (GitGuardian, Checkmarx, etc.)

## GitHub Repositories

- continuedev/continue (final v2.0.0, telemetry removed)
- cline/cline
- aider-ai/aider (and original paul-gauthier/aider)

## Community & Forums

- Hacker News threads on Grok Build uploads
- Reddit discussions (r/vscode, etc.)
- Cursor forums and GitHub issues

## Notes on Source Quality

- Wire captures and independent technical reports are treated as high-value for observed behavior.
- Official docs are authoritative for claimed behavior but were cross-checked against real-world reports for discrepancies.
- All claims should be re-verified on current versions of the tools.

**Last major refresh**: 2026-07-13
