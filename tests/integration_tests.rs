use skill_loop_verifier::parser::handoff::parse_transition_rules;
use skill_loop_verifier::parser::loop_::parse_loop_contract;
use skill_loop_verifier::parser::skill::discover_skills;
use skill_loop_verifier::types::Repo;
use skill_loop_verifier::types::{
    detect_entry_points, detect_terminal_states, is_standard_halt_reason, is_standard_verb,
    validate_halt_reason, validate_verb,
};
use skill_loop_verifier::validators::run_all;
use std::collections::HashSet;
use tempfile::TempDir;

fn write_skill(dir: &TempDir, category: &str, name: &str, content: &str) {
    let skill_dir = dir.path().join("skills").join(category).join(name);
    std::fs::create_dir_all(&skill_dir).unwrap();
    std::fs::write(skill_dir.join("SKILL.md"), content).unwrap();
}

fn write_loop_md(dir: &TempDir, category: &str, name: &str, content: &str) {
    let skill_dir = dir.path().join("skills").join(category).join(name);
    std::fs::create_dir_all(&skill_dir).unwrap();
    std::fs::write(skill_dir.join("LOOP.md"), content).unwrap();
}

#[test]
fn test_parse_single_skill() {
    let dir = TempDir::new().unwrap();
    write_skill(
        &dir,
        "discovery",
        "my-skill",
        "---\nname: my-skill\nlevel: L2-GUIDED\nowner: dev-agent\n---\n\n## Description\nDoes things.\n\n## Rules\n- Rule one.\n\n## State Model\nSome state model.\n",
    );

    let skills = discover_skills(&dir.path().join("skills")).unwrap();
    assert_eq!(skills.len(), 1);
    let s = &skills[0];
    assert_eq!(s.name, "my-skill");
    assert_eq!(s.category, "discovery");
    assert_eq!(s.level, "L2-GUIDED");
    assert_eq!(s.owner, vec!["dev-agent".to_string()]);
}

#[test]
fn test_transition_rule_parsing() {
    let content = "\
## State Transition Rule

transition in-dev → ready-for-deskcheck
  trigger all acceptance tests green
  handoff running-desk-checks to qa-agent

transition in-dev → halted-stall
  halt stall after 5 iterations
";
    let rules = parse_transition_rules(content, "my-skill");
    assert_eq!(rules.len(), 2);
    assert_eq!(rules[0].from, "in-dev");
    assert_eq!(rules[0].to, "ready-for-deskcheck");
    assert_eq!(rules[0].trigger.as_deref(), Some("all acceptance tests green"));
    assert_eq!(rules[0].handoff_skill.as_deref(), Some("running-desk-checks"));
    assert_eq!(rules[0].handoff_agent.as_deref(), Some("qa-agent"));
    assert_eq!(rules[1].halt_reason.as_deref(), Some("stall"));
    assert_eq!(rules[1].halt_after, Some(5));
}

#[test]
fn test_entry_point_detection() {
    use skill_loop_verifier::types::Transition;
    let transitions = vec![
        Transition { from: "in-dev".into(), to: "done".into(), trigger: "ok".into(), condition: None },
    ];
    let entries = detect_entry_points(&transitions);
    assert!(entries.contains("in-dev"));
    assert!(!entries.contains("done"));
}

#[test]
fn test_terminal_detection() {
    use skill_loop_verifier::types::Transition;
    let transitions = vec![
        Transition { from: "in-dev".into(), to: "done".into(), trigger: "ok".into(), condition: None },
    ];
    let terminals = detect_terminal_states(&transitions);
    assert!(terminals.contains("done"));
    assert!(!terminals.contains("in-dev"));
}

#[test]
fn test_standard_vocabulary() {
    assert!(is_standard_verb("verify"));
    assert!(is_standard_verb("handoff"));
    assert!(is_standard_verb("halt"));
    assert!(!is_standard_verb("frobnicate"));

    assert!(is_standard_halt_reason("stall"));
    assert!(is_standard_halt_reason("ambiguous"));
    assert!(is_standard_halt_reason("budget"));
    assert!(!is_standard_halt_reason("tired"));

    assert!(validate_verb("verify").is_none());
    assert!(validate_verb("unknown_verb").is_some());
    assert!(validate_halt_reason("stall").is_none());
    assert!(validate_halt_reason("unknown_reason").is_some());
}

