# Grok Build evidence — 2026-07-20

**Purpose:** Maintenance re-verification of local-posture keys after channel
movement `0.2.102 → 0.2.106`. Source-reading only (no wire capture, no lab
run). Continues the 2026-07-17 intake; does not replace it.

**Owner decisions (unchanged from 2026-07-17):**

1. Local-posture rules cite OSS primary sources + in-tree user guide.
2. Prefer OSS at pinned `SOURCE_REV` over lagging public docs when they disagree.
3. Behavior/wire claims still require a lab run under a later date directory.

## Version pin

| Signal | Value | Retrieved |
| --- | --- | --- |
| Channel pointer `https://x.ai/cli/stable` | `0.2.106` | 2026-07-20 |
| npm `@xai-official/grok` latest | `0.2.106` | 2026-07-20 |
| OSS monorepo `SOURCE_REV` file | `ba69d70c2f7d70a130a323b2becdf137af784c7f` | 2026-07-20 |
| GitHub `main` tip at retrieval | `ba76b0a683fa52e4e60685017b85905451be17bc` | 2026-07-20 |

**Rules `tested_versions`:** pin `min`/`max` to `0.2.106` (with `<=` on min
per loader invariant that `<=`-prefixed min equals max) and
`verified_on: 2026-07-20`.

## Claim re-verification

All four local-posture keys still match live OSS sources:

| Key | Type / notes | Verdict |
| --- | --- | --- |
| `features.telemetry` | bool or `"session_metrics"`; TelemetryMode default `Disabled` | PASS |
| `features.feedback` | bool; guide default `true` | PASS |
| `telemetry.trace_upload` | `Option<bool>`; None inherits master switch | PASS |
| `telemetry.otel_log_user_prompts` | `Option<bool>`; external OTEL content gate | PASS |

User-guide Telemetry section was rewritten (independent privacy knobs framing)
but keys, defaults, and inheritance semantics are unchanged. `config.rs` types
for these four observations are byte-stable since the 2026-07-16 OSS publish
(raw semantic hash unchanged).

## Retrieved artifacts (`raw/`)

Semantic hashes via `scripts/freshness/normalize.sh` (2026-07-20):

| File | Semantic sha256 |
| --- | --- |
| `user-guide-05-configuration.md` | `cc07bcd55b38aba8c18c84108e21787e7478c374d94bb0ee4df48e1ffa7338c3` |
| `telemetry-config.rs` | `11044f207a0894d0c0e2f729bc8c88fcc51b1b98a85d9eaadf85491b124a8e36` |
| `update-version.rs` | `55f91336fb79fc0680459776f0fed55bf3577742b64417a50f952108cc293d39` |
| `stable` | `5a90f3e4768115c6c779133a207de770a3125da821127dc4900cf19d2d827f15` |
| `install.sh` | `aacadba2136e5ba6378ddfa84bee36851dcbb29c03985c5e3cd1355d2453e270` |

These raw hashes are what rules store under the GitHub blob citation URLs
(same convention as the 2026-07-17 intake: stable content hash of the source
file, not the GitHub HTML chrome).

## Official source URLs for rule citations

- User guide config:
  `https://github.com/xai-org/grok-build/blob/main/crates/codegen/xai-grok-pager/docs/user-guide/05-configuration.md`
- Telemetry types:
  `https://github.com/xai-org/grok-build/blob/main/crates/codegen/xai-grok-telemetry/src/config.rs`
- Version/detection:
  `https://github.com/xai-org/grok-build/blob/main/crates/codegen/xai-grok-update/src/version.rs`
- Channel pointer: `https://x.ai/cli/stable`
- Installer: `https://x.ai/cli/install.sh`

## Not in this intake

- No lab-run / independent-reproduction behavior claims.
- No new observation keys.
- Copilot and other non-implemented harnesses are freshness-watched only.
