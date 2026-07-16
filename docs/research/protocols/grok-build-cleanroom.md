# Grok Build clean-room reproduction protocol

**Status:** Release-gating for 0.0.1 (spec §7.3). This is maintainer lab work
performed OUTSIDE the product. Harness Guard itself never captures traffic,
never executes `grok`, and never phones home; this protocol produces the
dated evidence artifacts that Grok Build rules cite.

**Prior work is quarantined.** The v0.2.93 reproduction
(github.com/cereblab/grok-build-exfil-repro) and everything under
`docs/research/per-tool/grok-build.md` and `data/` are leads only. No risk
claim ships without a fresh run against the then-current version.

## 1. Environment

- A fresh, disposable VM or container per run; macOS and Linux runs both
  recorded. Destroy the environment after artifact extraction.
- No personal or client data anywhere inside the environment.
- An owner-provisioned **disposable** xAI account — never a personal or
  company account.
- A purpose-built canary repository: unique, never-published canary tokens
  embedded in files the model is NOT asked to read, plus files it IS asked
  to read. Token uniqueness is what makes payload search conclusive.

## 2. Version pinning

Before any run, record:
- the exact Grok Build version string as the product reports it;
- the install channel (npm package name + dist-tag, installer URL, or other);
- the SHA-256 of the installed binary/package artifact.

This run's findings apply to exactly that version. Rules cite it with
`tested_versions` min == max (no `<=` prefix) unless a separate written
justification widens the range.

## 3. Capture

- System-level egress observation scoped to the VM: mitmproxy with a locally
  installed CA **plus** raw packet capture (tcpdump/pcap) as corroboration.
- Record request targets, sizes, and payload structure for xAI endpoints.
- The transmission test is a canary-token search over all captured payloads
  (decoded/decompressed where applicable).

## 4. Matrix

Run each row in a fresh environment state; record config file + account UI
state alongside each capture:
1. Documented-default configuration (no config file edits).
2. Each currently documented mitigation key from docs.x.ai/build/settings and
   docs.x.ai/build/settings/reference toggled independently (retrieve those
   pages fresh at run time; do NOT reuse key names from prior research — the
   old GROK_TELEMETRY_* / [telemetry] trace_upload keys are retired and
   test-banned).
3. Account/server-side flag states (e.g. any remote codebase-upload disable
   feature) recorded as user-confirmed observations of the account UI where
   visible; otherwise recorded as unknown. Never inferred.

## 5. Artifacts

Store under `docs/research/evidence/grok-build/<YYYY-MM-DD>/`:
- dated, sanitized capture summaries (endpoints, sizes, payload structure,
  canary hit/no-hit per matrix row) — canary tokens are fine to include; no
  credentials, no personal data, no full payload dumps;
- the exact configuration file used per matrix row;
- the version-pinning record from §2;
- semantic hashes (`scripts/freshness/normalize.sh <url>`) and Wayback
  anchors for every docs page cited.

## 6. Rule authoring consequences

- Locally observable `~/.grok/config.toml` posture keys cite
  `official-documentation` sources.
- Upload/telemetry *behavior* claims cite `independent-reproduction` with the
  pinned version and are never generalized beyond it.
- Server-side/account state is `unknown` with a `verify_url`.
- Version drift after the run ⇒ `stale-ruleset` by construction — the honest
  state, not a defect.

## 7. Retired-keys tripwire

`GROK_TELEMETRY_ENABLED`, `GROK_TELEMETRY_TRACE_UPLOAD`, and the
`[telemetry]`/`trace_upload` key pair must never reappear in any rule,
remediation, or user-facing string. Enforced by
`crates/harness-guard-rules/tests/tripwires.rs` and the cli_surface corpus
test.

## Release gate

0.0.1 does not tag until this protocol has been executed against the
then-current Grok Build version and the shipped Grok rules cite that run. If
upstream releases between the run and the tag, follow the
`docs/maintenance/runbook.md` triage flow to decide re-run vs. shipping with
the pinned version honestly reflected (newer detected versions then yield
`stale-ruleset`, which is correct behavior).
