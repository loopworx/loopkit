use crate::types::Transition;
use loopkit_core::types::{Config, Diagnostic, FileLocation, Severity};
use std::collections::HashMap;
use std::path::PathBuf;

/// Validate the deskcheck sub-pattern. Fully config-driven via config fields.
/// When disabled (default), returns empty. When enabled, checks:
///   entry_from → deskcheck_state → (feedback_to | forward_to)
pub fn validate(transitions: &[Transition], config: &Config) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    if !config.deskcheck_enabled || config.deskcheck_state.is_empty() {
        return diagnostics;
    }

    let adj = build_adjacency_map(transitions);
    let ds = config.deskcheck_state.as_str();
    if !adj.contains_key(ds) {
        return diagnostics;
    }

    // Rule 1: entry_from must have a direct transition to deskcheck_state
    if !config.deskcheck_entry_from.is_empty()
        && !has_edge(&adj, config.deskcheck_entry_from.as_str(), ds)
    {
        diagnostics.push(Diagnostic {
            severity: Severity::Error,
            code: "state-missing-deskcheck-entry".to_string(),
            message: format!(
                "{} must have a transition to {}",
                config.deskcheck_entry_from, ds
            ),
            location: FileLocation::new(PathBuf::from("skills")),
            help: format!("Add: transition {} → {}", config.deskcheck_entry_from, ds),
        });
    }

    // Rule 2: deskcheck_state must have a path back to feedback_to (bug feedback)
    if !config.deskcheck_feedback_to.is_empty()
        && !has_edge(&adj, ds, config.deskcheck_feedback_to.as_str())
    {
        diagnostics.push(Diagnostic {
            severity: Severity::Error,
            code: "state-missing-deskcheck-feedback".to_string(),
            message: format!(
                "{} must have a transition back to {}",
                ds, config.deskcheck_feedback_to
            ),
            location: FileLocation::new(PathBuf::from("skills")),
            help: format!("Add: transition {} → {}", ds, config.deskcheck_feedback_to),
        });
    }

    // Rule 3: deskcheck_state must have a path to forward_to
    if !config.deskcheck_forward_to.is_empty()
        && !has_edge(&adj, ds, config.deskcheck_forward_to.as_str())
    {
        diagnostics.push(Diagnostic {
            severity: Severity::Error,
            code: "state-missing-deskcheck-completion".to_string(),
            message: format!(
                "{} must have a transition to {}",
                ds, config.deskcheck_forward_to
            ),
            location: FileLocation::new(PathBuf::from("skills")),
            help: format!("Add: transition {} → {}", ds, config.deskcheck_forward_to),
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
    adj.get(from)
        .map(|targets| targets.contains(&to))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(from: &str, to: &str) -> Transition {
        Transition {
            from: from.into(),
            to: to.into(),
            skill: "test".into(),
            defined_in: std::path::PathBuf::from("t/LOOP.md"),
        }
    }

    fn test_config() -> Config {
        Config {
            deskcheck_enabled: true,
            deskcheck_state: "in-deskcheck".into(),
            deskcheck_entry_from: "in-dev".into(),
            deskcheck_feedback_to: "in-dev".into(),
            deskcheck_forward_to: "in-qa".into(),
            ..Config::default()
        }
    }

    #[test]
    fn disabled_when_not_enabled() {
        let transitions = vec![
            t("custom-start", "custom-check"),
            t("custom-check", "custom-done"),
        ];
        let diags = validate(&transitions, &Config::default());
        assert!(diags.is_empty());
    }

    #[test]
    fn valid_deskcheck_pattern_no_diagnostics() {
        let config = test_config();
        let transitions = vec![
            t("in-dev", "in-deskcheck"),
            t("in-deskcheck", "in-dev"),
            t("in-deskcheck", "in-qa"),
        ];
        let diags = validate(&transitions, &config);
        assert!(diags.is_empty());
    }

    #[test]
    fn missing_entry_reports_error() {
        let config = test_config();
        let transitions = vec![t("in-deskcheck", "in-dev"), t("in-deskcheck", "in-qa")];
        let diags = validate(&transitions, &config);
        assert!(diags
            .iter()
            .any(|d| d.code == "state-missing-deskcheck-entry"));
    }

    #[test]
    fn missing_feedback_reports_error() {
        let config = test_config();
        let transitions = vec![t("in-dev", "in-deskcheck"), t("in-deskcheck", "in-qa")];
        let diags = validate(&transitions, &config);
        assert!(diags
            .iter()
            .any(|d| d.code == "state-missing-deskcheck-feedback"));
    }

    #[test]
    fn no_deskcheck_no_diagnostics() {
        let config = test_config();
        let transitions = vec![t("in-dev", "in-qa")];
        let diags = validate(&transitions, &config);
        assert!(diags.is_empty());
    }
}
