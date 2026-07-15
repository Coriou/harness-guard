//! `explain <rule-id>` renders the complete, bundled evidence record.
//!
//! The command is offline by construction: it only formats validated rule
//! data compiled into the binary.
use harness_guard_rules::loader::ValidatedRule;
use std::fmt::Write;

pub fn render_rule(rule: &ValidatedRule) -> String {
    let rule = rule.raw();
    let mut output = String::new();

    let _ = writeln!(output, "{} — {}\n", rule.id, rule.title);
    let _ = writeln!(output, "why it matters\n  {}\n", rule.why_it_matters);
    let _ = writeln!(
        output,
        "observes\n  {} · key {} · rendered values: {}\n",
        rule.observation.file,
        rule.observation.key,
        rule.observation.allowed_render.join(", ")
    );

    let _ = writeln!(output, "outcomes");
    for outcome in &rule.outcomes {
        let severity = outcome.severity.as_deref().unwrap_or("-");
        let confidence = outcome.confidence.as_deref().unwrap_or("-");
        let _ = writeln!(
            output,
            "  [{}/{}] when {}\n      confidence: {}\n      {}",
            outcome.status, severity, outcome.when, confidence, outcome.message
        );
        if let Some(remediation) = &outcome.remediation {
            let _ = writeln!(output, "      fix: {}", remediation.summary);
            let _ = writeln!(output, "      command: {}", remediation.command);
        }
        if let Some(unknown_reason) = &outcome.unknown_reason {
            let _ = writeln!(output, "      unknown reason: {unknown_reason}");
        }
        if let Some(verify_url) = &outcome.verify_url {
            let _ = writeln!(output, "      verify: {verify_url}");
        }
    }

    let _ = writeln!(output, "\ntested versions");
    for tested in &rule.tested_versions {
        let _ = writeln!(
            output,
            "  {} → {} (verified on {})",
            tested.min, tested.max, tested.verified_on
        );
    }

    let _ = writeln!(output, "\nsources");
    for source in &rule.sources {
        let _ = writeln!(
            output,
            "  {} — {} ({})",
            source.publisher, source.title, source.evidence_class
        );
        let _ = writeln!(output, "    url: {}", source.url);
        let _ = writeln!(
            output,
            "    retrieved: {} · content_hash: {}",
            source.retrieved, source.content_hash
        );
        if let Some(archived_url) = &source.archived_url {
            let _ = writeln!(output, "    archived: {archived_url}");
        }
        if let Some(notes) = &source.notes {
            let _ = writeln!(output, "    notes: {notes}");
        }
    }

    let _ = writeln!(output, "\nlimitations");
    for limitation in &rule.limitations {
        let _ = writeln!(output, "  - {limitation}");
    }
    let _ = writeln!(output, "\nunknown conditions");
    for condition in &rule.unknown_conditions {
        let _ = writeln!(output, "  - {condition}");
    }

    output
}

pub fn nearest<'a>(needle: &str, ids: &[&'a str]) -> Option<&'a str> {
    ids.iter()
        .copied()
        .min_by_key(|candidate| levenshtein(needle, candidate))
}

fn levenshtein(left: &str, right: &str) -> usize {
    let left: Vec<char> = left.chars().collect();
    let right: Vec<char> = right.chars().collect();
    let mut previous: Vec<usize> = (0..=right.len()).collect();

    for (left_index, left_char) in left.iter().enumerate() {
        let mut current = vec![left_index + 1];
        for (right_index, right_char) in right.iter().enumerate() {
            let substitution_cost = usize::from(left_char != right_char);
            current.push(
                (previous[right_index + 1] + 1)
                    .min(current[right_index] + 1)
                    .min(previous[right_index] + substitution_cost),
            );
        }
        previous = current;
    }

    previous[right.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearest_handles_unicode_without_byte_indexing() {
        let ids = ["codex-history-persist-01", "codex-résumé-01"];
        assert_eq!(nearest("codex-resume-01", &ids), Some("codex-résumé-01"));
    }
}
