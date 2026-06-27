use loopkit_core::types::{Diagnostic, Skill};

pub fn check(skill: &Skill) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let path = skill.skill_md.clone();

    let content = std::fs::read_to_string(&path).unwrap_or_default();

    // Check: frontmatter must exist (--- ... ---)
    let has_frontmatter = content.starts_with("---");
    if !has_frontmatter {
        diags.push(Diagnostic::error(
            "skill-missing-frontmatter",
            "SKILL.md must start with YAML frontmatter (---)".into(),
            path.clone(),
        ));
    }

    // Helper: find the line number of a YAML key in the frontmatter
    let find_yaml_line = |key: &str| -> Option<u32> {
        content.lines().enumerate().find_map(|(i, line)| {
            if line.trim_start().starts_with(&format!("{}:", key)) {
                Some((i + 1) as u32)
            } else {
                None
            }
        })
    };

    // Helper: find the --- end marker of frontmatter
    let find_frontmatter_end = || -> Option<u32> {
        let mut in_frontmatter = false;
        for (i, line) in content.lines().enumerate() {
            if i == 0 && line.trim() == "---" {
                in_frontmatter = true;
                continue;
            }
            if in_frontmatter && line.trim() == "---" {
                return Some((i + 1) as u32);
            }
        }
        None
    };

    if skill.name.is_empty() {
        let line = find_yaml_line("name").or_else(find_frontmatter_end);
        let mut diag = Diagnostic::error(
            "skill-missing-name",
            "name field missing from frontmatter".into(),
            path.clone(),
        );
        if let Some(l) = line {
            diag = diag.at_line(l);
        }
        diags.push(diag);
    } else {
        if skill.name.len() > 64 {
            let line = find_yaml_line("name");
            let mut diag = Diagnostic::error(
                "skill-name-too-long",
                format!(
                    "name '{}' exceeds 64 characters ({} chars)",
                    skill.name,
                    skill.name.len()
                ),
                path.clone(),
            );
            if let Some(l) = line {
                diag = diag.at_line(l);
            }
            diags.push(diag);
        }
        if skill
            .name
            .chars()
            .any(|c| !c.is_ascii_lowercase() && !c.is_ascii_digit() && c != '-')
        {
            let line = find_yaml_line("name");
            let mut diag = Diagnostic::error(
                "skill-name-invalid-chars",
                format!(
                    "name '{}' contains invalid characters (only [a-z0-9-] allowed)",
                    skill.name
                ),
                path.clone(),
            );
            if let Some(l) = line {
                diag = diag.at_line(l);
            }
            diags.push(diag);
        }
        for reserved in &["anthropic", "claude"] {
            if skill.name.contains(reserved) {
                let line = find_yaml_line("name");
                let mut diag = Diagnostic::error(
                    "skill-name-reserved-word",
                    format!(
                        "name '{}' contains reserved word '{}'",
                        skill.name, reserved
                    ),
                    path.clone(),
                );
                if let Some(l) = line {
                    diag = diag.at_line(l);
                }
                diags.push(diag);
            }
        }
    }

    let desc = &skill.description;
    if desc.is_empty() {
        let line = find_yaml_line("description").or_else(find_frontmatter_end);
        let mut diag = Diagnostic::error(
            "skill-missing-description",
            "description field missing from frontmatter".into(),
            path.clone(),
        );
        if let Some(l) = line {
            diag = diag.at_line(l);
        }
        diags.push(diag);
    } else {
        if desc.len() > 1024 {
            let line = find_yaml_line("description");
            let mut diag = Diagnostic::error(
                "skill-description-too-long",
                format!("description exceeds 1024 characters ({} chars)", desc.len()),
                path.clone(),
            );
            if let Some(l) = line {
                diag = diag.at_line(l);
            }
            diags.push(diag);
        }
        if desc.contains('<') && desc.contains('>') {
            let line = find_yaml_line("description");
            let mut diag = Diagnostic::error(
                "skill-description-xml-tag",
                "description contains XML tags".into(),
                path.clone(),
            );
            if let Some(l) = line {
                diag = diag.at_line(l);
            }
            diags.push(diag);
        }
        let lower = desc.to_lowercase();
        if lower.starts_with("i ") || lower.starts_with("you ") || lower.starts_with("we ") {
            let line = find_yaml_line("description");
            let mut diag = Diagnostic::warning(
                "skill-description-not-third-person",
                "description appears to use first/second person. Use third person: 'Processes...' not 'I can...'".into(),
                path.clone(),
            );
            if let Some(l) = line {
                diag = diag.at_line(l);
            }
            diags.push(diag);
        }
    }

    // Optional frontmatter fields: compatibility, license, metadata, allowed-tools
    // Parse raw frontmatter to check these fields
    let raw_frontmatter = {
        if let Ok(c) = std::fs::read_to_string(&path) {
            let (fm, _) = loopkit_core::parser::skill::parse_frontmatter(&c);
            fm
        } else {
            return diags;
        }
    };

    // Validate compatibility length (max 500 chars)
    if let Some(compat) = raw_frontmatter.get("compatibility") {
        if compat.len() > 500 {
            let line = find_yaml_line("compatibility");
            let mut diag = Diagnostic::error(
                "skill-compatibility-too-long",
                format!(
                    "compatibility field exceeds 500 characters ({} chars)",
                    compat.len()
                ),
                path.clone(),
            );
            if let Some(l) = line {
                diag = diag.at_line(l);
            }
            diags.push(diag);
        }
    }

    // Validate allowed-tools is space-separated if present
    if let Some(tools) = raw_frontmatter.get("allowed-tools") {
        if tools.contains(',') {
            let line = find_yaml_line("allowed-tools");
            let mut diag = Diagnostic::warning(
                "skill-allowed-tools-format",
                "allowed-tools should be space-separated, not comma-separated".into(),
                path.clone(),
            );
            if let Some(l) = line {
                diag = diag.at_line(l);
            }
            diags.push(diag);
        }
    }

    // Validate metadata: must be string-to-string map
    // Check metadata inline values for proper nesting
    if let Some(meta_val) = raw_frontmatter.get("metadata") {
        if meta_val.contains('{') || meta_val.contains('[') {
            let line = find_yaml_line("metadata");
            let mut diag = Diagnostic::warning(
                "skill-metadata-inline-map",
                "metadata should use indented sub-keys, not inline maps. Use:\n  metadata:\n    category: my-category".into(),
                path.clone(),
            );
            if let Some(l) = line {
                diag = diag.at_line(l);
            }
            diags.push(diag);
        }
    }

    // Check that metadata sub-keys are not further nested (must be string values)
    for key in raw_frontmatter.keys() {
        if key.starts_with("metadata.") {
            let parts: Vec<&str> = key.split('.').collect();
            if parts.len() > 2 {
                let line = find_yaml_line(parts.last().unwrap_or(&""));
                let mut diag = Diagnostic::warning(
                    "skill-metadata-deep-nesting",
                    format!(
                        "metadata key '{}' is nested more than one level deep. metadata supports only string-to-string maps",
                        key
                    ),
                    path.clone(),
                );
                if let Some(l) = line.or_else(|| find_yaml_line("metadata")) {
                    diag = diag.at_line(l);
                }
                diags.push(diag);
            }
        }
    }

    // Check for unknown frontmatter keys (beyond spec + common extensions)
    let known_keys: &[&str] = &[
        "name",
        "description",
        "license",
        "compatibility",
        "metadata",
        "allowed-tools",
        "level",
        "owner",
        "trigger",
    ];
    for key in raw_frontmatter.keys() {
        // Allow dotted sub-keys of known parents (e.g., metadata.category)
        let base_key = key.split('.').next().unwrap_or(key);
        if !known_keys.contains(&base_key) && !key.starts_with("x-") {
            let term = key.split('.').next().unwrap_or(key);
            let line = find_yaml_line(term);
            let mut diag = Diagnostic::warning(
                "skill-unknown-frontmatter-key",
                format!(
                    "Unknown frontmatter key '{}'. Use only spec-defined keys or prefix custom keys with 'x-'",
                    key
                ),
                path.clone(),
            );
            if let Some(l) = line {
                diag = diag.at_line(l);
            }
            diags.push(diag);
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
        let skill = make_skill(&"a".repeat(65), "A test skill");
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
        assert!(diags
            .iter()
            .any(|d| d.code == "skill-description-not-third-person"));
    }

    #[test]
    fn valid_skill_no_diagnostics() {
        let dir = tempfile::tempdir().unwrap();
        let md_path = dir.path().join("SKILL.md");
        std::fs::write(
            &md_path,
            "---\nname: test-skill\ndescription: Processes test data\n---\n",
        )
        .unwrap();

        let skill = Skill {
            name: "test-skill".into(),
            description: "Processes test data".into(),
            level: String::new(),
            owner: vec![],
            category: String::new(),
            path: dir.path().to_path_buf(),
            skill_md: md_path,
            sections: vec![],
            states: vec![],
        };
        let diags = check(&skill);
        assert!(diags.is_empty());
    }

    #[test]
    fn compatibility_too_long_reports_error() {
        let dir = tempfile::tempdir().unwrap();
        let md_path = dir.path().join("SKILL.md");
        let compat = "x".repeat(501);
        std::fs::write(
            &md_path,
            format!(
                "---\nname: test-skill\ndescription: A test\ncompatibility: {}\n---\n",
                compat
            ),
        )
        .unwrap();

        let skill = Skill {
            name: "test-skill".into(),
            description: "A test".into(),
            level: String::new(),
            owner: vec![],
            category: String::new(),
            path: dir.path().to_path_buf(),
            skill_md: md_path,
            sections: vec![],
            states: vec![],
        };
        let diags = check(&skill);
        assert!(diags
            .iter()
            .any(|d| d.code == "skill-compatibility-too-long"));
    }

    #[test]
    fn allowed_tools_comma_separated_reports_warning() {
        let dir = tempfile::tempdir().unwrap();
        let md_path = dir.path().join("SKILL.md");
        std::fs::write(
            &md_path,
            "---\nname: test-skill\ndescription: A test\nallowed-tools: tool-a,tool-b\n---\n",
        )
        .unwrap();

        let skill = Skill {
            name: "test-skill".into(),
            description: "A test".into(),
            level: String::new(),
            owner: vec![],
            category: String::new(),
            path: dir.path().to_path_buf(),
            skill_md: md_path,
            sections: vec![],
            states: vec![],
        };
        let diags = check(&skill);
        assert!(diags.iter().any(|d| d.code == "skill-allowed-tools-format"));
    }

    #[test]
    fn metadata_inline_map_reports_warning() {
        let dir = tempfile::tempdir().unwrap();
        let md_path = dir.path().join("SKILL.md");
        std::fs::write(
            &md_path,
            "---\nname: test-skill\ndescription: A test\nmetadata: {\"key\": \"val\"}\n---\n",
        )
        .unwrap();

        let skill = Skill {
            name: "test-skill".into(),
            description: "A test".into(),
            level: String::new(),
            owner: vec![],
            category: String::new(),
            path: dir.path().to_path_buf(),
            skill_md: md_path,
            sections: vec![],
            states: vec![],
        };
        let diags = check(&skill);
        assert!(diags.iter().any(|d| d.code == "skill-metadata-inline-map"));
    }

    #[test]
    fn unknown_frontmatter_key_reports_warning() {
        let dir = tempfile::tempdir().unwrap();
        let md_path = dir.path().join("SKILL.md");
        std::fs::write(
            &md_path,
            "---\nname: test-skill\ndescription: A test\nunknown-key: value\n---\n",
        )
        .unwrap();

        let skill = Skill {
            name: "test-skill".into(),
            description: "A test".into(),
            level: String::new(),
            owner: vec![],
            category: String::new(),
            path: dir.path().to_path_buf(),
            skill_md: md_path,
            sections: vec![],
            states: vec![],
        };
        let diags = check(&skill);
        assert!(diags
            .iter()
            .any(|d| d.code == "skill-unknown-frontmatter-key"));
    }

    #[test]
    fn x_prefixed_custom_key_no_warning() {
        let dir = tempfile::tempdir().unwrap();
        let md_path = dir.path().join("SKILL.md");
        std::fs::write(
            &md_path,
            "---\nname: test-skill\ndescription: A test\nx-custom: value\n---\n",
        )
        .unwrap();

        let skill = Skill {
            name: "test-skill".into(),
            description: "A test".into(),
            level: String::new(),
            owner: vec![],
            category: String::new(),
            path: dir.path().to_path_buf(),
            skill_md: md_path,
            sections: vec![],
            states: vec![],
        };
        let diags = check(&skill);
        assert!(!diags
            .iter()
            .any(|d| d.code == "skill-unknown-frontmatter-key"));
    }

    #[test]
    fn second_person_description_reports_warning() {
        let skill = make_skill("test-skill", "You can do things");
        let diags = check(&skill);
        assert!(diags
            .iter()
            .any(|d| d.code == "skill-description-not-third-person"));
    }

    #[test]
    fn we_description_reports_warning() {
        let skill = make_skill("test-skill", "We process data");
        let diags = check(&skill);
        assert!(diags
            .iter()
            .any(|d| d.code == "skill-description-not-third-person"));
    }

    #[test]
    fn anthropic_reserved_word_reports_error() {
        let skill = make_skill("anthropic-skill", "test");
        let diags = check(&skill);
        assert!(diags.iter().any(|d| d.code == "skill-name-reserved-word"));
    }

    #[test]
    fn missing_frontmatter_reports_error() {
        let dir = tempfile::tempdir().unwrap();
        let md_path = dir.path().join("SKILL.md");
        std::fs::write(&md_path, "Just some content, no frontmatter.").unwrap();

        let skill = Skill {
            name: "test-skill".into(),
            description: "A test".into(),
            level: String::new(),
            owner: vec![],
            category: String::new(),
            path: dir.path().to_path_buf(),
            skill_md: md_path,
            sections: vec![],
            states: vec![],
        };
        let diags = check(&skill);
        assert!(diags.iter().any(|d| d.code == "skill-missing-frontmatter"));
        // Should return early — no other checks should fire
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn with_frontmatter_no_missing_frontmatter_error() {
        let dir = tempfile::tempdir().unwrap();
        let md_path = dir.path().join("SKILL.md");
        std::fs::write(
            &md_path,
            "---\nname: test-skill\ndescription: A test\n---\n",
        )
        .unwrap();

        let skill = Skill {
            name: "test-skill".into(),
            description: "A test".into(),
            level: String::new(),
            owner: vec![],
            category: String::new(),
            path: dir.path().to_path_buf(),
            skill_md: md_path,
            sections: vec![],
            states: vec![],
        };
        let diags = check(&skill);
        assert!(!diags.iter().any(|d| d.code == "skill-missing-frontmatter"));
    }
}
