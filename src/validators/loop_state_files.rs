//! Loop state files validator: checks that operational state files
//! exist if they are referenced by the project.

use crate::types::{Diagnostic, FileLocation, Repo, Severity};

/// Check that loop state tracking files exist in the docs/ directory.
/// These are optional but recommended for projects using the loop framework.
pub fn validate_loop_state_files(repo: &Repo) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    let expected_files = [
        ("docs/inception.loop.md", "inception phase tracking"),
        ("docs/iteration-board.loop.md", "iteration governance tracking"),
    ];

    for (path, desc) in &expected_files {
        let full_path = repo.root.join(path);
        if !full_path.exists() {
            diags.push(Diagnostic {
                severity: Severity::Info,
                code: "loop-state-file-missing".to_string(),
                message: format!(
                    "Loop state file `{}` ({}) is missing",
                    path, desc
                ),
                location: FileLocation {
                    path: full_path,
                    line: None,
                    column: None,
                },
                help: format!(
                    "Create `{}` to track {} state across sessions.",
                    path, desc
                ),
            });
        }
    }

    diags
}
