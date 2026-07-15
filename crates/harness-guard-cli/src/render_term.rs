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
}

pub fn render(report: &Report, opts: &TermOpts) -> String {
    let mut output = String::new();

    if !opts.quiet {
        let _ = writeln!(
            output,
            "harness-guard {} · ruleset {} · scanned {} · no network requests made\n",
            report.harness_guard_version, report.ruleset_version, report.scanned_at
        );
        let _ = writeln!(output, "detected tools");
        if report.tools.is_empty() {
            let _ = writeln!(output, "  ○ codex — not detected");
        }
        for tool in &report.tools {
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
        let _ = writeln!(output);
    }

    for tool in &report.tools {
        render_tool(&mut output, tool, opts);
    }

    let summary = &report.summary;
    let _ = writeln!(
        output,
        "{} warning · {} info · {} unknown · {} stale · {} passed — 0 network requests made",
        summary.warning,
        summary.info,
        summary.unknown,
        summary.stale,
        summary.passed.green()
    );
    let _ = writeln!(
        output,
        "No numeric score is produced — read findings individually."
    );
    output
}

fn render_tool(output: &mut String, tool: &ToolReport, opts: &TermOpts) {
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
