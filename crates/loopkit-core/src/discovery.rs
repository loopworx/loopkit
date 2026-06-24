use std::path::Path;

use walkdir::WalkDir;

use crate::parser::skill::parse_skill_dir;
use crate::types::{Diagnostic, Skill};

/// Discovers all skills under the given directory.
/// Returns skills that parse successfully, plus any diagnostics from failed parses.
pub fn discover_skills(skills_dir: &Path) -> (Vec<Skill>, Vec<Diagnostic>) {
    if !skills_dir.exists() {
        return (Vec::new(), Vec::new());
    }

    let mut skills = Vec::new();
    let mut diagnostics = Vec::new();

    for entry in WalkDir::new(skills_dir).min_depth(2).max_depth(2) {
        if let Ok(entry) = entry {
            if entry.file_type().is_dir() {
                match parse_skill_dir(entry.path()) {
                    Ok(Some(skill)) => skills.push(skill),
                    Ok(None) => {}
                    Err(diags) => diagnostics.extend(diags),
                }
            }
        }
    }

    (skills, diagnostics)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write_skill_md(dir: &Path, frontmatter: &str, body: &str) {
        let content = format!("---\n{}\n---\n\n{}", frontmatter, body);
        fs::write(dir.join("SKILL.md"), content).unwrap();
    }

    #[test]
    fn test_discover_skills_empty_when_dir_does_not_exist() {
        let (skills, diags) =
            discover_skills(Path::new("/tmp/__nonexistent_loopkit_skills_test__"));
        assert!(skills.is_empty());
        assert!(diags.is_empty());
    }

    #[test]
    fn test_discover_skills_discovers_skills_in_temp_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let skills_dir = tmp.path();

        let category_dir = skills_dir.join("general");
        let skill_dir = category_dir.join("my-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        write_skill_md(
            &skill_dir,
            "name: my-skill\ndescription: A test skill\nlevel: beginner",
            "# My Skill\n\n## Description\n\nThis is a test.",
        );

        let (skills, diags) = discover_skills(skills_dir);
        assert_eq!(skills.len(), 1);
        assert!(diags.is_empty());
        assert_eq!(skills[0].name, "my-skill");
        assert_eq!(skills[0].category, "general");
        assert_eq!(skills[0].description, "A test skill");
        assert_eq!(skills[0].level, "beginner");
    }

    #[test]
    fn test_discover_skills_multiple_skills() {
        let tmp = tempfile::tempdir().unwrap();
        let skills_dir = tmp.path();

        let skill_a = skills_dir.join("cat-a").join("skill-a");
        let skill_b = skills_dir.join("cat-b").join("skill-b");
        fs::create_dir_all(&skill_a).unwrap();
        fs::create_dir_all(&skill_b).unwrap();
        write_skill_md(&skill_a, "name: skill-a\ndescription: First", "# Skill A");
        write_skill_md(&skill_b, "name: skill-b\ndescription: Second", "# Skill B");

        let (skills, diags) = discover_skills(skills_dir);
        assert_eq!(skills.len(), 2);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_discover_skills_diagnostics_for_parse_errors() {
        let tmp = tempfile::tempdir().unwrap();
        let skills_dir = tmp.path();

        let skill_dir = skills_dir.join("cat").join("bad-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        // Missing 'name' in frontmatter → parse error
        write_skill_md(
            &skill_dir,
            "description: No name here",
            "# Bad Skill",
        );

        let (skills, diags) = discover_skills(skills_dir);
        assert!(skills.is_empty());
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, "skill-missing-name");
    }

    #[test]
    fn test_discover_skills_skips_dirs_without_skill_md() {
        let tmp = tempfile::tempdir().unwrap();
        let skills_dir = tmp.path();

        let empty_dir = skills_dir.join("cat").join("empty");
        fs::create_dir_all(&empty_dir).unwrap();
        // No SKILL.md → parse_skill_dir returns Ok(None), skipped silently

        let (skills, diags) = discover_skills(skills_dir);
        assert!(skills.is_empty());
        assert!(diags.is_empty());
    }
}
