use crate::parser::loop_::parse_loop_contract;
use crate::types::{Diagnostic, FileLocation, Repo, Severity};

/// Validate LOOP.md section structure for all skills that have one.
pub fn validate_loop_sections(repo: &Repo) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    for skill in &repo.skills {
        if !skill.has_loop_md {
            continue;
        }

        let loop_path = skill.loop_md();
        if let Some(contract) = parse_loop_contract(&loop_path, &skill.name) {
            // Check for all 7 canonical sections
            let missing = crate::parser::loop_::missing_sections(&contract.sections);
            for section in &missing {
                diags.push(Diagnostic {
                    severity: Severity::Error,
                    code: "loop-missing-section".to_string(),
                    message: format!(
                        "LOOP.md for `{}` is missing required section '## {}'",
                        skill.name, section
                    ),
                    location: FileLocation {
                        path: loop_path.clone(),
                        line: None,
                        column: None,
                    },
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
                    location: FileLocation {
                        path: loop_path.clone(),
                        line: None,
                        column: None,
                    },
                    help: "Canonical order: Entry Conditions → Loop State Schema → Single Iteration Step → Proof of Progress → State Transition Rule → Halt Conditions → Handoff Target".to_string(),
                });
            }

            // Check for unknown section headings
            for section in &contract.sections {
                if let crate::types::LoopSection::Unknown(body) = section {
                    diags.push(Diagnostic {
                    severity: Severity::Error,
                    code: "loop-unknown-section".to_string(),
                        message: format!(
                            "LOOP.md for `{}` has unknown section with body: '{}...'; use only the 7 canonical headings.",
                            skill.name,
                            &body[..body.len().min(50)]
                        ),
                        location: FileLocation {
                            path: loop_path.clone(),
                            line: None,
                            column: None,
                        },
                        help: "Only these 7 headings are valid: Entry Conditions, Loop State Schema, Single Iteration Step, Proof of Progress, State Transition Rule, Halt Conditions, Handoff Target".to_string(),
                    });
                }
            }
        }
    }

    diags
}
