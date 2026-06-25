use loopkit_core::types::{Diagnostic, Skill};

pub fn check(skill: &Skill) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let path = skill.skill_md.clone();

    let content = std::fs::read_to_string(&path).unwrap_or_default();
    let find_name_line = || -> Option<u32> {
        content.lines().enumerate().find_map(|(i, line)| {
            if line.trim_start().starts_with("name:") {
                Some((i + 1) as u32)
            } else {
                None
            }
        })
    };

    if !skill.name.is_empty() {
        // Check parent directory name matches skill name
        let dir_name = skill
            .path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        if !dir_name.is_empty() && skill.name != dir_name {
            let line = find_name_line();
            let mut diag = Diagnostic::error(
                "skill-name-mismatch-dir",
                format!(
                    "name '{}' does not match parent directory '{}'. The name field must match the directory name",
                    skill.name, dir_name
                ),
                path.clone(),
            );
            if let Some(l) = line {
                diag = diag.at_line(l);
            }
            diags.push(diag);
        }

        // No leading hyphen
        if skill.name.starts_with('-') {
            let line = find_name_line();
            let mut diag = Diagnostic::error(
                "skill-name-leading-hyphen",
                format!(
                    "name '{}' starts with a hyphen. Names must not start with a hyphen",
                    skill.name
                ),
                path.clone(),
            );
            if let Some(l) = line {
                diag = diag.at_line(l);
            }
            diags.push(diag);
        }

        // No trailing hyphen
        if skill.name.ends_with('-') {
            let line = find_name_line();
            let mut diag = Diagnostic::error(
                "skill-name-trailing-hyphen",
                format!(
                    "name '{}' ends with a hyphen. Names must not end with a hyphen",
                    skill.name
                ),
                path.clone(),
            );
            if let Some(l) = line {
                diag = diag.at_line(l);
            }
            diags.push(diag);
        }

        // No consecutive hyphens
        if skill.name.contains("--") {
            let line = find_name_line();
            let mut diag = Diagnostic::error(
                "skill-name-consecutive-hyphens",
                format!(
                    "name '{}' contains consecutive hyphens. Names must not contain '--'",
                    skill.name
                ),
                path.clone(),
            );
            if let Some(l) = line {
                diag = diag.at_line(l);
            }
            diags.push(diag);
        }

        let first_word = skill.name.split('-').next().unwrap_or("");
        if !first_word.ends_with("ing") {
            let line = find_name_line();
            let mut diag = Diagnostic::warning(
                "skill-name-not-gerund",
                format!(
                    "name '{}' does not start with a gerund (-ing word). Consider a verb form like 'running-tests'",
                    skill.name
                ),
                path.clone(),
            );
            if let Some(l) = line {
                diag = diag.at_line(l);
            }
            diags.push(diag);
        }
    }

    let vague_names = ["helper", "utils", "tools", "misc"];
    for vague in &vague_names {
        if skill.name.contains(vague) {
            let line = find_name_line();
            let mut diag = Diagnostic::warning(
                "skill-name-vague",
                format!(
                    "name '{}' contains vague term '{}'. Use a specific, descriptive name",
                    skill.name, vague
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

pub fn check_consistency(skills: &[Skill]) -> Vec<Diagnostic> {
    let mut gerund_count = 0u32;
    let mut noun_count = 0u32;
    let mut action_count = 0u32;

    for skill in skills {
        if skill.name.is_empty() {
            continue;
        }
        if skill.name.ends_with("-ing") {
            gerund_count += 1;
        } else if skill.name.contains('-') {
            action_count += 1;
        } else {
            noun_count += 1;
        }
    }

    let total = gerund_count + noun_count + action_count;
    if total < 2 {
        return vec![];
    }

    let max_count = gerund_count.max(noun_count).max(action_count);
    if max_count < total {
        let skills_path = skills
            .first()
            .map(|s| s.path.clone())
            .unwrap_or(std::path::PathBuf::from("skills"));
        return vec![Diagnostic::warning(
            "skill-naming-inconsistent",
            format!(
                "Project has mixed naming patterns: {} gerund, {} noun, {} action-prefix names. \
                 Consider using a consistent convention.",
                gerund_count, noun_count, action_count
            ),
            skills_path,
        )];
    }

    vec![]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_skill(name: &str) -> Skill {
        Skill {
            name: name.into(),
            description: String::new(),
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
    fn non_gerund_name_reports_warning() {
        let skill = make_skill("test-skill");
        let diags = check(&skill);
        assert!(diags.iter().any(|d| d.code == "skill-name-not-gerund"));
    }

    #[test]
    fn gerund_name_no_warning() {
        let skill = make_skill("running-tests");
        let diags = check(&skill);
        assert!(!diags.iter().any(|d| d.code == "skill-name-not-gerund"));
    }

    #[test]
    fn vague_name_reports_warning() {
        let skill = make_skill("helper-tool");
        let diags = check(&skill);
        assert!(diags.iter().any(|d| d.code == "skill-name-vague"));
    }

    #[test]
    fn mixed_patterns_reports_warning() {
        let skills = vec![make_skill("running-tests"), make_skill("helper")];
        let diags = check_consistency(&skills);
        assert!(diags.iter().any(|d| d.code == "skill-naming-inconsistent"));
    }

    #[test]
    fn consistent_patterns_no_warning() {
        let skills = vec![make_skill("running-tests"), make_skill("building-code")];
        let diags = check_consistency(&skills);
        assert!(diags.is_empty());
    }

    #[test]
    fn name_mismatch_dir_reports_error() {
        let skill = Skill {
            name: "my-skill".into(),
            description: String::new(),
            level: String::new(),
            owner: vec![],
            category: String::new(),
            path: std::path::PathBuf::from("skills/wrong-dir"),
            skill_md: std::path::PathBuf::from("skills/wrong-dir/SKILL.md"),
            sections: vec![],
            states: vec![],
        };
        let diags = check(&skill);
        assert!(diags.iter().any(|d| d.code == "skill-name-mismatch-dir"));
    }

    #[test]
    fn leading_hyphen_reports_error() {
        let skill = make_skill("-bad-name");
        let diags = check(&skill);
        assert!(diags.iter().any(|d| d.code == "skill-name-leading-hyphen"));
    }

    #[test]
    fn trailing_hyphen_reports_error() {
        let skill = make_skill("bad-name-");
        let diags = check(&skill);
        assert!(diags.iter().any(|d| d.code == "skill-name-trailing-hyphen"));
    }

    #[test]
    fn consecutive_hyphens_reports_error() {
        let skill = make_skill("bad--name");
        let diags = check(&skill);
        assert!(diags
            .iter()
            .any(|d| d.code == "skill-name-consecutive-hyphens"));
    }

    #[test]
    fn empty_name_no_naming_diagnostics() {
        let skill = make_skill("");
        let diags = check(&skill);
        // empty name: dir check skipped, vague check doesn't match empty
        assert!(!diags.iter().any(|d| d.code == "skill-name-mismatch-dir"));
    }

    #[test]
    fn consistency_with_all_nouns_no_warning() {
        let skills = vec![make_skill("tests"), make_skill("builds")];
        let diags = check_consistency(&skills);
        assert!(diags.is_empty());
    }

    #[test]
    fn consistency_with_single_skill_no_warning() {
        let skills = vec![make_skill("running-tests")];
        let diags = check_consistency(&skills);
        assert!(diags.is_empty());
    }

    #[test]
    fn consistency_with_empty_names_skipped() {
        let skills = vec![make_skill(""), make_skill("")];
        let diags = check_consistency(&skills);
        assert!(diags.is_empty());
    }
}
