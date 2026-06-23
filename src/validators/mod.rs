use crate::types::{Diagnostic, Repo};
use crate::simulation;

mod constraints;
mod cross_references;
mod graph;
mod loop_language;
mod loop_sections;
mod loop_state_files;
mod skill_completeness;
mod state_consistency;

pub use self::constraints::*;
pub use self::cross_references::*;
pub use self::graph::*;
pub use self::loop_language::*;
pub use self::loop_sections::*;
pub use self::loop_state_files::*;
pub use self::skill_completeness::*;
pub use self::state_consistency::*;

/// Run all validators and return all diagnostics.
pub fn run_all(repo: &Repo) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    // Skill-level checks
    diagnostics.extend(validate_skill_completeness(repo));
    diagnostics.extend(validate_loop_language(repo));

    // Loop contract checks
    diagnostics.extend(validate_loop_completeness(repo));
    diagnostics.extend(validate_loop_sections(repo));

    // Graph checks
    diagnostics.extend(validate_graph(repo));

    // Cross-cutting checks
    diagnostics.extend(validate_constraints(repo));
    diagnostics.extend(validate_cross_references(repo));
    diagnostics.extend(validate_state_consistency(repo));
    diagnostics.extend(validate_loop_state_files(repo));

    // Simulation (budget-bounded reachability)
    diagnostics.extend(simulation::run_all(repo, 20));

    diagnostics
}
