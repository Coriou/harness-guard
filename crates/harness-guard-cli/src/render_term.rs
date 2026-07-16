//! §7.1 terminal view. The terminal and JSON views share the same sanitized
//! `Report` structs, so raw configuration cannot enter either renderer.

use harness_guard_rules::report::{
    Confidence, FindingRecord, Report, Severity, Status, Summary, ToolReport,
};
use owo_colors::OwoColorize;
use std::fmt::Write;

pub struct TermOpts {
    /// `None` shows every finding severity. This option never filters unknown
    /// or stale-ruleset records.
    pub min_severity: Option<Severity>,
    pub quiet: bool,
    pub verbose: bool,
    /// Alphabetical harness ids the scan covered. Anything here that is not
    /// in `report.tools` renders as an explicit "not detected" row.
    pub requested: Vec<String>,
}

pub fn render(report: &Report, opts: &TermOpts) -> String {
    let mut output = String::new();

    if !opts.quiet {
        let network_summary = if report.network_requests_made == 0 {
            "no network requests made".to_string()
        } else {
            format!("{} network requests made", report.network_requests_made)
        };
        let _ = writeln!(
            output,
            "harness-guard {} · ruleset {} · scanned {} · {network_summary}\n",
            report.harness_guard_version, report.ruleset_version, report.scanned_at
        );
        let _ = writeln!(output, "detected tools");
        let mut detected: std::collections::BTreeMap<&str, &ToolReport> = report
            .tools
            .iter()
            .map(|tool| (tool.tool.as_str(), tool))
            .collect();
        for requested in &opts.requested {
            match detected.remove(requested.as_str()) {
                Some(tool) => {
                    let version = tool
                        .detected_version
                        .as_deref()
                        .unwrap_or("version not detected");
                    let path = tool
                        .config_paths
                        .first()
                        .map(String::as_str)
                        .unwrap_or("no config file");
                    let _ = writeln!(
                        output,
                        "  ● {} {} · config {} · confidence {}",
                        tool.tool,
                        version,
                        path,
                        confidence_label(tool.detection_confidence)
                    );
                }
                None => {
                    let _ = writeln!(output, "  ○ {requested} — not detected");
                }
            }
        }
        let _ = writeln!(output);
    }

    for tool in &report.tools {
        render_tool(&mut output, tool, opts);
    }

    let summary = &report.summary;
    let _ = writeln!(
        output,
        "{} warning · {} info · {} unknown · {} stale · {} passed — {} network requests made",
        summary.warning,
        summary.info,
        summary.unknown,
        summary.stale,
        summary.passed.green(),
        report.network_requests_made
    );
    let _ = writeln!(
        output,
        "No numeric score is produced — read findings individually."
    );
    output
}

fn render_tool(output: &mut String, tool: &ToolReport, opts: &TermOpts) {
    if tool.findings.is_empty() {
        // A harness can be detected yet carry zero bundled rules (grok-build
        // until its rule work package lands, §7.3).
        // Falling through to the normal summary line below would print
        // "0 warning · 0 info · 0 unknown · 0 stale · 0 passed", which reads
        // as a clean, verified audit — it is not: nothing was evaluated.
        if !opts.quiet {
            let version = tool
                .detected_version
                .as_deref()
                .unwrap_or("version not detected");
            let _ = writeln!(
                output,
                "{} {} — no rules bundled for this tool yet",
                tool.tool, version
            );
            let _ = writeln!(
                output,
                "{}\n",
                "0 findings because there is nothing to evaluate — this is not a clean audit"
                    .dimmed()
            );
        }
        return;
    }

    if !opts.quiet {
        let version = tool
            .detected_version
            .as_deref()
            .unwrap_or("version not detected");
        let last_verified = tool.rules_last_verified_version.as_deref().unwrap_or("?");
        let verified_date = tool.rules_verified_date.as_deref().unwrap_or("?");
        let _ = writeln!(
            output,
            "{} {} — rules verified ≤{} · {}",
            tool.tool, version, last_verified, verified_date
        );

        if !tool.version_in_range {
            // This is a hint, not an error: the finding itself remains visible
            // below as an explicitly unverified stale-ruleset result.
            let detected = tool
                .detected_version
                .as_deref()
                .map(|version| format!("you have {version}"))
                .unwrap_or_else(|| "version not detected".to_string());
            let banner = format!(
                "rules verified ≤{last_verified} — {detected}, showing last-known rules as unverified"
            );
            let _ = writeln!(output, "{}", banner.dimmed());
        }

        let counts = Summary::from_tools(std::slice::from_ref(tool));
        let _ = writeln!(
            output,
            "{} warning · {} info · {} unknown · {} stale · {} passed\n",
            counts.warning,
            counts.info,
            counts.unknown,
            counts.stale,
            counts.passed.green()
        );
    }

    for finding in &tool.findings {
        render_finding(output, finding, opts);
    }
}

