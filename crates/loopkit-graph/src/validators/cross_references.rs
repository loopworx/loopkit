use loopkit_core::types::{Config, Diagnostic, FileLocation, Severity, Skill};
use regex::Regex;
use std::collections::HashSet;

/// Check that backtick-quoted skill references in README.md and
/// `handoff <skill> to <agent>` directives in LOOP.md files point to
/// skills that actually exist.
pub fn validate(skills: &[Skill], _all_skills: &[Skill], config: &Config) -> Vec<Diagnostic> {
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
                if is_known_exception(token, config) {
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
fn is_known_exception(token: &str, config: &Config) -> bool {
    // Generic non-skill tokens
    if matches!(
        token,
        "story-id"
            | "capability-slug"
            | "story-123"
            | "story-bs01"
            | "ADR-XXX"
            | "PROJ-28"
            | "npm"
            | "cargo"
            | "dune"
            | "coqc"
            | "test"
            | "Linear"
            | "all-agents"
            | "in-progress"
    ) {
        return true;
    }
    // Config-driven: enforced states from .loopkit.yaml
    if config.enforced_states.iter().any(|s| s.name == token) {
        return true;
    }
    // Config-driven: halted-* states derived from halt_reasons
    if config
        .halt_reasons
        .iter()
        .any(|r| format!("halted-{}", r) == token)
    {
        return true;
    }
    token.ends_with("-agent") || token.starts_with("story-")
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

    fn test_config() -> Config {
        let mut config = Config::default();
        config.enforced_states = vec![
            loopkit_core::types::EnforcedState {
                name: "in-analysis".into(),
                agent: "".into(),
                description: "".into(),
            },
            loopkit_core::types::EnforcedState {
                name: "in-dev".into(),
                agent: "".into(),
                description: "".into(),
            },
            loopkit_core::types::EnforcedState {
                name: "in-deskcheck".into(),
                agent: "".into(),
                description: "".into(),
            },
            loopkit_core::types::EnforcedState {
                name: "in-qa".into(),
                agent: "".into(),
                description: "".into(),
            },
            loopkit_core::types::EnforcedState {
                name: "in-acceptance".into(),
                agent: "".into(),
                description: "".into(),
            },
            loopkit_core::types::EnforcedState {
                name: "done".into(),
                agent: "".into(),
                description: "".into(),
            },
        ];
        config.halt_reasons = vec![
            "stall".into(),
            "ambiguous".into(),
            "human-gate".into(),
            "unsafe".into(),
        ];
        config
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
        let config = test_config();
        let diags = validate(&skills, &skills, &config);
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
        let config = test_config();
        let diags = validate(&skills, &skills, &config);
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
        let config = test_config();
        let diags = validate(&skills, &skills, &config);
        assert!(diags.is_empty());
    }

    #[test]
    fn no_files_no_diagnostics() {
        let skills: Vec<Skill> = vec![];
        let config = Config::default();
        let diags = validate(&skills, &skills, &config);
        assert!(diags.is_empty());
    }

    #[test]
    fn known_exceptions_are_recognized() {
        let config = test_config();
        // Generic exceptions
        assert!(is_known_exception("story-id", &config));
        assert!(is_known_exception("capability-slug", &config));
        assert!(is_known_exception("story-123", &config));
        assert!(is_known_exception("story-bs01", &config));
        assert!(is_known_exception("ADR-XXX", &config));
        assert!(is_known_exception("PROJ-28", &config));
        assert!(is_known_exception("npm", &config));
        assert!(is_known_exception("cargo", &config));
        assert!(is_known_exception("dune", &config));
        assert!(is_known_exception("coqc", &config));
        assert!(is_known_exception("test", &config));
        assert!(is_known_exception("Linear", &config));
        assert!(is_known_exception("all-agents", &config));
        assert!(is_known_exception("in-progress", &config));
        // Config-driven: from enforced_states
        assert!(is_known_exception("in-analysis", &config));
        assert!(is_known_exception("in-dev", &config));
        assert!(is_known_exception("in-deskcheck", &config));
        assert!(is_known_exception("in-qa", &config));
        assert!(is_known_exception("in-acceptance", &config));
        assert!(is_known_exception("done", &config));
        // Config-driven: from halt_reasons
        assert!(is_known_exception("halted-stall", &config));
        assert!(is_known_exception("halted-ambiguous", &config));
        assert!(is_known_exception("halted-human-gate", &config));
        assert!(is_known_exception("halted-unsafe", &config));
        // Patterns
        assert!(is_known_exception("some-agent", &config)); // ends with -agent
        assert!(is_known_exception("story-abc123", &config)); // starts with story-
        assert!(!is_known_exception("some-real-skill", &config));
    }
}
