use loopkit_core::types::{Diagnostic, Skill};

/// Check for script-related best practices:
/// - Skills with complex multi-step workflows should bundle scripts
/// - Detect inline dependency declarations in script files
pub fn check(skill: &Skill) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let path = skill.skill_md.clone();

    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return diags,
    };

    let scripts_dir = skill.path.join("scripts");
    let has_scripts_dir = scripts_dir.exists() && scripts_dir.is_dir();

    // Count shell/python commands in SKILL.md code blocks
    let has_commands = content.contains("```bash")
        || content.contains("```sh")
        || content.contains("```python")
        || content.contains("```py")
        || content.contains("pipx run")
        || content.contains("uvx ")
        || content.contains("uv run")
        || content.contains("npx ")
        || content.contains("bunx ")
        || content.contains("deno run");

    // If SKILL.md references commands but no scripts/ directory exists, warn
    if has_commands && !has_scripts_dir {
        let line = content.lines()
            .enumerate()
            .find(|(_, l)| {
                let trimmed = l.trim();
                trimmed.starts_with("```bash")
                    || trimmed.starts_with("```sh")
                    || trimmed.starts_with("```python")
                    || trimmed.starts_with("```py")
                    || trimmed.contains("pipx run")
                    || trimmed.contains("uvx ")
                    || trimmed.contains("uv run")
                    || trimmed.contains("npx ")
                    || trimmed.contains("bunx ")
                    || trimmed.contains("deno run")
            })
            .map(|(i, _)| (i + 1) as u32);

        let mut diag = Diagnostic::info(
            "skill-scripts-suggested",
            "SKILL.md references executable commands but no scripts/ directory exists. \
             Consider bundling reusable, tested scripts for reliability".into(),
            path.clone(),
        );
        if let Some(l) = line {
            diag = diag.at_line(l);
        }
        diags.push(diag);
    }

    // If scripts/ exists, check for inline dependency declarations
    if has_scripts_dir {
        if let Ok(entries) = std::fs::read_dir(&scripts_dir) {
            for entry in entries.flatten() {
                let script_path = entry.path();
                if let Ok(body) = std::fs::read_to_string(&script_path) {
                    let has_inline_deps = body.contains("# /// script")
                        || body.contains("# dependencies =")
                        || body.contains("npm:")
                        || body.contains("jsr:")
                        || body.contains("require 'bundler/inline'");

                    if !has_inline_deps {
                        let script_name = script_path
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default();
                        diags.push(Diagnostic::info(
                            "skill-script-no-inline-deps",
                            format!(
                                "Script '{}' has no inline dependency declarations. \
                                 Consider using PEP 723 (# /// script), npm: specifiers, \
                                 or bundler/inline for self-contained scripts",
                                script_name
                            ),
                            script_path.clone(),
                        ));
                    } else {
                        // Check for --help flag support
                        let has_help = body.contains("--help")
                            || body.contains("-h")
                            || body.contains("usage:")
                            || body.contains("Usage:");
                        if !has_help {
                            let script_name = script_path
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_default();
                            diags.push(Diagnostic::info(
                                "skill-script-no-help",
                                format!(
                                    "Script '{}' has no visible --help or usage documentation. \
                                     Agents learn your script interface from --help output",
                                    script_name
                                ),
                                script_path.clone(),
                            ));
                        }
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
    fn commands_without_scripts_dir_suggests_bundling() {
        let dir = tempdir().unwrap();
        let md_path = dir.path().join("SKILL.md");
        std::fs::write(&md_path, "```bash\nnpm test\n```").unwrap();

        let skill = make_skill("test-skill", dir.path().to_path_buf(), md_path);
        let diags = check(&skill);
        assert!(diags.iter().any(|d| d.code == "skill-scripts-suggested"));
    }

    #[test]
    fn commands_with_scripts_dir_no_suggestion() {
        let dir = tempdir().unwrap();
        let md_path = dir.path().join("SKILL.md");
        std::fs::write(&md_path, "```bash\nnpm test\n```").unwrap();
        std::fs::create_dir(dir.path().join("scripts")).unwrap();

        let skill = make_skill("test-skill", dir.path().to_path_buf(), md_path);
        let diags = check(&skill);
        // "scripts" dir exists → no "scripts-suggested" diagnostic
        assert!(!diags.iter().any(|d| d.code == "skill-scripts-suggested"));
    }

    #[test]
    fn no_commands_no_diagnostic() {
        let dir = tempdir().unwrap();
        let md_path = dir.path().join("SKILL.md");
        std::fs::write(&md_path, "Just text, no commands.").unwrap();

        let skill = make_skill("test-skill", dir.path().to_path_buf(), md_path);
        let diags = check(&skill);
        assert!(diags.is_empty());
    }

    #[test]
    fn script_without_inline_deps_reports_info() {
        let dir = tempdir().unwrap();
        let md_path = dir.path().join("SKILL.md");
        std::fs::write(&md_path, "See scripts/validate.py").unwrap();
        let scripts = dir.path().join("scripts");
        std::fs::create_dir(&scripts).unwrap();
        std::fs::write(scripts.join("validate.py"), "print('ok')").unwrap();

        let skill = make_skill("test-skill", dir.path().to_path_buf(), md_path);
        let diags = check(&skill);
        assert!(diags.iter().any(|d| d.code == "skill-script-no-inline-deps"));
    }

    #[test]
    fn script_with_inline_deps_no_deps_warning() {
        let dir = tempdir().unwrap();
        let md_path = dir.path().join("SKILL.md");
        std::fs::write(&md_path, "See scripts/validate.py").unwrap();
        let scripts = dir.path().join("scripts");
        std::fs::create_dir(&scripts).unwrap();
        std::fs::write(
            scripts.join("validate.py"),
            "# /// script\n# dependencies = [\"requests\"]\n# ///\nimport requests\nprint('ok')",
        )
        .unwrap();

        let skill = make_skill("test-skill", dir.path().to_path_buf(), md_path);
        let diags = check(&skill);
        assert!(!diags.iter().any(|d| d.code == "skill-script-no-inline-deps"));
        // Still warns about --help
        assert!(diags.iter().any(|d| d.code == "skill-script-no-help"));
    }

    #[test]
    fn script_with_help_no_warnings() {
        let dir = tempdir().unwrap();
        let md_path = dir.path().join("SKILL.md");
        std::fs::write(&md_path, "See scripts/validate.py").unwrap();
        let scripts = dir.path().join("scripts");
        std::fs::create_dir(&scripts).unwrap();
        std::fs::write(
            scripts.join("validate.py"),
            "# /// script\n# dependencies = [\"requests\"]\n# ///\nimport sys\nif '--help' in sys.argv:\n    print('usage: validate.py <file>')\n",
        )
        .unwrap();

        let skill = make_skill("test-skill", dir.path().to_path_buf(), md_path);
        let diags = check(&skill);
        assert!(diags.is_empty());
    }
}
