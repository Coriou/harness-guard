//! Discovery, bounded reads, safe parsing, and rule evaluation.
//! Everything takes an explicit DiscoveryRoot — no env, no network,
//! no process spawning (clippy + cargo-deny enforced).
#![forbid(unsafe_code)]

pub mod discovery;
pub mod evaluate;
pub mod parse;
pub mod readfs;
pub mod scan;
pub mod version;
