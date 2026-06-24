use crate::types::Transition;
use loopkit_core::types::Diagnostic;

/// Validate deskcheck pattern: state transitions from in-dev should
/// route through a deskcheck state before QA.
/// (Full implementation in Task 2.6)
pub fn validate(_transitions: &[Transition]) -> Vec<Diagnostic> {
    vec![]
}
