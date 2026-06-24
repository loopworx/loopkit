use crate::types::Transition;
use loopkit_core::types::{Config, Diagnostic, FileLocation, Severity, Skill};
use std::collections::HashSet;
use std::path::PathBuf;

/// Validate configuration constraints.
/// Checks that:
/// 1. Graph is not empty (at least one transition exists)
/// 2. Handoff targets reference existing skills
pub fn validate(transitions: &[Transition], skills: &[Skill], _config: &Config) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    // Graph should not be empty
    if transitions.is_empty() {
        diags.push(Diagnostic {
            severity: Severity::Error,
            code: "constraints-empty-graph".to_string(),
            message: "No transitions found. No transition rules defined in any LOOP.md."
                .to_string(),
            location: FileLocation::new(PathBuf::from("skills")),
            help: "Add transition rules to at least one LOOP.md file using: \
                transition <from> -> <to>"
                .to_string(),
        });
    }

    // Handoff targets should reference existing skills
    let known: HashSet<&str> = skills.iter().map(|s| s.name.as_str()).collect();

    // The cross_references validator already checks handoff directives.
    // Here we do a basic consistency check: every skill referenced as a
    // handoff target should exist.
    for t in transitions {
        // Transition edges represent skill -> state mappings.
        // The skill field is the owning skill. If a transition target
        // looks like a skill name (gerund, no hyphen) and doesn't exist,
        // it might be a handoff to a missing skill.
        let target = &t.to;
        // Only check targets that look like skill names (not state names)
        // Skill names end in -ing and don't start with typical state prefixes
        if target.split('-').any(|p| p.ends_with("ing")) && !known.contains(target.as_str()) {
            diags.push(Diagnostic {
                severity: Severity::Error,
                code: "constraints-unknown-handoff-target".to_string(),
                message: format!(
                    "Skill `{}` transitions to unknown handoff target `{}`",
                    t.skill, target
                ),
                location: FileLocation::new(PathBuf::from("skills")),
                help: format!("Skill `{}` must exist in the skills directory.", target),
            });
        }
    }

    diags
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(from: &str, to: &str) -> Transition {
        Transition {
            from: from.into(),
            to: to.into(),
            skill: "s".into(),
            defined_in: PathBuf::from("x"),
        }
    }

    fn make_skill(name: &str) -> Skill {
        Skill {
            name: name.into(),
            level: "L3".into(),
            owner: vec![],
            description: "".into(),
            category: "".into(),
            path: PathBuf::from("skills").join(name),
            skill_md: PathBuf::from("skills").join(name).join("SKILL.md"),
            sections: vec![],
            states: vec![],
        }
    }

    #[test]
    fn empty_transitions_emits_error() {
        let config = Config::default();
        let diags = validate(&[], &[], &config);
        assert!(diags.iter().any(|d| d.code == "constraints-empty-graph"));
    }

    #[test]
    fn unknown_gerund_handoff_target_emits_error() {
        let config = Config::default();
        let skills = vec![make_skill("my-skill")];
        let transitions = vec![t("a", "running-desk-checks")];
        let diags = validate(&transitions, &skills, &config);
        assert!(diags
            .iter()
            .any(|d| d.code == "constraints-unknown-handoff-target"));
    }

    #[test]
    fn valid_transitions_no_diagnostics() {
        let config = Config::default();
        let skills = vec![make_skill("my-skill")];
        let transitions = vec![t("in-dev", "in-qa")];
        let diags = validate(&transitions, &skills, &config);
        assert!(diags.is_empty());
    }

    #[test]
    fn transitions_to_known_skill_no_error() {
        let config = Config::default();
        let skills = vec![make_skill("running-desk-checks"), make_skill("my-skill")];
        let transitions = vec![t("a", "running-desk-checks")];
        let diags = validate(&transitions, &skills, &config);
        assert!(diags
            .iter()
            .all(|d| d.code != "constraints-unknown-handoff-target"));
    }
}
