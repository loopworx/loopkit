//! Deterministic loop simulation engine.
//!
//! Checks that the handoff graph is *executable*: every entry point can
//! reach a terminal state within the configured budget, and no transition
//! leads to an unknown state.

use crate::types::{build_adjacency, Diagnostic, FileLocation, Repo, Severity};
use std::collections::{HashMap, HashSet, VecDeque};

/// A simulation violation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Violation {
    /// An entry point cannot reach any terminal state.
    UnreachableTerminal { entry: String },
    /// A transition leads to a state not in the graph.
    TransitionToUnknownState { from: String, to: String },
    /// A self-loop is the only transition from a non-terminal state.
    SelfLoopOnly { state: String },
}

/// Run the full simulation and return diagnostics.
pub fn run_all(repo: &Repo, max_iterations: u32) -> Vec<Diagnostic> {
    let violations = simulate_loop(repo, max_iterations);
    violations
        .into_iter()
        .map(|v| violation_to_diagnostic(v, repo))
        .collect()
}

/// Simulate the loop: BFS from each entry point, confirm reachability
/// to a terminal within `max_iterations` hops.
pub fn simulate_loop(repo: &Repo, max_iterations: u32) -> Vec<Violation> {
    let mut violations = Vec::new();

    let graph = &repo.handoff_graph;
    let adj = build_adjacency(&graph.edges);

    // Collect all known state names
    let all_states: HashSet<String> = graph.nodes.iter().map(|n| n.name.clone()).collect();

    // Check for transitions to unknown states
    for edge in &graph.edges {
        if !all_states.contains(&edge.to) {
            violations.push(Violation::TransitionToUnknownState {
                from: edge.from.clone(),
                to: edge.to.clone(),
            });
        }
    }

    // Terminal states (auto-detected as sinks)
    let terminal_set: HashSet<String> = graph
        .nodes
        .iter()
        .filter(|n| n.is_terminal)
        .map(|n| n.name.clone())
        .collect();

    // If there are no terminals, every entry point is unreachable
    if terminal_set.is_empty() && !graph.entry_points.is_empty() {
        for ep in &graph.entry_points {
            violations.push(Violation::UnreachableTerminal {
                entry: ep.name.clone(),
            });
        }
        return violations;
    }

    // BFS from each entry point, bounded by max_iterations
    for ep in &graph.entry_points {
        if !bfs_reaches_terminal(&ep.name, &adj, &terminal_set, max_iterations) {
            violations.push(Violation::UnreachableTerminal {
                entry: ep.name.clone(),
            });
        }
    }

    // Check for self-loop-only states (non-terminal, only self-loop)
    let outbound: HashMap<&str, Vec<&str>> = {
        let mut map: HashMap<&str, Vec<&str>> = HashMap::new();
        for e in &graph.edges {
            map.entry(&e.from).or_default().push(&e.to);
        }
        map
    };
    for node in &graph.nodes {
        if node.is_terminal {
            continue;
        }
        if let Some(dests) = outbound.get(node.name.as_str()) {
            let has_non_self = dests.iter().any(|d| *d != node.name.as_str());
            if !has_non_self && dests.len() == 1 {
                violations.push(Violation::SelfLoopOnly {
                    state: node.name.clone(),
                });
            }
        }
    }

    violations
}

/// BFS from `start` to any terminal state, bounded by `max_iterations` hops.
fn bfs_reaches_terminal(
    start: &str,
    adj: &HashMap<String, Vec<String>>,
    terminals: &HashSet<String>,
    max_iterations: u32,
) -> bool {
    if terminals.contains(start) {
        return true;
    }

    let mut visited: HashSet<String> = HashSet::new();
    visited.insert(start.to_string());
    let mut frontier: VecDeque<(String, u32)> = VecDeque::new();
    frontier.push_back((start.to_string(), 0));

    while let Some((node, depth)) = frontier.pop_front() {
        if depth >= max_iterations {
            continue;
        }
        if let Some(neighbors) = adj.get(&node) {
            for next in neighbors {
                if visited.contains(next) {
                    continue;
                }
                if terminals.contains(next) {
                    return true;
                }
                visited.insert(next.clone());
                frontier.push_back((next.clone(), depth + 1));
            }
        }
    }

    false
}

