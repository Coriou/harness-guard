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

/// Rule-relevant key extraction, typed (§5.2). Only the requested dotted key
/// is retained; Str is held only until the engine checks the rule's domain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtractedValue {
    Unset,
    /// Held only until the engine checks the rule's rendering domain.
    Str(String),
    Bool(bool),
    /// Floats and out-of-i64 numbers become Other, never Int.
    Int(i64),
    /// Present but not representable — never rendered.
    Other,
}

pub fn extract_key(document: &toml::Value, dotted_key: &str) -> ExtractedValue {
    let mut current = document;
    for part in dotted_key.split('.') {
        match current {
            toml::Value::Table(table) => match table.get(part) {
                Some(next) => current = next,
                None => return ExtractedValue::Unset,
            },
            // Tables-only traversal: a path hitting an array or scalar is
            // present-but-not-representable (§5.2).
            _ => return ExtractedValue::Other,
        }
    }
    match current {
        toml::Value::String(text) => ExtractedValue::Str(text.clone()),
        toml::Value::Boolean(flag) => ExtractedValue::Bool(*flag),
        toml::Value::Integer(number) => ExtractedValue::Int(*number),
        _ => ExtractedValue::Other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_values_are_extracted() {
        let doc = parse_config(
            "[history]\npersistence = \"none\"\nenabled = true\ndays = 30\nratio = 1.5\n",
        )
        .unwrap();
        assert!(
            matches!(extract_key(&doc, "history.persistence"), ExtractedValue::Str(ref s) if s == "none")
        );
        assert!(matches!(
            extract_key(&doc, "history.enabled"),
            ExtractedValue::Bool(true)
        ));
        assert!(matches!(
            extract_key(&doc, "history.days"),
            ExtractedValue::Int(30)
        ));
        assert!(matches!(
            extract_key(&doc, "history.ratio"),
            ExtractedValue::Other
        ));
    }

    #[test]
    fn key_path_through_an_array_or_scalar_is_other_not_unset() {
        // §5.2: traversal is tables-only; array indexing is unsupported in 0.0.1.
        let doc = parse_config("history = [1, 2]\n").unwrap();
        assert!(matches!(
            extract_key(&doc, "history.persistence"),
            ExtractedValue::Other
        ));
        let doc = parse_config("history = \"flat\"\n").unwrap();
        assert!(matches!(
            extract_key(&doc, "history.persistence"),
            ExtractedValue::Other
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
