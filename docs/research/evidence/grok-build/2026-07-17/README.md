# Grok Build evidence — 2026-07-17

**Purpose:** Release-gating evidence intake for Task 19 (0.0.1 multi-harness).
**Owner decisions (session 2026-07-17):**

1. **Evidence scope:** local-posture rules from source + official docs; targeted
   lab run only for any independent-reproduction *behavior* claims we ship.
   No full clean-room matrix is required for local-posture rules.
2. **Key authority:** prefer OSS source + in-tree user guide at
   `github.com/xai-org/grok-build` (pinned `SOURCE_REV`) over lagging
   `docs.x.ai` pages when they disagree. Public docs still retrieved and
   hashed for cross-check.

This package is **not** a network capture. Behavior claims (canary uploads,
wire-level telemetry fire) remain out of scope until a targeted lab run lands
under a later date directory.

## Version pin

| Signal | Value | Retrieved |
| --- | --- | --- |
| Channel pointer `https://x.ai/cli/stable` | `0.2.102` | 2026-07-17 |
| npm `@xai-official/grok` latest | `0.2.102` | 2026-07-17 |
| OSS monorepo `SOURCE_REV` | `124d85bc5dc6e7805560215fcc6d5413944920e1` | 2026-07-17 |
| Public changelog page "Latest" | still listed `v0.2.101` at retrieval (lags channel) | 2026-07-17 |

**Rules `tested_versions`:** pin `min`/`max` to `0.2.102` (with `<=` on min
per MDN convention used by peer rules) and `verified_on: 2026-07-17`. Local
posture keys are evidenced from the OSS tree at the SOURCE_REV above, which
ships with the product at this channel pin.

## Packaging and detection (execution-free)

From `https://x.ai/cli/install.sh` and OSS
`crates/codegen/xai-grok-update/src/version.rs`:

| Fact | Value | Evidence |
| --- | --- | --- |
| PATH binary | `grok` (also `agent` alias) | install.sh; README |
| Home | `~/.grok` / `$GROK_HOME` | docs.x.ai/settings; user guide |
| User config | `$GROK_HOME/config.toml` | docs + user guide |
| Config format | TOML | same |
| Primary install channel | internal installer `https://x.ai/cli/install.sh` | install.sh |
| Channel pointers | `https://x.ai/cli/{stable,alpha,enterprise}` | install.sh; version.rs |
| Artifact URL pattern | `{base}/grok-{version}-{platform}` | install.sh |
| npm package (still published) | `@xai-official/grok` | version.rs `NPM_PACKAGE` + npm registry |
| Managed on-disk version probe | parse symlink target `grok-<version>-<platform>` **without exec** | version.rs `installed_on_disk_version` + `version_from_versioned_binary_name` |

