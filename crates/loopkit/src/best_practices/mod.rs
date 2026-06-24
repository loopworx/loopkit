pub mod anti_patterns;
pub mod frontmatter;
pub mod naming;
pub mod progressive;
pub mod scripts;
pub mod structure;
pub mod terminology;
pub mod workflow;

use loopkit_core::types::{Diagnostic, Skill};

/// Run all best-practices checks across all skills.
pub fn check_all(skills: &[Skill], verbose: bool) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    if verbose {
        eprintln!("=== best-practices validators ===");
    }

    macro_rules! run_check {
        ($label:expr, $call:expr) => {{
            let before = diagnostics.len();
            for skill in skills {
                diagnostics.extend($call(skill));
            }
            let count = diagnostics.len() - before;
            if verbose {
                if count > 0 {
                    eprintln!("  {:>30}  {} diagnostics", $label, count);
                } else {
                    eprintln!("  {:>30}  ✓", $label);
                }
            }
        }};
    }

    run_check!("frontmatter", frontmatter::check);
    run_check!("naming", naming::check);
    run_check!("structure", structure::check);
    run_check!("progressive", progressive::check);
    run_check!("terminology", terminology::check);
    run_check!("workflow", workflow::check);
    run_check!("anti_patterns", anti_patterns::check);
    run_check!("scripts", scripts::check);

    let before = diagnostics.len();
    diagnostics.extend(naming::check_consistency(skills));
    let count = diagnostics.len() - before;
    if verbose && count > 0 {
        eprintln!("  {:>30}  {} diagnostics", "naming_consistency", count);
    }

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
        let diags = check_all(&[], false);
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

        let diags = check_all(&[s1, s2], false);
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

        let diags = check_all(&[s1, s2], false);
        assert!(diags.iter().any(|d| d.code == "skill-naming-inconsistent"));
    }
}
