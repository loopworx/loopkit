use crate::types::{Diagnostic, Severity};
use std::sync::OnceLock;

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const BLUE: &str = "\x1b[34m";
const CYAN: &str = "\x1b[36m";
const WHITE: &str = "\x1b[37m";

static COLOR_ENABLED: OnceLock<bool> = OnceLock::new();

fn should_color() -> bool {
    *COLOR_ENABLED.get_or_init(|| std::env::var("NO_COLOR").is_err())
}

fn paint(text: &str, color: &str) -> String {
    if should_color() {
        format!("{color}{text}{RESET}")
    } else {
        text.to_string()
    }
}

fn paint_bold(text: &str, color: &str) -> String {
    if should_color() {
        format!("{BOLD}{color}{text}{RESET}")
    } else {
        text.to_string()
    }
}

pub fn format_header(version: &str, path: &std::path::Path) -> String {
    let mut out = String::new();
    let line = "─".repeat(60);
    out.push_str(&paint(&line, DIM));
    out.push('\n');
    out.push_str(&paint_bold("  loopkit", CYAN));
    out.push(' ');
    out.push_str(&paint(&format!("v{}", version), DIM));
    out.push('\n');
    out.push_str(&paint(&format!("  {}", path.display()), DIM));
    out.push('\n');
    out.push_str(&paint(&line, DIM));
    out.push('\n');
    out
}

pub fn format_diagnostics(diagnostics: &[Diagnostic]) -> String {
    let mut out = String::new();
    for d in diagnostics {
        let (severity_label, color) = match d.severity {
            Severity::Error => ("error", RED),
            Severity::Warning => ("warn", YELLOW),
            Severity::Info => ("info", BLUE),
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

        let badge = paint_bold(&format!(" {:<5} ", severity_label), color);

        let code = paint(&d.code, DIM);
        let loc_str = paint(&loc, DIM);

        out.push_str(&format!("{} {}\n", badge, loc_str));
        out.push_str(&format!("       {}\n", code));
        out.push_str(&format!("       {}\n", d.message));
        if !d.help.is_empty() {
            out.push_str(&format!(
                "       {}\n",
                paint(&format!("hint: {}", d.help), DIM)
            ));
        }
        out.push('\n');
    }
    out
}

pub fn diagnostics_json(
    diagnostics: &[Diagnostic],
    skills_checked: usize,
    verifications: usize,
) -> String {
    use serde::Serialize;

    #[derive(Serialize)]
    struct Summary {
        errors: usize,
        warnings: usize,
        info: usize,
        verifications: usize,
    }

    #[derive(Serialize)]
    struct Output<'a> {
        skills_checked: usize,
        verifications: usize,
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
        verifications,
        diagnostics,
        summary: Summary {
            errors,
            warnings,
            info,
            verifications,
        },
    };

    serde_json::to_string_pretty(&output).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e))
}

