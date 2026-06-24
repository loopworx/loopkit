pub mod anti_patterns;
pub mod frontmatter;
pub mod naming;
pub mod progressive;
pub mod structure;
pub mod terminology;
pub mod workflow;

use loopkit_core::types::{Diagnostic, Skill};

/// Run all best-practices checks across all skills.
pub fn check_all(skills: &[Skill]) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for skill in skills {
        diagnostics.extend(frontmatter::check(skill));
        diagnostics.extend(naming::check(skill));
        diagnostics.extend(structure::check(skill));
        diagnostics.extend(progressive::check(skill));
        diagnostics.extend(terminology::check(skill));
        diagnostics.extend(workflow::check(skill));
        diagnostics.extend(anti_patterns::check(skill));
    }
    diagnostics.extend(naming::check_consistency(skills));
    diagnostics
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn make_skill(
        name: &str,
        description: &str,
        path: std::path::PathBuf,
        skill_md: std::path::PathBuf,
    ) -> Skill {
        Skill {
            name: name.into(),
            description: description.into(),
            level: String::new(),
            owner: vec![],
            category: String::new(),
            path,
            skill_md,
            sections: vec![],
            states: vec![],
        }
    }

    #[test]
    fn check_all_with_empty_skills_returns_empty() {
        let diags = check_all(&[]);
        assert!(diags.is_empty());
    }

    #[test]
    fn check_all_with_multiple_skills_returns_combined_diagnostics() {
        let dir = tempdir().unwrap();
        let md = dir.path().join("SKILL.md");
        std::fs::write(&md, "content\n").unwrap();
        let s1 = make_skill("", "", dir.path().to_path_buf(), md);

        let dir2 = tempdir().unwrap();
        let md2 = dir2.path().join("SKILL.md");
        std::fs::write(&md2, "content\n").unwrap();
        let s2 = make_skill("", "", dir2.path().to_path_buf(), md2);

        let diags = check_all(&[s1, s2]);
        let missing_name_count = diags.iter().filter(|d| d.code == "skill-missing-name").count();
        assert_eq!(missing_name_count, 2);
    }

    #[test]
    fn check_all_includes_cross_skill_naming_consistency() {
        let dir = tempdir().unwrap();
        let md = dir.path().join("SKILL.md");
        std::fs::write(&md, "# Test\n\nsome content\n").unwrap();
        let s1 = make_skill(
            "running-tests",
            "Processes test data",
            dir.path().to_path_buf(),
            md.clone(),
        );

        let dir2 = tempdir().unwrap();
        let md2 = dir2.path().join("SKILL.md");
        std::fs::write(&md2, "# Test\n\nsome content\n").unwrap();
        let s2 = make_skill(
            "helper",
            "Helps with things",
            dir2.path().to_path_buf(),
            md2.clone(),
        );

        let diags = check_all(&[s1, s2]);
        assert!(diags.iter().any(|d| d.code == "skill-naming-inconsistent"));
    }
}
