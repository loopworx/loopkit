use crate::types::{Diagnostic, FileLocation, Repo, Severity, STATE_MODEL_ALIASES};

/// Validate that every skill has required SKILL.md sections.
pub fn validate_skill_completeness(repo: &Repo) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    for skill in &repo.skills {
        let sections = &skill.sections;

        // Check for Description
        if !sections.iter().any(|s| s == "Description") {
            diags.push(Diagnostic {
                severity: Severity::Error,
                code: "skill-missing-description".to_string(),
                message: format!("Skill `{}` is missing '## Description' section", skill.name),
                location: FileLocation {
                    path: skill.skill_md(),
                    line: None,
                    column: None,
                },
                help: "Every SKILL.md must have a ## Description section.".to_string(),
            });
        }

        // Check for Rules
        if !sections.iter().any(|s| s == "Rules") {
            diags.push(Diagnostic {
                severity: Severity::Error,
                code: "skill-missing-rules".to_string(),
                message: format!("Skill `{}` is missing '## Rules' section", skill.name),
                location: FileLocation {
                    path: skill.skill_md(),
                    line: None,
                    column: None,
                },
                help: "Every SKILL.md must have a ## Rules section.".to_string(),
            });
        }

        // Check for State Model (or alias)
        if !sections.iter().any(|s| crate::types::is_state_model_alias(s)) {
            let aliases = STATE_MODEL_ALIASES.join(", ");
            diags.push(Diagnostic {
                severity: Severity::Error,
                code: "skill-missing-state-model".to_string(),
                message: format!(
                    "Skill `{}` is missing a state model section. Expected one of: {}",
                    skill.name, aliases
                ),
                location: FileLocation {
                    path: skill.skill_md(),
                    line: None,
                    column: None,
                },
                help: format!("Every SKILL.md must define a state model under one of: {}.", aliases),
            });
        }

        // L1-RIGID requires Entry Conditions and Halt Conditions
        if skill.level == "L1-RIGID" {
            if !sections.iter().any(|s| s == "Entry Conditions") {
                diags.push(Diagnostic {
                    severity: Severity::Error,
                    code: "skill-l1-missing-entry-conditions".to_string(),
                    message: format!(
                        "L1-RIGID skill `{}` is missing '## Entry Conditions' section",
                        skill.name
                    ),
                    location: FileLocation {
                        path: skill.skill_md(),
                        line: None,
                        column: None,
                    },
                    help: "L1-RIGID skills must declare ## Entry Conditions directly in SKILL.md.".to_string(),
                });
            }
            if !sections.iter().any(|s| s == "Halt Conditions") {
                diags.push(Diagnostic {
                    severity: Severity::Error,
                    code: "skill-l1-missing-halt-conditions".to_string(),
                    message: format!(
                        "L1-RIGID skill `{}` is missing '## Halt Conditions' section",
                        skill.name
                    ),
                    location: FileLocation {
                        path: skill.skill_md(),
                        line: None,
                        column: None,
                    },
                    help: "L1-RIGID skills must declare ## Halt Conditions directly in SKILL.md.".to_string(),
                });
            }
        }

        // Warn about duplicate Rules headings
        let rules_count = sections.iter().filter(|s| *s == "Rules").count();
        if rules_count > 1 {
            diags.push(Diagnostic {
                severity: Severity::Error,
                code: "skill-duplicate-rules".to_string(),
                message: format!(
                    "Skill `{}` has {} '## Rules' sections (duplicate).",
                    skill.name, rules_count
                ),
                location: FileLocation {
                    path: skill.skill_md(),
                    line: None,
                    column: None,
                },
                help: "Merge duplicate ## Rules sections into one.".to_string(),
            });
        }
    }

    diags
}

/// Check for LOOP.md presence in skills that define transition rules (loop-worthy).
pub fn validate_loop_completeness(repo: &Repo) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    for skill in &repo.skills {
        // A skill is loop-worthy if it defines transition rules
        let has_transitions = !skill.transitions.is_empty();

        if has_transitions && !skill.has_loop_md {
            diags.push(Diagnostic {
                severity: Severity::Error,
                code: "loop-missing-for-transition-skill".to_string(),
                message: format!(
                    "Skill `{}` defines transition rules but has no LOOP.md",
                    skill.name
                ),
                location: FileLocation {
                    path: skill.dir().to_path_buf(),
                    line: None,
                    column: None,
                },
                help: "Skills with transition rules should have a LOOP.md defining entry conditions, proof of progress, and halt conditions.".to_string(),
            });
        }
    }

    diags
}
