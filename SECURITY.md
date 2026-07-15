# Security policy

## Reporting a vulnerability

Please use [GitHub private vulnerability reporting](https://github.com/Coriou/harness-guard/security/advisories/new).
Do not open a public issue for an undisclosed vulnerability.

Reports must not contain real configuration files, source code, transcripts,
shell history, `.env` contents, tokens, usernames, or absolute home paths.
Use the synthetic fixtures in `fixtures/` or a minimal fully invented example.

Include:

- the affected commit and platform;
- the smallest synthetic reproduction;
- the expected and observed behavior; and
- whether the issue could cause network access, execution, an unintended read,
  or unsanitized output.

## Supported version

Harness Guard is an early preview. Security fixes target the current `main`
branch until versioned releases exist.

## Security invariants

A scan must remain network-free and execution-free. It must read only
explicitly supported, bounded configuration/version files; refuse unsafe file
types; and emit sanitized, per-finding-cited output. Tests and examples must use
synthetic fixtures—never a developer's real tool configuration.
