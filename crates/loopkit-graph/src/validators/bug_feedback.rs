use crate::types::Transition;
use loopkit_core::types::Diagnostic;

/// Validate bug feedback loop: deskcheck/QA states should have transitions
/// back to in-dev for bug fixes.
/// (Full implementation in Task 2.7)
pub fn validate(_transitions: &[Transition]) -> Vec<Diagnostic> {
    vec![]
}
