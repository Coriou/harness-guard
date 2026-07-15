//! Safe TOML parsing (§9): crate recursion limit as backstop, our own
//! depth ≤ 32 bound, line/col computed locally, raw text dropped
//! immediately after extraction. NEVER enable toml's `unbounded` feature.

pub const MAX_NESTING_DEPTH: usize = 32;

/// Value-free parse failure. `message` comes from toml::de::Error::message()
/// (never its Display, which can render source snippets); `key_path` is set
/// only for extraction-stage issues where WE know the key — the toml error
/// type does not expose one.
#[derive(Debug, Clone)]
pub struct ParseFailure {
    pub line: Option<usize>,
    pub col: Option<usize>,
    pub key_path: Option<String>,
    pub message: String,
}

pub fn parse_config(text: &str) -> Result<toml::Value, ParseFailure> {
    let document: toml::Value = toml::from_str(text).map_err(|error| {
        let (line, col) = match error.span() {
            Some(span) => {
                let (line, col) = line_col(text, span.start.min(text.len()));
                (Some(line), Some(col))
            }
            None => (None, None),
        };
        ParseFailure {
            line,
            col,
            key_path: None,
            message: error.message().to_string(),
        }
    })?;

    let depth = value_depth(&document);
    if depth > MAX_NESTING_DEPTH {
        return Err(ParseFailure {
            line: None,
            col: None,
            key_path: None,
            message: format!(
                "nesting depth {depth} exceeds the safety bound of {MAX_NESTING_DEPTH}"
            ),
        });
    }
    Ok(document)
}

fn value_depth(value: &toml::Value) -> usize {
    match value {
        toml::Value::Table(table) => 1 + table.values().map(value_depth).max().unwrap_or_default(),
        toml::Value::Array(array) => 1 + array.iter().map(value_depth).max().unwrap_or_default(),
        _ => 0,
    }
}

/// 1-based line/column for a byte offset. Diagnostics carry these as plain
/// numbers and never retain the source text.
pub fn line_col(source: &str, byte: usize) -> (usize, usize) {
    let clamped = byte.min(source.len());
    let before = &source[..clamped];
    let line = before.bytes().filter(|&byte| byte == b'\n').count() + 1;
    let col = before
        .rfind('\n')
        .map(|newline| clamped - newline)
        .unwrap_or(clamped + 1);
    (line, col)
}

/// Rule-relevant key extraction. Only the requested dotted key is retained.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtractedValue {
    Unset,
    /// Held only until evaluation checks the rule's rendering allowlist.
    Str(String),
    /// Present but not a string — never rendered.
    NonString,
}

pub fn extract_key(document: &toml::Value, dotted_key: &str) -> ExtractedValue {
    let mut current = document;
    for part in dotted_key.split('.') {
        match current.get(part) {
            Some(next) => current = next,
            None => return ExtractedValue::Unset,
        }
    }

    current
        .as_str()
        .map(|value| ExtractedValue::Str(value.to_string()))
        .unwrap_or(ExtractedValue::NonString)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_toml_parses_and_extracts() {
        let doc = parse_config("[history]\npersistence = \"none\"\n").unwrap();
        assert!(matches!(
            extract_key(&doc, "history.persistence"),
            ExtractedValue::Str(ref s) if s == "none"
        ));
    }

    #[test]
    fn missing_key_is_unset() {
        let doc = parse_config("model = \"gpt-5\"\n").unwrap();
        assert!(matches!(
            extract_key(&doc, "history.persistence"),
            ExtractedValue::Unset
        ));
    }

    #[test]
    fn non_string_value_is_nonstring_and_never_carried() {
        let doc = parse_config("[history]\npersistence = 3\n").unwrap();
        assert!(matches!(
            extract_key(&doc, "history.persistence"),
            ExtractedValue::NonString
        ));
    }

    #[test]
    fn malformed_toml_reports_line_col_without_raw_text() {
        let err = parse_config("[history\npersistence = \"none\"\n").unwrap_err();
        assert_eq!(err.line, Some(1));
        assert!(err.col.is_some());
        assert!(!err.message.contains("persistence = "));
    }

    #[test]
    fn depth_over_32_is_rejected() {
        let mut s = String::from("a = ");
        for _ in 0..40 {
            s.push_str("{a = ");
        }
        s.push('1');
        for _ in 0..40 {
            s.push('}');
        }
        s.push('\n');
        let err = parse_config(&s).unwrap_err();
        assert!(err.message.contains("nesting depth"));
    }

    #[test]
    fn depth_at_32_is_accepted() {
        let mut s = String::from("a = ");
        for _ in 0..31 {
            s.push_str("{a = ");
        }
        s.push('1');
        for _ in 0..31 {
            s.push('}');
        }
        s.push('\n');
        assert!(parse_config(&s).is_ok());
    }

    #[test]
    fn hostile_deep_nesting_never_panics() {
        let mut s = String::from("a = ");
        for _ in 0..20_000 {
            s.push_str("[[");
        }
        assert!(parse_config(&s).is_err());
    }

    #[test]
    fn line_col_counts_from_one() {
        assert_eq!(line_col("ab\ncd", 0), (1, 1));
        assert_eq!(line_col("ab\ncd", 3), (2, 1));
        assert_eq!(line_col("ab\ncd", 4), (2, 2));
    }
}
