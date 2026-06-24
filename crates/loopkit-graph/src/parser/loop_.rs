use crate::types::{LoopContract, LoopSection, CANONICAL_LOOP_SECTIONS};
use loopkit_core::parser::skill::parse_sections;
use std::path::Path;

/// Parse a LOOP.md file and extract its section structure.
pub fn parse_loop_contract(path: &Path, skill_name: &str) -> Option<LoopContract> {
    let content = std::fs::read_to_string(path).ok()?;
    let sections_raw = parse_sections(&content);

    let sections: Vec<LoopSection> = sections_raw
        .iter()
        .map(|s| match s.name.as_str() {
            "Entry Conditions" => LoopSection::EntryConditions(s.body.clone()),
            "Loop State Schema" => LoopSection::LoopStateSchema(s.body.clone()),
            "Single Iteration Step" => LoopSection::SingleIterationStep(s.body.clone()),
            "Proof of Progress" => LoopSection::ProofOfProgress(s.body.clone()),
            "State Transition Rule" => LoopSection::StateTransitionRule(s.body.clone()),
            "Halt Conditions" => LoopSection::HaltConditions(s.body.clone()),
            "Handoff Target" => LoopSection::HandoffTarget(s.body.clone()),
            _ => LoopSection::Unknown(s.body.clone()),
        })
        .collect();

    let known_headings: Vec<&str> = sections
        .iter()
        .filter(|s| is_canonical_loop_section(s))
        .map(|s| s.name())
        .collect();

    let section_order_valid = verify_section_order(&known_headings);

    Some(LoopContract {
        skill: skill_name.to_string(),
        sections,
        section_order_valid,
        transitions: Vec::new(),
        loop_md_path: path.to_path_buf(),
    })
}

fn is_canonical_loop_section(section: &LoopSection) -> bool {
    let name = section.name();
    !matches!(section, LoopSection::Unknown(_)) && CANONICAL_LOOP_SECTIONS.contains(&name)
}

/// Verify that known canonical sections appear in the expected order.
/// Order: Entry Conditions, Loop State Schema, Single Iteration Step,
/// Proof of Progress, State Transition Rule, Halt Conditions, Handoff Target
fn verify_section_order(headings: &[&str]) -> bool {
    let mut last_idx: Option<usize> = None;

    for heading in headings {
        if let Some(pos) = CANONICAL_LOOP_SECTIONS.iter().position(|s| s == heading) {
            match last_idx {
                Some(prev) if pos < prev => return false,
                _ => last_idx = Some(pos),
            }
        }
    }

    true
}

/// Check if all 7 canonical sections are present (uniquely).
pub fn has_all_canonical_sections(sections: &[LoopSection]) -> bool {
    let unique_canonical: std::collections::HashSet<&str> = sections
        .iter()
        .filter_map(|s| match s {
            LoopSection::EntryConditions(_) => Some("Entry Conditions"),
            LoopSection::LoopStateSchema(_) => Some("Loop State Schema"),
            LoopSection::SingleIterationStep(_) => Some("Single Iteration Step"),
            LoopSection::ProofOfProgress(_) => Some("Proof of Progress"),
            LoopSection::StateTransitionRule(_) => Some("State Transition Rule"),
            LoopSection::HaltConditions(_) => Some("Halt Conditions"),
            LoopSection::HandoffTarget(_) => Some("Handoff Target"),
            LoopSection::Unknown(_) => None,
        })
        .collect();
    unique_canonical.len() == 7
}

/// Get missing canonical section names.
pub fn missing_sections(sections: &[LoopSection]) -> Vec<&'static str> {
    let present: std::collections::HashSet<&str> = sections
        .iter()
        .filter_map(|s| match s {
            LoopSection::EntryConditions(_) => Some("Entry Conditions"),
            LoopSection::LoopStateSchema(_) => Some("Loop State Schema"),
            LoopSection::SingleIterationStep(_) => Some("Single Iteration Step"),
            LoopSection::ProofOfProgress(_) => Some("Proof of Progress"),
            LoopSection::StateTransitionRule(_) => Some("State Transition Rule"),
            LoopSection::HaltConditions(_) => Some("Halt Conditions"),
            LoopSection::HandoffTarget(_) => Some("Handoff Target"),
            LoopSection::Unknown(_) => None,
        })
        .collect();

    CANONICAL_LOOP_SECTIONS
        .iter()
        .filter(|s| !present.contains(*s))
        .copied()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn parse_valid_loop_contract() {
        let dir = TempDir::new().unwrap();
        let loop_path = dir.path().join("LOOP.md");
        std::fs::write(
            &loop_path,
            "\
## Entry Conditions
Story is in-dev

## Loop State Schema
| field | type |
|-------|------|
| story | str  |

## Single Iteration Step
1. verify entry conditions
2. write test

## Proof of Progress
`cargo test`

## State Transition Rule
transition in-dev → in-deskcheck
  trigger all-ACs-green

## Halt Conditions
halt stall after 5 iterations

## Handoff Target
handoff running-desk-checks to qa-agent
",
        )
        .unwrap();

        let contract = parse_loop_contract(&loop_path, "test-skill").unwrap();
        assert_eq!(contract.skill, "test-skill");
        assert_eq!(contract.sections.len(), 7);
        assert!(contract.section_order_valid);
        assert!(has_all_canonical_sections(&contract.sections));
        assert!(missing_sections(&contract.sections).is_empty());
    }

    #[test]
    fn detect_missing_sections() {
        let dir = TempDir::new().unwrap();
        let loop_path = dir.path().join("LOOP.md");
        std::fs::write(
            &loop_path,
            "\
## Entry Conditions
foo

## Loop State Schema
bar

## Single Iteration Step
baz
",
        )
        .unwrap();

        let contract = parse_loop_contract(&loop_path, "test-skill").unwrap();
        assert!(!has_all_canonical_sections(&contract.sections));
        let missing = missing_sections(&contract.sections);
        assert!(missing.contains(&"Proof of Progress"));
        assert!(missing.contains(&"State Transition Rule"));
        assert!(missing.contains(&"Halt Conditions"));
        assert!(missing.contains(&"Handoff Target"));
    }

    #[test]
    fn section_order_out_of_order() {
        let dir = TempDir::new().unwrap();
        let loop_path = dir.path().join("LOOP.md");
        std::fs::write(
            &loop_path,
            "\
## Halt Conditions
halt stall

## Entry Conditions
foo

## Loop State Schema
bar

## Single Iteration Step
baz

## Proof of Progress
qux

## State Transition Rule
transition a → b

## Handoff Target
handoff x to y
",
        )
        .unwrap();

        let contract = parse_loop_contract(&loop_path, "test-skill").unwrap();
        assert!(!contract.section_order_valid);
    }
}