**Harness Guard detection strategy (authorized by design §5.3 "may be added
behind the same interface with recorded evidence"):**

1. Existing npm walk when PATH binary resolves near `package.json` named
   `@xai-official/grok`.
2. Fallback: if the resolved PATH entry is a symlink whose target basename
   matches the documented `grok-<semver>[-platform…]` pattern, parse the
   version exactly as Grok's own `version_from_versioned_binary_name` does.
3. Otherwise `None` → `stale-ruleset` ("version not detected") — never invent.

Do **not** run `grok --version` from product code. Do **not** read ambient
developer `~/.grok` stores in tests — synthetic fixtures only.

## Local-posture keys (for rules)

From OSS user guide `05-configuration.md` and
`xai-grok-telemetry/src/config.rs` (retrieved 2026-07-17):

| Key | Type | Documented meaning | Rule posture |
| --- | --- | --- | --- |
| `features.telemetry` | bool *or* `"session_metrics"` string | Master switch for SpaceXAI product telemetry. `TelemetryMode` default in source is `Disabled`. | Primary telemetry rule (bool observation; string modes fall to unrecognized/unknown) |
| `features.feedback` | bool | Feedback system; user guide default `true` | Feedback rule |
| `telemetry.trace_upload` | optional bool | `None` inherits master telemetry; `Some(false)` disables GCS session/trace uploads only | Secondary telemetry rule (explicit false = pass; true = finding; unset = unknown) |
| `telemetry.mixpanel_enabled` | bool | Product analytics sub-switch | Optional / skip for 0.0.1 if redundant with master |
| `telemetry.otel_log_user_prompts` | optional bool | External OTEL content gate; default off | OTEL prompt-log rule (mirrors Codex otel rule) |

**Retired-keys tripwire must be revised.** The July-13 audit banned
`GROK_TELEMETRY_*`, `[telemetry]`, and `trace_upload` because public docs
omitted them. The OSS product source and in-tree user guide **re-document**
them as live controls. Owner decision: OSS is authoritative. Ban only keys
that remain genuinely dead after this intake (none of the four prior strings
are dead). Replace the tripwire with a tighter ban list if any truly dead
legacy remains; otherwise remove the ban and pin that rules cite current keys
with evidence.

**Not in 0.0.1 rules (local posture insufficient or no key found):**

- Runtime upload/canary behavior (needs targeted lab)
- Account/server `disable_codebase_upload` (unknown + verify_url only if we
  ever add an observation-less advisory — out of scope)
- Session retention period (no documented user-scope cleanup key found)

## Targeted lab brief (owner, later)

If shipping any `independent-reproduction` behavior claim, run a **reduced**
matrix (not the full §4 protocol):

1. Pin exact `0.2.102` (or then-current stable) in a disposable VM.
2. Default config vs `features.telemetry = false` vs
   `telemetry.trace_upload = false` — one capture each.
3. Canary-token search over mitmproxy+pcap for repo content.
4. Land sanitized summaries under
   `docs/research/evidence/grok-build/<lab-date>/`.

Until that exists, **no rule may use `evidence_class: independent-reproduction`.**

## Retrieved artifacts (`raw/`)

Semantic hashes via `scripts/freshness/normalize.sh` (2026-07-17):

| File | Semantic sha256 |
| --- | --- |
| `user-guide-05-configuration.md` | `7b19d8b0cb8c589f672bc49a47d217de206f25ad4893b1233c02031cfd3a8319` |
| `telemetry-config.rs` | `11044f207a0894d0c0e2f729bc8c88fcc51b1b98a85d9eaadf85491b124a8e36` |
| `update-version.rs` | `55f91336fb79fc0680459776f0fed55bf3577742b64417a50f952108cc293d39` |
| `SOURCE_REV` | `2f6ae6f7faea2519c41216a606680ff74749bc93e2404ff65f87f45cc7208d34` |
| `install.sh` | `aacadba2136e5ba6378ddfa84bee36851dcbb29c03985c5e3cd1355d2453e270` |
| `stable` | `060c2b1a4f89732010aad18e5ab90b0f306013e2382fe1eaa3ff96daf9c06840` |
| `settings.html` | `81e696a6cb988f072a0d246317717614339ae8fe08a227016963c75208719c97` |
| `settings-reference.html` | `4a5a07abd1fb24e50e724ec4c7320d29efd10fc66a930930369395af5134ea2b` |

Wayback (best-effort):

- settings: `http://web.archive.org/web/20260622235728/https://docs.x.ai/build/settings`
- settings/reference: no snapshot at retrieval
- OSS user guide raw URL: no snapshot at retrieval

## Official source URLs for rule citations

Prefer stable GitHub blob URLs with commit pin where possible:

- User guide config:
  `https://github.com/xai-org/grok-build/blob/main/crates/codegen/xai-grok-pager/docs/user-guide/05-configuration.md`
  (notes must name SOURCE_REV `124d85bc…` and retrieval date)
- Telemetry types:
  `https://github.com/xai-org/grok-build/blob/main/crates/codegen/xai-grok-telemetry/src/config.rs`
- Version/detection:
  `https://github.com/xai-org/grok-build/blob/main/crates/codegen/xai-grok-update/src/version.rs`
- Settings overview: `https://docs.x.ai/build/settings`
- Installer: `https://x.ai/cli/install.sh`
- Open-source announcement: `https://x.ai/news/grok-build-open-source` (2026-07-15)
