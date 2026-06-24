use loopkit_core::types::{Diagnostic, Skill};

pub fn check(skill: &Skill) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let path = skill.skill_md.clone();

    if skill.name.is_empty() {
        diags.push(Diagnostic::error(
            "skill-missing-name",
            "name field missing from frontmatter".into(),
            path.clone(),
        ));
    } else {
        if skill.name.len() > 64 {
            diags.push(Diagnostic::error(
                "skill-name-too-long",
                format!(
                    "name '{}' exceeds 64 characters ({} chars)",
                    skill.name,
                    skill.name.len()
                ),
                path.clone(),
            ));
        }
        if skill
            .name
            .chars()
            .any(|c| !c.is_ascii_lowercase() && !c.is_ascii_digit() && c != '-')
        {
            diags.push(Diagnostic::error(
                "skill-name-invalid-chars",
                format!(
                    "name '{}' contains invalid characters (only [a-z0-9-] allowed)",
                    skill.name
                ),
                path.clone(),
            ));
        }
        for reserved in &["anthropic", "claude"] {
            if skill.name.contains(reserved) {
                diags.push(Diagnostic::error(
                    "skill-name-reserved-word",
                    format!(
                        "name '{}' contains reserved word '{}'",
                        skill.name, reserved
                    ),
                    path.clone(),
                ));
            }
        }
    }

    let desc = &skill.description;
    if desc.is_empty() {
        diags.push(Diagnostic::error(
            "skill-missing-description",
            "description field missing from frontmatter".into(),
            path.clone(),
        ));
    } else {
        if desc.len() > 1024 {
            diags.push(Diagnostic::error(
                "skill-description-too-long",
                format!(
                    "description exceeds 1024 characters ({} chars)",
                    desc.len()
                ),
                path.clone(),
            ));
        }
        if desc.contains('<') && desc.contains('>') {
            diags.push(Diagnostic::error(
                "skill-description-xml-tag",
                "description contains XML tags".into(),
                path.clone(),
            ));
        }
        let lower = desc.to_lowercase();
        if lower.starts_with("i ")
            || lower.starts_with("you ")
            || lower.starts_with("we ")
        {
            diags.push(Diagnostic::warning(
                "skill-description-not-third-person",
                "description appears to use first/second person. Use third person: 'Processes...' not 'I can...'".into(),
                path.clone(),
            ));
        }
    }

    diags
}

#[cfg(test)]
mod tests {
    use super::*;
    

    fn make_skill(name: &str, description: &str) -> Skill {
        Skill {
            name: name.into(),
            description: description.into(),
            level: String::new(),
            owner: vec![],
            category: String::new(),
            path: std::path::PathBuf::from("skills/test"),
            skill_md: std::path::PathBuf::from("skills/test/SKILL.md"),
            sections: vec![],
            states: vec![],
        }
    }

    #[test]
    fn missing_name_reports_error() {
        let skill = make_skill("", "A test skill");
        let diags = check(&skill);
        assert!(diags.iter().any(|d| d.code == "skill-missing-name"));
    }

    #[test]
    fn name_too_long_reports_error() {
        let skill = make_skill(
            &"a".repeat(65),
            "A test skill",
        );
        let diags = check(&skill);
        assert!(diags.iter().any(|d| d.code == "skill-name-too-long"));
    }

    #[test]
    fn name_invalid_chars_reports_error() {
        let skill = make_skill("Bad Name!", "test");
        let diags = check(&skill);
        assert!(diags.iter().any(|d| d.code == "skill-name-invalid-chars"));
    }

    #[test]
    fn reserved_word_reports_error() {
        let skill = make_skill("claude-skill", "test");
        let diags = check(&skill);
        assert!(diags.iter().any(|d| d.code == "skill-name-reserved-word"));
    }

    #[test]
    fn missing_description_reports_error() {
        let skill = make_skill("test-skill", "");
        let diags = check(&skill);
        assert!(diags.iter().any(|d| d.code == "skill-missing-description"));
    }

    #[test]
    fn description_too_long_reports_error() {
        let skill = make_skill("test-skill", &"x".repeat(1025));
        let diags = check(&skill);
        assert!(diags.iter().any(|d| d.code == "skill-description-too-long"));
    }

    #[test]
    fn xml_tags_reports_error() {
        let skill = make_skill("test-skill", "A <description> with tags");
        let diags = check(&skill);
        assert!(diags.iter().any(|d| d.code == "skill-description-xml-tag"));
    }

    #[test]
    fn first_person_reports_warning() {
        let skill = make_skill("test-skill", "I can do things");
        let diags = check(&skill);
        assert!(diags.iter().any(|d| d.code == "skill-description-not-third-person"));
    }

    #[test]
    fn valid_skill_no_diagnostics() {
        let skill = make_skill("test-skill", "Processes test data");
        let diags = check(&skill);
        assert!(diags.is_empty());
    }
}
