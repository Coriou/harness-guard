//! Discovery, bounded reads, safe parsing, and rule evaluation.
//! Everything takes an explicit DiscoveryRoot — no env, no network,
//! no process spawning (clippy + cargo-deny enforced).
pub mod discovery;
pub mod parse;
pub mod readfs;
