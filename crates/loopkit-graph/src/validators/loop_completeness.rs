use crate::types::LoopContract;
use loopkit_core::types::{Config, Diagnostic, FileLocation, Severity, Skill};
use std::collections::HashMap;

/// Validate skill completeness + loop completeness.
pub fn validate(
    skills: &[Skill],
    all_handoffs: &HashMap<String, LoopContract>,
    config: &Config,
) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    // Skill-level checks
    diags.extend(check_skill_completeness(skills, config));

    // Loop completeness checks
    diags.extend(check_loop_completeness(skills, all_handoffs));

    diags
}

fn check_skill_completeness(skills: &[Skill], config: &Config) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    for skill in skills {
        let section_names: Vec<&str> = skill.sections.iter().map(|s| s.name.as_str()).collect();

        // Check for Description
        if !section_names.iter().any(|s| *s == "Description") {
            diags.push(Diagnostic {
                severity: Severity::Error,
                code: "skill-missing-description".to_string(),
                message: format!("Skill `{}` is missing '## Description' section", skill.name),
                location: FileLocation::new(skill.skill_md.clone()),
                help: "Every SKILL.md must have a ## Description section.".to_string(),
            });
        }

        // Check for Rules
        if !section_names.iter().any(|s| *s == "Rules") {
            diags.push(Diagnostic {
                severity: Severity::Error,
                code: "skill-missing-rules".to_string(),
                message: format!("Skill `{}` is missing '## Rules' section", skill.name),
                location: FileLocation::new(skill.skill_md.clone()),
                help: "Every SKILL.md must have a ## Rules section.".to_string(),
            });
        }

        // Check for State Model (or alias)
        let has_state_model = section_names
            .iter()
            .any(|s| config.state_model_aliases.iter().any(|alias| alias == *s));

        if !has_state_model {
            let aliases = config.state_model_aliases.join(", ");
            diags.push(Diagnostic {
                severity: Severity::Error,
                code: "skill-missing-state-model".to_string(),
                message: format!(
                    "Skill `{}` is missing a state model section. Expected one of: {}",
                    skill.name, aliases
                ),
                location: FileLocation::new(skill.skill_md.clone()),
                help: format!("Every SKILL.md must define a state model under one of: {}.", aliases),
            });
        }

        // L1-RIGID: requires Entry Conditions and Halt Conditions
        if skill.level == "L1-RIGID" {
            if !section_names.iter().any(|s| *s == "Entry Conditions") {
                diags.push(Diagnostic {
                    severity: Severity::Error,
                    code: "skill-l1-missing-entry-conditions".to_string(),
                    message: format!(
                        "L1-RIGID skill `{}` is missing '## Entry Conditions' section",
                        skill.name
                    ),
                    location: FileLocation::new(skill.skill_md.clone()),
                    help: "L1-RIGID skills must declare ## Entry Conditions directly in SKILL.md."
                        .to_string(),
                });
            }
            if !section_names.iter().any(|s| *s == "Halt Conditions") {
                diags.push(Diagnostic {
                    severity: Severity::Error,
                    code: "skill-l1-missing-halt-conditions".to_string(),
                    message: format!(
                        "L1-RIGID skill `{}` is missing '## Halt Conditions' section",
                        skill.name
                    ),
                    location: FileLocation::new(skill.skill_md.clone()),
                    help: "L1-RIGID skills must declare ## Halt Conditions directly in SKILL.md."
                        .to_string(),
                });
            }
        }

        // Warn about duplicate Rules headings
        let rules_count = section_names.iter().filter(|s| *s == "Rules").count();
        if rules_count > 1 {
            diags.push(Diagnostic {
                severity: Severity::Error,
                code: "skill-duplicate-rules".to_string(),
                message: format!(
                    "Skill `{}` has {} '## Rules' sections (duplicate).",
                    skill.name, rules_count
                ),
                location: FileLocation::new(skill.skill_md.clone()),
                help: "Merge duplicate ## Rules sections into one.".to_string(),
            });
        }
    }

    diags
}

/// Check for LOOP.md presence in skills that define transition rules.
fn check_loop_completeness(
    skills: &[Skill],
    all_handoffs: &HashMap<String, LoopContract>,
) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    for skill in skills {
        let has_transitions = all_handoffs
            .get(&skill.name)
            .map(|c| !c.transitions.is_empty())
            .unwrap_or(false);

        if has_transitions && !skill.loop_md().exists() {
            diags.push(Diagnostic {
                severity: Severity::Error,
                code: "loop-missing-for-transition-skill".to_string(),
                message: format!(
                    "Skill `{}` defines transition rules but has no LOOP.md",
                    skill.name
                ),
                location: FileLocation::new(skill.path.clone()),
                help: "Skills with transition rules should have a LOOP.md defining entry conditions, \
                    proof of progress, and halt conditions."
                    .to_string(),
            });
        }
    }

    diags
}
