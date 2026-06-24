use crate::types::Transition;
use loopkit_core::parser::skill::extract_section_body;
use loopkit_core::state_name::is_state_like;
use loopkit_core::types::{Config, Diagnostic, FileLocation, Severity, Skill};
use std::collections::HashSet;

/// Check bidirectional consistency between SKILL.md state declarations
/// and the handoff graph.
///
/// Forward: every state declared in a SKILL.md State Model (or alias) must
/// appear in the graph.
/// Reverse: every graph node must be declared in at least one State Model section.
pub fn validate(skills: &[Skill], transitions: &[Transition], config: &Config) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    let graph_states: HashSet<&str> = {
        let mut s = HashSet::new();
        for t in transitions {
            s.insert(t.from.as_str());
            s.insert(t.to.as_str());
        }
        s
    };

    // --- Forward check: skill-declared states must be in the graph ---
    for skill in skills {
        if let Ok(content) = std::fs::read_to_string(&skill.skill_md) {
            let mut prose_states = Vec::new();

            for alias in &config.state_model_aliases {
                if let Some(body) = extract_section_body(&content, alias) {
                    prose_states.extend(extract_prose_states(&body));
                }
            }

            prose_states.sort();
            prose_states.dedup();

            for state in &prose_states {
                if !graph_states.contains(state.as_str()) {
                    diags.push(Diagnostic {
                        severity: Severity::Warning,
                        code: "state-undefined-in-graph".to_string(),
                        message: format!(
                            "Skill `{}` references state `{}` in its State Model but it is not in the handoff graph",
                            skill.name, state
                        ),
                        location: FileLocation::new(skill.skill_md.clone()),
                        help: format!(
                            "Add a transition involving `{}` to some LOOP.md, or remove it from the State Model.",
                            state
                        ),
                    });
                }
            }
        }
    }

    // --- Reverse check: graph nodes must be declared in some State Model ---
    let mut graph_declared: HashSet<String> = HashSet::new();

    for skill in skills {
        if let Ok(content) = std::fs::read_to_string(&skill.skill_md) {
            for alias in &config.state_model_aliases {
                if let Some(body) = extract_section_body(&content, alias) {
                    for state in extract_prose_states(&body) {
                        graph_declared.insert(state);
                    }
                }
            }
        }
    }

    for state in &graph_states {
        if !graph_declared.contains(*state) {
            diags.push(Diagnostic {
                severity: Severity::Warning,
                code: "state-undeclared-in-skill".to_string(),
                message: format!(
                    "State '{}' appears in the handoff graph but is not declared in any SKILL.md State Model section",
                    state
                ),
                location: FileLocation::new(skills.first().map_or_else(
                    || std::path::PathBuf::from("skills"),
                    |s| s.path.parent().unwrap_or(&s.path).to_path_buf(),
                )),
                help: format!(
                    "Declare '{}' in a SKILL.md under one of: {}.",
                    state,
                    config.state_model_aliases.join(", ")
                ),
            });
        }
    }

    diags
}

