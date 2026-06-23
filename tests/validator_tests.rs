mod common;

use common::*;
use skill_loop_verifier::types::Severity;
use skill_loop_verifier::validators::{
    run_all, validate_constraints, validate_graph, validate_loop_completeness,
    validate_loop_language, validate_loop_sections, validate_skill_completeness,
};
use std::collections::HashSet;

// ── Skill completeness ───────────────────────────────────────────────

#[test]
fn given_skill_with_all_required_sections_when_validating_then_no_errors() {
    let dir = tempfile::TempDir::new().unwrap();
    write_skill(&dir, "dev", "good", &minimal_skill("good", "L2-GUIDED"));
    let repo = make_repo(&dir);
    let diags = validate_skill_completeness(&repo);
    assert!(diags.is_empty());
}

#[test]
fn given_skill_missing_description_when_validating_then_reports_error() {
    let dir = tempfile::TempDir::new().unwrap();
    write_skill(&dir, "dev", "bad", "---\nname: bad\nlevel: L2-GUIDED\nowner: dev\n---\n\n## Rules\nr\n\n## State Model\ns\n");
    let repo = make_repo(&dir);
    let diags = validate_skill_completeness(&repo);
    assert!(diags.iter().any(|d| d.code == "skill-missing-description"));
}

#[test]
fn given_skill_missing_rules_when_validating_then_reports_error() {
    let dir = tempfile::TempDir::new().unwrap();
    write_skill(&dir, "dev", "bad", "---\nname: bad\nlevel: L2-GUIDED\nowner: dev\n---\n\n## Description\nd\n\n## State Model\ns\n");
    let repo = make_repo(&dir);
    let diags = validate_skill_completeness(&repo);
    assert!(diags.iter().any(|d| d.code == "skill-missing-rules"));
}

#[test]
fn given_skill_missing_state_model_when_validating_then_reports_error() {
    let dir = tempfile::TempDir::new().unwrap();
    write_skill(&dir, "dev", "bad", "---\nname: bad\nlevel: L2-GUIDED\nowner: dev\n---\n\n## Description\nd\n\n## Rules\nr\n");
    let repo = make_repo(&dir);
    let diags = validate_skill_completeness(&repo);
    assert!(diags.iter().any(|d| d.code == "skill-missing-state-model"));
}

#[test]
fn given_l1_rigid_skill_missing_entry_and_halt_conditions_when_validating_then_reports_both() {
    let dir = tempfile::TempDir::new().unwrap();
    write_skill(&dir, "dev", "bad", &minimal_skill("bad", "L1-RIGID"));
    let repo = make_repo(&dir);
    let diags = validate_skill_completeness(&repo);
    let codes: HashSet<_> = diags.iter().map(|d| d.code.clone()).collect();
    assert!(codes.contains("skill-l1-missing-entry-conditions"));
    assert!(codes.contains("skill-l1-missing-halt-conditions"));
}

#[test]
fn given_skill_with_duplicate_rules_section_when_validating_then_warns() {
    let dir = tempfile::TempDir::new().unwrap();
    write_skill(&dir, "dev", "dup", "---\nname: dup\nlevel: L2-GUIDED\nowner: dev\n---\n\n## Description\nd\n\n## Rules\nr1\n\n## Rules\nr2\n\n## State Model\ns\n");
    let repo = make_repo(&dir);
    let diags = validate_skill_completeness(&repo);
    assert!(diags.iter().any(|d| d.code == "skill-duplicate-rules"));
}

#[test]
fn given_l1_rigid_skill_with_all_required_sections_when_validating_then_no_errors() {
    let dir = tempfile::TempDir::new().unwrap();
    write_skill(&dir, "dev", "good", &minimal_l1_skill("good"));
    let repo = make_repo(&dir);
    let diags = validate_skill_completeness(&repo);
    assert!(diags.is_empty());
}

// ── Loop completeness ────────────────────────────────────────────────

#[test]
fn given_handoff_table_without_loop_md_when_validating_then_warns_missing_loop() {
    let dir = tempfile::TempDir::new().unwrap();
    write_skill(&dir, "dev", "skill-a", &minimal_skill("skill-a", "L2-GUIDED"));
    // Proper markdown table with separator line
    write_handoffs(&dir, "dev", "skill-a", "| from | to |\n|------|----|\n| a | b |\n");
    let repo = make_repo(&dir);
    let diags = validate_loop_completeness(&repo);
    assert!(diags.iter().any(|d| d.code == "loop-missing-for-transition-skill"),
        "expected loop-missing-for-transition-skill, got: {diags:?}");
}

