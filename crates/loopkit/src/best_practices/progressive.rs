use loopkit_core::types::{Diagnostic, Skill};
use regex::Regex;

pub fn check(skill: &Skill) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let path = skill.skill_md.clone();

    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return diags,
    };

    let line_count = content.lines().count();

    if line_count > 200 {
        let ref_re = Regex::new(r"\[([^\]]+)\]\(([^)]+\.md)\)").expect("hardcoded regex");
        let has_reference_files = ref_re.is_match(&content);

        if !has_reference_files {
            diags.push(Diagnostic::warning(
                "skill-no-progressive-disclosure",
                format!(
                    "SKILL.md is {} lines with no separate reference files. \
                     Consider extracting content into linked reference files for progressive disclosure",
                    line_count
                ),
                path.clone(),
            ));
        }
    }

    // Orphan reference files: .md files in skill dir not linked from SKILL.md
    let ref_re = Regex::new(r"\[([^\]]+)\]\(([^)]+\.md)\)").expect("hardcoded regex");
    let linked_refs: std::collections::HashSet<String> = ref_re
        .captures_iter(&content)
        .map(|cap| cap[2].to_string().to_lowercase())
        .collect();

    if let Ok(entries) = std::fs::read_dir(&skill.path) {
        for entry in entries.flatten() {
            let file_name = entry.file_name().to_string_lossy().to_string();
            let lower = file_name.to_lowercase();
            if lower.ends_with(".md") && lower != "skill.md" && lower != "loop.md" {
                if !linked_refs.contains(&lower) {
                    diags.push(Diagnostic::warning(
                        "skill-orphan-reference",
                        format!(
                            "File '{}' exists in skill directory but is not linked from SKILL.md",
                            file_name
                        ),
                        entry.path(),
                    ));
                }
            }
        }
    }

    diags
}

#[cfg(test)]
mod tests {
    use super::*;
    
    
    use tempfile::tempdir;

    fn make_skill(name: &str, path: std::path::PathBuf, skill_md: std::path::PathBuf) -> Skill {
        Skill {
            name: name.into(),
            description: String::new(),
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
    fn long_skill_without_refs_reports_warning() {
        let dir = tempdir().unwrap();
        let md_path = dir.path().join("SKILL.md");
        let body = "line\n".repeat(201);
        std::fs::write(&md_path, &body).unwrap();

        let skill = make_skill("test-skill", dir.path().to_path_buf(), md_path);
        let diags = check(&skill);
        assert!(diags.iter().any(|d| d.code == "skill-no-progressive-disclosure"));
    }

    #[test]
    fn short_skill_no_warning() {
        let dir = tempdir().unwrap();
        let md_path = dir.path().join("SKILL.md");
        std::fs::write(&md_path, "short body").unwrap();

        let skill = make_skill("test-skill", dir.path().to_path_buf(), md_path);
        let diags = check(&skill);
        assert!(diags.is_empty());
    }

    #[test]
    fn orphan_reference() {
        let dir = tempdir().unwrap();
        let md_path = dir.path().join("SKILL.md");
        std::fs::write(&md_path, "# Test\n\nNo links here.").unwrap();
        std::fs::write(dir.path().join("EXTRA.md"), "Extra file content").unwrap();

        let skill = make_skill("test-skill", dir.path().to_path_buf(), md_path);
        let diags = check(&skill);
        assert!(diags.iter().any(|d| d.code == "skill-orphan-reference"));
    }

    #[test]
    fn short_skill_with_refs() {
        let dir = tempdir().unwrap();
        let md_path = dir.path().join("SKILL.md");
        std::fs::write(&md_path, "See [ref](ref.md) for details.").unwrap();

        let skill = make_skill("test-skill", dir.path().to_path_buf(), md_path);
        let diags = check(&skill);
        assert!(!diags.iter().any(|d| d.code == "skill-no-progressive-disclosure"));
    }

    #[test]
    fn orphan_reference_skill_md_ignored() {
        let dir = tempdir().unwrap();
        let md_path = dir.path().join("SKILL.md");
        std::fs::write(&md_path, "# Test\n\nNo links.").unwrap();
        std::fs::write(dir.path().join("LOOP.md"), "loop content").unwrap();

        let skill = make_skill("test-skill", dir.path().to_path_buf(), md_path);
        let diags = check(&skill);
        assert!(!diags.iter().any(|d| d.code == "skill-orphan-reference"));
    }
}