pub fn format_summary(
    diagnostics: &[Diagnostic],
    skills_count: usize,
    verifications: usize,
) -> String {
    let errors = diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .count();
    let warnings = diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Warning)
        .count();
    let infos = diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Info)
        .count();

    let status = if errors > 0 {
        paint_bold("FAIL", RED)
    } else if warnings > 0 {
        paint_bold("PASS", YELLOW)
    } else {
        paint_bold("PASS", GREEN)
    };

    let line = "─".repeat(60);

    let mut out = String::new();
    out.push_str(&paint(&line, DIM));
    out.push('\n');
    out.push_str(&format!("  {}  ", status));
    out.push_str(&format!(
        "{} skills  {} verifications",
        paint_bold(&skills_count.to_string(), WHITE),
        paint_bold(&verifications.to_string(), WHITE),
    ));
    out.push('\n');
    out.push_str(&format!(
        "  {}  {}  {}\n",
        paint(
            &format!("{} errors", errors),
            if errors > 0 { RED } else { DIM }
        ),
        paint(
            &format!("{} warnings", warnings),
            if warnings > 0 { YELLOW } else { DIM }
        ),
        paint(
            &format!("{} info", infos),
            if infos > 0 { BLUE } else { DIM }
        ),
    ));
    out.push_str(&paint(&line, DIM));
    out
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
        assert!(out.contains("error"));
        assert!(out.contains("E001"));
        assert!(out.contains("something broke"));
        assert!(out.contains("foo.md"));
    }

    #[test]
    fn test_format_diagnostics_single_warning() {
        let diag = make_diag(Severity::Warning, "W001", "deprecated", "bar.md");
        let out = format_diagnostics(&[diag]);
        assert!(out.contains("warn"));
        assert!(out.contains("W001"));
    }

    #[test]
    fn test_format_diagnostics_single_info() {
        let diag = make_diag(Severity::Info, "I001", "note", "baz.md");
        let out = format_diagnostics(&[diag]);
        assert!(out.contains("info"));
        assert!(out.contains("I001"));
    }

    #[test]
    fn test_format_diagnostics_with_line() {
        let diag = make_diag_with_line(Severity::Error, "E002", "line error", "foo.md", 42);
        let out = format_diagnostics(&[diag]);
        assert!(out.contains("foo.md:42"));
    }

    #[test]
    fn test_format_diagnostics_help_shown() {
        let mut diag = make_diag(Severity::Error, "E001", "err", "a.md");
        diag.help = "try this fix".to_string();
        let out = format_diagnostics(&[diag]);
        assert!(out.contains("hint: try this fix"));
    }

    // ── diagnostics_json ────────────────────────────────────────────────

    #[test]
    fn test_diagnostics_json_empty() {
        let json = diagnostics_json(&[], 0, 0);
        assert!(json.contains(r#""skills_checked": 0"#));
        assert!(json.contains(r#""verifications": 0"#));
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
        let json = diagnostics_json(&diags, 5, 42);
        let v: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
        assert_eq!(v["skills_checked"], 5);
        assert_eq!(v["verifications"], 42);
        assert_eq!(v["diagnostics"].as_array().unwrap().len(), 4);
        assert_eq!(v["summary"]["errors"], 1);
        assert_eq!(v["summary"]["warnings"], 2);
        assert_eq!(v["summary"]["info"], 1);
        assert_eq!(v["summary"]["verifications"], 42);
    }

    #[test]
    fn test_diagnostics_json_is_valid_json() {
        let diags = vec![make_diag(Severity::Error, "E001", "err", "a.md")];
        let json = diagnostics_json(&diags, 1, 10);
        serde_json::from_str::<serde_json::Value>(&json).expect("should be valid JSON");
    }

    // ── format_summary ──────────────────────────────────────────────────

    #[test]
    fn test_format_summary_zero_diagnostics() {
        let s = format_summary(&[], 0, 0);
        assert!(s.contains("0 skills"));
        assert!(s.contains("0 verifications"));
        assert!(s.contains("0 errors"));
        assert!(s.contains("0 warnings"));
        assert!(s.contains("PASS"));
    }

    #[test]
    fn test_format_summary_with_errors_shows_fail() {
        let diags = vec![
            make_diag(Severity::Error, "E001", "err", "a.md"),
            make_diag(Severity::Error, "E002", "err", "a.md"),
            make_diag(Severity::Warning, "W001", "warn", "b.md"),
            make_diag(Severity::Info, "I001", "info", "c.md"),
        ];
        let s = format_summary(&diags, 3, 180);
        assert!(s.contains("FAIL"));
        assert!(s.contains("3 skills"));
        assert!(s.contains("180 verifications"));
        assert!(s.contains("2 errors"));
        assert!(s.contains("1 warnings"));
    }

    #[test]
    fn test_format_summary_only_warnings_shows_pass() {
        let diags = vec![make_diag(Severity::Warning, "W001", "warn", "b.md")];
        let s = format_summary(&diags, 10, 90);
        assert!(s.contains("PASS"));
        assert!(s.contains("1 warnings"));
    }

    #[test]
    fn test_format_summary_only_infos_shows_pass() {
        let diags = vec![make_diag(Severity::Info, "I001", "info", "c.md")];
        let s = format_summary(&diags, 10, 90);
        assert!(s.contains("PASS"));
        assert!(s.contains("1 info"));
    }

    #[test]
    fn test_format_header_contains_version_and_path() {
        let s = format_header("1.2.3", std::path::Path::new("/test/path"));
        assert!(s.contains("1.2.3"));
        assert!(s.contains("/test/path"));
        assert!(s.contains("loopkit"));
    }

    #[test]
    fn test_format_diagnostics_with_column() {
        let mut diag = make_diag(Severity::Error, "E001", "err", "foo.md");
        diag.location = FileLocation::at(PathBuf::from("foo.md"), 10, 5);
        let out = format_diagnostics(&[diag]);
        assert!(out.contains("foo.md:10:5"));
    }

    #[test]
    fn test_format_diagnostics_multiple_severities() {
        let diags = vec![
            make_diag(Severity::Error, "E001", "err", "a.md"),
            make_diag(Severity::Warning, "W001", "warn", "b.md"),
            make_diag(Severity::Info, "I001", "info", "c.md"),
        ];
        let out = format_diagnostics(&diags);
        assert!(out.contains("error"));
        assert!(out.contains("warn"));
        assert!(out.contains("info"));
    }

    #[test]
    fn test_format_diagnostics_no_path_line_only() {
        let mut diag = make_diag(Severity::Error, "E001", "err", "a.md");
        diag.location.line = Some(10);
        diag.location.column = None;
        let out = format_diagnostics(&[diag]);
        assert!(out.contains("a.md:10"));
    }
}
