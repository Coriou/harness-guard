//! Discovery, bounded reads, safe parsing, and rule evaluation.
//! Everything takes an explicit DiscoveryRoot — no env, no network,
//! no process spawning (clippy + cargo-deny enforced).
#![forbid(unsafe_code)]

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
compile_error!("harness-guard supports only macOS and Linux");

pub mod discovery;
pub mod engine;
pub mod harness;
pub mod parse;
pub mod parse_json;
pub mod readfs;
pub mod scan;
pub mod version;
