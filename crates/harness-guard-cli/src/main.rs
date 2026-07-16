#![forbid(unsafe_code)]

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
compile_error!("harness-guard supports only macOS and Linux");

mod diagnostics;
mod explain;
mod redact;
mod render_json;
mod render_term;

use clap::{CommandFactory, FromArgMatches, Parser, Subcommand, ValueEnum};
use harness_guard_core::discovery::DiscoveryRoot;
use harness_guard_core::harness::{HarnessId, descriptor};
use harness_guard_core::parse::ParseFailure;
use harness_guard_core::scan::{ScanResult, scan_harness};
use harness_guard_rules::loader::{load_rules, ruleset_version};
use harness_guard_rules::report::{Platform, Report, Severity, Status, Summary};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::OnceLock;

/// local, execution-free, per-finding-cited config auditor for
/// privacy/retention/telemetry posture
#[derive(Parser)]
#[command(
    name = "harness-guard",
    version,
    about,
    long_about = None,
    before_help = "Examples:\n  harness-guard scan\n  harness-guard explain codex-history-persist-01\n  harness-guard scan --json"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
    #[command(flatten)]
    color: colorchoice_clap::Color,
}

#[derive(Subcommand)]
enum Cmd {
    /// Scan detected tools' local config (reads files only; never executes tools)
    #[command(
        before_help = "Examples:\n  harness-guard scan\n  harness-guard scan --json\n  harness-guard scan --tool codex --color never"
    )]
    Scan(ScanArgs),
    /// Show detected tools, versions, and config paths — no rule evaluation
    #[command(before_help = "Examples:\n  harness-guard list")]
    List,
    /// Show a rule's full evidence record (works offline)
    #[command(before_help = "Examples:\n  harness-guard explain codex-history-persist-01")]
    Explain { rule_id: String },
    /// Show binary version and ruleset version separately
    #[command(before_help = "Examples:\n  harness-guard version")]
    Version,
    /// Generate shell completions
    #[command(before_help = "Examples:\n  harness-guard completions bash")]
    Completions { shell: clap_complete::Shell },
}

#[derive(clap::Args)]
struct ScanArgs {
    /// Restrict to specific tools (repeatable; default: every detected tool)
    #[arg(long, value_parser = ["codex", "claude-code", "grok-build"])]
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
    let matches = cli_command().get_matches();
    let cli = Cli::from_arg_matches(&matches).unwrap_or_else(|error| error.exit());
    cli.color.write_global();
    match cli.cmd {
        Cmd::Scan(args) => cmd_scan(args),
        Cmd::List => cmd_list(),
        Cmd::Explain { rule_id } => cmd_explain(&rule_id),
        Cmd::Version => cmd_version(),
        Cmd::Completions { shell } => cmd_completions(shell),
    }
}

fn cmd_completions(shell: clap_complete::Shell) -> ExitCode {
    let mut command = cli_command();

    if shell == clap_complete::Shell::Bash {
        let mut output = Vec::new();
        clap_complete::generate(shell, &mut command, "harness-guard", &mut output);

        // clap_complete 4.6 renders a hyphenated root command differently in
        // Bash state assignments and case arms. Normalize the case-arm form so
        // every generated subcommand state is reachable.
        let output = String::from_utf8(output)
            .expect("clap_complete generated non-UTF-8 Bash completions")
            .replace("harness__subcmd__guard", "harness__guard");
        std::io::stdout()
            .write_all(output.as_bytes())
            .expect("failed to write Bash completions");
    } else {
        clap_complete::generate(shell, &mut command, "harness-guard", &mut std::io::stdout());
    }

    ExitCode::SUCCESS
}

fn cli_command() -> clap::Command {
    static VERSION: OnceLock<String> = OnceLock::new();
    let version = VERSION.get_or_init(|| {
        format!(
            "{}\nruleset {}",
            env!("CARGO_PKG_VERSION"),
            ruleset_version()
        )
    });
    Cli::command().version(version.as_str())
}

