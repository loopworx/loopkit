use std::path::PathBuf;

/// Helper: get the fixture root (examples/test-fixture) relative to the workspace.
fn fixture_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .join("..")
        .join("..")
        .join("examples")
        .join("test-fixture")
}

/// Helper: run the full pipeline against the fixture.
fn run_fixture() -> (Vec<loopkit_core::types::Diagnostic>, usize) {
    let root = fixture_root();
    let config = loopkit_core::config::load_config(&root);
    let skills_dir = root.join(&config.skills_dir);
    let (skills, discovery_diags) = loopkit_core::discovery::discover_skills(&skills_dir);

    let mut all_diags = discovery_diags;
    all_diags.extend(loopkit_graph::validators::run_all(
        &root, &config, &skills, false,
    ));
    all_diags.extend(loopkit::best_practices::check_all(&skills, false));

    (all_diags, skills.len())
}

fn codes(diags: &[loopkit_core::types::Diagnostic]) -> Vec<&str> {
    let mut v: Vec<&str> = diags.iter().map(|d| d.code.as_str()).collect();
    v.sort();
    v.dedup();
    v
}

fn errors(diags: &[loopkit_core::types::Diagnostic]) -> Vec<&loopkit_core::types::Diagnostic> {
    diags
        .iter()
        .filter(|d| d.severity == loopkit_core::types::Severity::Error)
        .collect()
}

fn warnings(diags: &[loopkit_core::types::Diagnostic]) -> Vec<&loopkit_core::types::Diagnostic> {
    diags
        .iter()
        .filter(|d| d.severity == loopkit_core::types::Severity::Warning)
        .collect()
}

#[test]
fn fixture_discovers_both_skills() {
    let (_, count) = run_fixture();
    assert_eq!(count, 2, "should discover both skills");
}

#[test]
fn fixture_catches_missing_description() {
    let (diags, _) = run_fixture();
    assert!(
        diags.iter().any(|d| d.code == "skill-missing-description"),
        "should catch missing description in broken-helper"
    );
}

#[test]
fn fixture_catches_duplicate_rules() {
    let (diags, _) = run_fixture();
    assert!(
        diags.iter().any(|d| d.code == "skill-duplicate-rules"),
        "should catch duplicate Rules section"
    );
}

#[test]
fn fixture_catches_non_gerund_name() {
    let (diags, _) = run_fixture();
    assert!(
        diags.iter().any(|d| d.code == "skill-name-not-gerund"),
        "should flag non-gerund name"
    );
}

#[test]
fn fixture_catches_vague_name() {
    let (diags, _) = run_fixture();
    assert!(
        diags.iter().any(|d| d.code == "skill-name-vague"),
        "should flag vague name containing 'helper'"
    );
}

#[test]
fn fixture_catches_windows_path() {
    let (diags, _) = run_fixture();
    assert!(
        diags.iter().any(|d| d.code == "skill-windows-path"),
        "should flag Windows-style paths"
    );
}

#[test]
fn fixture_catches_time_sensitive_language() {
    let (diags, _) = run_fixture();
    assert!(
        diags.iter().any(|d| d.code == "skill-time-sensitive"),
        "should flag date-specific language"
    );
}

#[test]
fn fixture_catches_too_many_options() {
    let (diags, _) = run_fixture();
    assert!(
        diags.iter().any(|d| d.code == "skill-too-many-options"),
        "should flag too many equivalent options"
    );
}

#[test]
fn fixture_catches_magic_numbers() {
    let (diags, _) = run_fixture();
    assert!(
        diags.iter().any(|d| d.code == "skill-magic-numbers"),
        "should flag undocumented magic numbers in code blocks"
    );
}

#[test]
fn fixture_catches_missing_checklist() {
    let (diags, _) = run_fixture();
    assert!(
        diags.iter().any(|d| d.code == "skill-missing-checklist"),
        "should flag multi-step workflow without checklist"
    );
}

#[test]
fn fixture_catches_missing_feedback_loop() {
    let (diags, _) = run_fixture();
    assert!(
        diags
            .iter()
            .any(|d| d.code == "skill-missing-feedback-loop"),
        "should flag missing feedback loop pattern"
    );
}

#[test]
fn fixture_catches_loop_section_order() {
    let (diags, _) = run_fixture();
    assert!(
        diags.iter().any(|d| d.code == "loop-section-order"),
        "should flag LOOP.md with non-canonical section order"
    );
}

#[test]
fn fixture_catches_unknown_halt_reason() {
    let (diags, _) = run_fixture();
    assert!(
        diags.iter().any(|d| d.code == "loop-unknown-halt-reason"),
        "should flag halt with non-standard reason"
    );
}

#[test]
fn fixture_catches_unknown_handoff_target() {
    let (diags, _) = run_fixture();
    assert!(
        diags
            .iter()
            .any(|d| d.code == "constraints-unknown-handoff-target"),
        "should flag handoff to non-existent skill"
    );
}

#[test]
fn fixture_catches_nonstandard_verbs() {
    let (diags, _) = run_fixture();
    assert!(
        diags.iter().any(|d| d.code == "loop-nonstandard-verb"),
        "should flag non-standard verbs in LOOP.md"
    );
}

#[test]
fn fixture_catches_undeclared_states() {
    let (diags, _) = run_fixture();
    assert!(
        diags.iter().any(|d| d.code == "state-undeclared-in-skill"),
        "should flag graph states not declared in any SKILL.md State Model"
    );
}

