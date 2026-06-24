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
    if line_count > 500 {
        diags.push(Diagnostic::error(
            "skill-body-too-long",
            format!(
                "SKILL.md body exceeds 500 lines ({} lines). Consider extracting content into reference files",
                line_count
            ),
            path.clone(),
        ));
    }

    // Windows-style path detection
    if let Some(pos) = content.find('\\') {
        let line = content[..pos].lines().count() as u32 + 1;
        diags.push(
            Diagnostic::error(
                "skill-windows-path",
                "SKILL.md contains Windows-style paths (backslash). Use forward slashes".into(),
                path.clone(),
            )
            .at_line(line),
        );
    }

    // Time-sensitive language detection
    let time_re = Regex::new(
        r"(?i)(january|february|march|april|may|june|july|august|september|october|november|december)\s+\d{4}"
    ).expect("hardcoded regex");
    if let Some(m) = time_re.find(&content) {
        let line = content[..m.start()].lines().count() as u32 + 1;
        diags.push(Diagnostic::warning(
            "skill-time-sensitive",
            "SKILL.md contains date-specific language (e.g., 'March 2024'). Consider using relative framing".into(),
            path.clone(),
        ).at_line(line));
    }

    // Reference chain depth detection (A references B, B references C = depth 2+)
    let ref_re = Regex::new(r"\[([^\]]+)\]\(([^)]+\.md)\)").expect("hardcoded regex");
    let mut refs: Vec<String> = Vec::new();
    for cap in ref_re.captures_iter(&content) {
        refs.push(cap[2].to_string());
    }
    if !refs.is_empty() {
        let skill_dir = skill.path.clone();
        for ref_file in &refs {
            let ref_path = skill_dir.join(ref_file);
            if let Ok(ref_content) = std::fs::read_to_string(&ref_path) {
                if ref_re.is_match(&ref_content) {
                    diags.push(Diagnostic::warning(
                        "skill-deep-reference",
                        format!(
                            "Reference chain deeper than one level: {} references another reference file",
                            ref_file
                        ),
                        path.clone(),
                    ));
                }
                let ref_lines = ref_content.lines().count();
                if ref_lines > 100 {
                    let has_toc = ref_content.contains("## Table of Contents")
                        || ref_content.contains("## Contents")
                        || ref_content.contains("- [")
                            && ref_content.lines().filter(|l| l.starts_with("- [")).count() > 2;
                    if !has_toc {
                        diags.push(Diagnostic::warning(
                            "skill-ref-missing-toc",
                            format!(
                                "Reference file '{}' has {} lines but no table of contents",
                                ref_file, ref_lines
                            ),
                            ref_path.clone(),
                        ));
                    }
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
    fn long_body_reports_error() {
        let dir = tempdir().unwrap();
        let md_path = dir.path().join("SKILL.md");
        let body = "line\n".repeat(501);
        std::fs::write(&md_path, &body).unwrap();

        let skill = make_skill("test-skill", dir.path().to_path_buf(), md_path);
        let diags = check(&skill);
        assert!(diags.iter().any(|d| d.code == "skill-body-too-long"));
    }

    #[test]
    fn windows_path_reports_error() {
        let dir = tempdir().unwrap();
        let md_path = dir.path().join("SKILL.md");
        std::fs::write(&md_path, "See C:\\Users\\test\\file.md").unwrap();

        let skill = make_skill("test-skill", dir.path().to_path_buf(), md_path);
        let diags = check(&skill);
        assert!(diags.iter().any(|d| d.code == "skill-windows-path"));
    }

    #[test]
    fn short_body_no_error() {
        let dir = tempdir().unwrap();
        let md_path = dir.path().join("SKILL.md");
        std::fs::write(&md_path, "short body").unwrap();

        let skill = make_skill("test-skill", dir.path().to_path_buf(), md_path);
        let diags = check(&skill);
        assert!(diags.is_empty());
    }

    #[test]
    fn no_deep_reference() {
        let dir = tempdir().unwrap();
        let md_path = dir.path().join("SKILL.md");
        std::fs::write(&md_path, "See [ref](ref.md)").unwrap();
        std::fs::write(
            dir.path().join("ref.md"),
            "Just some content, no references here.",
        )
        .unwrap();

        let skill = make_skill("test-skill", dir.path().to_path_buf(), md_path);
        let diags = check(&skill);
        assert!(!diags.iter().any(|d| d.code == "skill-deep-reference"));
    }

    #[test]
    fn deep_reference() {
        let dir = tempdir().unwrap();
        let md_path = dir.path().join("SKILL.md");
        std::fs::write(&md_path, "See [ref](ref.md)").unwrap();
        std::fs::write(dir.path().join("ref.md"), "See [other](other.md) for more.").unwrap();

        let skill = make_skill("test-skill", dir.path().to_path_buf(), md_path);
        let diags = check(&skill);
        assert!(diags.iter().any(|d| d.code == "skill-deep-reference"));
    }

    #[test]
    fn ref_missing_toc() {
        let dir = tempdir().unwrap();
        let md_path = dir.path().join("SKILL.md");
        std::fs::write(&md_path, "See [bigref](bigref.md)").unwrap();
        let big_content = (0..101)
            .map(|i| format!("line {}\n", i))
            .collect::<String>();
        std::fs::write(dir.path().join("bigref.md"), big_content).unwrap();

        let skill = make_skill("test-skill", dir.path().to_path_buf(), md_path);
        let diags = check(&skill);
        assert!(diags.iter().any(|d| d.code == "skill-ref-missing-toc"));
    }

    #[test]
    fn ref_with_toc() {
        let dir = tempdir().unwrap();
        let md_path = dir.path().join("SKILL.md");
        std::fs::write(&md_path, "See [bigref](bigref.md)").unwrap();
        let mut big_content = String::new();
        for i in 0..100 {
            big_content.push_str(&format!("line {}\n", i));
        }
        big_content.push_str("## Table of Contents\n");
        std::fs::write(dir.path().join("bigref.md"), big_content).unwrap();

        let skill = make_skill("test-skill", dir.path().to_path_buf(), md_path);
        let diags = check(&skill);
        assert!(!diags.iter().any(|d| d.code == "skill-ref-missing-toc"));
    }

    #[test]
    fn time_sensitive_language() {
        let dir = tempdir().unwrap();
        let md_path = dir.path().join("SKILL.md");
        std::fs::write(&md_path, "Current as of March 2024.").unwrap();

        let skill = make_skill("test-skill", dir.path().to_path_buf(), md_path);
        let diags = check(&skill);
        assert!(diags.iter().any(|d| d.code == "skill-time-sensitive"));
    }

    #[test]
    fn no_time_sensitive() {
        let dir = tempdir().unwrap();
        let md_path = dir.path().join("SKILL.md");
        std::fs::write(&md_path, "Current as of the latest version.").unwrap();

        let skill = make_skill("test-skill", dir.path().to_path_buf(), md_path);
        let diags = check(&skill);
        assert!(!diags.iter().any(|d| d.code == "skill-time-sensitive"));
    }
}
