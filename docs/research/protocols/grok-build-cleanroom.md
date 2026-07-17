# Grok Build clean-room reproduction protocol

**Status:** Release-gating for 0.0.1 (spec §7.3), amended 2026-07-17. This is
maintainer lab work performed OUTSIDE the product. Harness Guard itself never
captures traffic, never executes `grok`, and never phones home; this protocol
produces the dated evidence artifacts that Grok Build rules may cite for
*behavior* claims.

**Prior work is quarantined.** The v0.2.93 reproduction
(github.com/cereblab/grok-build-exfil-repro) and everything under
`docs/research/per-tool/grok-build.md` and `data/` are leads only. No risk
claim ships without fresh primary evidence appropriate to the claim class.

## Two evidence paths (owner decision 2026-07-17)

| Path | What it authorizes | Required artifacts |
| --- | --- | --- |
| **Source-reading (local posture)** | Rules that observe user-scope config keys only (`features.telemetry`, `features.feedback`, `telemetry.trace_upload`, `telemetry.otel_log_user_prompts`, …). `evidence_class: official-documentation`. | Dated intake under `docs/research/evidence/grok-build/<date>/` with pinned version/channel, `SOURCE_REV`, semantic hashes of OSS primary sources and/or official docs, and notes that no wire capture was performed. |
| **Lab-run (behavior)** | Upload/telemetry *behavior* claims (canary hit/no-hit, endpoints, payload structure). `evidence_class: independent-reproduction`. | §§1–5 of this protocol executed; sanitized capture summaries under the same dated directory. |

Local-posture rules **may** cite OSS primary sources (pinned `SOURCE_REV` +
in-tree user guide) as authoritative for key names and documented defaults,
preferring them over lagging public docs pages when they disagree. Behavior
claims still require lab artifacts. No rule may use
`evidence_class: independent-reproduction` without a lab run under that date
directory.

The 2026-07-17 intake is **source-reading only** (local posture). See
`docs/research/evidence/grok-build/2026-07-17/README.md`.

## 1. Environment (lab-run path)

- A fresh, disposable VM or container per run; macOS and Linux runs both
  recorded. Destroy the environment after artifact extraction.
- No personal or client data anywhere inside the environment.
- An owner-provisioned **disposable** xAI account — never a personal or
  company account.
- A purpose-built canary repository: unique, never-published canary tokens
  embedded in files the model is NOT asked to read, plus files it IS asked
  to read. Token uniqueness is what makes payload search conclusive.

## 2. Version pinning

Before any run (lab or source-reading intake), record:
- the exact Grok Build version string (channel pointer and/or npm dist-tag);
- the install channel (npm package name + dist-tag, installer URL / channel
  pointer, or other);
- for source-reading: the OSS monorepo `SOURCE_REV` when keys are taken from
  source;
- for lab-run: the SHA-256 of the installed binary/package artifact.

This run's findings apply to exactly that version. Behavior rules cite it with
`tested_versions` min == max (no `<=` prefix) unless a separate written
justification widens the range. Local-posture rules may use the peer MDN-style
`<=` min convention when the source evidence justifies the lower bound.

## 3. Capture (lab-run path)

- System-level egress observation scoped to the VM: mitmproxy with a locally
  installed CA **plus** raw packet capture (tcpdump/pcap) as corroboration.
- Record request targets, sizes, and payload structure for xAI endpoints.
- The transmission test is a canary-token search over all captured payloads
  (decoded/decompressed where applicable).

## 4. Matrix (lab-run path)

Run each row in a fresh environment state; record config file + account UI
state alongside each capture:
1. Documented-default configuration (no config file edits).
2. Each currently documented mitigation key from the OSS user guide /
   telemetry config (and cross-checked public docs) toggled independently —
   retrieve keys fresh at run time from the pinned `SOURCE_REV` and official
   pages. Owner decision 2026-07-17: OSS source is primary; do **not** treat
   the July-13 audit's "keys are retired" conclusion as current.
3. Account/server-side flag states (e.g. any remote codebase-upload disable
   feature) recorded as user-confirmed observations of the account UI where
   visible; otherwise recorded as unknown. Never inferred.

A reduced matrix is acceptable for a targeted behavior claim (see the dated
evidence README "Targeted lab brief").

## 5. Artifacts

Store under `docs/research/evidence/grok-build/<YYYY-MM-DD>/`:
- for source-reading: version/channel pin, `SOURCE_REV`, semantic hashes
  (`scripts/freshness/normalize.sh`), compact raw excerpts, and an explicit
  note that no network capture was performed;
- for lab-run: dated, sanitized capture summaries (endpoints, sizes, payload
  structure, canary hit/no-hit per matrix row) — canary tokens are fine to
  include; no credentials, no personal data, no full payload dumps — plus the
  exact configuration file used per matrix row;
- the version-pinning record from §2;
- Wayback anchors when available for every docs page cited.

## 6. Rule authoring consequences

- Locally observable `~/.grok/config.toml` posture keys cite
  `official-documentation` sources (OSS user guide / telemetry types and/or
  docs.x.ai), with `notes` naming `SOURCE_REV` and retrieval date when OSS is
  primary.
- Upload/telemetry *behavior* claims cite `independent-reproduction` with the
  pinned version and are never generalized beyond it.
- Server-side/account state is `unknown` with a `verify_url`.
- Version drift after the run ⇒ `stale-ruleset` by construction — the honest
  state, not a defect.
- Rules must not claim wire-level upload behavior from source-reading alone.

## 7. Live-keys tripwire (amended 2026-07-17)

**Owner decision supersedes the plan's original "ban forever" wording.**

The July-13 audit banned `GROK_TELEMETRY_ENABLED`, `GROK_TELEMETRY_TRACE_UPLOAD`,
`[telemetry]`, and `trace_upload` because public docs omitted them. The OSS
product source and in-tree user guide at the 2026-07-17 `SOURCE_REV`
**re-document** those controls as live. No currently-live key is banned.

Tripwires in `crates/harness-guard-rules/tests/tripwires.rs` and
`crates/harness-guard-cli/tests/cli_surface.rs` no longer ban those strings.
Rules that cite them must still:

- use `evidence_class: official-documentation` (or a future lab-backed
  `independent-reproduction`) with dated sources;
- never recommend the *legacy research-only* claim that those keys alone stop
  canary-repo wire uploads without a lab artifact.

If a key is later proven genuinely dead, re-introduce a narrow ban for that
key only, with a citation to the intake that retired it.

## Release gate

0.0.1 may ship **local-posture** Grok Build rules that cite a completed
source-reading intake (2026-07-17 or later) with pinned version and primary
sources. Behavior claims remain blocked until a lab run lands. If upstream
releases between intake and tag, follow the `docs/maintenance/runbook.md`
triage flow to decide re-intake vs. shipping with the pinned version honestly
reflected (newer detected versions then yield `stale-ruleset`, which is
correct behavior).