#[test]
fn fixture_catches_missing_loop_state_files() {
    let (diags, _) = run_fixture();
    assert!(
        diags.iter().any(|d| d.code == "loop-state-file-missing"),
        "should flag missing loop state tracking files"
    );
}

#[test]
fn fixture_catches_unknown_handoff_reference() {
    let (diags, _) = run_fixture();
    assert!(
        diags.iter().any(|d| d.code == "xref-unknown-handoff-skill"),
        "should flag backtick references to unknown skills"
    );
}

#[test]
fn valid_skill_has_no_frontmatter_errors() {
    let (diags, _) = run_fixture();
    let running_diags: Vec<_> = diags
        .iter()
        .filter(|d| {
            d.location
                .path
                .to_string_lossy()
                .contains("running-tdd-loops")
        })
        .filter(|d| d.severity == loopkit_core::types::Severity::Error)
        .collect();
    let frontmatter_codes: Vec<_> = running_diags
        .iter()
        .filter(|d| {
            d.code.starts_with("skill-name-")
                || d.code.starts_with("skill-missing-name")
                || d.code.starts_with("skill-description-")
                || d.code.starts_with("skill-missing-description")
        })
        .map(|d| d.code.as_str())
        .collect();
    assert!(
        frontmatter_codes.is_empty(),
        "valid skill should have no frontmatter errors, got: {:?}",
        frontmatter_codes
    );
}

#[test]
fn valid_skill_has_no_loop_section_errors() {
    let (diags, _) = run_fixture();
    let running_diags: Vec<_> = diags
        .iter()
        .filter(|d| {
            d.location
                .path
                .to_string_lossy()
                .contains("running-tdd-loops")
        })
        .filter(|d| d.severity == loopkit_core::types::Severity::Error)
        .filter(|d| d.code.starts_with("loop-"))
        .collect();
    assert!(
        running_diags.is_empty(),
        "valid skill should have no loop section errors, got: {:?}",
        running_diags.iter().map(|d| &d.code).collect::<Vec<_>>()
    );
}

#[test]
fn valid_skill_has_no_structure_errors() {
    let (diags, _) = run_fixture();
    let running_diags: Vec<_> = diags
        .iter()
        .filter(|d| {
            d.location
                .path
                .to_string_lossy()
                .contains("running-tdd-loops")
        })
        .filter(|d| d.severity == loopkit_core::types::Severity::Error)
        .filter(|d| d.code.starts_with("skill-"))
        .collect();
    assert!(
        running_diags.is_empty(),
        "valid skill should have no structure errors, got: {:?}",
        running_diags.iter().map(|d| &d.code).collect::<Vec<_>>()
    );
}

#[test]
fn json_output_format_is_valid() {
    let (diags, skills_count) = run_fixture();
    let json = loopkit_core::diagnostic::diagnostics_json(&diags, skills_count);
    let parsed: serde_json::Value =
        serde_json::from_str(&json).expect("JSON output should be valid");
    assert_eq!(parsed["skills_checked"], skills_count as u64);
    assert!(parsed["diagnostics"].is_array());
    assert!(parsed["summary"]["errors"].is_u64());
    assert!(parsed["summary"]["warnings"].is_u64());
}

#[test]
fn summary_counts_match_diagnostics() {
    let (diags, skills_count) = run_fixture();
    let err_count = errors(&diags).len();
    let warn_count = warnings(&diags).len();
    let json = loopkit_core::diagnostic::diagnostics_json(&diags, skills_count);
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["summary"]["errors"], err_count as u64);
    assert_eq!(parsed["summary"]["warnings"], warn_count as u64);
}

#[test]
fn text_output_contains_all_diagnostics() {
    let (diags, skills_count) = run_fixture();
    let text = loopkit_core::diagnostic::format_diagnostics(&diags);
    let summary = loopkit_core::diagnostic::format_summary(&diags, skills_count);

    for d in &diags {
        assert!(
            text.contains(&d.code),
            "text output should contain code {}",
            d.code
        );
    }
    assert!(summary.contains(&format!("{} skills checked", skills_count)));
    assert!(summary.contains("error(s)"));
    assert!(summary.contains("warning(s)"));
}

#[test]
fn enforced_state_check_runs() {
    let (diags, _) = run_fixture();
    let enforced_missing: Vec<_> = diags
        .iter()
        .filter(|d| d.code == "state-enforced-missing")
        .map(|d| d.message.as_str())
        .collect();
    // With only 2 skills covering in-dev through done (happy path), some enforced
    // states like backlog, halted-* will be missing.
    assert!(
        !enforced_missing.is_empty(),
        "should report some enforced states as missing"
    );
}

#[test]
fn bug_feedback_and_deskcheck_checks_run() {
    let (diags, _) = run_fixture();
    let codes = codes(&diags);
    // The valid skill has proper deskcheck and bug feedback transitions,
    // so these specific codes should NOT appear (proper transitions exist).
    assert!(
        !codes.contains(&"state-missing-deskcheck-entry"),
        "valid skill has in-dev → in-deskcheck transition"
    );
    assert!(
        !codes.contains(&"state-missing-deskcheck-completion"),
        "valid skill has in-deskcheck → in-qa transition"
    );
    assert!(
        !codes.contains(&"state-missing-bug-feedback"),
        "valid skill has bug feedback transitions (in-qa → in-dev, in-acceptance → in-dev)"
    );
}
