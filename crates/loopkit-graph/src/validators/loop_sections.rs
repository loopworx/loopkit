use crate::parser::loop_::{missing_sections, parse_loop_contract};
use crate::types::{LoopContract, LoopSection};
use loopkit_core::types::{Config, Diagnostic, FileLocation, Severity, Skill};
use std::collections::HashMap;

/// Validate LOOP.md section structure for all skills that have one.
pub fn validate(
    skills: &[Skill],
    all_handoffs: &HashMap<String, LoopContract>,
    _config: &Config,
) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    for skill in skills {
        let loop_path = skill.loop_md();
        if !loop_path.exists() {
            continue;
        }

        if let Some(contract) = parse_loop_contract(&loop_path, &skill.name) {
            // Check for all 7 canonical sections
            let missing = missing_sections(&contract.sections);
            for section in &missing {
                diags.push(Diagnostic {
                    severity: Severity::Error,
                    code: "loop-missing-section".to_string(),
                    message: format!(
                        "LOOP.md for `{}` is missing required section '## {}'",
                        skill.name, section
                    ),
                    location: FileLocation::new(loop_path.clone()),
                    help: format!(
                        "Add '## {}' section to LOOP.md. All 7 canonical sections are required.",
                        section
                    ),
                });
            }

            // Check section ordering
            if !contract.section_order_valid {
                diags.push(Diagnostic {
                    severity: Severity::Error,
                    code: "loop-section-order".to_string(),
                    message: format!(
                        "LOOP.md for `{}` has sections in non-canonical order",
                        skill.name
                    ),
                    location: FileLocation::new(loop_path.clone()),
                    help: "Canonical order: Entry Conditions -> Loop State Schema -> \
                        Single Iteration Step -> Proof of Progress -> State Transition Rule -> \
                        Halt Conditions -> Handoff Target"
                        .to_string(),
                });
            }

            // Check for unknown section headings
            for section in &contract.sections {
                if let LoopSection::Unknown(body) = section {
                    diags.push(Diagnostic {
                        severity: Severity::Error,
                        code: "loop-unknown-section".to_string(),
                        message: format!(
                            "LOOP.md for `{}` has unknown section with body: '{}...'; \
                            use only the 7 canonical headings.",
                            skill.name,
                            &body[..body.len().min(50)]
                        ),
                        location: FileLocation::new(loop_path.clone()),
                        help:
                            "Only these 7 headings are valid: Entry Conditions, Loop State Schema, \
                            Single Iteration Step, Proof of Progress, State Transition Rule, \
                            Halt Conditions, Handoff Target"
                                .to_string(),
                    });
                }
            }
        }
    }

    // Suppress unused variable warning
    let _ = all_handoffs;

    diags
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn make_skill(name: &str, path: PathBuf) -> Skill {
        let skill_md = path.join("SKILL.md");
        Skill {
            name: name.into(),
            level: "L3".into(),
            owner: vec![],
            description: "".into(),
            category: "".into(),
            path,
            skill_md,
            sections: vec![],
            states: vec![],
        }
    }

    #[test]
    fn valid_loop_md_no_errors() {
        let dir = TempDir::new().unwrap();
        let skill_dir = dir.path().join("test-skill");
        std::fs::create_dir(&skill_dir).unwrap();
        std::fs::write(skill_dir.join("SKILL.md"), "").unwrap();
        std::fs::write(
            skill_dir.join("LOOP.md"),
            "\
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

## Halt Conditions
halt stall

## Handoff Target
handoff x to y
",
        )
        .unwrap();

        let skills = vec![make_skill("test-skill", skill_dir)];
        let all_handoffs: HashMap<String, LoopContract> = HashMap::new();
        let config = Config::default();
        let diags = validate(&skills, &all_handoffs, &config);
        assert!(
            diags.is_empty(),
            "Expected no diagnostics but got: {:?}",
            diags
        );
    }

    #[test]
    fn missing_sections_emits_error() {
        let dir = TempDir::new().unwrap();
        let skill_dir = dir.path().join("test-skill");
        std::fs::create_dir(&skill_dir).unwrap();
        std::fs::write(skill_dir.join("SKILL.md"), "").unwrap();
        std::fs::write(
            skill_dir.join("LOOP.md"),
            "\
## Entry Conditions
foo

## Loop State Schema
bar
",
        )
        .unwrap();

        let skills = vec![make_skill("test-skill", skill_dir)];
        let all_handoffs: HashMap<String, LoopContract> = HashMap::new();
        let config = Config::default();
        let diags = validate(&skills, &all_handoffs, &config);
        assert!(diags.iter().any(|d| d.code == "loop-missing-section"));
    }

    #[test]
    fn wrong_section_order_emits_error() {
        let dir = TempDir::new().unwrap();
        let skill_dir = dir.path().join("test-skill");
        std::fs::create_dir(&skill_dir).unwrap();
        std::fs::write(skill_dir.join("SKILL.md"), "").unwrap();
        std::fs::write(
            skill_dir.join("LOOP.md"),
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

        let skills = vec![make_skill("test-skill", skill_dir)];
        let all_handoffs: HashMap<String, LoopContract> = HashMap::new();
        let config = Config::default();
        let diags = validate(&skills, &all_handoffs, &config);
        assert!(diags.iter().any(|d| d.code == "loop-section-order"));
    }

    #[test]
    fn unknown_section_emits_error() {
        let dir = TempDir::new().unwrap();
        let skill_dir = dir.path().join("test-skill");
        std::fs::create_dir(&skill_dir).unwrap();
        std::fs::write(skill_dir.join("SKILL.md"), "").unwrap();
        std::fs::write(
            skill_dir.join("LOOP.md"),
            "\
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

## Halt Conditions
halt stall

## Handoff Target
handoff x to y

## My Custom Section
nobody expects this
",
        )
        .unwrap();

        let skills = vec![make_skill("test-skill", skill_dir)];
        let all_handoffs: HashMap<String, LoopContract> = HashMap::new();
        let config = Config::default();
        let diags = validate(&skills, &all_handoffs, &config);
        assert!(diags.iter().any(|d| d.code == "loop-unknown-section"));
    }

    #[test]
    fn no_loop_md_no_diagnostics() {
        let dir = TempDir::new().unwrap();
        let skill_dir = dir.path().join("test-skill");
        std::fs::create_dir(&skill_dir).unwrap();
        std::fs::write(skill_dir.join("SKILL.md"), "").unwrap();

        let skills = vec![make_skill("test-skill", skill_dir)];
        let all_handoffs: HashMap<String, LoopContract> = HashMap::new();
        let config = Config::default();
        let diags = validate(&skills, &all_handoffs, &config);
        assert!(diags.is_empty());
    }
}
