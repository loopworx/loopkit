use crate::types::Transition;
use loopkit_core::types::{Diagnostic, Severity, FileLocation};
use std::collections::HashMap;
use std::path::PathBuf;

/// Validate bug-feedback transitions:
/// - in-qa must have a path back to in-dev (bug found during full QA check)
/// - in-acceptance must have a path back to in-dev (PO/UX finds issues)
pub fn validate(transitions: &[Transition]) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let adj = build_adjacency_map(transitions);

    if adj.contains_key("in-qa") && !has_edge(&adj, "in-qa", "in-dev") {
        diagnostics.push(Diagnostic {
            severity: Severity::Error,
            code: "state-missing-bug-feedback".to_string(),
            message: "in-qa must have a transition back to in-dev (bug found — assign back to developer with bug report)".to_string(),
            location: FileLocation::new(PathBuf::from("skills")),
            help: "Add: transition in-qa → in-dev".to_string(),
        });
    }

    if adj.contains_key("in-acceptance") && !has_edge(&adj, "in-acceptance", "in-dev") {
        diagnostics.push(Diagnostic {
            severity: Severity::Error,
            code: "state-missing-bug-feedback".to_string(),
            message: "in-acceptance must have a transition back to in-dev (PO/UX finds issues — bug report)".to_string(),
            location: FileLocation::new(PathBuf::from("skills")),
            help: "Add: transition in-acceptance → in-dev".to_string(),
        });
    }

    diagnostics
}

fn build_adjacency_map(transitions: &[Transition]) -> HashMap<&str, Vec<&str>> {
    let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
    for t in transitions {
        adj.entry(&t.from).or_default().push(&t.to);
    }
    adj
}

fn has_edge(adj: &HashMap<&str, Vec<&str>>, from: &str, to: &str) -> bool {
    adj.get(from).map(|targets| targets.contains(&to)).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(from: &str, to: &str) -> Transition {
        Transition { from: from.into(), to: to.into(), skill: "test".into(), defined_in: std::path::PathBuf::from("t/LOOP.md") }
    }

    #[test]
    fn valid_feedback_no_diagnostics() {
        let transitions = vec![
            t("in-qa", "in-dev"),
            t("in-qa", "in-acceptance"),
            t("in-acceptance", "in-dev"),
            t("in-acceptance", "ready-for-deploy"),
        ];
        assert!(validate(&transitions).is_empty());
    }

    #[test]
    fn missing_qa_feedback_reports_error() {
        let transitions = vec![t("in-qa", "in-acceptance")];
        let diags = validate(&transitions);
        assert!(diags.iter().any(|d| d.code == "state-missing-bug-feedback"));
    }

    #[test]
    fn missing_acceptance_feedback_reports_error() {
        let transitions = vec![t("in-acceptance", "ready-for-deploy")];
        let diags = validate(&transitions);
        assert!(diags.iter().any(|d| d.code == "state-missing-bug-feedback"));
    }

    #[test]
    fn no_qa_or_acceptance_no_diagnostics() {
        assert!(validate(&[t("in-dev", "in-deskcheck")]).is_empty());
    }
}