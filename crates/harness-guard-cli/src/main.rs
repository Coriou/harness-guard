mod redact;
mod render_json;

use clap::{Parser, Subcommand, ValueEnum};
use harness_guard_core::discovery::DiscoveryRoot;
use harness_guard_core::scan::{ScanResult, scan_codex};
use harness_guard_rules::loader::{load_rules, ruleset_version};
use harness_guard_rules::report::{Platform, Report, Severity, Status, Summary};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// local, execution-free, per-finding-cited config auditor for
/// privacy/retention/telemetry posture
#[derive(Parser)]
#[command(name = "harness-guard", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
    #[command(flatten)]
    color: colorchoice_clap::Color,
}

#[derive(Subcommand)]
enum Cmd {
    /// Scan detected tools' local config (reads files only; never executes tools)
    Scan(ScanArgs),
    /// Show detected tools, versions, and config paths — no rule evaluation
    List,
    /// Show a rule's full evidence record (works offline)
    Explain { rule_id: String },
    /// Show binary version and ruleset version separately
    Version,
    /// Generate shell completions
    Completions { shell: clap_complete::Shell },
}

#[derive(clap::Args)]
struct ScanArgs {
    /// Restrict to specific tools (v1: only `codex` is implemented)
    #[arg(long, value_parser = ["codex"])]
    tool: Vec<String>,
    /// Emit the sanitized report as JSON (the schemas/report contract)
    #[arg(long)]
    json: bool,
    /// Lowest finding severity to display (never hides unknown/stale blocks)
    #[arg(long, value_enum, default_value_t = MinSeverity::Info)]
    min_severity: MinSeverity,
    /// Findings at or above this severity set exit code 1
    #[arg(long, value_enum, default_value_t = FailOn::Warning)]
    fail_on: FailOn,
    /// Suppress header, detection block, and banners
    #[arg(long)]
    quiet: bool,
    /// Additionally itemize pass results
    #[arg(long)]
    verbose: bool,
}

#[derive(Clone, Copy, ValueEnum, PartialEq)]
enum MinSeverity {
    Info,
    Warning,
}

#[derive(Clone, Copy, ValueEnum, PartialEq)]
enum FailOn {
    Never,
    Info,
    Warning,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    cli.color.write_global();
    match cli.cmd {
        Cmd::Scan(args) => cmd_scan(args),
        Cmd::List => todo!("Task 13"),
        Cmd::Explain { rule_id: _ } => todo!("Task 13"),
        Cmd::Version => todo!("Task 13"),
        Cmd::Completions { shell: _ } => todo!("Task 13"),
    }
}

fn discovery_root_from_env() -> (DiscoveryRoot, Option<PathBuf>) {
    // This is the only ambient-environment boundary. Core always receives an
    // explicit root and therefore cannot fall through to the real home.
    let home = directories::BaseDirs::new().map(|dirs| dirs.home_dir().to_path_buf());
    let codex_home = std::env::var_os("CODEX_HOME")
        .map(PathBuf::from)
        .or_else(|| home.as_ref().map(|home| home.join(".codex")))
        .unwrap_or_else(|| PathBuf::from(".codex"));
    let path_dirs = std::env::var_os("PATH")
        .map(|path| std::env::split_paths(&path).collect())
        .unwrap_or_default();

    (
        DiscoveryRoot {
            codex_home,
            path_dirs,
        },
        home,
    )
}

fn cmd_scan(args: ScanArgs) -> ExitCode {
    let (root, home) = discovery_root_from_env();
    let rules = load_rules();
    let results: Vec<ScanResult> = scan_codex(&root, &rules).into_iter().collect();
    let degraded = results.iter().any(|result| result.degraded);
    let parse_failures: Vec<_> = results
        .iter()
        .filter_map(|result| result.parse_failure.clone())
        .collect();

    let report = build_report(&results, home.as_deref());

    if args.json {
        println!("{}", render_json::render(&report));
    } else {
        // Task 12 replaces this with the §7.1 terminal renderer.
        render_terminal_stub(&report, &args);
    }
    for failure in &parse_failures {
        // Source text is deliberately never attached or printed. Task 14
        // replaces this structural line/column diagnostic with miette.
        eprintln!(
            "config parse failure at line {:?} col {:?}: {}",
            failure.line, failure.col, failure.message
        );
    }

    let threshold = match args.fail_on {
        FailOn::Never => None,
        FailOn::Info => Some(Severity::Info),
        FailOn::Warning => Some(Severity::Warning),
    };
    let failing = threshold.is_some_and(|threshold| {
        report.tools.iter().any(|tool| {
            tool.findings.iter().any(|finding| {
                finding.status == Status::Finding
                    && finding
                        .severity
                        .is_some_and(|severity| severity >= threshold)
            })
        })
    });

    if degraded {
        ExitCode::from(2)
    } else if failing {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn build_report(results: &[ScanResult], home: Option<&Path>) -> Report {
    let mut tools: Vec<_> = results
        .iter()
        .map(|result| {
            let mut tool = result.tool_report.clone();
            tool.config_paths = tool
                .config_paths
                .iter()
                .map(|path| redact::redact_home(path, home))
                .collect();
            tool
        })
        .collect();
    tools.sort_by(|left, right| left.tool.cmp(&right.tool));
    let summary = Summary::from_tools(&tools);
    let scanned_at = time::OffsetDateTime::now_local()
        .unwrap_or_else(|_| time::OffsetDateTime::now_utc())
        .format(&time::format_description::well_known::Rfc3339)
        .expect("RFC3339 formatting");

    Report {
        schema_version: "1.0".to_string(),
        harness_guard_version: env!("CARGO_PKG_VERSION").to_string(),
        ruleset_version: ruleset_version(),
        scanned_at,
        network_requests_made: 0,
        platform: Platform { os: current_os() },
        tools,
        summary,
    }
}

fn current_os() -> String {
    if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        "linux"
    }
    .to_string()
}

fn render_terminal_stub(report: &Report, _args: &ScanArgs) {
    println!(
        "{} warning · {} info · {} unknown · {} stale · {} passed — 0 network requests made",
        report.summary.warning,
        report.summary.info,
        report.summary.unknown,
        report.summary.stale,
        report.summary.passed
    );
}
