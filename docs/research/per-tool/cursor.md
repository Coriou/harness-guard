# Cursor — Quick Reference

> **Legacy summary.** Current Cursor privacy choices are not accurately represented by a single binary toggle, and decisive account state may not be locally observable. Read [`../verification-audit-2026-07-13.md`](../verification-audit-2026-07-13.md) before using this file.

**Risk Level**: Not expressible as one account-independent rating.

**Key control family**: Cursor's current privacy/data-sharing modes and workspace policy.

Current documentation distinguishes Share Data, Privacy Mode with Storage, and Privacy Mode. Their retention/training properties should be checked directly in the current documentation and account UI.

## Important Locations
- Logs: `~/Library/Application Support/Cursor/logs/`
- Config: `~/.cursor/`, `.cursorignore`, `.cursor/mcp.json`
- Even with Bring-Your-Own-Key: requests still go through Cursor backend.

## Recommendations
1. Verify the active account/workspace privacy mode in the UI; do not infer it from local files.
2. Review telemetry and indexing settings separately.
3. Treat `.cursorignore` as a scoping aid, not proof that no content can be sent by every feature.
4. Confirm organization policy for commercial work.

See the [current privacy documentation](https://docs.cursor.com/account/privacy) and [verification audit](../verification-audit-2026-07-13.md).
