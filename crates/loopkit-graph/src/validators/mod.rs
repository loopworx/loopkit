pub mod bug_feedback;
pub mod constraints;
pub mod cross_references;
pub mod deskcheck;
pub mod enforced_states;
pub mod graph;
pub mod loop_completeness;
pub mod loop_language;
pub mod loop_sections;
pub mod loop_state_files;
pub mod state_consistency;

use crate::graph::build_transitions;
use crate::parser::handoff::parse_all_handoffs;
use crate::simulation;
use crate::types::{LoopContract, Transition};
use loopkit_core::types::{Config, Diagnostic, Severity, Skill};
use std::collections::HashMap;

/// Run all validators and return unified diagnostics.
pub fn run_all(
    root: &std::path::Path,
    config: &Config,
    skills: &[Skill],
    verbose: bool,
) -> Vec<Diagnostic> {
    let all_handoffs: HashMap<String, LoopContract> =
        parse_all_handoffs(&config.skills_dir, skills);
    let transitions: Vec<Transition> = build_transitions(skills, &all_handoffs);

    let mut diagnostics = Vec::new();

    macro_rules! run {
        ($label:expr, $call:expr) => {{
            let diags = $call;
            if verbose {
                let e = diags
                    .iter()
                    .filter(|d| d.severity == Severity::Error)
                    .count();
                let w = diags
                    .iter()
                    .filter(|d| d.severity == Severity::Warning)
                    .count();
                let i = diags
                    .iter()
                    .filter(|d| d.severity == Severity::Info)
                    .count();
                if e + w + i > 0 {
                    eprintln!("  {:>30}  {}E  {}W  {}I", $label, e, w, i);
                } else {
                    eprintln!("  {:>30}  ✓", $label);
                }
            }
            diagnostics.extend(diags);
        }};
    }

    // Graph validators
    run!("graph", graph::validate(&transitions));

    // Simulation
    run!(
        "simulation",
        simulation::run_all(&transitions, config.max_iterations)
    );

    // Loop language
    run!(
        "loop_language",
        loop_language::validate(skills, &all_handoffs, config)
    );

    // Loop sections
    run!(
        "loop_sections",
        loop_sections::validate(skills, &all_handoffs, config)
    );

    // State consistency (forward + reverse)
    run!(
        "state_consistency",
        state_consistency::validate(skills, &transitions, config)
    );

    // Enforced states
    run!(
        "enforced_states",
        enforced_states::validate(&transitions, config)
    );

    // Deskcheck pattern
    run!("deskcheck", deskcheck::validate(&transitions, config));

    // Bug feedback
    run!("bug_feedback", bug_feedback::validate(&transitions, config));

    // Loop completeness (skill + loop)
    run!(
        "loop_completeness",
        loop_completeness::validate(skills, &all_handoffs, config)
    );

    // Loop state files
    run!("loop_state_files", loop_state_files::validate(root, config));

    // Cross references
    run!(
        "cross_references",
        cross_references::validate(skills, skills, config)
    );

    // Constraints
    run!(
        "constraints",
        constraints::validate(&transitions, skills, config)
    );

    diagnostics
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn make_skill(name: &str, path: PathBuf) -> Skill {
        let skill_md = path.join("SKILL.md");
        Skill {
            name: name.into(),
            level: "L3".into(),
            owner: vec![],
            description: "".into(),
            category: "".into(),
            path,
            skill_md,
            sections: vec![],
            states: vec![],
        }
    }

    #[test]
    fn run_all_with_empty_skills_produces_diagnostics() {
        let config = Config::default();
        let root = std::path::PathBuf::from(".");
        let diags = run_all(&root, &config, &[], false);
        // With empty skills, we expect diagnostics from:
        // - enforced_states (all missing)
        // - constraints (empty graph)
        // - loop_state_files (missing docs files)
        assert!(!diags.is_empty());
        // It should be a Vec<Diagnostic>
        let _: &Vec<Diagnostic> = &diags;
    }

    #[test]
    fn run_all_with_skills_produces_diagnostics_vec() {
        let dir = TempDir::new().unwrap();
        let skill_dir = dir.path().join("test-skill");
        std::fs::create_dir(&skill_dir).unwrap();
        std::fs::write(skill_dir.join("SKILL.md"), "").unwrap();
        // Don't create a LOOP.md -- this will generate errors from completeness check etc.

        let skills = vec![make_skill("test-skill", skill_dir)];
        // Need a valid skills_dir for parse_all_handoffs
        let mut config = Config::default();
        config.skills_dir = dir.path().to_string_lossy().to_string();

        let diags = run_all(dir.path(), &config, &skills, false);
        // Just verify it returns diagnostics (will have some from various validators)
        assert!(!diags.is_empty());
        let _: &Vec<Diagnostic> = &diags;
    }
}
