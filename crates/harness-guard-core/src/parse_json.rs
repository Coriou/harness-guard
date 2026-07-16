//! Safe JSON parsing at TOML-equivalent hostile rigor (§5.2), mirroring
//! parse.rs invariant-for-invariant. Reads arrive through the same readfs
//! layer (no new read path); serde_json's default recursion limit (128) is
//! the overflow backstop; the shared MAX_NESTING_DEPTH = 32 bound is
//! enforced identically to TOML; diagnostics are categorical only because
//! serde_json error text can embed source fragments. Duplicate keys resolve
//! last-value-wins — matching the harness's own parser (test-pinned). Raw
//! text and the parsed Value are dropped inside the scan, as for TOML.
use crate::parse::{ExtractedValue, MAX_NESTING_DEPTH, ParseFailure};

pub fn parse_config_json(text: &str) -> Result<serde_json::Value, ParseFailure> {
    let document: serde_json::Value = serde_json::from_str(text).map_err(|error| {
        let line = error.line();
        let column = error.column();
        ParseFailure {
            line: (line > 0).then_some(line),
            col: (column > 0).then_some(column),
            key_path: None,
            message: categorical_message(&error),
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

/// Only the error CATEGORY reaches diagnostics — never serde_json's Display
/// text, which can quote source content (test-pinned).
fn categorical_message(error: &serde_json::Error) -> String {
    match error.classify() {
        serde_json::error::Category::Eof => "unexpected end of JSON input".to_string(),
        serde_json::error::Category::Syntax => "invalid JSON syntax".to_string(),
        serde_json::error::Category::Data => "JSON structure is not interpretable".to_string(),
        serde_json::error::Category::Io => "JSON could not be read".to_string(),
    }
}

fn value_depth(value: &serde_json::Value) -> usize {
    match value {
        serde_json::Value::Object(map) => {
            1 + map.values().map(value_depth).max().unwrap_or_default()
        }
        serde_json::Value::Array(array) => {
            1 + array.iter().map(value_depth).max().unwrap_or_default()
        }
        _ => 0,
    }
}

/// Objects-only dotted-key traversal (§5.2): array indexing is unsupported in
/// 0.0.1; a path hitting an array or scalar yields Other.
pub fn extract_key_json(document: &serde_json::Value, dotted_key: &str) -> ExtractedValue {
    let mut current = document;
    for part in dotted_key.split('.') {
        match current {
            serde_json::Value::Object(map) => match map.get(part) {
                Some(next) => current = next,
                None => return ExtractedValue::Unset,
            },
            _ => return ExtractedValue::Other,
        }
    }
    match current {
        serde_json::Value::String(text) => ExtractedValue::Str(text.clone()),
        serde_json::Value::Bool(flag) => ExtractedValue::Bool(*flag),
        serde_json::Value::Number(number) => number
            .as_i64()
            .map(ExtractedValue::Int)
            .unwrap_or(ExtractedValue::Other),
        _ => ExtractedValue::Other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_json_parses_and_extracts_typed_values() {
        let doc =
            parse_config_json(r#"{"cleanupPeriodDays": 30, "env": {"FOO": "bar"}, "flag": true}"#)
                .unwrap();
        assert!(matches!(
            extract_key_json(&doc, "cleanupPeriodDays"),
            ExtractedValue::Int(30)
        ));
        assert!(
            matches!(extract_key_json(&doc, "env.FOO"), ExtractedValue::Str(ref s) if s == "bar")
        );
        assert!(matches!(
            extract_key_json(&doc, "flag"),
            ExtractedValue::Bool(true)
        ));
        assert!(matches!(
            extract_key_json(&doc, "absent"),
            ExtractedValue::Unset
        ));
        assert!(matches!(
            extract_key_json(&doc, "flag.nested"),
            ExtractedValue::Other
        ));
    }

    #[test]
    fn floats_and_out_of_i64_numbers_are_other_never_rendered() {
        let doc =
            parse_config_json(r#"{"days": 1.5, "huge": 99999999999999999999999999, "neg": -3}"#)
                .unwrap();
        assert!(matches!(
            extract_key_json(&doc, "days"),
            ExtractedValue::Other
        ));
        assert!(matches!(
            extract_key_json(&doc, "huge"),
            ExtractedValue::Other
        ));
        assert!(matches!(
            extract_key_json(&doc, "neg"),
            ExtractedValue::Int(-3)
        ));
    }

    #[test]
    fn duplicate_keys_are_last_value_wins_pinned() {
        // serde_json keeps the LAST value for a repeated key — the same
        // resolution the harness's own JSON parser applies, so it is the
        // correct observation (§5.2). This test pins the dependency behavior.
        let doc = parse_config_json(r#"{"key": "first", "key": "second"}"#).unwrap();
        assert!(
            matches!(extract_key_json(&doc, "key"), ExtractedValue::Str(ref s) if s == "second")
        );
    }

    #[test]
    fn key_path_hitting_an_array_is_other() {
        let doc = parse_config_json(r#"{"list": [1, 2, 3]}"#).unwrap();
        assert!(matches!(
            extract_key_json(&doc, "list.0"),
            ExtractedValue::Other
        ));
    }

    #[test]
    fn parse_failures_are_categorical_and_never_quote_source() {
        // serde_json Display output can embed source fragments; our failure
        // must strip to the categorical message (§5.2, test-pinned).
        const SECRET: &str = "sk-hostile-secret-value";
        let text = format!(r#"{{"key": {SECRET}}}"#);
        let failure = parse_config_json(&text).unwrap_err();
        assert!(
            !failure.message.contains(SECRET),
            "secret leaked: {}",
            failure.message
        );
        assert!(!failure.message.contains("sk-"), "secret fragment leaked");
        assert!(failure.line.is_some() && failure.col.is_some());
    }

    #[test]
    fn truncated_json_is_categorical_eof() {
        let failure = parse_config_json(r#"{"key": "value"#).unwrap_err();
        assert_eq!(failure.message, "unexpected end of JSON input");
    }

    #[test]
    fn depth_over_shared_bound_is_rejected_and_at_bound_accepted() {
        // Identical bound to TOML — the shared MAX_NESTING_DEPTH constant.
        let over = format!("{}1{}", "{\"a\":".repeat(33), "}".repeat(33));
        let failure = parse_config_json(&over).unwrap_err();
        assert!(failure.message.contains("nesting depth"));
        let at = format!("{}1{}", "{\"a\":".repeat(32), "}".repeat(32));
        assert!(parse_config_json(&at).is_ok());
    }

    #[test]
    fn hostile_deep_nesting_never_panics() {
        // serde_json's default recursion limit (128) is the backstop.
        let hostile = "[".repeat(20_000);
        assert!(parse_config_json(&hostile).is_err());
    }
}
