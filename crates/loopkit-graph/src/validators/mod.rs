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

use crate::graph;
use crate::parser::handoff::parse_all_handoffs;
use crate::simulation;
use crate::types::{LoopContract, Transition};
use loopkit_core::types::{Config, Diagnostic, Skill};
use std::collections::HashMap;

/// Run all validators and return unified diagnostics.
pub fn run_all(config: &Config, skills: &[Skill]) -> Vec<Diagnostic> {
    let all_handoffs: HashMap<String, LoopContract> =
        parse_all_handoffs(&config.skills_dir, skills);
    let transitions: Vec<Transition> = graph::build_transitions(skills, &all_handoffs);

    let mut diagnostics = Vec::new();

    // Graph validators
    diagnostics.extend(graph::validate(&transitions));

    // Simulation
    diagnostics.extend(simulation::run_all(&transitions, config.max_iterations));

    // Loop language
    diagnostics.extend(loop_language::validate(skills, &all_handoffs, config));

    // Loop sections
    diagnostics.extend(loop_sections::validate(skills, &all_handoffs, config));

    // State consistency (forward + reverse)
    diagnostics.extend(state_consistency::validate(skills, &transitions, config));

    // Enforced states (stub -- Task 2.5)
    diagnostics.extend(enforced_states::validate(&transitions, config));

    // Deskcheck pattern (stub -- Task 2.6)
    diagnostics.extend(deskcheck::validate(&transitions));

    // Bug feedback (stub -- Task 2.7)
    diagnostics.extend(bug_feedback::validate(&transitions));

    // Loop completeness (skill + loop)
    diagnostics.extend(loop_completeness::validate(skills, &all_handoffs, config));

    // Loop state files
    diagnostics.extend(loop_state_files::validate(config));

    // Cross references
    diagnostics.extend(cross_references::validate(skills, skills));

    // Constraints
    diagnostics.extend(constraints::validate(&transitions, skills, config));

    diagnostics
}