fn discovery_root_from_env() -> (DiscoveryRoot, Option<PathBuf>) {
    // This is the only ambient-environment boundary. Core always receives an
    // explicit root and therefore cannot fall through to the real home.
    let home = directories::BaseDirs::new().map(|dirs| dirs.home_dir().to_path_buf());
    let home_join = |dotdir: &str, fallback: &str| {
        home.as_ref()
            .map(|home| home.join(dotdir))
            .unwrap_or_else(|| PathBuf::from(fallback))
    };
    let codex_home = std::env::var_os("CODEX_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home_join(".codex", ".codex"));
    // §5.1 fresh-retrieval item (Task 15, retrieved 2026-07-16): checked
    // https://code.claude.com/docs/en/settings and
    // https://code.claude.com/docs/en/env-vars — neither documents an
    // environment variable that relocates the `~/.claude` config directory.
    // (Community reports, e.g. github.com/anthropics/claude-code/issues/33430
    // "Document CLAUDE_CONFIG_DIR", describe an undocumented variable Claude
    // Code itself honors, but the brief's decision rule turns on *official*
    // documentation, which does not exist as of this retrieval.) No override
    // is wired: home stays HOME/.claude, and the descriptor keeps the
    // defensive "$CLAUDE_HOME" token for the (currently unreachable) case
    // where a config path falls outside HOME.
    let claude_home = home_join(".claude", ".claude");
    // §5.1: no Grok home-override env var is assumed either.
    let grok_home = home_join(".grok", ".grok");
    let path_dirs = std::env::var_os("PATH")
        .map(|path| std::env::split_paths(&path).collect())
        .unwrap_or_default();

    (
        DiscoveryRoot {
            codex_home,
            claude_home,
            grok_home,
            path_dirs,
        },
        home,
    )
}

fn cmd_scan(args: ScanArgs) -> ExitCode {
    let (root, home) = discovery_root_from_env();
    let rules = load_rules();

    let selected: Vec<HarnessId> = if args.tool.is_empty() {
        HarnessId::ALL.to_vec()
    } else {
        let mut ids: Vec<HarnessId> = args
            .tool
            .iter()
            .filter_map(|tool| HarnessId::parse(tool))
            .collect();
        ids.sort_unstable();
        ids.dedup();
        ids
    };
    let results: Vec<ScanResult> = selected
        .iter()
        .filter_map(|&harness| scan_harness(&root, harness, &rules))
        .collect();
    let degraded = results.iter().any(|result| result.degraded);

    let report = build_report(&results, home.as_deref(), &root);

    // `selected` is sorted HarnessId order (alphabetical); `results` preserves
    // that order (filter_map only drops undetected harnesses); `report.tools`
    // is a per-element transform of `results` re-sorted the same alphabetical
    // way — so `zip` pairs each failure with ITS OWN tool's redacted path (a
    // fixed `tools.first()` shortcut would mislabel every tool but the first
    // once more than one is selected).
    let parse_failures: Vec<(String, ParseFailure)> = report
        .tools
        .iter()
        .zip(results.iter())
        .filter_map(|(tool, result)| {
            debug_assert_eq!(tool.tool, result.tool_report.tool);
            result.parse_failure.clone().map(|failure| {
                (
                    tool.config_paths.first().cloned().unwrap_or_default(),
                    failure,
                )
            })
        })
        .collect();

    if args.json {
        println!("{}", render_json::render(&report));
    } else {
        let opts = render_term::TermOpts {
            min_severity: match args.min_severity {
                MinSeverity::Info => None,
                MinSeverity::Warning => Some(Severity::Warning),
            },
            quiet: args.quiet,
            verbose: args.verbose,
            requested: selected
                .iter()
                .map(|harness| harness.as_str().to_string())
                .collect(),
        };
        anstream::print!("{}", render_term::render(&report, &opts));
    }
    for (path, failure) in &parse_failures {
        eprint!("{}", diagnostics::report_parse_failure(failure, path));
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

fn cmd_list() -> ExitCode {
    // Detection only: this command never loads or evaluates rules.
    let (root, home) = discovery_root_from_env();
    let mut table = comfy_table::Table::new();
    table.set_header(["tool", "version", "config", "confidence"]);

    for harness in HarnessId::ALL {
        let facts = descriptor(harness);
        let home_detected = harness_guard_core::readfs::probe_directory(root.home(harness))
            != harness_guard_core::readfs::PathProbe::Missing;
        let on_path = harness_guard_core::version::binary_on_path(&root, harness);
        if home_detected || on_path {
            let version = harness_guard_core::version::detect_version(&root, harness)
                .unwrap_or_else(|| "version not detected".to_string());
            let config_path = root.config_path(harness);
            let config = match harness_guard_core::readfs::probe_regular_file(&config_path) {
                harness_guard_core::readfs::PathProbe::Present => redact::redact_config_path(
                    &config_path.to_string_lossy(),
                    home.as_deref(),
                    root.home(harness),
                    facts.home_token,
                    facts.config_file,
                ),
                harness_guard_core::readfs::PathProbe::Missing => "no config file".to_string(),
                harness_guard_core::readfs::PathProbe::Refused => "config path refused".to_string(),
            };
            let confidence = match harness_guard_core::scan::detection_confidence(
                (version != "version not detected").then_some(version.as_str()),
                home_detected,
            ) {
                harness_guard_rules::report::Confidence::Low => "low",
                harness_guard_rules::report::Confidence::Medium => "medium",
                harness_guard_rules::report::Confidence::High => "high",
            };
            table.add_row([
                harness.as_str(),
                version.as_str(),
                config.as_str(),
                confidence,
            ]);
        } else {
            table.add_row([harness.as_str(), "not detected", "-", "-"]);
        }
    }

    anstream::println!("{table}");
    ExitCode::SUCCESS
}

fn cmd_explain(rule_id: &str) -> ExitCode {
    let rules = load_rules();
    match rules.iter().find(|rule| rule.raw().id == rule_id) {
        Some(rule) => {
            anstream::print!("{}", explain::render_rule(rule));
            ExitCode::SUCCESS
        }
        None => {
            let ids: Vec<&str> = rules.iter().map(|rule| rule.raw().id.as_str()).collect();
            match explain::nearest(rule_id, &ids) {
                Some(nearest) => {
                    eprintln!("unknown rule id {rule_id:?} — did you mean {nearest:?}?")
                }
                None => eprintln!("unknown rule id {rule_id:?}"),
            }
            ExitCode::from(2)
        }
    }
}

fn cmd_version() -> ExitCode {
    // These versions are deliberately separate because bundled rules can be
    // revised independently of the binary release.
    println!("harness-guard {}", env!("CARGO_PKG_VERSION"));
    println!("ruleset {}", ruleset_version());
    ExitCode::SUCCESS
}

fn build_report(results: &[ScanResult], home: Option<&Path>, root: &DiscoveryRoot) -> Report {
    let mut tools: Vec<_> = results
        .iter()
        .map(|result| {
            let mut tool = result.tool_report.clone();
            let harness = HarnessId::parse(&tool.tool).expect("scan produces known tool ids");
            let facts = descriptor(harness);
            tool.config_paths = tool
                .config_paths
                .iter()
                .map(|path| {
                    redact::redact_config_path(
                        path,
                        home,
                        root.home(harness),
                        facts.home_token,
                        facts.config_file,
                    )
                })
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
        schema_version: "1.1".to_string(),
        harness_guard_version: env!("CARGO_PKG_VERSION").to_string(),
        ruleset_version: ruleset_version(),
        scanned_at,
        network_requests_made: 0,
        platform: Platform { os: current_os() },
        tools,
        summary,
    }
}

#[cfg(target_os = "macos")]
fn current_os() -> String {
    "macos".to_string()
}

#[cfg(target_os = "linux")]
fn current_os() -> String {
    "linux".to_string()
}
