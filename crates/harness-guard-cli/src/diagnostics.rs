//! miette is used only for config-parse failures and never attaches source
//! text. Line, column, and key path travel as plain strings so rendering
//! cannot expose a config-value snippet.

use harness_guard_core::parse::ParseFailure;

#[derive(Debug)]
struct ConfigParseDiagnostic {
    message: String,
    location: String,
    path: String,
}

impl std::fmt::Display for ConfigParseDiagnostic {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "config not safely parseable: {}", self.message)
    }
}

impl std::error::Error for ConfigParseDiagnostic {}

impl miette::Diagnostic for ConfigParseDiagnostic {
    fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        Some(Box::new("harness_guard::config_parse"))
    }

    fn help<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        Some(Box::new(format!(
            "{} in {} — fix the file and re-run; raw file content is never shown",
            self.location, self.path
        )))
    }

    // Deliberately no source_code() implementation: without source text,
    // miette cannot render a snippet containing raw config values.
}

pub fn report_parse_failure(failure: &ParseFailure, redacted_path: &str) -> String {
    let location = match (failure.line, failure.col, &failure.key_path) {
        (Some(line), Some(column), Some(key)) => {
            format!("line {line}, column {column}, key {key}")
        }
        (Some(line), Some(column), None) => format!("line {line}, column {column}"),
        (_, _, Some(key)) => format!("key {key}"),
        _ => "unknown location".to_string(),
    };
    let diagnostic = ConfigParseDiagnostic {
        message: failure.message.clone(),
        location,
        path: redacted_path.to_string(),
    };
    format!("{:?}", miette::Report::new(diagnostic))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_key_path_is_plain_text_without_a_source_snippet() {
        let failure = ParseFailure {
            line: Some(4),
            col: Some(9),
            key_path: Some("history.persistence".to_string()),
            message: "value has an unsupported type".to_string(),
        };

        let rendered = report_parse_failure(&failure, "~/.codex/config.toml");
        assert!(rendered.contains("line 4, column 9, key history.persistence"));
        assert!(rendered.contains("~/.codex/config.toml"));
        assert!(!rendered.contains("source code"));
    }
}
