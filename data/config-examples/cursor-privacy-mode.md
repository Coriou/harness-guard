# Cursor Privacy Recommendations (from research)

> **Legacy, not a verified fix profile.** Current Cursor documentation distinguishes Share Data, Privacy Mode with Storage, and Privacy Mode. Account state may not be locally observable. Verify the current options in [Cursor's privacy documentation](https://docs.cursor.com/account/privacy) and the application UI.

1. Go to Settings (Cmd/Ctrl + ,)
2. General → Privacy → review and select the mode appropriate to the account/workspace policy
3. Telemetry → Off (separate setting)
4. Create `.cursorignore` in project roots for:
   - node_modules/
   - .env*
   - secrets/
   - large binary dirs
   - .git/

Even with your own API keys, data still routes through Cursor backend for final prompt assembly.
