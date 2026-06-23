use crate::types::Config;
use std::path::PathBuf;

/// Load configuration from a `.loop-verifier.yaml` file.
/// Looks in the given root directory, then walks up.
pub fn load_config(root: &PathBuf) -> Config {
    let config_path = root.join(".loop-verifier.yaml");
    if config_path.exists() {
        match std::fs::read_to_string(&config_path) {
            Ok(content) => match serde_yml::from_str::<Config>(&content) {
                Ok(config) => return config,
                Err(e) => {
                    eprintln!(
                        "warning: failed to parse {}: {}. Using defaults.",
                        config_path.display(),
                        e
                    );
                }
            },
            Err(e) => {
                eprintln!(
                    "warning: failed to read {}: {}. Using defaults.",
                    config_path.display(),
                    e
                );
            }
        }
    }
    Config::default()
}