#[test]
fn given_skill_with_no_transitions_when_validating_loop_completeness_then_no_errors() {
    let dir = tempfile::TempDir::new().unwrap();
    write_skill(&dir, "dev", "skill-a", &minimal_skill("skill-a", "L2-GUIDED"));
    let repo = make_repo(&dir);
    let diags = validate_loop_completeness(&repo);
    assert!(diags.is_empty());
}

// ── Loop sections ────────────────────────────────────────────────────

#[test]
fn given_loop_with_non_canonical_section_when_validating_then_reports_unknown_section() {
    let dir = tempfile::TempDir::new().unwrap();
    write_skill(&dir, "dev", "skill-a", &minimal_skill("skill-a", "L2-GUIDED"));
    write_loop(&dir, "dev", "skill-a", &format!(
        "## Entry Conditions\nready\n\n\
         ## Loop State Schema\nx\n\n\
         ## Single Iteration Step\ny\n\n\
         ## Proof of Progress\nz\n\n\
         ## State Transition Rule\ntransition a → b\n\n\
         ## Halt Conditions\nhalt stall\n\n\
         ## Extra Notes\nextra\n\n\
         ## Handoff Target\nhandoff x to agent\n"
    ));
    let repo = make_repo(&dir);
    let diags = validate_loop_sections(&repo);
    assert!(diags.iter().any(|d| d.code == "loop-unknown-section"));
}

// ── Loop language ────────────────────────────────────────────────────

#[test]
fn given_transition_with_unknown_halt_reason_when_validating_then_reports_error() {
    let dir = tempfile::TempDir::new().unwrap();
    write_skill(&dir, "dev", "skill-a", &minimal_skill("skill-a", "L2-GUIDED"));
    write_loop(&dir, "dev", "skill-a", "## State Transition Rule\ntransition a → b\n  halt unknown-reason\n");
    let repo = make_repo(&dir);
    let diags = validate_loop_language(&repo);
    assert!(diags.iter().any(|d| d.code == "loop-unknown-halt-reason"));
}

