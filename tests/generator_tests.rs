mod common;

use common::*;
use skill_loop_verifier::generator::{emit_checker_ml, emit_generated, shortest_path_to_any};
use skill_loop_verifier::types::{
    HandoffGraph, Repo, State, Transition,
};
use std::collections::{HashMap, HashSet};

fn make_test_repo(nodes: Vec<State>, edges: Vec<Transition>) -> Repo {
    Repo {
        root: std::path::PathBuf::from("."),
        skills: vec![],
        handoff_graph: HandoffGraph {
            nodes,
            edges,
            entry_points: vec![],
        },
    }
}

// ── Generator: shortest path ────────────────────────────────────────

#[test]
fn given_disconnected_graph_when_finding_shortest_path_then_returns_none() {
    let mut adj: HashMap<String, Vec<String>> = HashMap::new();
    adj.insert("a".into(), vec!["b".into()]);
    adj.insert("c".into(), vec!["d".into()]);
    let targets = HashSet::from(["done".to_string()]);
    assert_eq!(shortest_path_to_any("a", &targets, &adj), None);
}

#[test]
fn given_longer_path_when_finding_shortest_then_takes_direct_route() {
    let mut adj: HashMap<String, Vec<String>> = HashMap::new();
    adj.insert("a".into(), vec!["b".into(), "done".into()]);
    adj.insert("b".into(), vec!["done".into()]);
    let targets = HashSet::from(["done".to_string()]);
    // BFS finds a->done directly (length 2), not a->b->done (length 3)
    let path = shortest_path_to_any("a", &targets, &adj).unwrap();
    assert_eq!(path, vec!["a".to_string(), "done".to_string()]);
}

// ── Generator: emit_generated ───────────────────────────────────────

#[test]
fn given_repo_with_two_states_one_transition_when_generating_then_output_contains_states() {
    let nodes = vec![
        State { name: "start".into(), defined_in: vec![], is_entry: true, is_terminal: false },
        State { name: "done".into(), defined_in: vec![], is_entry: false, is_terminal: true },
    ];
    let edges = vec![
        Transition { from: "start".into(), to: "done".into(), trigger: "finish".into(), condition: None },
    ];
    let repo = make_test_repo(nodes, edges);
    let output = emit_generated(&repo);
    assert!(output.contains("S_start"));
    assert!(output.contains("S_done"));
    assert!(output.contains("T_start_done"));
    // Terminal states should be listed
    assert!(output.contains("terminal_states"));
    // Entry points
    assert!(output.contains("entry_points"));
}

#[test]
fn given_repo_with_self_loop_when_generating_then_self_loop_excluded_from_edges() {
    let nodes = vec![
        State { name: "loop".into(), defined_in: vec![], is_entry: true, is_terminal: false },
        State { name: "done".into(), defined_in: vec![], is_entry: false, is_terminal: true },
    ];
    let edges = vec![
        Transition { from: "loop".into(), to: "loop".into(), trigger: "retry".into(), condition: None },
        Transition { from: "loop".into(), to: "done".into(), trigger: "finish".into(), condition: None },
    ];
    let repo = make_test_repo(nodes, edges);
    let output = emit_generated(&repo);
    // Self-loop should not appear as a Transition constructor
    assert!(!output.contains("T_loop_loop"));
    assert!(output.contains("T_loop_done"));
}

#[test]
fn given_repo_with_no_edges_when_generating_then_produces_t_none_fallback() {
    let nodes = vec![
        State { name: "done".into(), defined_in: vec![], is_entry: false, is_terminal: true },
    ];
    let repo = make_test_repo(nodes, vec![]);
    let output = emit_generated(&repo);
    assert!(output.contains("T_none"));
}

#[test]
fn given_repo_with_only_terminal_states_when_generating_then_all_are_terminal() {
    let nodes = vec![
        State { name: "done".into(), defined_in: vec![], is_entry: false, is_terminal: true },
    ];
    let repo = make_test_repo(nodes, vec![]);
    let output = emit_generated(&repo);
    // is_terminal should return true for S_done
    assert!(output.contains("S_done => true"));
}

#[test]
fn given_repo_with_multiple_terminal_states_when_generating_then_all_listed() {
    let nodes = vec![
        State { name: "a".into(), defined_in: vec![], is_entry: true, is_terminal: false },
        State { name: "done".into(), defined_in: vec![], is_entry: false, is_terminal: true },
        State { name: "halted".into(), defined_in: vec![], is_entry: false, is_terminal: true },
    ];
    let edges = vec![
        Transition { from: "a".into(), to: "done".into(), trigger: "ok".into(), condition: None },
    ];
    let repo = make_test_repo(nodes, edges);
    let output = emit_generated(&repo);
    assert!(output.contains("S_done"));
    assert!(output.contains("S_halted"));
}

// ── Generator: emit_checker_ml ──────────────────────────────────────

#[test]
fn given_repo_with_states_when_emitting_checker_then_includes_string_of_state() {
    let nodes = vec![
        State { name: "x".into(), defined_in: vec![], is_entry: false, is_terminal: true },
    ];
    let repo = make_test_repo(nodes, vec![]);
    let output = emit_checker_ml(&repo);
    assert!(output.contains("string_of_state"));
    assert!(output.contains("S_x -> \"x\""));
}

#[test]
fn given_repo_when_emitting_checker_then_includes_all_checks() {
    let nodes = vec![
        State { name: "start".into(), defined_in: vec![], is_entry: true, is_terminal: false },
        State { name: "done".into(), defined_in: vec![], is_entry: false, is_terminal: true },
    ];
    let edges = vec![
        Transition { from: "start".into(), to: "done".into(), trigger: "go".into(), condition: None },
    ];
    let repo = make_test_repo(nodes, edges);
    let output = emit_checker_ml(&repo);
    assert!(output.contains("check_no_dead_ends"));
    assert!(output.contains("check_terminals_are_sinks"));
    assert!(output.contains("check_entry_points"));
    assert!(output.contains("check_reachability"));
}
