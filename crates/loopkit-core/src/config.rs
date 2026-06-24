use crate::types::Config;
use std::path::Path;

/// Load configuration from a `.loopkit.yaml` file.
/// Looks in the given root directory then falls back to defaults.
pub fn load_config(root: &Path) -> Config {
    let config_path = root.join(".loopkit.yaml");
    if config_path.exists() {
        match std::fs::read_to_string(&config_path) {
            Ok(content) => match serde_yml::from_str::<Config>(&content) {
                Ok(config) => return config,
                Err(e) => {
                    eprintln!(
                        "warning: failed to parse {}: {}. Using defaults.",
                        config_path.display(), e
                    );
                }
            },
            Err(e) => {
                eprintln!(
                    "warning: failed to read {}: {}. Using defaults.",
                    config_path.display(), e
                );
            }
        }
    }
    Config::default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_load_config_default_when_no_file() {
        let dir = tempfile::tempdir().unwrap();
        let config = load_config(dir.path());
        assert_eq!(config.skills_dir, "skills/");
        assert_eq!(config.max_iterations, 20);
    }

    #[test]
    fn test_load_config_valid_yaml() {
        let dir = tempfile::tempdir().unwrap();
        let yaml = "skills_dir: custom-skills/\nmax_iterations: 42\n";
        fs::write(dir.path().join(".loopkit.yaml"), yaml).unwrap();

        let config = load_config(dir.path());
        assert_eq!(config.skills_dir, "custom-skills/");
        assert_eq!(config.max_iterations, 42);
    }

    #[test]
    fn test_load_config_invalid_yaml_falls_back_to_defaults() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(".loopkit.yaml"), "invalid: [broken: yaml").unwrap();

        let config = load_config(dir.path());
        // Should fall back to defaults
        assert_eq!(config.skills_dir, "skills/");
        assert_eq!(config.max_iterations, 20);
    }

    #[test]
    fn test_load_config_missing_file_falls_back() {
        // Point to a non-existent directory
        let config = load_config(Path::new("/tmp/__nonexistent_loopkit_test_dir__"));
        assert_eq!(config.skills_dir, "skills/");
        assert_eq!(config.max_iterations, 20);
    }
}
