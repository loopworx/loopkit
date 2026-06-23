mod common;

use common::*;
use skill_loop_verifier::parser::handoff::{parse_all_handoffs, parse_handoff_table, parse_transition_rules};
use skill_loop_verifier::parser::loop_::{missing_sections, parse_loop_contract};
use skill_loop_verifier::parser::skill::{discover_skills, parse_frontmatter, parse_sections, parse_skill_dir};

// ── Frontmatter ──────────────────────────────────────────────────────

#[test]
fn given_valid_frontmatter_when_parsing_then_extracts_fields() {
    let (fm, lines) = parse_frontmatter("---\nname: foo\nlevel: L1-RIGID\n---\n\nbody\n");
    assert_eq!(fm.get("name"), Some(&"foo".to_string()));
    assert_eq!(fm.get("level"), Some(&"L1-RIGID".to_string()));
    assert_eq!(lines, 5);
}

#[test]
fn given_no_frontmatter_when_parsing_then_returns_empty_0_lines() {
    let (fm, lines) = parse_frontmatter("## Heading\nbody\n");
    assert!(fm.is_empty());
    assert_eq!(lines, 0);
}

#[test]
fn given_markdown_when_parsing_sections_then_returns_h2_headings_only() {
    let sections = parse_sections("# H1\n## A\ntext\n### H3\n## B\nmore\n");
    assert_eq!(sections, vec!["A".to_string(), "B".to_string()]);
}

// ── Skill discovery ──────────────────────────────────────────────────

#[test]
fn given_valid_skill_directory_when_parsing_then_returns_skill_struct() {
    let dir = tempfile::TempDir::new().unwrap();
    write_skill(&dir, "discovery", "my-skill", &minimal_skill("my-skill", "L2-GUIDED"));
    let skills = discover_skills(&dir.path().join("skills")).unwrap();
    assert_eq!(skills.len(), 1);
    assert_eq!(skills[0].name, "my-skill");
    assert_eq!(skills[0].category, "discovery");
    assert_eq!(skills[0].level, "L2-GUIDED");
    assert_eq!(skills[0].owner, vec!["dev-agent".to_string()]);
}

#[test]
fn given_skill_without_frontmatter_name_when_parsing_then_returns_none() {
    let dir = tempfile::TempDir::new().unwrap();
    let sd = dir.path().join("skills").join("x").join("bare");
    std::fs::create_dir_all(&sd).unwrap();
    std::fs::write(sd.join("SKILL.md"), "# bare\n\n## Description\nx\n\n## Rules\nr\n\n## State Model\ns\n").unwrap();
    assert!(parse_skill_dir(&sd).is_none());
}

#[test]
fn given_missing_skills_directory_when_discovering_then_returns_empty_vec() {
    let dir = tempfile::TempDir::new().unwrap();
    let skills = discover_skills(&dir.path().join("nope")).unwrap();
    assert!(skills.is_empty());
}

// ── Transition rule parsing ──────────────────────────────────────────

#[test]
fn given_transition_with_trigger_and_handoff_when_parsing_then_all_fields_extracted() {
    let content = "## State Transition Rule\n\ntransition in-dev → ready-for-deskcheck\n  trigger all tests green\n  handoff running-desk-checks to qa-agent\n";
    let rules = parse_transition_rules(content, "my-skill");
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].from, "in-dev");
    assert_eq!(rules[0].to, "ready-for-deskcheck");
    assert_eq!(rules[0].trigger.as_deref(), Some("all tests green"));
    assert_eq!(rules[0].handoff_skill.as_deref(), Some("running-desk-checks"));
    assert_eq!(rules[0].handoff_agent.as_deref(), Some("qa-agent"));
}

#[test]
fn given_transition_with_halt_after_iterations_when_parsing_then_halt_fields_set() {
    let content = "## State Transition Rule\n\ntransition in-dev → halted-stall\n  halt stall after 5 iterations\n";
    let rules = parse_transition_rules(content, "my-skill");
    assert_eq!(rules[0].halt_reason.as_deref(), Some("stall"));
    assert_eq!(rules[0].halt_after, Some(5));
}

