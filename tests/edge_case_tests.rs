mod common;

use common::*;
use skill_loop_verifier::types::{Diagnostic, FileLocation, HandoffGraph, Repo, Severity, State, Transition};
use skill_loop_verifier::validators::{validate_graph, validate_loop_language, validate_loop_sections};
use std::path::PathBuf;

fn make_repo_with_graph(nodes: Vec<State>, edges: Vec<Transition>) -> Repo {
    Repo {
        root: PathBuf::from("."),
        skills: vec![],
        handoff_graph: HandoffGraph {
            entry_points: nodes.iter().filter(|n| n.is_entry).cloned().collect(),
            nodes,
            edges,
        },
    }
}

// ── Graph: dead-end non-terminal (manually constructed mismatch) ────

#[test]
fn given_non_terminal_node_with_no_outbound_when_validating_graph_then_dead_end_reported() {
    let nodes = vec![
        State { name: "orphan".into(), defined_in: vec![], is_entry: true, is_terminal: false },
    ];
    let repo = make_repo_with_graph(nodes, vec![]);
    let diags = validate_graph(&repo);
    assert!(diags.iter().any(|d| d.code == "graph-dead-end"));
}

// ── Graph: terminal with outbound (manually constructed mismatch) ────

#[test]
fn given_terminal_node_with_outbound_when_validating_graph_then_terminal_outbound_reported() {
    let nodes = vec![
        State { name: "bad-terminal".into(), defined_in: vec![], is_entry: false, is_terminal: true },
        State { name: "next".into(), defined_in: vec![], is_entry: false, is_terminal: false },
    ];
    let edges = vec![
        Transition { from: "bad-terminal".into(), to: "next".into(), trigger: "x".into(), condition: None },
    ];
    let repo = make_repo_with_graph(nodes, edges);
    let diags = validate_graph(&repo);
    assert!(diags.iter().any(|d| d.code == "graph-terminal-with-outbound"));
}

// ── Graph: unreachable non-entry ────────────────────────────────────

#[test]
fn given_state_without_inbound_and_not_entry_when_validating_then_unreachable_reported() {
    let nodes = vec![
        State { name: "unreachable".into(), defined_in: vec![], is_entry: false, is_terminal: false },
        State { name: "done".into(), defined_in: vec![], is_entry: false, is_terminal: true },
    ];
    let edges = vec![
        Transition { from: "unreachable".into(), to: "done".into(), trigger: "x".into(), condition: None },
    ];
    let repo = make_repo_with_graph(nodes, edges);
    let diags = validate_graph(&repo);
    assert!(diags.iter().any(|d| d.code == "graph-unreachable"));
}

// ── Loop language: verb detection in step lists ─────────────────────

#[test]
fn given_loop_with_imperative_step_list_when_validating_then_nonstandard_verbs_warned() {
    let dir = tempfile::TempDir::new().unwrap();
    write_skill(&dir, "dev", "skill-a", &minimal_skill("skill-a", "L2-GUIDED"));
    // LOOP.md with step-by-step imperatives - "Execute" not in standard set
    write_loop(&dir, "dev", "skill-a", "\
## Entry Conditions
ready

## Loop State Schema
| f | t |
|---|
| x | y |

## Single Iteration Step
1. Execute the task
2. verify the result

## Proof of Progress
`test`

## State Transition Rule
transition a → b
  trigger done

## Halt Conditions
halt stall

## Handoff Target
handoff x to agent
");
    let repo = make_repo(&dir);
    let diags = validate_loop_language(&repo);
    assert!(diags.iter().any(|d| d.code == "loop-nonstandard-verb"),
        "expected verb warning, got: {diags:?}");
}

// ── Loop sections: missing section detection in full context ─────────

#[test]
fn given_loop_missing_halt_conditions_when_validating_sections_then_missing_reported() {
    let dir = tempfile::TempDir::new().unwrap();
    write_skill(&dir, "dev", "skill-a", &minimal_skill("skill-a", "L2-GUIDED"));
    // LOOP.md without Halt Conditions
    write_loop(&dir, "dev", "skill-a", "\
## Entry Conditions
ready

## Loop State Schema
| f | t |
|---|
| x | y |

## Single Iteration Step
1. verify x

## Proof of Progress
`test`

## State Transition Rule
transition a → b
  trigger t

## Handoff Target
handoff x to agent
");
    let repo = make_repo(&dir);
    let diags = validate_loop_sections(&repo);
    assert!(diags.iter().any(|d| d.code == "loop-missing-section"));
}
