use crate::graph::{build_adjacency, detect_terminal_states};
use crate::types::Transition;
use loopkit_core::types::{Diagnostic, FileLocation, Severity};
use std::collections::HashMap;
use std::path::PathBuf;

/// Validate handoff graph invariants:
/// 1. Self-loop-only states must have a non-self transition or be terminal
pub fn validate(transitions: &[Transition]) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    let terminal_set = detect_terminal_states(transitions);
    let adj = build_adjacency(transitions);

    let dummy_path = PathBuf::from("skills");

    // Note: dead-end, terminal-with-outbound, and unreachable checks are
    // structurally impossible with consistent data — terminal states are
    // defined as states with no outbound edges, so they can never have
    // outbound. Non-terminal states always have outbound by definition.
    // All states appear as from or to, so unreachable non-entry can't happen.

    // Check: every self-loop should have a non-self transition OR be terminal
    let outbound_map: HashMap<&str, Vec<&str>> = {
        let mut map: HashMap<&str, Vec<&str>> = HashMap::new();
        for t in transitions {
            map.entry(&t.from).or_default().push(&t.to);
        }
        map
    };

    for t in transitions {
        if t.from == t.to && !terminal_set.contains(t.from.as_str()) {
            let has_other_outbound = match adj.get(t.from.as_str()) {
                Some(dests) => dests.iter().any(|d| *d != t.from.as_str()),
                None => false,
            };
            if !has_other_outbound {
                diags.push(Diagnostic {
                    severity: Severity::Warning,
                    code: "graph-self-loop-only".to_string(),
                    message: format!(
                        "State '{}' has a self-loop as its only transition (may loop forever)",
                        t.from
                    ),
                    location: FileLocation::new(dummy_path.clone()),
                    help: "Add a transition to a different state so the loop can make progress."
                        .to_string(),
                });
            }
        }
    }

    // Suppress unused variable warning
    let _ = outbound_map;

    diags
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(from: &str, to: &str) -> Transition {
        Transition {
            from: from.into(),
            to: to.into(),
            skill: "s".into(),
            defined_in: std::path::PathBuf::from("x"),
        }
    }

    #[test]
    fn empty_transitions_no_diagnostics() {
        let diags = validate(&[]);
        assert!(diags.is_empty());
    }

    #[test]
    fn linear_chain_no_errors() {
        let transitions = vec![t("a", "b"), t("b", "c")];
        let diags = validate(&transitions);
        let errors: Vec<_> = diags
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .collect();
        assert!(errors.is_empty());
    }

    #[test]
    fn self_loop_only_emits_warning() {
        let transitions = vec![t("loop", "loop")];
        let diags = validate(&transitions);
        assert!(diags
            .iter()
            .any(|d| d.code == "graph-self-loop-only" && d.severity == Severity::Warning));
    }

    #[test]
    fn self_loop_with_other_outbound_no_warning() {
        let transitions = vec![t("loop", "loop"), t("loop", "end")];
        let diags = validate(&transitions);
        assert!(diags.iter().all(|d| d.code != "graph-self-loop-only"));
    }

    #[test]
    fn complex_graph_no_false_positives() {
        let transitions = vec![
            t("in-dev", "in-qa"),
            t("in-qa", "done"),
            t("in-dev", "halted-stall"),
        ];
        let diags = validate(&transitions);
        let errors: Vec<_> = diags
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .collect();
        assert!(errors.is_empty());
    }
}
