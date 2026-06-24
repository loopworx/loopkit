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
