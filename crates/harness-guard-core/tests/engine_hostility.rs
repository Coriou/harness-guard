//! §6.7: for arbitrary ExtractedValue × every bundled rule × every
//! ConfigState × version state, the engine returns a schema-valid record and
//! never renders a string outside the rule's derivable renderings — the
//! hostile-archive-value test generalized.
use harness_guard_core::engine::{ConfigState, evaluate_rule};
use harness_guard_core::parse::{ExtractedValue, ParseFailure};
use harness_guard_core::readfs::RefusalReason;
use harness_guard_rules::loader::load_rules;
use std::collections::BTreeMap;

const HOSTILE_STRINGS: [&str; 8] = [
    "hostile-archive-value",
    "",
    "unset",
    "none\" } { \"injected",
    "{unknown_subject}",
    "$(curl evil)",
    "línea-ünicode-💥",
    "very-long-aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
];

fn hostile_values() -> Vec<ExtractedValue> {
    let mut values = vec![
        ExtractedValue::Unset,
        ExtractedValue::Other,
        ExtractedValue::Bool(true),
        ExtractedValue::Bool(false),
        ExtractedValue::Int(i64::MIN),
        ExtractedValue::Int(-1),
        ExtractedValue::Int(0),
        ExtractedValue::Int(30),
        ExtractedValue::Int(i64::MAX),
    ];
    values.extend(
        HOSTILE_STRINGS
            .iter()
            .map(|s| ExtractedValue::Str(s.to_string())),
    );
    values
}

fn config_states(rule_key: &str, value: &ExtractedValue) -> Vec<ConfigState> {
    let mut parsed = BTreeMap::new();
    parsed.insert(rule_key.to_string(), value.clone());
    vec![
        ConfigState::Missing,
        ConfigState::Unreadable(RefusalReason::PermissionDenied),
        ConfigState::Unreadable(RefusalReason::Symlink),
        ConfigState::Unreadable(RefusalReason::Oversized),
        ConfigState::Unreadable(RefusalReason::NotUtf8),
        ConfigState::Unparseable(ParseFailure {
            line: Some(1),
            col: Some(1),
            key_path: None,
            message: "invalid JSON syntax".to_string(),
        }),
        ConfigState::Parsed(parsed),
    ]
}

#[test]
fn engine_is_total_schema_valid_and_never_leaks() {
    for rule in load_rules() {
        let key = rule.raw().observation.key.clone();
        // Renderings derivable from rule data: domain values, bools, and any
        // in-bounds integer render from the parsed value; hostile strings
        // outside the domain must never appear anywhere in the record.
        for value in hostile_values() {
            for config in config_states(&key, &value) {
                for version in [None, Some("0.144.5"), Some("9.9.9"), Some("not-a-version")] {
                    let finding = evaluate_rule(&rule, &config, version);
                    finding.validate().unwrap_or_else(|error| {
                        panic!("invalid record from rule {} ({error})", rule.raw().id)
                    });
                    let serialized = serde_json::to_string(&finding).unwrap();
                    // Leak-check only when the hostile string actually flowed
                    // through as the observed value (ConfigState::Parsed) —
                    // for Missing/Unreadable/Unparseable states `value` never
                    // reaches the engine, so the record's content is
                    // unrelated to it. "unset" is excluded from the
                    // leak-checked set entirely: it is the engine's own
                    // legitimate unset-outcome rendering token (e.g. "history
                    // .persistence is unset in the user-level config"), not a
                    // leaked raw value, so asserting its absence would be a
                    // false failure rather than a real leak check.
                    if let (ExtractedValue::Str(text), ConfigState::Parsed(_)) = (&value, &config) {
                        let in_domain = rule
                            .raw()
                            .observation
                            .allowed_render
                            .iter()
                            .any(|r| r == text);
                        if !in_domain && !text.is_empty() && text != "unset" {
                            assert!(
                                !serialized.contains(text.as_str()),
                                "rule {} leaked hostile value {text:?}",
                                rule.raw().id
                            );
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn conservative_direction_never_inverts() {
    // §6.3: nothing falls through to pass. Any non-domain value must yield
    // unknown (in range) or stale-ruleset (out of range) — never pass/finding.
    use harness_guard_rules::report::Status;
    for rule in load_rules() {
        let key = rule.raw().observation.key.clone();
        for value in [
            ExtractedValue::Other,
            ExtractedValue::Str("hostile-archive-value".into()),
        ] {
            let mut parsed = BTreeMap::new();
            parsed.insert(key.clone(), value);
            let finding = evaluate_rule(&rule, &ConfigState::Parsed(parsed), Some("0.144.5"));
            assert!(
                matches!(finding.status, Status::Unknown | Status::StaleRuleset),
                "rule {} let a non-domain value reach {:?}",
                rule.raw().id,
                finding.status
            );
        }
    }
}
