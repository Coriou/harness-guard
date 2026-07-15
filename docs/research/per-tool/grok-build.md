# Grok Build (xAI) — Quick Reference

> **Legacy and materially superseded.** Do not use the telemetry/configuration keys below. Read [`../verification-audit-2026-07-13.md`](../verification-audit-2026-07-13.md) and current [xAI settings documentation](https://docs.x.ai/build/settings/reference). The independent upload reproduction applies to a tested version/setup and does not establish the current universal default.

**Risk Level**: Potentially high where the reproduced upload behavior applies; current applicability is unknown.

**Key Finding**: A reproducible v0.2.93 wire capture reconstructed a repository git bundle containing tracked unread files and history. The tested training choice did not stop it. Public follow-up reporting indicates a later remote disable flag, so this evidence must not be generalized to every current version/account.

## Critical Paths (macOS / Unix)
- Config: `~/.grok/config.toml`
- Logs: use the current documented logging configuration; the legacy default log paths are unverified
- Worktrees: `~/.grok/worktrees/`

## High-Value Audit Commands
Use the documented `grok inspect` command and compare the result with the settings reference for the exact installed version. Do not execute the legacy command YAML.

## Recommended Starting Hardening
No copyable hardening profile is currently verified. The former `[telemetry]` example and `GROK_TELEMETRY_*` variables are absent from current official documentation. For sensitive repositories, use only controls documented for the exact version and account, and treat current codebase-upload behavior as requiring independent verification.

## References
- [Current settings reference](https://docs.x.ai/build/settings/reference)
- [Independent v0.2.93 reproduction](https://github.com/cereblab/grok-build-exfil-repro)
- [Verification audit](../verification-audit-2026-07-13.md)
