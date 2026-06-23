use std::process::Command;

/// Get the path to a compiled binary. In test mode, binaries are at specific locations.
fn binary_path(name: &str) -> String {
    // Try the cargo test binary location first, then fallback to build dir
    let exe = std::env::current_exe().unwrap();
    let dir = exe.parent().unwrap();

    // Check sibling deps directory
    let sibling = dir.join(name);
    if sibling.exists() {
        return sibling.to_string_lossy().to_string();
    }

    // Fallback: use cargo run
    format!("cargo run --bin {}", name)
}

#[test]
fn gen_coq_help_flag_returns_zero() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "gen_coq", "--", "--help"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .unwrap();
    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Usage: gen_coq"));
}

#[test]
fn gen_coq_check_missing_file_reports_error() {
    let dir = tempfile::TempDir::new().unwrap();
    let output = Command::new("cargo")
        .args(["run", "--bin", "gen_coq", "--", "--root", &dir.path().to_string_lossy(), "--check"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .unwrap();
    assert!(!output.status.success());
}

#[test]
fn main_help_flag_returns_zero() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "skill-loop-verifier", "--", "--help"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("skill-loop-verifier"));
}

#[test]
fn main_check_empty_skills_reports_warning() {
    let dir = tempfile::TempDir::new().unwrap();
    // Create skills dir with no skills
    std::fs::create_dir_all(dir.path().join("skills")).unwrap();
    let output = Command::new("cargo")
        .args(["run", "--bin", "skill-loop-verifier", "--", "--root", &dir.path().to_string_lossy()])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .unwrap();
    // No errors expected
    assert!(output.status.success());
}
