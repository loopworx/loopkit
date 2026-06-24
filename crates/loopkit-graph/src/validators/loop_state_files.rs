use loopkit_core::types::{Config, Diagnostic, FileLocation, Severity};
use std::path::Path;

/// Check that loop state tracking files exist in the docs/ directory.
pub fn validate(root: &Path, config: &Config) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    let expected_files = [
        ("docs/inception.loop.md", "inception phase tracking"),
        ("docs/iteration-board.loop.md", "iteration governance tracking"),
    ];

    for (path, desc) in &expected_files {
        let full_path = root.join(path);
        if !full_path.exists() {
            diags.push(Diagnostic {
                severity: Severity::Info,
                code: "loop-state-file-missing".to_string(),
                message: format!("Loop state file `{}` ({}) is missing", path, desc),
                location: FileLocation::new(full_path),
                help: format!(
                    "Create `{}` to track {} state across sessions.",
                    path, desc
                ),
            });
        }
    }

    // Suppress unused config variable (used for future extensibility)
    let _ = config;

    diags
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_docs_files_emits_info() {
        let config = Config::default();
        let root = std::path::PathBuf::from(tempfile::TempDir::new().unwrap().path());
        let diags = validate(&root, &config);
        assert!(!diags.is_empty());
        assert!(diags.iter().all(|d| d.code == "loop-state-file-missing"));
        assert!(diags.iter().all(|d| d.severity == Severity::Info));
    }

    #[test]
    fn with_docs_files_present_no_diagnostics() {
        let dir = tempfile::TempDir::new().unwrap();
        let docs = dir.path().join("docs");
        std::fs::create_dir(&docs).unwrap();
        std::fs::write(docs.join("inception.loop.md"), "").unwrap();
        std::fs::write(docs.join("iteration-board.loop.md"), "").unwrap();
        let config = Config::default();
        let diags = validate(dir.path(), &config);
        assert!(diags.is_empty(), "Expected no diagnostics but got: {:?}", diags);
    }
}
