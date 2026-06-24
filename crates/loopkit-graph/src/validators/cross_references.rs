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
                            help: format!("Skill `{}` must exist in the skills directory.", target),
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
            path: path.clone(),
            skill_md,
            sections: vec![],
            states: vec![],
        }
    }

    #[test]
    fn loop_md_with_unknown_handoff_skill_warning() {
        let dir = TempDir::new().unwrap();
        let skill_dir = dir.path().join("my-skill");
        std::fs::create_dir(&skill_dir).unwrap();
        std::fs::write(skill_dir.join("SKILL.md"), "").unwrap();
        std::fs::write(
            skill_dir.join("LOOP.md"),
            "handoff unknown-skill to some-agent\n",
        )
        .unwrap();

        let skills = vec![make_skill("my-skill", skill_dir.clone())];
        let diags = validate(&skills, &skills);
        assert!(diags.iter().any(|d| d.code == "xref-unknown-handoff-skill"));
    }

    #[test]
    fn loop_md_with_known_skill_no_error() {
        let dir = TempDir::new().unwrap();
        let skill_dir = dir.path().join("my-skill");
        std::fs::create_dir(&skill_dir).unwrap();
        std::fs::write(skill_dir.join("SKILL.md"), "").unwrap();
        std::fs::write(
            skill_dir.join("LOOP.md"),
            "handoff known-skill to some-agent\n",
        )
        .unwrap();

        let skills = vec![
            make_skill("my-skill", skill_dir.clone()),
            make_skill("known-skill", dir.path().join("known-skill")),
        ];
        let diags = validate(&skills, &skills);
        assert!(!diags.iter().any(|d| d.code == "xref-unknown-handoff-skill"));
    }

    #[test]
    fn loop_md_handoff_done_no_warning() {
        let dir = TempDir::new().unwrap();
        let skill_dir = dir.path().join("my-skill");
        std::fs::create_dir(&skill_dir).unwrap();
        std::fs::write(skill_dir.join("SKILL.md"), "").unwrap();
        std::fs::write(skill_dir.join("LOOP.md"), "handoff done to all-agents\n").unwrap();

        let skills = vec![make_skill("my-skill", skill_dir.clone())];
        let diags = validate(&skills, &skills);
        assert!(diags.is_empty());
    }

    #[test]
    fn no_files_no_diagnostics() {
        let skills: Vec<Skill> = vec![];
        let diags = validate(&skills, &skills);
        assert!(diags.is_empty());
    }

    #[test]
    fn known_exceptions_are_recognized() {
        // Test various known exception tokens
        assert!(is_known_exception("story-id"));
        assert!(is_known_exception("capability-slug"));
        assert!(is_known_exception("story-123"));
        assert!(is_known_exception("story-bs01"));
        assert!(is_known_exception("ADR-XXX"));
        assert!(is_known_exception("PROJ-28"));
        assert!(is_known_exception("npm"));
        assert!(is_known_exception("cargo"));
        assert!(is_known_exception("dune"));
        assert!(is_known_exception("coqc"));
        assert!(is_known_exception("test"));
        assert!(is_known_exception("Linear"));
        assert!(is_known_exception("in-analysis"));
        assert!(is_known_exception("ready-for-dev"));
        assert!(is_known_exception("in-dev"));
        assert!(is_known_exception("ready-for-deskcheck"));
        assert!(is_known_exception("in-deskcheck"));
        assert!(is_known_exception("ready-for-qa"));
        assert!(is_known_exception("in-qa"));
        assert!(is_known_exception("ready-for-acceptance"));
        assert!(is_known_exception("in-acceptance"));
        assert!(is_known_exception("ready-to-deploy"));
        assert!(is_known_exception("done"));
        assert!(is_known_exception("halted-stall"));
        assert!(is_known_exception("halted-ambiguous"));
        assert!(is_known_exception("halted-human-gate"));
        assert!(is_known_exception("halted-unsafe"));
        assert!(is_known_exception("in-progress"));
        assert!(is_known_exception("some-agent")); // ends with -agent
        assert!(is_known_exception("story-abc123")); // starts with story-
        assert!(!is_known_exception("some-real-skill"));
    }
}
