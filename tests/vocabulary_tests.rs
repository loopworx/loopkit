mod common;

use skill_loop_verifier::types::{
    build_adjacency, detect_entry_points, detect_terminal_states, is_canonical_loop_section,
    is_canonical_skill_section, is_standard_halt_reason, is_standard_verb, validate_halt_reason,
    validate_verb, Transition,
};
use std::collections::HashSet;

// ── Canonical sections ───────────────────────────────────────────────

#[test]
fn given_canonical_loop_section_names_when_checking_then_returns_true() {
    assert!(is_canonical_loop_section("Entry Conditions"));
    assert!(is_canonical_loop_section("Handoff Target"));
}

#[test]
fn given_unknown_heading_when_checking_canonical_loop_section_then_returns_false() {
    assert!(!is_canonical_loop_section("Random Section"));
}

#[test]
fn given_state_model_aliases_when_checking_skill_section_then_all_recognized() {
    assert!(is_canonical_skill_section("State Model"));
    assert!(is_canonical_skill_section("The Loop"));
    assert!(is_canonical_skill_section("Loop States"));
    assert!(is_canonical_skill_section("Loop State"));
    assert!(is_canonical_skill_section("Description"));
    assert!(is_canonical_skill_section("Rules"));
}

#[test]
fn given_non_canonical_heading_when_checking_skill_section_then_returns_false() {
    assert!(!is_canonical_skill_section("Random Notes"));
}

// ── Verb vocabulary ──────────────────────────────────────────────────

#[test]
fn given_standard_verb_when_validating_then_returns_ok() {
    assert!(validate_verb("verify").is_none());
    assert!(validate_verb("write").is_none());
    assert!(validate_verb("commit").is_none());
    assert!(validate_verb("handoff").is_none());
    assert!(validate_verb("gate").is_none());
    assert!(validate_verb("pull").is_none());
    assert!(validate_verb("read").is_none());
    assert!(validate_verb("run").is_none());
    assert!(validate_verb("check").is_none());
    assert!(validate_verb("confirm").is_none());
    assert!(validate_verb("create").is_none());
    assert!(validate_verb("update").is_none());
    assert!(validate_verb("define").is_none());
}

#[test]
fn given_standard_verb_with_different_case_when_checking_then_still_matches() {
    assert!(is_standard_verb("VERIFY"));
    assert!(is_standard_verb("Handoff"));
}

#[test]
fn given_unknown_verb_when_validating_then_returns_error_message() {
    let err = validate_verb("frobnicate").unwrap();
    assert!(err.contains("frobnicate"));
    assert!(err.contains("verify"));
}

#[test]
fn given_common_word_not_a_standard_verb_when_checking_then_returns_false() {
    assert!(!is_standard_verb("frobnicate"));
}

// ── Halt reason vocabulary ───────────────────────────────────────────

#[test]
fn given_standard_halt_reasons_when_validating_then_returns_ok() {
    assert!(validate_halt_reason("stall").is_none());
    assert!(validate_halt_reason("ambiguous").is_none());
    assert!(validate_halt_reason("human-gate").is_none());
    assert!(validate_halt_reason("unsafe").is_none());
    assert!(validate_halt_reason("budget").is_none());
}

#[test]
fn given_standard_halt_reason_with_different_case_when_checking_then_still_matches() {
    assert!(is_standard_halt_reason("STALL"));
    assert!(is_standard_halt_reason("Human-Gate"));
}

#[test]
fn given_unknown_halt_reason_when_validating_then_returns_error_message() {
    let err = validate_halt_reason("tired").unwrap();
    assert!(err.contains("tired"));
    assert!(err.contains("stall"));
}

// ── Graph analysis helpers ──────────────────────────────────────────

#[test]
fn given_transition_with_self_loop_when_building_adjacency_then_self_loop_ignored() {
    let ts = vec![
        Transition { from: "a".into(), to: "a".into(), trigger: "x".into(), condition: None },
        Transition { from: "a".into(), to: "b".into(), trigger: "y".into(), condition: None },
    ];
    let adj = build_adjacency(&ts);
    assert_eq!(adj.get("a"), Some(&vec!["b".to_string()]));
}

#[test]
fn given_2_transitions_same_source_when_building_adjacency_then_both_destinations_present_sorted() {
    let ts = vec![
        Transition { from: "a".into(), to: "c".into(), trigger: "x".into(), condition: None },
        Transition { from: "a".into(), to: "b".into(), trigger: "y".into(), condition: None },
    ];
    let adj = build_adjacency(&ts);
    assert_eq!(adj.get("a"), Some(&vec!["b".to_string(), "c".to_string()]));
}

#[test]
fn given_no_transitions_when_detecting_entry_points_then_returns_empty() {
    assert!(detect_entry_points(&[]).is_empty());
}

#[test]
fn given_linear_chain_when_detecting_entry_points_then_only_first_state_is_entry() {
    let ts = vec![
        Transition { from: "a".into(), to: "b".into(), trigger: "x".into(), condition: None },
        Transition { from: "b".into(), to: "c".into(), trigger: "y".into(), condition: None },
    ];
    let entries = detect_entry_points(&ts);
    assert_eq!(entries, HashSet::from(["a".to_string()]));
}

#[test]
fn given_no_transitions_when_detecting_terminal_states_then_returns_empty() {
    assert!(detect_terminal_states(&[]).is_empty());
}

#[test]
fn given_state_with_outbound_but_no_inbound_edges_when_detecting_terminals_then_is_terminal() {
    let ts = vec![
        Transition { from: "a".into(), to: "b".into(), trigger: "x".into(), condition: None },
    ];
    let terminals = detect_terminal_states(&ts);
    assert!(terminals.contains("b"));
    assert!(!terminals.contains("a"));
}

#[test]
fn given_two_terminal_states_when_detecting_terminals_then_both_found() {
    let ts = vec![
        Transition { from: "a".into(), to: "b".into(), trigger: "x".into(), condition: None },
        Transition { from: "a".into(), to: "c".into(), trigger: "y".into(), condition: None },
    ];
    let terminals = detect_terminal_states(&ts);
    assert_eq!(terminals, HashSet::from(["b".to_string(), "c".to_string()]));
}
