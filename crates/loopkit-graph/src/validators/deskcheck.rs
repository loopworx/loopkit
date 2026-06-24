use crate::types::Transition;
use loopkit_core::types::{Diagnostic, Severity, FileLocation};
use std::collections::HashMap;
use std::path::PathBuf;

/// Validate the deskcheck sub-pattern: in-dev → in-deskcheck → (in-dev | in-qa)
/// Only fires if in-deskcheck appears in the graph (it's an enforced state,
/// so it always should — but defensively check).
pub fn validate(transitions: &[Transition]) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    let adj = build_adjacency_map(transitions);
    let has_deskcheck = adj.contains_key("in-deskcheck");

    if !has_deskcheck {
        return diagnostics; // deskcheck not used, nothing to check
    }

    // Rule 1: in-dev must have a direct transition to in-deskcheck
    if !has_edge(&adj, "in-dev", "in-deskcheck") {
        diagnostics.push(Diagnostic {
            severity: Severity::Error,
            code: "state-missing-deskcheck-entry".to_string(),
            message: "in-dev must have a transition to in-deskcheck (developer requests QA review per AC)".to_string(),
            location: FileLocation::new(PathBuf::from("skills")),
            help: "Add: transition in-dev → in-deskcheck".to_string(),
        });
    }

    // Rule 2: in-deskcheck must have a path back to in-dev (bug feedback)
    if !has_edge(&adj, "in-deskcheck", "in-dev") {
        diagnostics.push(Diagnostic {
            severity: Severity::Error,
            code: "state-missing-deskcheck-feedback".to_string(),
            message: "in-deskcheck must have a transition back to in-dev (QA returns bug report)".to_string(),
            location: FileLocation::new(PathBuf::from("skills")),
            help: "Add: transition in-deskcheck → in-dev".to_string(),
        });
    }

    // Rule 3: in-deskcheck must have a path to in-qa (all ACs finalized)
    if !has_edge(&adj, "in-deskcheck", "in-qa") {
        diagnostics.push(Diagnostic {
            severity: Severity::Error,
            code: "state-missing-deskcheck-completion".to_string(),
            message: "in-deskcheck must have a transition to in-qa (all ACs finalized)".to_string(),
            location: FileLocation::new(PathBuf::from("skills")),
            help: "Add: transition in-deskcheck → in-qa".to_string(),
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
        Transition {
            from: from.into(), to: to.into(),
            skill: "test".into(),
            defined_in: std::path::PathBuf::from("t/LOOP.md"),
        }
    }

    #[test]
    fn valid_deskcheck_pattern_no_diagnostics() {
        let transitions = vec![
            t("in-dev", "in-deskcheck"),
            t("in-deskcheck", "in-dev"),
            t("in-deskcheck", "in-qa"),
        ];
        let diags = validate(&transitions);
        assert!(diags.is_empty());
    }

    #[test]
    fn missing_entry_reports_error() {
        let transitions = vec![
            t("in-deskcheck", "in-dev"),
            t("in-deskcheck", "in-qa"),
        ];
        let diags = validate(&transitions);
        assert!(diags.iter().any(|d| d.code == "state-missing-deskcheck-entry"));
    }

    #[test]
    fn missing_feedback_reports_error() {
        let transitions = vec![
            t("in-dev", "in-deskcheck"),
            t("in-deskcheck", "in-qa"),
        ];
        let diags = validate(&transitions);
        assert!(diags.iter().any(|d| d.code == "state-missing-deskcheck-feedback"));
    }

    #[test]
    fn no_deskcheck_no_diagnostics() {
        let transitions = vec![t("in-dev", "in-qa")];
        let diags = validate(&transitions);
        assert!(diags.is_empty());
    }
}