/// Extract state-like tokens from section body text.
/// Looks for backtick-quoted words.
fn extract_prose_states(section_body: &str) -> Vec<String> {
    if section_body.trim().is_empty() {
        return Vec::new();
    }

    let mut states: Vec<String> = Vec::new();

    let mut in_backtick = false;
    let mut current_start = 0usize;

    for (i, ch) in section_body.char_indices() {
        if ch == '`' {
            if in_backtick {
                let token = &section_body[current_start..i];
                if is_state_like(token) {
                    states.push(token.to_string());
                }
            }
            in_backtick = !in_backtick;
            current_start = i + 1;
        }
    }

    states.sort();
    states.dedup();
    states
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn t(from: &str, to: &str) -> Transition {
        Transition {
            from: from.into(),
            to: to.into(),
            skill: "s".into(),
            defined_in: PathBuf::from("x"),
        }
    }

    fn make_skill(name: &str, path: PathBuf) -> Skill {
        let skill_md = path.join("SKILL.md");
        Skill {
            name: name.into(),
            level: "L3".into(),
            owner: vec![],
            description: "".into(),
            category: "".into(),
            path,
            skill_md: skill_md.clone(),
            sections: vec![],
            states: vec![],
        }
    }

    #[test]
    fn extract_prose_states_parses_backtick_states() {
        let body = "state is `in-dev` and then `in-qa`";
        let states = extract_prose_states(body);
        assert_eq!(states.len(), 2);
        assert!(states.contains(&"in-dev".to_string()));
        assert!(states.contains(&"in-qa".to_string()));
    }

    #[test]
    fn extract_prose_states_filters_non_state_like() {
        let body = "skill is `running-tdd-loops` which is gerund";
        let states = extract_prose_states(body);
        assert!(states.is_empty());
    }

    #[test]
    fn extract_prose_states_empty_body() {
        let states = extract_prose_states("");
        assert!(states.is_empty());
    }

    #[test]
    fn extract_prose_states_whitespace_only() {
        let states = extract_prose_states("   \n  ");
        assert!(states.is_empty());
    }

    #[test]
    fn graph_state_declared_in_skill_no_error() {
        let dir = TempDir::new().unwrap();
        let skill_dir = dir.path().join("test-skill");
        std::fs::create_dir(&skill_dir).unwrap();
        std::fs::write(
            skill_dir.join("SKILL.md"),
            "\
## State Model
Current state is `in-dev`. Next is `in-qa`.
",
        )
        .unwrap();

        let skills = vec![make_skill("test-skill", skill_dir)];
        let transitions = vec![t("in-dev", "in-qa")];
        let mut config = Config::default();
        config.state_model_aliases = vec!["State Model".to_string()];

        let diags = validate(&skills, &transitions, &config);
        assert!(diags.is_empty(), "Expected no diagnostics but got: {:?}", diags);
    }

    #[test]
    fn graph_state_not_declared_in_skill_emits_warning() {
        let dir = TempDir::new().unwrap();
        let skill_dir = dir.path().join("test-skill");
        std::fs::create_dir(&skill_dir).unwrap();
        std::fs::write(
            skill_dir.join("SKILL.md"),
            "\
## State Model
This skill handles `in-dev`.
",
        )
        .unwrap();

        let skills = vec![make_skill("test-skill", skill_dir)];
        let transitions = vec![t("in-dev", "in-qa")];
        let mut config = Config::default();
        config.state_model_aliases = vec!["State Model".to_string()];

        let diags = validate(&skills, &transitions, &config);
        assert!(diags.iter().any(|d| d.code == "state-undeclared-in-skill"
            && d.message.contains("in-qa")));
    }

    #[test]
    fn skill_state_not_in_graph_emits_warning() {
        let dir = TempDir::new().unwrap();
        let skill_dir = dir.path().join("test-skill");
        std::fs::create_dir(&skill_dir).unwrap();
        std::fs::write(
            skill_dir.join("SKILL.md"),
            "\
## State Model
States: `in-dev`, `in-qa`, `orphan-state`.
",
        )
        .unwrap();

        let skills = vec![make_skill("test-skill", skill_dir)];
        let transitions = vec![t("in-dev", "in-qa")];
        let mut config = Config::default();
        config.state_model_aliases = vec!["State Model".to_string()];

        let diags = validate(&skills, &transitions, &config);
        assert!(diags.iter().any(|d| d.code == "state-undefined-in-graph"
            && d.message.contains("orphan-state")));
    }

    #[test]
    fn empty_transitions_no_warnings() {
        let dir = TempDir::new().unwrap();
        let skill_dir = dir.path().join("test-skill");
        std::fs::create_dir(&skill_dir).unwrap();
        std::fs::write(skill_dir.join("SKILL.md"), "").unwrap();

        let skills = vec![make_skill("test-skill", skill_dir)];
        let config = Config::default();
        let diags = validate(&skills, &[], &config);
        assert!(diags.is_empty());
    }

    #[test]
    fn empty_skills_with_graph_states_uses_fallback_location() {
        // Tests the map_or_else path when skills is empty
        let _dir = TempDir::new().unwrap();
        let transitions = vec![t("in-dev", "in-qa")];
        let config = Config::default();
        let diags = validate(&[], &transitions, &config);
        // "in-dev" and "in-qa" are in graph but no skills declare them
        assert!(diags.iter().any(|d| d.code == "state-undeclared-in-skill"));
    }

    #[test]
    fn unreadable_skill_md_is_skipped() {
        // Create a skill with a SKILL.md path that doesn't exist / can't be read
        let dir = TempDir::new().unwrap();
        let skill_dir = dir.path().join("test-skill");
        std::fs::create_dir(&skill_dir).unwrap();
        // Don't create SKILL.md — read_to_string will fail
        let skill = Skill {
            name: "test-skill".into(),
            level: "L3".into(),
            owner: vec![],
            description: "".into(),
            category: "".into(),
            path: skill_dir.clone(),
            skill_md: skill_dir.join("SKILL.md"),
            sections: vec![],
            states: vec![],
        };

        let transitions = vec![t("in-dev", "in-qa")];
        let config = Config::default();
        let diags = validate(&[skill], &transitions, &config);
        // Both states undeclared since skill wasn't read
        assert!(diags.iter().any(|d| d.code == "state-undeclared-in-skill"));
    }

    #[test]
    fn empty_section_body_is_skipped() {
        // Tests the empty body early return in extract_prose_states
        let dir = TempDir::new().unwrap();
        let skill_dir = dir.path().join("test-skill");
        std::fs::create_dir(&skill_dir).unwrap();
        std::fs::write(
            skill_dir.join("SKILL.md"),
            "\
## State Model

",
        )
        .unwrap();

        let skills = vec![make_skill("test-skill", skill_dir)];
        let transitions = vec![t("in-dev", "in-qa")];
        let mut config = Config::default();
        config.state_model_aliases = vec!["State Model".to_string()];
        let diags = validate(&skills, &transitions, &config);
        // Both states undeclared in skill (empty section body → no states extracted)
        assert!(diags.iter().any(|d| d.code == "state-undeclared-in-skill"));
    }
}
