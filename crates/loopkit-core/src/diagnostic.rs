use crate::types::{Diagnostic, Severity};

pub fn format_diagnostics(diagnostics: &[Diagnostic]) -> String {
    let mut out = String::new();
    for d in diagnostics {
        let severity = match d.severity {
            Severity::Error => "Error",
            Severity::Warning => "Warning",
            Severity::Info => "Info",
        };

        let loc = if let Some(line) = d.location.line {
            let line_str = if let Some(col) = d.location.column {
                format!(":{}:{}", line, col)
            } else {
                format!(":{}", line)
            };
            format!("{}{}", d.location.path.display(), line_str)
        } else {
            d.location.path.display().to_string()
        };

        out.push_str(&format!(
            "{:<60} {:<8} {:<35} {}\n",
            loc, severity, d.code, d.message
        ));
    }
    out
}

pub fn diagnostics_json(diagnostics: &[Diagnostic], skills_checked: usize) -> String {
    use serde::Serialize;

    #[derive(Serialize)]
    struct Summary {
        errors: usize,
        warnings: usize,
        info: usize,
    }

    #[derive(Serialize)]
    struct Output<'a> {
        skills_checked: usize,
        diagnostics: &'a [Diagnostic],
        summary: Summary,
    }

    let errors = diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .count();
    let warnings = diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Warning)
        .count();
    let info = diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Info)
        .count();

    let output = Output {
        skills_checked,
        diagnostics,
        summary: Summary {
            errors,
            warnings,
            info,
        },
    };

    serde_json::to_string_pretty(&output).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e))
}

pub fn format_summary(diagnostics: &[Diagnostic], skills_count: usize) -> String {
    let errors = diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .count();
    let warnings = diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Warning)
        .count();
    format!(
        "\n{} skills checked. {} error(s), {} warning(s).",
        skills_count, errors, warnings
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Diagnostic, FileLocation, Severity};
    use std::path::PathBuf;

    fn make_diag(severity: Severity, code: &str, msg: &str, path: &str) -> Diagnostic {
        Diagnostic {
            severity,
            code: code.to_string(),
            message: msg.to_string(),
            location: FileLocation::new(PathBuf::from(path)),
            help: String::new(),
        }
    }

    fn make_diag_with_line(
        severity: Severity,
        code: &str,
        msg: &str,
        path: &str,
        line: u32,
    ) -> Diagnostic {
        Diagnostic {
            severity,
            code: code.to_string(),
            message: msg.to_string(),
            location: FileLocation::at(PathBuf::from(path), line, 0),
            help: String::new(),
        }
    }

    // ── format_diagnostics ──────────────────────────────────────────────

    #[test]
    fn test_format_diagnostics_empty() {
        let out = format_diagnostics(&[]);
        assert!(out.is_empty());
    }

    #[test]
    fn test_format_diagnostics_single_error() {
        let diag = make_diag(Severity::Error, "E001", "something broke", "foo.md");
        let out = format_diagnostics(&[diag]);
        assert!(out.contains("foo.md"));
        assert!(out.contains("Error"));
        assert!(out.contains("E001"));
        assert!(out.contains("something broke"));
    }

    #[test]
    fn test_format_diagnostics_single_warning() {
        let diag = make_diag(Severity::Warning, "W001", "deprecated", "bar.md");
        let out = format_diagnostics(&[diag]);
        assert!(out.contains("Warning"));
        assert!(out.contains("W001"));
    }

    #[test]
    fn test_format_diagnostics_single_info() {
        let diag = make_diag(Severity::Info, "I001", "note", "baz.md");
        let out = format_diagnostics(&[diag]);
        assert!(out.contains("Info"));
        assert!(out.contains("I001"));
    }

    #[test]
    fn test_format_diagnostics_with_line() {
        let diag = make_diag_with_line(Severity::Error, "E002", "line error", "foo.md", 42);
        let out = format_diagnostics(&[diag]);
        assert!(out.contains("foo.md:42:0"));
    }

    #[test]
    fn test_format_diagnostics_line_only() {
        let mut diag = make_diag(Severity::Error, "E003", "line only", "foo.md");
        diag.location.line = Some(10);
        diag.location.column = None;
        let out = format_diagnostics(&[diag]);
        assert!(out.contains("foo.md:10"));
        assert!(!out.contains("foo.md:10:"));
    }

    #[test]
    fn test_format_diagnostics_multiple_severities() {
        let diags = vec![
            make_diag(Severity::Error, "E001", "err", "a.md"),
            make_diag(Severity::Warning, "W001", "warn", "b.md"),
            make_diag(Severity::Info, "I001", "info", "c.md"),
        ];
        let out = format_diagnostics(&diags);
        assert!(out.contains("Error"));
        assert!(out.contains("Warning"));
        assert!(out.contains("Info"));
        // 3 diagnostics → 3 newlines
        assert_eq!(out.lines().count(), 3);
    }

    // ── diagnostics_json ────────────────────────────────────────────────

    #[test]
    fn test_diagnostics_json_empty() {
        let json = diagnostics_json(&[], 0);
        assert!(json.contains(r#""skills_checked": 0"#));
        assert!(json.contains(r#""diagnostics": []"#));
        assert!(json.contains(r#""errors": 0"#));
    }

    #[test]
    fn test_diagnostics_json_with_diagnostics() {
        let diags = vec![
            make_diag(Severity::Error, "E001", "err", "a.md"),
            make_diag(Severity::Warning, "W001", "warn", "b.md"),
            make_diag(Severity::Warning, "W002", "warn2", "c.md"),
            make_diag(Severity::Info, "I001", "info", "d.md"),
        ];
        let json = diagnostics_json(&diags, 5);
        // Parse it back to verify
        let v: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
        assert_eq!(v["skills_checked"], 5);
        assert_eq!(v["diagnostics"].as_array().unwrap().len(), 4);
        assert_eq!(v["summary"]["errors"], 1);
        assert_eq!(v["summary"]["warnings"], 2);
        assert_eq!(v["summary"]["info"], 1);
    }

    #[test]
    fn test_diagnostics_json_is_valid_json() {
        let diags = vec![make_diag(Severity::Error, "E001", "err", "a.md")];
        let json = diagnostics_json(&diags, 1);
        serde_json::from_str::<serde_json::Value>(&json).expect("should be valid JSON");
    }

    // ── format_summary ──────────────────────────────────────────────────

    #[test]
    fn test_format_summary_zero_diagnostics() {
        let s = format_summary(&[], 0);
        assert!(s.contains("0 skills checked"));
        assert!(s.contains("0 error(s)"));
        assert!(s.contains("0 warning(s)"));
    }

    #[test]
    fn test_format_summary_mixed() {
        let diags = vec![
            make_diag(Severity::Error, "E001", "err", "a.md"),
            make_diag(Severity::Error, "E002", "err", "a.md"),
            make_diag(Severity::Warning, "W001", "warn", "b.md"),
            make_diag(Severity::Info, "I001", "info", "c.md"),
        ];
        let s = format_summary(&diags, 3);
        assert!(s.contains("3 skills checked"));
        assert!(s.contains("2 error(s)"));
        assert!(s.contains("1 warning(s)"));
    }

    #[test]
    fn test_format_summary_only_infos() {
        let diags = vec![make_diag(Severity::Info, "I001", "info", "c.md")];
        let s = format_summary(&diags, 10);
        assert!(s.contains("10 skills checked"));
        assert!(s.contains("0 error(s)"));
        assert!(s.contains("0 warning(s)"));
    }
}
