use loopkit_core::types::{Diagnostic, Skill};
use regex::Regex;

pub fn check(skill: &Skill) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let path = skill.skill_md.clone();

    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return diags,
    };

    // Too many equivalent options
    let option_re = Regex::new(r"(?i)(\d+)\s*(?:options?|ways?|approaches?|methods?)\s*(?:to|for)")
        .expect("hardcoded regex");
    for cap in option_re.captures_iter(&content) {
        if let Ok(n) = cap[1].parse::<u32>() {
            if n > 3 {
                let line = content[..cap.get(0).unwrap().start()].lines().count() as u32 + 1;
                diags.push(Diagnostic::warning(
                    "skill-too-many-options",
                    format!(
                        "{} equivalent options offered for the same task. Consider picking one recommended approach",
                        n
                    ),
                    path.clone(),
                ).at_line(line));
            }
        }
    }

    // Magic numbers in code blocks
    let code_block_re = Regex::new(r"```[\s\S]*?```").expect("hardcoded regex");
    let magic_re = Regex::new(r"\b\d{2,}\b").expect("hardcoded regex");
    let documented_re = Regex::new(r"(?i)(max|min|limit|timeout|retry|threshold|default)")
        .expect("hardcoded regex");

    for m in code_block_re.find_iter(&content) {
        let block = m.as_str();
        let block_start_line = content[..m.start()].lines().count();
        for magic in magic_re.find_iter(block) {
            let num = magic.as_str();
            // Check surrounding context for documentation
            let block_content = block.to_string();
            let idx = magic.start();
            let context_start = idx.saturating_sub(50);
            let context_end = (idx + num.len() + 50).min(block_content.len());
            let context = &block_content[context_start..context_end];

            if !documented_re.is_match(context) {
                let rel_line = block[..idx].lines().count();
                let line_num = (block_start_line + rel_line + 1) as u32;
                diags.push(
                    Diagnostic::warning(
                        "skill-magic-numbers",
                        format!(
                        "Magic number '{}' found in code block at line ~{} without documentation",
                        num,
                        line_num
                    ),
                        path.clone(),
                    )
                    .at_line(line_num),
                );
            }
        }
    }

    // Bare raise / punting to Claude
    let bare_raise_re = Regex::new(r"(?m)^(?:try\s*:|except\s+\w*\s*:\s*\n\s*(?:raise|pass))")
        .expect("hardcoded regex");
    if let Some(m) = bare_raise_re.find(&content) {
        let line = content[..m.start()].lines().count() as u32 + 1;
        diags.push(Diagnostic::warning(
            "skill-punts-to-claude",
            "Bare except clause with raise/pass detected. Consider providing error handling guidance".into(),
            path.clone(),
        ).at_line(line));
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
    fn too_many_options_reports_warning() {
        let dir = tempdir().unwrap();
        let md_path = dir.path().join("SKILL.md");
        std::fs::write(&md_path, "There are 5 ways to do this task").unwrap();

        let skill = make_skill("test-skill", dir.path().to_path_buf(), md_path);
        let diags = check(&skill);
        assert!(diags.iter().any(|d| d.code == "skill-too-many-options"));
    }

    #[test]
    fn magic_number_in_code_reports_warning() {
        let dir = tempdir().unwrap();
        let md_path = dir.path().join("SKILL.md");
        std::fs::write(&md_path, "```\nlet x = 42;\n```").unwrap();

        let skill = make_skill("test-skill", dir.path().to_path_buf(), md_path);
        let diags = check(&skill);
        assert!(diags.iter().any(|d| d.code == "skill-magic-numbers"));
    }

    #[test]
    fn documented_magic_number_no_warning() {
        let dir = tempdir().unwrap();
        let md_path = dir.path().join("SKILL.md");
        std::fs::write(&md_path, "```\nlet max_retries = 42; // documented\n```").unwrap();

        let skill = make_skill("test-skill", dir.path().to_path_buf(), md_path);
        let diags = check(&skill);
        assert!(!diags.iter().any(|d| d.code == "skill-magic-numbers"));
    }
}
