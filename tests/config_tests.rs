mod common;

use skill_loop_verifier::config::load_config;
use skill_loop_verifier::parser::yaml::parse_file;
use skill_loop_verifier::types::Config;
use std::path::PathBuf;

// ── Config ───────────────────────────────────────────────────────────

#[test]
fn given_no_config_file_when_loading_then_defaults_applied() {
    let dir = tempfile::TempDir::new().unwrap();
    let cfg = load_config(&dir.path().to_path_buf());
    assert_eq!(cfg.skills_dir, "skills/");
    assert_eq!(cfg.max_iterations, 20);
}

#[test]
fn given_valid_yaml_config_when_loading_then_values_parsed() {
    let dir = tempfile::TempDir::new().unwrap();
    std::fs::write(
        dir.path().join(".loop-verifier.yaml"),
        "skills_dir: my-skills/\nmax_iterations: 42\n",
    )
    .unwrap();
    let cfg = load_config(&dir.path().to_path_buf());
    assert_eq!(cfg.skills_dir, "my-skills/");
    assert_eq!(cfg.max_iterations, 42);
}

#[test]
fn given_invalid_yaml_config_when_loading_then_defaults_used() {
    let dir = tempfile::TempDir::new().unwrap();
    std::fs::write(dir.path().join(".loop-verifier.yaml"), "::: not valid :::").unwrap();
    let cfg = load_config(&dir.path().to_path_buf());
    assert_eq!(cfg.skills_dir, "skills/");
}

// ── YAML parser ──────────────────────────────────────────────────────

#[test]
fn given_valid_yaml_file_when_parsing_then_returns_config() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("c.yaml");
    std::fs::write(&path, "skills_dir: foo\nmax_iterations: 5\n").unwrap();
    let cfg: Config = parse_file(&path).unwrap();
    assert_eq!(cfg.skills_dir, "foo");
    assert_eq!(cfg.max_iterations, 5);
}

#[test]
fn given_invalid_yaml_syntax_when_parsing_then_returns_error() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("bad.yaml");
    std::fs::write(&path, "skills_dir: [unclosed").unwrap();
    let result: Result<Config, String> = parse_file(&path);
    assert!(result.is_err());
}

#[test]
fn given_missing_file_when_parsing_yaml_then_returns_error() {
    let result: Result<Config, String> = parse_file(&PathBuf::from("/nonexistent/config.yaml"));
    assert!(result.is_err());
}
