use serde::de::DeserializeOwned;
use std::path::Path;

/// Parse a YAML file into any deserializable type.
pub fn parse_file<T: DeserializeOwned + 'static>(path: &Path) -> Result<T, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read {}: {}", path.display(), e))?;
    serde_yml::from_str(&content).map_err(|e| format!("cannot parse {}: {}", path.display(), e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq)]
    struct SimpleConfig {
        name: String,
        version: u32,
    }

    #[test]
    fn test_parse_file_valid_yaml() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.yaml");
        std::fs::write(&path, "name: test\nversion: 1\n").unwrap();

        let result: Result<SimpleConfig, _> = parse_file(&path);
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.name, "test");
        assert_eq!(config.version, 1);
    }

    #[test]
    fn test_parse_file_invalid_yaml() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.yaml");
        std::fs::write(&path, "invalid: [broken: yaml\n").unwrap();

        let result: Result<SimpleConfig, _> = parse_file(&path);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("cannot parse"));
    }

    #[test]
    fn test_parse_file_missing_file() {
        let result: Result<SimpleConfig, _> =
            parse_file(Path::new("/tmp/__nonexistent_yaml_test__.yaml"));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("cannot read"));
    }
}
