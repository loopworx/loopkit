use crate::types::{Diagnostic, FileLocation, Repo, Severity};

/// Validate configuration constraints.
/// Currently checks that max_iterations is reasonable.
pub fn validate_constraints(repo: &Repo) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    // Graph should not be empty
    if repo.handoff_graph.nodes.is_empty() {
        diags.push(Diagnostic {
            severity: Severity::Error,
            code: "constraints-empty-graph".to_string(),
            message: "No states found in the handoff graph. No transition rules defined in any LOOP.md or HANDOFFS.md.".to_string(),
            location: FileLocation {
                path: repo.root.join("skills"),
                line: None,
                column: None,
            },
            help: "Add transition rules to at least one LOOP.md file using: transition <from> → <to>".to_string(),
        });
    }

    // Handoff targets should reference existing skills
    for skill in &repo.skills {
        for rule in &skill.transitions {
            if let Some(ref target) = rule.handoff_skill {
                if !repo.skills.iter().any(|s| s.name == *target) {
                    diags.push(Diagnostic {
                    severity: Severity::Error,
                    code: "constraints-unknown-handoff-target".to_string(),
                        message: format!(
                            "Skill `{}` transitions to unknown handoff target `{}`",
                            skill.name, target
                        ),
                        location: FileLocation {
                            path: skill.dir().to_path_buf(),
                            line: None,
                            column: None,
                        },
                        help: format!(
                            "Skill `{}` must exist in the skills directory.",
                            target
                        ),
                    });
                }
            }
        }
    }

    diags
}
