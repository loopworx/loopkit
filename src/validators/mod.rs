use crate::types::{Diagnostic, Repo};

mod constraints;
mod graph;
mod loop_language;
mod loop_sections;
mod skill_completeness;

pub use self::constraints::*;
pub use self::graph::*;
pub use self::loop_language::*;
pub use self::loop_sections::*;
pub use self::skill_completeness::*;

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

    // Constraints
    diagnostics.extend(validate_constraints(repo));

    diagnostics
}
