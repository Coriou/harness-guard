use harness_guard_rules::report::{
    Confidence, FindingRecord, Severity, SourceCite, Status, Summary, ToolReport,
};
use harness_guard_rules::schema::Remediation;

#[test]
fn valid_status_matrices_are_accepted() {
    for status in [
        Status::Pass,
        Status::Finding,
        Status::Unknown,
        Status::StaleRuleset,
    ] {
        valid_record(status).validate().unwrap();
    }
}

#[test]
fn finding_without_severity_is_rejected_and_not_miscounted() {
    let mut finding = valid_record(Status::Finding);
    finding.severity = None;
    assert!(finding.validate().is_err());
    assert!(Summary::try_from_tools(&[tool_with(vec![finding])]).is_err());
}

#[test]
fn pass_status_matrix_is_enforced() {
    let mut finding = valid_record(Status::Pass);
    finding.severity = Some(Severity::Info);
    assert_invalid(finding);

    let mut finding = valid_record(Status::Pass);
    finding.confidence = None;
    assert_invalid(finding);

    let mut finding = valid_record(Status::Pass);
    finding.source = None;
    assert_invalid(finding);

    let mut finding = valid_record(Status::Pass);
    finding.remediation = Some(remediation());
    assert_invalid(finding);
}

#[test]
fn finding_status_matrix_is_enforced() {
    let mut finding = valid_record(Status::Finding);
    finding.confidence = None;
    assert_invalid(finding);

    let mut finding = valid_record(Status::Finding);
    finding.source = None;
    assert_invalid(finding);

    let mut finding = valid_record(Status::Finding);
    finding.unknown_reason = Some("wrong status".into());
    assert_invalid(finding);
}

#[test]
fn unknown_status_matrix_is_enforced() {
    let mut finding = valid_record(Status::Unknown);
    finding.severity = Some(Severity::Info);
    assert_invalid(finding);

    let mut finding = valid_record(Status::Unknown);
    finding.confidence = Some(Confidence::Low);
    assert_invalid(finding);

    let mut finding = valid_record(Status::Unknown);
    finding.source = Some(source());
    assert_invalid(finding);

    let mut finding = valid_record(Status::Unknown);
    finding.unknown_reason = None;
    assert_invalid(finding);

    let mut finding = valid_record(Status::Unknown);
    finding.remediation = Some(remediation());
    assert_invalid(finding);
}

#[test]
fn stale_status_matrix_is_enforced() {
    let mut finding = valid_record(Status::StaleRuleset);
    finding.confidence = Some(Confidence::Low);
    assert_invalid(finding);

    let mut finding = valid_record(Status::StaleRuleset);
    finding.source = None;
    assert_invalid(finding);

    let mut finding = valid_record(Status::StaleRuleset);
    finding.stale_reason = None;
    assert_invalid(finding);

    let mut finding = valid_record(Status::StaleRuleset);
    finding.remediation = Some(remediation());
    assert_invalid(finding);
}

#[test]
fn malformed_report_citations_are_rejected() {
    let mut finding = valid_record(Status::Finding);
    finding.source.as_mut().unwrap().url = "http://example.invalid".into();
    assert_invalid(finding);

    let mut finding = valid_record(Status::Finding);
    finding.source.as_mut().unwrap().retrieved = "2026-02-30".into();
    assert_invalid(finding);

    let mut finding = valid_record(Status::Unknown);
    finding.verify_url = Some("http://example.invalid".into());
    assert_invalid(finding);
}

#[test]
fn report_structs_deny_unknown_fields_during_deserialization() {
    let mut json = serde_json::to_value(valid_record(Status::Pass)).unwrap();
    json["raw_config"] = serde_json::json!("must never be accepted");
    assert!(serde_json::from_value::<FindingRecord>(json).is_err());
}

#[test]
fn valid_findings_are_counted_by_status_and_severity() {
    let mut info = valid_record(Status::Finding);
    info.severity = Some(Severity::Info);
    let tools = [tool_with(vec![
        valid_record(Status::Pass),
        info,
        valid_record(Status::Finding),
        valid_record(Status::Unknown),
        valid_record(Status::StaleRuleset),
    ])];

    let summary = Summary::try_from_tools(&tools).unwrap();
    assert_eq!(summary.tools_scanned, 1);
    assert_eq!(summary.warning, 1);
    assert_eq!(summary.info, 1);
    assert_eq!(summary.unknown, 1);
    assert_eq!(summary.stale, 1);
    assert_eq!(summary.passed, 1);
}

fn valid_record(status: Status) -> FindingRecord {
    let mut finding = FindingRecord {
        rule_id: "codex-history-persist-01".into(),
        status,
        severity: None,
        confidence: Some(Confidence::High),
        evidence_class: Some("official-documentation".into()),
        message: "Synthetic report validation test.".into(),
        observation: Some("history.persistence = allowlisted-value".into()),
        remediation: None,
        source: Some(source()),
        valid_from: Some("<=0.144.4".into()),
        valid_until: Some("0.144.4".into()),
        limitations: vec!["Synthetic limitation.".into()],
        unknown_reason: None,
        verify_url: None,
        stale_reason: None,
    };
    match status {
        Status::Pass => {}
        Status::Finding => {
            finding.severity = Some(Severity::Warning);
            finding.remediation = Some(remediation());
        }
        Status::Unknown => {
            finding.confidence = None;
            finding.evidence_class = None;
            finding.observation = None;
            finding.source = None;
            finding.unknown_reason = Some("Synthetic unknown reason.".into());
            finding.verify_url = Some("https://example.invalid/verify".into());
        }
        Status::StaleRuleset => {
            finding.confidence = None;
            finding.remediation = None;
            finding.stale_reason = Some("Synthetic stale reason.".into());
        }
    }
    finding
}

fn source() -> SourceCite {
    SourceCite {
        url: "https://example.invalid/official-documentation".into(),
        retrieved: "2026-07-15".into(),
    }
}

fn remediation() -> Remediation {
    Remediation {
        summary: "Synthetic remediation.".into(),
        command: "synthetic command".into(),
    }
}

fn tool_with(findings: Vec<FindingRecord>) -> ToolReport {
    ToolReport {
        tool: "codex".into(),
        detected_version: Some("0.144.4".into()),
        config_paths: vec!["~/.codex/config.toml".into()],
        detection_confidence: Confidence::High,
        rules_last_verified_version: Some("0.144.4".into()),
        rules_verified_date: Some("2026-07-15".into()),
        version_in_range: true,
        findings,
    }
}

fn assert_invalid(finding: FindingRecord) {
    assert!(finding.validate().is_err());
}