fn render_finding(output: &mut String, finding: &FindingRecord, opts: &TermOpts) {
    match finding.status {
        Status::Finding => {
            if let (Some(minimum), Some(severity)) = (opts.min_severity, finding.severity) {
                if severity < minimum {
                    return;
                }
            }
            let label = match finding.severity {
                Some(Severity::Warning) => format!("{}", "!! WARNING:".red().bold()),
                _ => "-- INFO:".to_string(),
            };
            let _ = writeln!(output, "{label} {}", finding.message);
            render_common_lines(output, finding);
        }
        Status::Unknown => {
            let _ = writeln!(output, "{} {}", "?? UNKNOWN:".cyan(), finding.message);
            if let Some(reason) = &finding.unknown_reason {
                let _ = writeln!(output, "   reason: {reason}");
            }
            if let Some(verify_url) = &finding.verify_url {
                let _ = writeln!(output, "   verify: {verify_url}");
            }
            let _ = writeln!(output, "   = harness-guard explain {}\n", finding.rule_id);
        }
        Status::StaleRuleset => {
            let _ = writeln!(
                output,
                "{} {}",
                "~ UNVERIFIED (stale ruleset):".yellow().dimmed(),
                finding.message
            );
            if let Some(reason) = &finding.stale_reason {
                let _ = writeln!(output, "   reason: {reason}");
            }
            render_common_lines(output, finding);
        }
        Status::Pass => {
            if !opts.verbose {
                return;
            }
            let _ = writeln!(output, "{} {}", "ok PASS:".green(), finding.message);
            render_common_lines(output, finding);
        }
    }
}

fn render_common_lines(output: &mut String, finding: &FindingRecord) {
    let _ = writeln!(
        output,
        "   rule {} · {}",
        finding.rule_id,
        finding.evidence_class.as_deref().unwrap_or("unverified")
    );
    if let Some(observation) = &finding.observation {
        let _ = writeln!(output, "   observed: {observation}");
    }
    if let Some(remediation) = &finding.remediation {
        let mut lines = remediation.command.lines();
        if let Some(first) = lines.next() {
            let _ = writeln!(output, "   fix: {first}");
            for line in lines {
                let _ = writeln!(output, "        {line}");
            }
        }
    }
    if let Some(source) = &finding.source {
        let _ = writeln!(output, "   = source: {} ({})", source.url, source.retrieved);
    }
    let _ = writeln!(output, "   = harness-guard explain {}\n", finding.rule_id);
}

fn confidence_label(confidence: Confidence) -> &'static str {
    match confidence {
        Confidence::Low => "low",
        Confidence::Medium => "medium",
        Confidence::High => "high",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use harness_guard_rules::report::Platform;

    #[test]
    fn network_wording_comes_from_the_report() {
        let report = Report {
            schema_version: "1.0".to_string(),
            harness_guard_version: "0.1.0".to_string(),
            ruleset_version: "test".to_string(),
            scanned_at: "2026-07-15T00:00:00Z".to_string(),
            network_requests_made: 3,
            platform: Platform {
                os: "test".to_string(),
            },
            tools: vec![],
            summary: Summary {
                tools_scanned: 0,
                warning: 0,
                info: 0,
                unknown: 0,
                stale: 0,
                passed: 0,
            },
        };

        let output = render(
            &report,
            &TermOpts {
                min_severity: None,
                quiet: false,
                verbose: false,
                requested: vec![],
            },
        );

        assert_eq!(output.matches("3 network requests made").count(), 2);
        assert!(!output.contains("no network requests made"));
        assert!(!output.contains("0 network requests made"));
    }

    #[test]
    fn zero_rule_tool_never_renders_as_a_clean_audit() {
        let tool = ToolReport {
            tool: "claude-code".to_string(),
            detected_version: Some("2.1.202".to_string()),
            config_paths: vec!["~/.claude/settings.json".to_string()],
            detection_confidence: Confidence::Medium,
            rules_last_verified_version: None,
            rules_verified_date: None,
            version_in_range: false,
            findings: vec![],
        };
        let report = Report {
            schema_version: "1.1".to_string(),
            harness_guard_version: "0.1.0".to_string(),
            ruleset_version: "test".to_string(),
            scanned_at: "2026-07-16T00:00:00Z".to_string(),
            network_requests_made: 0,
            platform: Platform {
                os: "test".to_string(),
            },
            tools: vec![tool],
            summary: Summary {
                tools_scanned: 1,
                warning: 0,
                info: 0,
                unknown: 0,
                stale: 0,
                passed: 0,
            },
        };

        let output = render(
            &report,
            &TermOpts {
                min_severity: None,
                quiet: false,
                verbose: false,
                requested: vec!["claude-code".to_string()],
            },
        );

        assert!(output.contains("no rules bundled for this tool yet"));
        assert!(output.contains("this is not a clean audit"));
        assert!(
            !output.contains("rules verified ≤"),
            "zero-rule tool must not render the normal rules-verified header"
        );
    }
}
