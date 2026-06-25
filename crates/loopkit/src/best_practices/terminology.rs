use loopkit_core::types::{Diagnostic, Skill};
use regex::Regex;

pub fn check(skill: &Skill) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let path = skill.skill_md.clone();

    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return diags,
    };

    let lower = content.to_lowercase();

    let synonym_groups: &[&[&str]] = &[
        &["endpoint", "route", "url"],
        &["field", "element", "control"],
        &["extract", "pull", "retrieve", "get"],
        &["write", "create", "generate", "produce"],
        &["check", "validate", "verify", "confirm"],
    ];

    for group in synonym_groups {
        let mut found: Vec<&str> = Vec::new();
        for word in *group {
            let re = Regex::new(&format!(r"\b{}\b", word)).expect("hardcoded regex");
            if re.is_match(&lower) {
                found.push(word);
            }
        }
        if found.len() >= 2 {
            // Find the line where the second conflicting word appears
            let mut conflict_line: Option<u32> = None;
            let first = found[0];
            for (i, line) in content.lines().enumerate() {
                let line_lower = line.to_lowercase();
                for w in found.iter().skip(1) {
                    let re = Regex::new(&format!(r"\b{}\b", w)).expect("hardcoded regex");
                    if re.is_match(&line_lower) && first != *w {
                        conflict_line = Some((i + 1) as u32);
                        break;
                    }
                }
                if conflict_line.is_some() {
                    break;
                }
            }

            let mut diag = Diagnostic::warning(
                "skill-term-inconsistency",
                format!(
                    "Multiple synonyms for the same concept found in SKILL.md: {}. \
                     Pick one term and use it consistently",
                    found.join(", ")
                ),
                path.clone(),
            );
            if let Some(line) = conflict_line {
                diag = diag.at_line(line);
            }
            diags.push(diag);
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
    fn synonym_mismatch_reports_warning() {
        let dir = tempdir().unwrap();
        let md_path = dir.path().join("SKILL.md");
        std::fs::write(&md_path, "Use the endpoint URL to retrieve data").unwrap();

        let skill = make_skill("test-skill", dir.path().to_path_buf(), md_path);
        let diags = check(&skill);
        assert!(diags.iter().any(|d| d.code == "skill-term-inconsistency"));
    }

    #[test]
    fn consistent_terms_no_warning() {
        let dir = tempdir().unwrap();
        let md_path = dir.path().join("SKILL.md");
        std::fs::write(&md_path, "Use the endpoint to fetch data").unwrap();

        let skill = make_skill("test-skill", dir.path().to_path_buf(), md_path);
        let diags = check(&skill);
        // "endpoint" is in group 1, "fetch" is not in any group — no conflicts
        assert!(diags.is_empty());
    }

    #[test]
    fn substring_does_not_trigger_false_positive() {
        let dir = tempdir().unwrap();
        let md_path = dir.path().join("SKILL.md");
        // "targets" contains "get" as substring but should not trigger
        std::fs::write(&md_path, "Set transition targets for the state machine").unwrap();

        let skill = make_skill("test-skill", dir.path().to_path_buf(), md_path);
        let diags = check(&skill);
        assert!(diags.is_empty());
    }
}