fn violation_to_diagnostic(v: Violation, repo: &Repo) -> Diagnostic {
    let skills_path = repo.root.join("skills");
    match v {
        Violation::UnreachableTerminal { entry } => Diagnostic {
            severity: Severity::Error,
            code: "sim-unreachable-terminal".to_string(),
            message: format!(
                "Entry point '{}' cannot reach any terminal state within the configured budget",
                entry
            ),
            location: FileLocation {
                path: skills_path,
                line: None,
                column: None,
            },
            help: "Add transitions so every entry point can reach a terminal (sink) state."
                .to_string(),
        },
        Violation::TransitionToUnknownState { from, to } => Diagnostic {
            severity: Severity::Error,
            code: "sim-unknown-state".to_string(),
            message: format!("Transition '{}' → '{}' targets a state not in the graph", from, to),
            location: FileLocation {
                path: skills_path,
                line: None,
                column: None,
            },
            help: "Ensure all transition targets are valid states defined in some LOOP.md."
                .to_string(),
        },
        Violation::SelfLoopOnly { state } => Diagnostic {
            severity: Severity::Warning,
            code: "sim-self-loop-only".to_string(),
            message: format!(
                "State '{}' has a self-loop as its only transition (may loop forever)",
                state
            ),
            location: FileLocation {
                path: skills_path,
                line: None,
                column: None,
            },
            help: "Add a transition to a different state so the loop can make progress."
                .to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{HandoffGraph, State, Transition};

    fn make_repo(nodes: Vec<State>, edges: Vec<Transition>) -> Repo {
        let entry_points: Vec<State> = nodes.iter().filter(|n| n.is_entry).cloned().collect();
        Repo {
            root: std::path::PathBuf::from("."),
            skills: vec![],
            handoff_graph: HandoffGraph {
                nodes,
                edges,
                entry_points,
            },
        }
    }

    #[test]
    fn given_linear_chain_when_simulating_then_no_violations() {
        let nodes = vec![
            State { name: "start".into(), defined_in: vec![], is_entry: true, is_terminal: false },
            State { name: "done".into(), defined_in: vec![], is_entry: false, is_terminal: true },
        ];
        let edges = vec![
            Transition { from: "start".into(), to: "done".into(), trigger: "go".into(), condition: None },
        ];
        let repo = make_repo(nodes, edges);
        let violations = simulate_loop(&repo, 20);
        assert!(violations.is_empty(), "expected no violations: {violations:?}");
    }

    #[test]
    fn given_entry_cannot_reach_terminal_when_simulating_then_unreachable() {
        let nodes = vec![
            State { name: "start".into(), defined_in: vec![], is_entry: true, is_terminal: false },
            State { name: "island".into(), defined_in: vec![], is_entry: false, is_terminal: true },
        ];
        // No edges — start can't reach island
        let repo = make_repo(nodes, vec![]);
        let violations = simulate_loop(&repo, 20);
        assert!(violations
            .iter()
            .any(|v| matches!(v, Violation::UnreachableTerminal { entry } if entry == "start")));
    }

    #[test]
    fn given_no_terminals_when_simulating_then_all_entries_unreachable() {
        let nodes = vec![
            State { name: "start".into(), defined_in: vec![], is_entry: true, is_terminal: false },
        ];
        let repo = make_repo(nodes, vec![]);
        let violations = simulate_loop(&repo, 20);
        assert!(violations
            .iter()
            .any(|v| matches!(v, Violation::UnreachableTerminal { entry } if entry == "start")));
    }

    #[test]
    fn given_self_loop_only_when_simulating_then_warned() {
        let nodes = vec![
            State { name: "start".into(), defined_in: vec![], is_entry: true, is_terminal: false },
            State { name: "done".into(), defined_in: vec![], is_entry: false, is_terminal: true },
            State { name: "loop".into(), defined_in: vec![], is_entry: false, is_terminal: false },
        ];
        let edges = vec![
            Transition { from: "start".into(), to: "loop".into(), trigger: "enter".into(), condition: None },
            Transition { from: "loop".into(), to: "loop".into(), trigger: "retry".into(), condition: None },
        ];
        let repo = make_repo(nodes, edges);
        let violations = simulate_loop(&repo, 20);
        assert!(violations
            .iter()
            .any(|v| matches!(v, Violation::SelfLoopOnly { state } if state == "loop")));
    }

    #[test]
    fn given_budget_too_small_when_simulating_then_unreachable() {
        // Chain: a → b → c → done (3 hops)
        let nodes = vec![
            State { name: "a".into(), defined_in: vec![], is_entry: true, is_terminal: false },
            State { name: "b".into(), defined_in: vec![], is_entry: false, is_terminal: false },
            State { name: "c".into(), defined_in: vec![], is_entry: false, is_terminal: false },
            State { name: "done".into(), defined_in: vec![], is_entry: false, is_terminal: true },
        ];
        let edges = vec![
            Transition { from: "a".into(), to: "b".into(), trigger: "x".into(), condition: None },
            Transition { from: "b".into(), to: "c".into(), trigger: "x".into(), condition: None },
            Transition { from: "c".into(), to: "done".into(), trigger: "x".into(), condition: None },
        ];
        let repo = make_repo(nodes, edges);
        // Budget of 2 is too small for 3-hop path
        let violations = simulate_loop(&repo, 2);
        assert!(violations
            .iter()
            .any(|v| matches!(v, Violation::UnreachableTerminal { entry } if entry == "a")));
    }

    #[test]
    fn given_budget_sufficient_when_simulating_then_reachable() {
        let nodes = vec![
            State { name: "a".into(), defined_in: vec![], is_entry: true, is_terminal: false },
            State { name: "b".into(), defined_in: vec![], is_entry: false, is_terminal: false },
            State { name: "done".into(), defined_in: vec![], is_entry: false, is_terminal: true },
        ];
        let edges = vec![
            Transition { from: "a".into(), to: "b".into(), trigger: "x".into(), condition: None },
            Transition { from: "b".into(), to: "done".into(), trigger: "x".into(), condition: None },
        ];
        let repo = make_repo(nodes, edges);
        let violations = simulate_loop(&repo, 20);
        assert!(violations.is_empty(), "expected no violations: {violations:?}");
    }
}
