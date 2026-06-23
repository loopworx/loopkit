//! Cross-reference validator: checks that skill names referenced in
//! README.md and LOOP.md handoff directives correspond to real skills.

use crate::types::{Diagnostic, FileLocation, Repo, Severity};
use regex::Regex;

/// Check that backtick-quoted skill references in README.md and
/// `handoff <skill> to <agent>` directives in LOOP.md files point to
/// skills that actually exist.
pub fn validate_cross_references(repo: &Repo) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let known_skills: std::collections::HashSet<&str> =
        repo.skills.iter().map(|s| s.name.as_str()).collect();

    // Check handoff directives in LOOP.md files
    let handoff_re = Regex::new(r"handoff\s+(\w[\w-]*)\s+to\s+\w[\w-]*").expect("regex");

    for skill in &repo.skills {
        if !skill.has_loop_md {
            continue;
        }
        let loop_path = skill.loop_md();
        if let Ok(content) = std::fs::read_to_string(&loop_path) {
            for (line_num, line) in content.lines().enumerate() {
                for cap in handoff_re.captures_iter(line) {
                    let target = &cap[1];
                    // "done" and "all-agents" are not skills — skip
                    if target == "done" || target == "all-agents" {
                        continue;
                    }
                    if !known_skills.contains(target) {
                        diags.push(Diagnostic {
                            severity: Severity::Warning,
                            code: "xref-unknown-handoff-skill".to_string(),
                            message: format!(
                                "LOOP.md for `{}` references unknown skill `{}` at line {}",
                                skill.name, target, line_num + 1
                            ),
                            location: FileLocation {
                                path: loop_path.clone(),
                                line: Some(line_num + 1),
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
    }

    // Check README.md for backtick-quoted skill references
    let readme_path = repo.root.join("README.md");
    if let Ok(content) = std::fs::read_to_string(&readme_path) {
        let backtick_re = Regex::new(r"`(\w[\w-]*)`").expect("regex");
        for (line_num, line) in content.lines().enumerate() {
            for cap in backtick_re.captures_iter(line) {
                let token = &cap[1];
                // Skip common non-skill tokens
                if is_known_exception(token) {
                    continue;
                }
                // Only flag if it looks like a skill name (contains a hyphen,
                // which is the naming convention) but isn't a known skill
                if token.contains('-') && !known_skills.contains(token) {
                    diags.push(Diagnostic {
                        severity: Severity::Warning,
                        code: "xref-unknown-readme-skill".to_string(),
                        message: format!(
                            "README.md references unknown skill `{}` at line {}",
                            token,
                            line_num + 1
                        ),
                        location: FileLocation {
                            path: readme_path.clone(),
                            line: Some(line_num + 1),
                            column: None,
                        },
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
            // Canonical delivery states — not skills
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
