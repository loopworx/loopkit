use crate::types::Transition;
use loopkit_core::types::{Config, Diagnostic};

/// Validate that all enforced states from Config appear in the graph.
/// (Full implementation in Task 2.5)
pub fn validate(_transitions: &[Transition], _config: &Config) -> Vec<Diagnostic> {
    vec![]
}
