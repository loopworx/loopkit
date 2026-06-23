use crate::types::{Diagnostic, FileLocation, Repo, Severity};
use std::collections::HashSet;

/// Validate the handoff graph invariants:
/// 1. No dead-end non-terminal states (every non-terminal has ≥1 outbound edge)
/// 2. Terminal states have no outgoing edges (sinks)
/// 3. No unreachable non-terminal states
pub fn validate_graph(repo: &Repo) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    let graph = &repo.handoff_graph;

    let terminal_set: HashSet<&str> = graph
        .nodes
        .iter()
        .filter(|s| s.is_terminal)
        .map(|s| s.name.as_str())
        .collect();

    let outbound: HashSet<&str> = graph.edges.iter().map(|e| e.from.as_str()).collect();
    let inbound: HashSet<&str> = graph.edges.iter().map(|e| e.to.as_str()).collect();

    for node in &graph.nodes {
        let name = &node.name;

        // Check: non-terminal states must have at least one outbound edge
        if !node.is_terminal && !outbound.contains(name.as_str()) {
            diags.push(Diagnostic {
                severity: Severity::Error,
                code: "graph-dead-end".to_string(),
                message: format!(
                    "Non-terminal state '{}' has no outgoing transitions (dead end)",
                    name
                ),
                location: FileLocation {
                    path: repo.root.join("skills"),
                    line: None,
                    column: None,
                },
                help: "Add at least one transition from this state to another state or mark it as terminal.".to_string(),
            });
        }

        // Check: terminal states must have no outgoing edges
        if node.is_terminal && outbound.contains(name.as_str()) {
            diags.push(Diagnostic {
                severity: Severity::Error,
                code: "graph-terminal-with-outbound".to_string(),
                message: format!(
                    "Terminal state '{}' has outgoing transitions but should be a sink",
                    name
                ),
                location: FileLocation {
                    path: repo.root.join("skills"),
                    line: None,
                    column: None,
                },
                help: "Terminal states (auto-detected as sinks) should have no outgoing edges. Remove outgoing transitions or reconsider the state's terminal status.".to_string(),
            });
        }
    }

    // Check: no unreachable non-terminal states (that aren't entry points)
    for node in &graph.nodes {
        if !node.is_entry && !inbound.contains(node.name.as_str()) {
            diags.push(Diagnostic {
                severity: Severity::Error,
                code: "graph-unreachable".to_string(),
                message: format!(
                    "State '{}' is not an entry point and has no inbound transitions (unreachable)",
                    node.name
                ),
                location: FileLocation {
                    path: repo.root.join("skills"),
                    line: None,
                    column: None,
                },
                help: "Add a transition into this state from another state, or make it an entry point if it's a valid starting point.".to_string(),
            });
        }
    }

    // Check: every self-loop should have a non-self transition OR be terminal
    for edge in &graph.edges {
        if edge.from == edge.to && !terminal_set.contains(edge.from.as_str()) {
            let has_other_outbound = graph
                .edges
                .iter()
                .any(|e| e.from == edge.from && e.to != edge.from);
            if !has_other_outbound {
                diags.push(Diagnostic {
                    severity: Severity::Warning,
                code: "graph-self-loop-only".to_string(),
                    message: format!(
                        "State '{}' has a self-loop as its only transition (may loop forever)",
                        edge.from
                    ),
                    location: FileLocation {
                        path: repo.root.join("skills"),
                        line: None,
                        column: None,
                    },
                    help: "Add a transition to a different state so the loop can make progress.".to_string(),
                });
            }
        }
    }

    diags
}
