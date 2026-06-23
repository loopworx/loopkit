use crate::types::{Diagnostic, Severity};

/// Format diagnostics as human-readable text.
pub fn format_diagnostics(diagnostics: &[Diagnostic]) -> String {
    if diagnostics.is_empty() {
        return "All contract checks passed ✅".to_string();
    }

    let mut out = String::new();
    for d in diagnostics {
        let prefix = match d.severity {
            Severity::Error => "ERROR",
            Severity::Warning => "ERROR",
            Severity::Info => "INFO",
        };
        out.push_str(&format!(
            "{} [{}] {}: {}\n    → {}\n",
            prefix,
            d.code,
            d.location.path.display(),
            d.message,
            d.help,
        ));
    }

    let errors = diagnostics.iter().filter(|d| d.severity == Severity::Error || d.severity == Severity::Warning).count();
    let infos = diagnostics.iter().filter(|d| d.severity == Severity::Info).count();
    out.push_str(&format!(
        "\n{} error(s), {} info(s), {} total\n",
        errors,
        infos,
        diagnostics.len()
    ));

    out
}

/// Format diagnostics as JSON.
pub fn diagnostics_json(diagnostics: &[Diagnostic]) -> String {
    serde_json::to_string_pretty(diagnostics).unwrap_or_default()
}
