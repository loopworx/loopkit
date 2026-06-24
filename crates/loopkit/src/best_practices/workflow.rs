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

    // Only check skills with substantial bodies
    if line_count < 20 {
        return diags;
    }

    // Check for numbered steps or checklist items
    let has_checklist = content.contains("- [ ]")
        || content.contains("- [x]")
        || content.contains("- [X]")
        || {
            let numbered_re = Regex::new(r"^\d+\.").expect("hardcoded regex");
            content.lines().filter(|l| numbered_re.is_match(l.trim())).count() >= 2
        };

    // Check for multi-step procedure without checklist
    let steps_re = Regex::new(r"(?i)(\bfirst\b.*\bthen\b|\bstep\s+\d)").expect("hardcoded regex");
    if steps_re.is_match(&content) && !has_checklist {
        diags.push(Diagnostic::warning(
            "skill-missing-checklist",
            "Multi-step workflow detected but no checklist or numbered steps found. Consider adding a checklist".into(),
            path.clone(),
        ));
    }

    // Check for feedback loop pattern (validate → fix → repeat)
    let has_feedback_loop = {
        let has_validate = content.to_lowercase().contains("validate")
            || content.to_lowercase().contains("verify")
            || content.to_lowercase().contains("check");
        let has_fix = content.to_lowercase().contains("fix")
            || content.to_lowercase().contains("correct")
            || content.to_lowercase().contains("resolve");
        let has_repeat = content.to_lowercase().contains("repeat")
            || content.to_lowercase().contains("iterate")
            || content.to_lowercase().contains("loop")
            || content.to_lowercase().contains("again");
        has_validate && has_fix && has_repeat
    };

    if !has_feedback_loop {
        let has_test_like = content.to_lowercase().contains("test")
            || content.to_lowercase().contains("validate");
        if has_test_like && content.to_lowercase().contains("implement") {
            diags.push(Diagnostic::warning(
                "skill-missing-feedback-loop",
                "Workflow mentions validation and implementation but no explicit feedback loop pattern. Consider adding a validate → fix → repeat cycle".into(),
                path.clone(),
            ));
        }
    }

    // Check for branching logic without conditionals
    let has_conditional = content.contains("if ")
        || content.contains("when ")
        || content.contains("unless ")
        || content.contains("otherwise");
    let has_choice = content.contains("either")
        || content.contains("choose")
        || content.contains("decide")
        || content.contains("determine");
    if has_choice && !has_conditional {
        diags.push(Diagnostic::warning(
            "skill-missing-conditionals",
            "Decision points detected but no explicit conditional structure (if/when/otherwise). Consider adding conditional branches".into(),
            path.clone(),
        ));
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
    fn missing_checklist_reports_warning() {
        let dir = tempdir().unwrap();
        let md_path = dir.path().join("SKILL.md");
        let content = "\n".repeat(20) + "First do this, then do that step 2 comes after\n";
        std::fs::write(&md_path, &content).unwrap();

        let skill = make_skill("test-skill", dir.path().to_path_buf(), md_path);
        let diags = check(&skill);
        assert!(diags.iter().any(|d| d.code == "skill-missing-checklist"));
    }

    #[test]
    fn has_checklist_no_warning() {
        let dir = tempdir().unwrap();
        let md_path = dir.path().join("SKILL.md");
        let mut content = "\n".repeat(20);
        content.push_str("- [ ] Step one\n- [ ] Step two\n");
        std::fs::write(&md_path, &content).unwrap();

        let skill = make_skill("test-skill", dir.path().to_path_buf(), md_path);
        let diags = check(&skill);
        assert!(!diags.iter().any(|d| d.code == "skill-missing-checklist"));
    }

    #[test]
    fn short_skill_skipped() {
        let dir = tempdir().unwrap();
        let md_path = dir.path().join("SKILL.md");
        std::fs::write(&md_path, "first do this then that").unwrap();

        let skill = make_skill("test-skill", dir.path().to_path_buf(), md_path);
        let diags = check(&skill);
        assert!(diags.is_empty());
    }
}
