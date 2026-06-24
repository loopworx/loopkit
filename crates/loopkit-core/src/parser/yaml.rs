use serde::de::DeserializeOwned;
use std::path::Path;

/// Parse a YAML file into any deserializable type.
pub fn parse_file<T: DeserializeOwned + 'static>(path: &Path) -> Result<T, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read {}: {}", path.display(), e))?;
    serde_yml::from_str(&content)
        .map_err(|e| format!("cannot parse {}: {}", path.display(), e))
}
