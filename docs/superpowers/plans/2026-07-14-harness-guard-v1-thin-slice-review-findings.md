# Adversarial review findings — v1 thin-slice plan (2026-07-14)

Companion to [2026-07-14-harness-guard-v1-thin-slice.md](./2026-07-14-harness-guard-v1-thin-slice.md). Verdict: **approve**, zero blocking findings. The nine non-blocking findings below should be applied by the executor at the relevant tasks (each is small and localized).

## Corrections already applied during the corrective round

- Repinned `toml` from nonexistent 0.22.8 to 1.1.2 (verified on crates.io; 0.22.x belongs to `toml_edit`), MSRV 1.85 matches workspace rust-version; confirmed `de::Error` exposes `message()`/`span()` and default recursion limit is 80 (`unbounded` feature stays banned).
- Resolved deny.toml license conflict: crate-pinned MPL-2.0 exception for `option-ext` only (transitive via `directories` → `dirs-sys`); MPL-2.0 stays off the general allowlist; copyleft policy rescoped to project-authored code; negative verification added (removing the exception must fail `cargo deny check licenses`).

## Non-blocking findings for the executor

1. **Task 17 README header:** file header says "Create: README.md" but instruction text says "do NOT create or rewrite" (README exists). Relabel to "Modify (conditionally)" so an agentic executor doesn't overwrite it.
2. **Hostile fixtures golden gap:** symlink-config, oversized, and permission-denied fixtures have committed `expected.json` files that are schema-validated but never compared to actual output (hostile.rs uses inline assertions). Reuse `assert_json_subset` against those `expected.json` files in hostile.rs per spec §10.4.
3. **Detection-confidence casing:** terminal detection line renders `detection_confidence` with `{:?}` (Debug → "High") while §7.1 mockup and `cmd_list` use lowercase "high". Render lowercase explicitly.
4. **Schema hole:** `rule.schema.json`'s finding-outcome conditional only requires severity/confidence *presence*; `"severity": null` would still validate. Add `"severity": {"enum": ["info","warning"]}` to the finding then-branch.
5. **toml pin semantics:** `toml = "1.1.2"` is a caret requirement; recursion-limit-80 and MSRV claims were verified against 1.1.2 specifically. Use `=1.1.2` if "pinned" is meant literally.
6. **Version reporting:** clap's auto `--version` prints only the binary version; spec §6 wants binary + ruleset. The `version` subcommand satisfies it, but consider clap's version string carrying the ruleset CalVer too (or document the deliberate split).
7. **--color placement test:** plan relies on `colorchoice_clap::Color`'s `--color` being `global=true` (it is, in 1.x) for `scan --color never` to parse; add a one-line cli_surface test pinning it.
8. **cargo fmt cadence:** no task runs `cargo fmt` during implementation but Task 17's sweep runs `cargo fmt --all -- --check`; run fmt at each task's commit step to avoid an end-of-slice churn commit.
9. **stale-ruleset empty-message rendering:** an unrecognized value yields `indicative.message == ""` and `obs == None`, producing "Unverified — last-known rule indicates:  Observed: n/a." Add a fallback phrase for that combination (and a fixture pinning it).
