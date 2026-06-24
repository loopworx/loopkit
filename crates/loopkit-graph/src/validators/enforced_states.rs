use crate::types::Transition;
use loopkit_core::types::{Config, Diagnostic, Severity};
use std::collections::HashSet;
use std::path::PathBuf;

pub fn validate(transitions: &[Transition], config: &Config) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    let graph_states: HashSet<&str> = transitions
        .iter()
        .flat_map(|t| [t.from.as_str(), t.to.as_str()])
        .collect();

    for enforced in &config.enforced_states {
        if !graph_states.contains(enforced.name.as_str()) {
            diagnostics.push(Diagnostic {
                severity: Severity::Error,
                code: "state-enforced-missing".to_string(),
                message: format!(
                    "Enforced state '{}' ({}) is not present in any transition graph",
                    enforced.name, enforced.agent
                ),
                location: loopkit_core::types::FileLocation::new(PathBuf::from("skills")),
                help: format!(
                    "Add a transition that includes '{}' as a source or target state. {}",
                    enforced.name, enforced.description
                ),
            });
        }
    }

    diagnostics
}

#[cfg(test)]
mod tests {
    use super::*;
    use loopkit_core::types::{Config, EnforcedState};

    fn make_transition(from: &str, to: &str) -> Transition {
        Transition {
            from: from.into(),
            to: to.into(),
            skill: "test-skill".into(),
            defined_in: std::path::PathBuf::from("test/LOOP.md"),
        }
    }

    fn test_config() -> Config {
        Config {
            enforced_states: vec![
                EnforcedState {
                    name: "in-dev".into(),
                    agent: "developer".into(),
                    description: "".into(),
                },
                EnforcedState {
                    name: "in-qa".into(),
                    agent: "qa-agent".into(),
                    description: "".into(),
                },
                EnforcedState {
                    name: "done".into(),
                    agent: "".into(),
                    description: "".into(),
                },
                EnforcedState {
                    name: "halted-stall".into(),
                    agent: "".into(),
                    description: "".into(),
                },
            ],
            ..Config::default()
        }
    }

    #[test]
    fn all_enforced_states_present_no_diagnostics() {
        let config = test_config();
        let transitions: Vec<Transition> = config
            .enforced_states
            .iter()
            .map(|s| make_transition(&s.name, &s.name))
            .collect();

        let diags = validate(&transitions, &config);
        assert!(
            diags.is_empty(),
            "Expected no diagnostics but got: {:?}",
            diags
        );
    }

    #[test]
    fn missing_enforced_state_emits_error() {
        let config = test_config();
        let transitions = vec![make_transition("in-dev", "in-qa")];
        let diags = validate(&transitions, &config);
        assert!(!diags.is_empty());
        assert!(diags.iter().any(|d| d.code == "state-enforced-missing"));
        assert!(diags.iter().all(|d| d.severity == Severity::Error));
    }

    #[test]
    fn empty_transitions_reports_all_missing() {
        let config = test_config();
        let transitions = vec![];
        let diags = validate(&transitions, &config);
        assert_eq!(diags.len(), config.enforced_states.len());
    }
}
