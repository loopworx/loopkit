use crate::types::{LoopContract, Transition};
use loopkit_core::types::Skill;
use std::collections::{HashMap, HashSet};

/// Build a list of resolved transitions from all skills' loop contracts.
/// Each skill's transitions become graph edges tagged with the skill name.
pub fn build_transitions(
    skills: &[Skill],
    all_handoffs: &HashMap<String, LoopContract>,
) -> Vec<Transition> {
    let mut transitions = Vec::new();

    for skill in skills {
        if let Some(contract) = all_handoffs.get(&skill.name) {
            for rule in &contract.transitions {
                transitions.push(Transition {
                    from: rule.from.clone(),
                    to: rule.to.clone(),
                    skill: skill.name.clone(),
                    defined_in: contract.loop_md_path.clone(),
                });
            }
        }
    }

    transitions
}

/// Build adjacency list from transitions.
pub fn build_adjacency<'a>(transitions: &'a [Transition]) -> HashMap<&'a str, Vec<&'a str>> {
    let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
    for t in transitions {
        adj.entry(&t.from).or_default().push(&t.to);
    }
    adj
}

/// Detect entry points: states with zero inbound edges.
pub fn detect_entry_points(transitions: &[Transition]) -> HashSet<String> {
    let mut has_inbound = HashSet::new();
    let mut all_from = HashSet::new();

    for t in transitions {
        has_inbound.insert(&t.to);
        all_from.insert(&t.from);
    }

    all_from
        .into_iter()
        .filter(|s| !has_inbound.contains(s))
        .map(String::from)
        .collect()
}

/// Detect terminal states: states with zero outbound edges (sinks).
pub fn detect_terminal_states(transitions: &[Transition]) -> HashSet<String> {
    let mut has_outbound = HashSet::new();
    let mut all_to = HashSet::new();

    for t in transitions {
        has_outbound.insert(&t.from);
        all_to.insert(&t.to);
    }

    all_to
        .into_iter()
        .filter(|s| !has_outbound.contains(s))
        .map(String::from)
        .collect()
}

/// Get all unique state names from transitions.
pub fn all_states(transitions: &[Transition]) -> HashSet<String> {
    let mut states = HashSet::new();
    for t in transitions {
        states.insert(t.from.clone());
        states.insert(t.to.clone());
    }
    states
}
