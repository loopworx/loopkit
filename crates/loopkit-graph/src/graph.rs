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
pub fn build_adjacency(transitions: &[Transition]) -> HashMap<&str, Vec<&str>> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::LoopContract;
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn t(from: &str, to: &str) -> Transition {
        Transition {
            from: from.into(),
            to: to.into(),
            skill: "s".into(),
            defined_in: PathBuf::from("x"),
        }
    }

    fn make_skill(name: &str) -> Skill {
        Skill {
            name: name.into(),
            level: "L3".into(),
            owner: vec![],
            description: "".into(),
            category: "".into(),
            path: PathBuf::from("skills").join(name),
            skill_md: PathBuf::from("skills").join(name).join("SKILL.md"),
            sections: vec![],
            states: vec![],
        }
    }

    #[test]
    fn build_transitions_empty_skills() {
        let skills: Vec<Skill> = vec![];
        let all_handoffs: HashMap<String, LoopContract> = HashMap::new();
        let transitions = build_transitions(&skills, &all_handoffs);
        assert!(transitions.is_empty());
    }

    #[test]
    fn build_transitions_skill_with_contract() {
        let skill = make_skill("my-skill");
        let mut all_handoffs: HashMap<String, LoopContract> = HashMap::new();
        all_handoffs.insert(
            "my-skill".into(),
            LoopContract {
                skill: "my-skill".into(),
                sections: vec![],
                section_order_valid: true,
                transitions: vec![crate::types::TransitionRule {
                    from: "in-dev".into(),
                    to: "in-qa".into(),
                    trigger: None,
                    handoff_target: None,
                    handoff_agent: None,
                    halt_reason: None,
                    halt_after: None,
                    defined_in: "my-skill".into(),
                }],
                loop_md_path: PathBuf::from("skills/my-skill/LOOP.md"),
            },
        );
        let transitions = build_transitions(&[skill], &all_handoffs);
        assert_eq!(transitions.len(), 1);
        assert_eq!(transitions[0].from, "in-dev");
        assert_eq!(transitions[0].to, "in-qa");
        assert_eq!(transitions[0].skill, "my-skill");
    }

    #[test]
    fn build_transitions_skill_without_contract() {
        let skill = make_skill("orphan");
        let all_handoffs: HashMap<String, LoopContract> = HashMap::new();
        let transitions = build_transitions(&[skill], &all_handoffs);
        assert!(transitions.is_empty());
    }

    #[test]
    fn build_adjacency_multiple_transitions() {
        let transitions = vec![t("a", "b"), t("a", "c"), t("b", "c")];
        let adj = build_adjacency(&transitions);
        assert_eq!(adj.len(), 2);
        assert_eq!(adj.get("a").unwrap(), &vec!["b", "c"]);
        assert_eq!(adj.get("b").unwrap(), &vec!["c"]);
    }

    #[test]
    fn build_adjacency_empty() {
        let adj = build_adjacency(&[]);
        assert!(adj.is_empty());
    }

    #[test]
    fn detect_entry_points_linear_chain() {
        let transitions = vec![t("a", "b"), t("b", "c")];
        let entries = detect_entry_points(&transitions);
        assert_eq!(entries.len(), 1);
        assert!(entries.contains("a"));
    }

    #[test]
    fn detect_entry_points_diamond() {
        let transitions = vec![
            t("start", "a"),
            t("start", "b"),
            t("a", "end"),
            t("b", "end"),
        ];
        let entries = detect_entry_points(&transitions);
        assert_eq!(entries.len(), 1);
        assert!(entries.contains("start"));
    }

    #[test]
    fn detect_entry_points_no_transitions() {
        let entries = detect_entry_points(&[]);
        assert!(entries.is_empty());
    }

    #[test]
    fn detect_terminal_states_linear_chain() {
        let transitions = vec![t("a", "b"), t("b", "c")];
        let terminals = detect_terminal_states(&transitions);
        assert_eq!(terminals.len(), 1);
        assert!(terminals.contains("c"));
    }

    #[test]
    fn detect_terminal_states_multiple_sinks() {
        let transitions = vec![t("a", "b"), t("a", "c")];
        let terminals = detect_terminal_states(&transitions);
        assert_eq!(terminals.len(), 2);
        assert!(terminals.contains("b"));
        assert!(terminals.contains("c"));
    }

    #[test]
    fn detect_terminal_states_no_transitions() {
        let terminals = detect_terminal_states(&[]);
        assert!(terminals.is_empty());
    }

    #[test]
    fn all_states_collects_every_unique_node() {
        let transitions = vec![t("a", "b"), t("b", "c"), t("c", "a")];
        let states = all_states(&transitions);
        assert_eq!(states.len(), 3);
        assert!(states.contains("a"));
        assert!(states.contains("b"));
        assert!(states.contains("c"));
    }

    #[test]
    fn all_states_empty() {
        let states = all_states(&[]);
        assert!(states.is_empty());
    }
}
