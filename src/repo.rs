use crate::parser::handoff::parse_all_handoffs;
use crate::parser::skill::{discover_skills, enrich_skill_states};
use crate::types::{
    detect_entry_points, detect_terminal_states, HandoffGraph, Repo, Skill, State, Transition,
    TransitionRule,
};
use std::collections::HashSet;
use std::path::PathBuf;

impl Repo {
    /// Load a repo from a root directory. Discovers skills, builds the handoff graph
    /// automatically from LOOP.md transition rules (or HANDOFFS.md tables as fallback).
    pub fn from_root(root: PathBuf, skills_dir_name: &str) -> std::io::Result<Self> {
        let skills_dir = root.join(skills_dir_name);

        let mut skills = discover_skills(&skills_dir)?;

        // Parse transition rules from all LOOP.md / HANDOFFS.md files
        let all_rules = parse_all_handoffs(&skills_dir);

        // Enrich each skill with its transition rules
        for skill in &mut skills {
            if let Some(rules) = all_rules.get(&skill.name) {
                skill.transitions = rules.clone();
            }
        }

        // Build the handoff graph from all transition rules
        let handoff_graph = build_graph(&skills, &all_rules);

        // Enrich skills with state information
        let all_state_names: HashSet<String> = handoff_graph
            .nodes
            .iter()
            .map(|s| s.name.clone())
            .collect();
        let entry_names: HashSet<String> = handoff_graph
            .entry_points
            .iter()
            .map(|s| s.name.clone())
            .collect();
        let terminal_names: HashSet<String> = handoff_graph
            .nodes
            .iter()
            .filter(|s| s.is_terminal)
            .map(|s| s.name.clone())
            .collect();

        enrich_skill_states(&mut skills, &all_state_names, &entry_names, &terminal_names);

        Ok(Repo {
            root,
            skills,
            handoff_graph,
        })
    }
}

/// Build the handoff graph from all skill transition rules.
/// Auto-discovers entry points (zero inbound) and terminal states (zero outbound).
fn build_graph(
    skills: &[Skill],
    all_rules: &std::collections::HashMap<String, Vec<TransitionRule>>,
) -> HandoffGraph {
    let mut transitions: Vec<Transition> = Vec::new();
    let mut seen: HashSet<(String, String)> = HashSet::new();

    for skill in skills {
        if let Some(rules) = all_rules.get(&skill.name) {
            for rule in rules {
                let pair = (rule.from.clone(), rule.to.clone());
                if seen.insert(pair.clone()) {
                    transitions.push(Transition {
                        from: rule.from.clone(),
                        to: rule.to.clone(),
                        trigger: rule.trigger.clone().unwrap_or_default(),
                        condition: None,
                    });
                }
            }
        }
    }

    let entry_set = detect_entry_points(&transitions);
    let terminal_set = detect_terminal_states(&transitions);

    // Collect all unique state names
    let mut all_state_names: HashSet<String> = HashSet::new();
    for t in &transitions {
        all_state_names.insert(t.from.clone());
        all_state_names.insert(t.to.clone());
    }

    let mut nodes: Vec<State> = all_state_names
        .into_iter()
        .map(|name| State {
            is_entry: entry_set.contains(&name),
            is_terminal: terminal_set.contains(&name),
            defined_in: Vec::new(),
            name,
        })
        .collect();
    nodes.sort_by(|a, b| a.name.cmp(&b.name));

    let entry_points: Vec<State> = nodes
        .iter()
        .filter(|s| s.is_entry)
        .cloned()
        .collect();

    HandoffGraph {
        nodes,
        edges: transitions,
        entry_points,
    }
}
