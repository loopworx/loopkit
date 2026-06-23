mod common;

use skill_loop_verifier::diagnostic::{diagnostics_json, format_diagnostics};
use skill_loop_verifier::types::{Diagnostic, FileLocation, Severity};
use std::path::PathBuf;

// ── Formatting ───────────────────────────────────────────────────────

#[test]
fn given_no_diagnostics_when_formatting_then_returns_clean_message() {
    assert_eq!(format_diagnostics(&[]), "All contract checks passed ✅");
}

#[test]
fn given_error_diagnostic_when_formatting_then_shows_error_code_and_help() {
    let d = Diagnostic {
        severity: Severity::Error,
        code: "E001".to_string(),
        message: "bad thing".to_string(),
        location: FileLocation {
            path: PathBuf::from("x.md"),
            line: Some(1),
            column: None,
        },
        help: "fix it".to_string(),
    };
    let s = format_diagnostics(&[d]);
    assert!(s.contains("ERROR"));
    assert!(s.contains("E001"));
    assert!(s.contains("bad thing"));
    assert!(s.contains("fix it"));
    assert!(s.contains("1 error(s)"));
}

#[test]
fn given_multiple_error_diagnostics_when_formatting_then_counts_correctly() {
    let d1 = Diagnostic {
        severity: Severity::Error,
        code: "W001".to_string(),
        message: "careful".to_string(),
        location: FileLocation {
            path: PathBuf::from("y.md"),
            line: None,
            column: None,
        },
        help: "note".to_string(),
    };
    let d2 = Diagnostic {
        severity: Severity::Error,
        code: "W002".to_string(),
        message: "also careful".to_string(),
        location: FileLocation {
            path: PathBuf::from("y.md"),
            line: None,
            column: None,
        },
        help: "also note".to_string(),
    };
    let s = format_diagnostics(&[d1, d2]);
    assert!(s.contains("ERROR"));
    assert!(s.contains("2 error(s)"));
}

#[test]
fn given_info_diagnostic_when_serializing_to_json_then_fields_present() {
    let d = Diagnostic {
        severity: Severity::Info,
        code: "I001".to_string(),
        message: "info msg".to_string(),
        location: FileLocation {
            path: PathBuf::from("z.md"),
            line: None,
            column: None,
        },
        help: "ok".to_string(),
    };
    let json = diagnostics_json(&[d]);
    assert!(json.contains("info msg"));
    assert!(json.contains("I001"));
}