#[test]
fn given_multiple_transitions_in_one_section_when_parsing_then_all_returned() {
    let content = "\
## State Transition Rule
transition a → b
  trigger first
transition a → c
  halt stall
transition a → d
  trigger third
";
    let rules = parse_transition_rules(content, "skill");
    assert_eq!(rules.len(), 3);
}

// ── Handoff table (backwards compat) ─────────────────────────────────

#[test]
fn given_handoff_markdown_table_when_parsing_then_extracts_from_to() {
    let content = "\
| from | to | trigger | condition |
|------|----|---------|-----------|
| in-dev | ready-for-qa | tests pass | all green |
";
    let rules = parse_handoff_table(content, "skill");
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].from, "in-dev");
    assert_eq!(rules[0].to, "ready-for-qa");
}

#[test]
fn given_handoff_with_only_transition_rules_when_table_parsing_then_falls_back_to_transition_parser() {
    let content = "## State Transition Rule\n\ntransition a → b\n  trigger x\n";
    let rules = parse_handoff_table(content, "skill");
    assert_eq!(rules[0].from, "a");
    assert_eq!(rules[0].to, "b");
}

#[test]
fn given_both_loop_and_handoff_files_when_parsing_all_then_loop_md_takes_precedence() {
    let dir = tempfile::TempDir::new().unwrap();
    write_skill(&dir, "dev", "skill-a", &minimal_skill("skill-a", "L2-GUIDED"));
    write_loop(&dir, "dev", "skill-a", "## State Transition Rule\ntransition loop-state → done\n  trigger from-loop\n");
    write_handoffs(&dir, "dev", "skill-a", "| from | to |\n| handoff-state | done |\n");
    let result = parse_all_handoffs(&dir.path().join("skills"));
    let rules = result.get("skill-a").unwrap();
    assert_eq!(rules[0].from, "loop-state", "LOOP.md transition rules take precedence");
}

#[test]
fn given_empty_skills_directory_when_parsing_all_handoffs_then_returns_empty() {
    let dir = tempfile::TempDir::new().unwrap();
    let result = parse_all_handoffs(&dir.path().join("skills"));
    assert!(result.is_empty());
}

// ── Loop contract ────────────────────────────────────────────────────

#[test]
fn given_loop_with_all_7_canonical_sections_when_parsing_then_valid_and_ordered() {
    let dir = tempfile::TempDir::new().unwrap();
    let content = "\
## Entry Conditions
ready

## Loop State Schema
| field | type |
|-------|------|
| s | str |

## Single Iteration Step
1. verify entry

## Proof of Progress
`test`

## State Transition Rule
transition a → b
  trigger t

## Halt Conditions
halt stall after 5 iterations

## Handoff Target
handoff x to agent
";
    let path = dir.path().join("LOOP.md");
    std::fs::write(&path, content).unwrap();
    let contract = parse_loop_contract(&path, "test-skill").unwrap();
    assert!(contract.section_order_valid);
    assert!(missing_sections(&contract.sections).is_empty());
    assert_eq!(contract.sections.len(), 7);
}

#[test]
fn given_loop_with_only_3_sections_when_checking_missing_then_returns_remaining_4() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("LOOP.md");
    std::fs::write(&path, "## Entry Conditions\nx\n\n## Loop State Schema\ny\n\n## Single Iteration Step\nz\n").unwrap();
    let contract = parse_loop_contract(&path, "test").unwrap();
    let missing = missing_sections(&contract.sections);
    assert!(missing.contains(&"Proof of Progress"));
    assert!(missing.contains(&"State Transition Rule"));
    assert!(missing.contains(&"Halt Conditions"));
    assert!(missing.contains(&"Handoff Target"));
}

#[test]
fn given_sections_in_wrong_order_when_parsing_then_section_order_invalid() {
    let dir = tempfile::TempDir::new().unwrap();
    std::fs::write(&dir.path().join("LOOP.md"), "\
## Halt Conditions
halt stall

## Entry Conditions
ready

## Loop State Schema
x

## Single Iteration Step
y

## Proof of Progress
z

## State Transition Rule
transition a → b

## Handoff Target
handoff x to y
").unwrap();
    let contract = parse_loop_contract(&dir.path().join("LOOP.md"), "test").unwrap();
    assert!(!contract.section_order_valid);
}
