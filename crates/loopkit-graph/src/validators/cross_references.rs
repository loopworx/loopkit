use loopkit_core::types::{Diagnostic, FileLocation, Severity, Skill};
use regex::Regex;
use std::collections::HashSet;

/// Check that backtick-quoted skill references in README.md and
/// `handoff <skill> to <agent>` directives in LOOP.md files point to
/// skills that actually exist.
pub fn validate(skills: &[Skill], _all_skills: &[Skill]) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let known_skills: HashSet<&str> = skills.iter().map(|s| s.name.as_str()).collect();

    // Check handoff directives in LOOP.md files
    let handoff_re = Regex::new(r"handoff\s+(\w[\w-]*)\s+to\s+\w[\w-]*").expect("regex");

    for skill in skills {
        let loop_path = skill.loop_md();
        if !loop_path.exists() {
            continue;
        }

        if let Ok(content) = std::fs::read_to_string(&loop_path) {
            for (line_num, line) in content.lines().enumerate() {
                for cap in handoff_re.captures_iter(line) {
                    let target = &cap[1];
                    // "done" and "all-agents" are not skills
                    if target == "done" || target == "all-agents" {
                        continue;
                    }
                    if !known_skills.contains(target) {
                        diags.push(Diagnostic {
                            severity: Severity::Warning,
                            code: "xref-unknown-handoff-skill".to_string(),
                            message: format!(
                                "LOOP.md for `{}` references unknown skill `{}` at line {}",
                                skill.name,
                                target,
                                line_num + 1
                            ),
                            location: FileLocation::new(loop_path.clone())
                                .at_line((line_num + 1) as u32),
                            help: format!(
                                "Skill `{}` must exist in the skills directory.",
                                target
                            ),
                        });
                    }
                }
            }
        }
    }

    // Check README.md for backtick-quoted skill references
    let readme_path = std::path::PathBuf::from("README.md");
    if let Ok(content) = std::fs::read_to_string(&readme_path) {
        let backtick_re = Regex::new(r"`(\w[\w-]*)`").expect("regex");
        for (line_num, line) in content.lines().enumerate() {
            for cap in backtick_re.captures_iter(line) {
                let token = &cap[1];
                if is_known_exception(token) {
                    continue;
                }
                if token.contains('-') && !known_skills.contains(token) {
                    diags.push(Diagnostic {
                        severity: Severity::Warning,
                        code: "xref-unknown-readme-skill".to_string(),
                        message: format!(
                            "README.md references unknown skill `{}` at line {}",
                            token,
                            line_num + 1
                        ),
                        location: FileLocation::new(readme_path.clone())
                            .at_line((line_num + 1) as u32),
                        help: format!(
                            "Skill `{}` must exist in the skills directory or be added to known exceptions.",
                            token
                        ),
                    });
                }
            }
        }
    }

    diags
}

/// Tokens that look like skill names but are known exceptions.
fn is_known_exception(token: &str) -> bool {
    matches!(
        token,
        "story-id" | "capability-slug" | "story-123" | "story-bs01"
            | "ADR-XXX" | "PROJ-28"
            | "npm" | "cargo" | "dune" | "coqc"
            | "test" | "Linear"
            // Canonical delivery states -- not skills
            | "in-analysis" | "ready-for-dev" | "in-dev"
            | "ready-for-deskcheck" | "in-deskcheck"
            | "ready-for-qa" | "in-qa"
            | "ready-for-acceptance" | "in-acceptance"
            | "ready-to-deploy" | "done"
            | "halted-stall" | "halted-ambiguous"
            | "halted-human-gate" | "halted-unsafe"
            | "in-progress"
    ) || token.ends_with("-agent")
        || token.starts_with("story-")
}