#[test]
fn test_full_graph_discovery() {
    let dir = TempDir::new().unwrap();

    // Create two skills with LOOP.md transition rules
    write_skill(
        &dir, "discovery", "skill-a",
        "---\nname: skill-a\nlevel: L2-GUIDED\nowner: dev\n---\n\n## Description\nA.\n\n## Rules\n- r1.\n\n## State Model\nstates.\n",
    );
    write_loop_md(&dir, "discovery", "skill-a", "\
## Entry Conditions
x

## Loop State Schema
y

## Single Iteration Step
z

## Proof of Progress
w

## State Transition Rule
transition in-analysis → ready-for-dev
  trigger start

## Halt Conditions
halt stall

## Handoff Target
handoff skill-b to dev
");

    write_skill(
        &dir, "development", "skill-b",
        "---\nname: skill-b\nlevel: L1-RIGID\nowner: dev\n---\n\n## Description\nB.\n\n## Rules\n- r2.\n\n## State Model\ns.\n\n## Entry Conditions\ne.\n\n## Halt Conditions\nh.\n",
    );
    write_loop_md(&dir, "development", "skill-b", "\
## Entry Conditions
x

## Loop State Schema
y

## Single Iteration Step
z

## Proof of Progress
w

## State Transition Rule
transition ready-for-dev → done
  trigger finish

## Halt Conditions
halt stall

## Handoff Target
handoff done to dev
");

    let repo = Repo::from_root(dir.path().to_path_buf(), "skills").unwrap();
    let diagnostics = run_all(&repo);

    // Should have a complete graph
    assert!(repo.handoff_graph.nodes.len() >= 3, "Expected at least 3 states: in-analysis, ready-for-dev, done");
    assert_eq!(repo.handoff_graph.edges.len(), 2, "Expected 2 transitions");

    // Check entry point detection
    let entries: HashSet<_> = repo.handoff_graph.entry_points.iter().map(|s| s.name.clone()).collect();
    assert!(entries.contains("in-analysis"), "in-analysis should be auto-detected as entry point (no inbound)");

    // Check terminal detection
    let terminals: HashSet<_> = repo.handoff_graph.nodes.iter()
        .filter(|s| s.is_terminal)
        .map(|s| s.name.clone())
        .collect();
    assert!(terminals.contains("done"), "done should be auto-detected as terminal (no outbound)");

    let errors = diagnostics.iter().filter(|d| d.severity == skill_loop_verifier::types::Severity::Error).count();
    assert_eq!(errors, 0, "Expected no errors, got: {diagnostics:?}");
}

#[test]
fn test_missing_loop_section_detected() {
    use skill_loop_verifier::parser::loop_::missing_sections;
    let dir = TempDir::new().unwrap();
    let loop_path = dir.path().join("LOOP.md");
    std::fs::write(
        &loop_path,
        "## Entry Conditions\nx\n\n## Single Iteration Step\ny\n",
    )
    .unwrap();
    let contract = parse_loop_contract(&loop_path, "test").unwrap();
    let missing = missing_sections(&contract.sections);
    assert!(missing.contains(&"Loop State Schema"));
    assert!(missing.contains(&"Proof of Progress"));
    assert!(missing.contains(&"State Transition Rule"));
    assert!(missing.contains(&"Halt Conditions"));
    assert!(missing.contains(&"Handoff Target"));
}

#[test]
fn test_duplicate_rules_detected() {
    let dir = TempDir::new().unwrap();
    write_skill(
        &dir, "discovery", "dup-rules",
        "---\nname: dup-rules\nlevel: L2-GUIDED\nowner: dev\n---\n\n## Description\nx\n\n## Rules\nr1\n\n## Rules\nr2\n\n## State Model\ns\n",
    );
    let repo = Repo::from_root(dir.path().to_path_buf(), "skills").unwrap();
    let diagnostics = run_all(&repo);
    let duplicate_warnings: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.code == "skill-duplicate-rules")
        .collect();
    assert!(!duplicate_warnings.is_empty(), "Should warn about duplicate Rules sections");
}
