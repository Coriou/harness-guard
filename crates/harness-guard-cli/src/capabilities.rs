//! `capabilities [--json]` — offline, deterministic, zero-network
//! introspection for agents deciding how to invoke the tool (§8.1). Sourced
//! from the same loaded-rules data as `scan`, so the two cannot drift.
use harness_guard_core::harness::HarnessId;
use harness_guard_core::scan::conservative_aggregates;
use harness_guard_rules::loader::{ValidatedRule, load_rules, ruleset_version};
use serde::Serialize;
use std::collections::BTreeSet;

#[derive(Serialize)]
pub struct Capabilities {
    pub schema_version: String,
    pub harness_guard_version: String,
    pub ruleset_version: String,
    pub report_schema_version: String,
    pub tools: Vec<ToolCapabilities>,
    pub commands: Vec<String>,
    pub exit_codes: ExitCodes,
}

#[derive(Serialize)]
pub struct ToolCapabilities {
    pub tool: String,
    pub rules: u32,
    pub categories: Vec<String>,
    pub rules_last_verified_version: Option<String>,
    pub rules_verified_date: Option<String>,
}

#[derive(Serialize)]
pub struct ExitCodes {
    #[serde(rename = "0")]
    pub ok: String,
    #[serde(rename = "1")]
    pub findings: String,
    #[serde(rename = "2")]
    pub degraded: String,
}

pub fn gather() -> Capabilities {
    let rules = load_rules();
    let tools = HarnessId::ALL
        .iter()
        .map(|&harness| {
            let tool_rules: Vec<&ValidatedRule> = rules
                .iter()
                .filter(|rule| rule.raw().tool == harness.as_str())
                .collect();
            let categories: BTreeSet<String> = tool_rules
                .iter()
                .map(|rule| rule.raw().category.clone())
                .collect();
            let (rules_last_verified_version, rules_verified_date) =
                conservative_aggregates(&tool_rules);
            ToolCapabilities {
                tool: harness.as_str().to_string(),
                rules: tool_rules.len() as u32,
                categories: categories.into_iter().collect(),
                rules_last_verified_version,
                rules_verified_date,
            }
        })
        .collect();

    Capabilities {
        schema_version: "1.0".to_string(),
        harness_guard_version: env!("CARGO_PKG_VERSION").to_string(),
        ruleset_version: ruleset_version(),
        report_schema_version: "1.1".to_string(),
        tools,
        commands: [
            "scan",
            "list",
            "explain",
            "version",
            "capabilities",
            "completions",
        ]
        .map(String::from)
        .to_vec(),
        exit_codes: ExitCodes {
            ok: "no findings at/above --fail-on".to_string(),
            findings: "findings at/above --fail-on".to_string(),
            degraded: "degraded or internal/usage error".to_string(),
        },
    }
}

pub fn render_json(capabilities: &Capabilities) -> String {
    serde_json::to_string_pretty(capabilities).expect("capabilities serialize")
}

pub fn render_table(capabilities: &Capabilities) -> String {
    let mut table = comfy_table::Table::new();
    table.set_header(["tool", "rules", "categories", "verified ≤", "verified on"]);
    for tool in &capabilities.tools {
        table.add_row([
            tool.tool.as_str(),
            &tool.rules.to_string(),
            &tool.categories.join(", "),
            tool.rules_last_verified_version.as_deref().unwrap_or("-"),
            tool.rules_verified_date.as_deref().unwrap_or("-"),
        ]);
    }
    format!(
        "harness-guard {} · ruleset {} · report schema {}\n{table}\n",
        capabilities.harness_guard_version,
        capabilities.ruleset_version,
        capabilities.report_schema_version
    )
}