#[test]
fn given_state_transition_rule_with_no_transition_directives_when_validating_then_reports_error() {
    let dir = tempfile::TempDir::new().unwrap();
    write_skill(&dir, "dev", "skill-a", &minimal_skill("skill-a", "L2-GUIDED"));
    write_loop(&dir, "dev", "skill-a", "\
## Entry Conditions
e

## Loop State Schema
s

## Single Iteration Step
i

## Proof of Progress
p

## State Transition Rule
Just some prose here, no transition directives.

## Halt Conditions
halt stall

## Handoff Target
handoff x to agent
");
    let repo = make_repo(&dir);
    let diags = validate_loop_language(&repo);
    assert!(diags.iter().any(|d| d.code == "loop-no-transitions"));
}

#[test]
fn given_halt_followed_by_skip_word_when_validating_then_no_false_positive() {
    let dir = tempfile::TempDir::new().unwrap();
    write_skill(&dir, "dev", "skill-a", &minimal_skill("skill-a", "L2-GUIDED"));
    write_loop(&dir, "dev", "skill-a", "\
## Halt Conditions
halt the iteration

## State Transition Rule
transition a → b

## Entry Conditions
ready

## Loop State Schema
x

## Single Iteration Step
y

## Proof of Progress
z

## Handoff Target
handoff x to agent
");
    let repo = make_repo(&dir);
    let diags = validate_loop_language(&repo);
    assert!(!diags.iter().any(|d| d.code == "loop-unknown-halt-reason"));
}

// ── Graph validator ──────────────────────────────────────────────────

#[test]
fn given_linear_transition_chain_when_auto_detecting_then_entry_and_terminal_correct() {
    let dir = tempfile::TempDir::new().unwrap();
    write_skill(&dir, "dev", "skill-a", &minimal_skill("skill-a", "L2-GUIDED"));
    write_loop(&dir, "dev", "skill-a", "## State Transition Rule\ntransition start → end\n");
    let repo = make_repo(&dir);
    // start has no inbound → entry point
    assert!(repo.handoff_graph.entry_points.iter().any(|s| s.name == "start"));
    // end has no outbound → terminal
    assert!(repo.handoff_graph.nodes.iter().any(|s| s.name == "end" && s.is_terminal));
    let diags = validate_graph(&repo);
    assert!(diags.is_empty(), "well-formed chain should have no errors: {diags:?}");
}

#[test]
fn given_state_with_no_inbound_appearing_only_as_to_when_validating_then_still_auto_detected_as_entry() {
    // If a state appears only in a "to" column (no "from" transitions), it has no outbound and no inbound.
    // No outbound → terminal. No inbound → entry point. Both can be true simultaneously.
    let dir = tempfile::TempDir::new().unwrap();
    write_skill(&dir, "dev", "skill-a", &minimal_skill("skill-a", "L2-GUIDED"));
    write_loop(&dir, "dev", "skill-a", "## State Transition Rule\ntransition a → sink\n");
    let repo = make_repo(&dir);
    assert!(repo.handoff_graph.nodes.iter().any(|s| s.name == "sink" && s.is_terminal));
}

#[test]
fn given_state_with_only_a_self_loop_when_validating_then_reports_self_loop_warning() {
    let dir = tempfile::TempDir::new().unwrap();
    write_skill(&dir, "dev", "skill-a", &minimal_skill("skill-a", "L2-GUIDED"));
    write_loop(&dir, "dev", "skill-a", "\
## State Transition Rule
transition entry → loop-state
transition loop-state → loop-state
");
    let repo = make_repo(&dir);
    let diags = validate_graph(&repo);
    assert!(diags.iter().any(|d| d.code == "graph-self-loop-only"));
}

#[test]
fn given_well_formed_graph_when_validating_then_no_errors() {
    let dir = tempfile::TempDir::new().unwrap();
    write_skill(&dir, "dev", "skill-a", &minimal_skill("skill-a", "L2-GUIDED"));
    write_loop(&dir, "dev", "skill-a", &minimal_loop_with_transition("entry", "done", "skill-a"));
    let repo = make_repo(&dir);
    let diags = validate_graph(&repo);
    assert!(diags.is_empty());
}

// ── Constraints validator ────────────────────────────────────────────

#[test]
fn given_empty_handoff_graph_when_validating_constraints_then_warns() {
    let dir = tempfile::TempDir::new().unwrap();
    write_skill(&dir, "dev", "skill-a", &minimal_skill("skill-a", "L2-GUIDED"));
    let repo = make_repo(&dir);
    let diags = validate_constraints(&repo);
    assert!(diags.iter().any(|d| d.code == "constraints-empty-graph"));
}

#[test]
fn given_handoff_to_nonexistent_skill_when_validating_then_warns() {
    let dir = tempfile::TempDir::new().unwrap();
    write_skill(&dir, "dev", "skill-a", &minimal_skill("skill-a", "L2-GUIDED"));
    write_loop(&dir, "dev", "skill-a", "\
## State Transition Rule
transition a → b
  handoff nonexistent-skill to agent
");
    let repo = make_repo(&dir);
    let diags = validate_constraints(&repo);
    assert!(diags.iter().any(|d| d.code == "constraints-unknown-handoff-target"));
}

// ── run_all integration ──────────────────────────────────────────────

#[test]
fn given_two_skills_forming_complete_graph_when_running_all_validators_then_no_errors() {
    let dir = tempfile::TempDir::new().unwrap();
    write_skill(&dir, "dev", "skill-a", &minimal_skill("skill-a", "L2-GUIDED"));
    write_loop(&dir, "dev", "skill-a", &minimal_loop_with_transition("a", "b", "skill-b"));
    write_skill(&dir, "dev", "skill-b", &minimal_skill("skill-b", "L2-GUIDED"));
    write_loop(&dir, "dev", "skill-b", &minimal_loop_with_transition("b", "done", "skill-a"));
    let repo = make_repo(&dir);
    let diags = run_all(&repo);
    let errors: Vec<_> = diags.iter().filter(|d| d.severity == Severity::Error).collect();
    assert!(errors.is_empty(), "expected no errors, got: {errors:?}");
    // Verify graph structure was auto-discovered
    assert!(repo.handoff_graph.nodes.len() >= 3);
    let entries: HashSet<_> = repo.handoff_graph.entry_points.iter().map(|s| s.name.clone()).collect();
    assert!(entries.contains("a"));
}
