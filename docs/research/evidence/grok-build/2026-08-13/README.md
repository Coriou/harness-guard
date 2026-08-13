# Grok Build evidence — 2026-08-13

**Purpose:** Maintenance re-verification of local-posture keys after the
channel moved `0.2.106 → 1.0.3` (major). Source-reading only (no wire
capture, no lab run). Continues the 2026-07-17 and 2026-07-20 intakes; does
not replace them.

**Owner decisions (unchanged from 2026-07-17):**

1. Local-posture rules cite OSS primary sources + in-tree user guide.
2. Prefer OSS at pinned `SOURCE_REV` over lagging public docs when they disagree.
3. Behavior/wire claims still require a lab run under a later date directory.

## Version pin

| Signal | Value | Retrieved |
| --- | --- | --- |
| Channel pointer `https://x.ai/cli/stable` | `1.0.3` | 2026-08-13 |
| npm `@xai-official/grok` latest | `1.0.3` | 2026-08-13 |
| OSS monorepo `SOURCE_REV` file | `ea094a8c369475f97c85540d01730baec0dce5d6` | 2026-08-13 |
| GitHub `main` tip at retrieval | `e5fd4816d43260c15ba785f103990c1ed6cea230` | 2026-08-13 |

**Rules `tested_versions`:** pin `min`/`max` to `1.0.3` (with `<=` on min
per loader invariant that `<=`-prefixed min equals max) and
`verified_on: 2026-08-13`. The `<=` convention is the same local-posture
convention used at 0.2.x; the major bump is called out because Grok's
key surface has historically moved, but the four observation keys remain
documented in the 1.0.3 user guide and `TelemetryConfig` types.

## Claim re-verification

All four local-posture keys still match live OSS sources at 1.0.3:

| Key | Type / notes | Verdict |
| --- | --- | --- |
| `features.telemetry` | bool or `"session_metrics"`; TelemetryMode default `Disabled` | PASS |
| `features.feedback` | bool; guide default `true` | PASS |
| `telemetry.trace_upload` | `Option<bool>`; None inherits master switch | PASS |
| `telemetry.otel_log_user_prompts` | `Option<bool>`; external OTEL content gate | PASS |

The user-guide Telemetry section still frames these as independent knobs and
now also documents `/privacy` coding-data sharing (account/settings UI, not
a `config.toml` key this product can observe), `mixpanel_enabled`, and
`otel_log_tool_details`. Those extra keys were evaluated for this pass and
were **not** added as rules (see CONTEXT.md).

## Retrieved artifacts (`raw/`)

Semantic hashes via `scripts/freshness/normalize.sh` (2026-08-13):

| File | Semantic sha256 |
| --- | --- |
| `user-guide-05-configuration.md` | `cd16d3a9758ce038de942f83b86e4c3ac271fbd31b01afe7b83c32ddae4ec2b9` |
| `telemetry-config.rs` | `4939cadc8d141187503eaad1a793a723b2d987333f6fb648a717c1f1f7da132f` |
| `update-version.rs` | `d155089bdd33ecbd19cb17a9516d8034b7c6b5de272633f8d8407f8727463e9e` |
| `stable` | `20d2cb096d1ab41a4140246d12f07bf6b8cb743fd48122b72532c03d44c5c14a` |
| `install.sh` | `ed21d8db9f031f12312a67b01fcf011fbe2b39c1f00ed8753592b380ceb608a4` |

These raw hashes are what rules store under the GitHub blob citation URLs
(same convention as the 2026-07-17/20 intakes: stable content hash of the
source file, not the GitHub HTML chrome).

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
- No new observation keys shipped.
- Copilot and other non-implemented harnesses are freshness-watched only.
