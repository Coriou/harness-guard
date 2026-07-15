//! The JSON view serializes the same sanitized `Report` used by the terminal
//! view, so the two output paths cannot drift.
use harness_guard_rules::report::Report;

pub fn render(report: &Report) -> String {
    serde_json::to_string_pretty(report).expect("validated Report is always serializable")
}
