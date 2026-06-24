use crate::types::Transition;
use loopkit_core::types::{Config, Diagnostic, Severity, FileLocation};
use std::collections::HashMap;
use std::path::PathBuf;

/// Validate bug-feedback transitions. Fully config-driven.
/// When disabled (default), returns empty. When enabled, checks:
/// - qa_state must have a path back to return_to (bug found during QA)
/// - acceptance_state must have a path back to return_to (PO/UX finds issues)
pub fn validate(transitions: &[Transition], config: &Config) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    if !config.bug_feedback_enabled {
        return diagnostics;
    }

    let adj = build_adjacency_map(transitions);

    if !config.bug_feedback_qa_state.is_empty()
        && adj.contains_key(config.bug_feedback_qa_state.as_str())
        && !has_edge(&adj, config.bug_feedback_qa_state.as_str(), config.bug_feedback_return_to.as_str())
    {
        diagnostics.push(Diagnostic {
            severity: Severity::Error,
            code: "state-missing-bug-feedback".to_string(),
            message: format!(
                "{} must have a transition back to {}",
                config.bug_feedback_qa_state, config.bug_feedback_return_to
            ),
            location: FileLocation::new(PathBuf::from("skills")),
            help: format!("Add: transition {} → {}", config.bug_feedback_qa_state, config.bug_feedback_return_to),
        });
    }

    if !config.bug_feedback_acceptance_state.is_empty()
        && adj.contains_key(config.bug_feedback_acceptance_state.as_str())
        && !has_edge(&adj, config.bug_feedback_acceptance_state.as_str(), config.bug_feedback_return_to.as_str())
    {
        diagnostics.push(Diagnostic {
            severity: Severity::Error,
            code: "state-missing-bug-feedback".to_string(),
            message: format!(
                "{} must have a transition back to {}",
                config.bug_feedback_acceptance_state, config.bug_feedback_return_to
            ),
            location: FileLocation::new(PathBuf::from("skills")),
            help: format!("Add: transition {} → {}", config.bug_feedback_acceptance_state, config.bug_feedback_return_to),
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

    fn test_config() -> Config {
        Config {
            bug_feedback_enabled: true,
            bug_feedback_qa_state: "in-qa".into(),
            bug_feedback_acceptance_state: "in-acceptance".into(),
            bug_feedback_return_to: "in-dev".into(),
            ..Config::default()
        }
    }

    #[test]
    fn disabled_when_not_enabled() {
        let transitions = vec![t("custom-qa", "custom-done")];
        let diags = validate(&transitions, &Config::default());
        assert!(diags.is_empty());
    }

    #[test]
    fn valid_feedback_no_diagnostics() {
        let config = test_config();
        let transitions = vec![
            t("in-qa", "in-dev"),
            t("in-qa", "in-acceptance"),
            t("in-acceptance", "in-dev"),
            t("in-acceptance", "done"),
        ];
        assert!(validate(&transitions, &config).is_empty());
    }

    #[test]
    fn missing_qa_feedback_reports_error() {
        let config = test_config();
        let transitions = vec![t("in-qa", "in-acceptance")];
        let diags = validate(&transitions, &config);
        assert!(diags.iter().any(|d| d.code == "state-missing-bug-feedback"));
    }

    #[test]
    fn missing_acceptance_feedback_reports_error() {
        let config = test_config();
        let transitions = vec![t("in-acceptance", "done")];
        let diags = validate(&transitions, &config);
        assert!(diags.iter().any(|d| d.code == "state-missing-bug-feedback"));
    }

    #[test]
    fn no_qa_or_acceptance_no_diagnostics() {
        let config = test_config();
        assert!(validate(&[t("in-dev", "in-deskcheck")], &config).is_empty());
    }
}
