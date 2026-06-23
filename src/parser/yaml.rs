use std::path::Path;

pub fn parse_file<T: for<'de> serde::Deserialize<'de> + 'static>(path: &Path) -> Result<T, String> {
    let content =
        std::fs::read_to_string(path).map_err(|e| format!("cannot read {}: {}", path.display(), e))?;
    serde_yml::from_str(&content).map_err(|e| format!("YAML parse error in {}: {}", path.display(), e))
}
