use crate::types::EnforcedState;

/// Validates a state name. Returns Ok(()) if valid, Err(reason) if not.
///
/// Rules:
/// - Not empty
/// - Max 128 chars
/// - Must contain at least one hyphen (kebab-case), unless it's a known
///   enforced state without a hyphen
/// - Only lowercase ASCII letters, digits, and hyphens
/// - No dots, slashes, backslashes, or whitespace
/// - Not a gerund form (ending in -ing, which indicates a skill name)
pub fn validate_state_name(s: &str, enforced: &[EnforcedState]) -> Result<(), String> {
    if s.is_empty() {
        return Err("empty state name".into());
    }
    if s.len() > 128 {
        return Err(format!("state name too long ({} chars, max 128)", s.len()));
    }
    if !s.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        return Err("state name must contain only lowercase letters, digits, and hyphens".into());
    }
    // Allow enforced states without hyphen (e.g. "done", "backlog")
    if enforced.iter().any(|es| es.name == s) && !s.contains('-') {
        return Ok(());
    }
    if !s.contains('-') {
        return Err("state name must contain a hyphen (kebab-case)".into());
    }
    if s.split('-').any(|part| part.ends_with("ing")) {
        return Err("state name looks like a gerund (skill name, not state)".into());
    }
    Ok(())
}

/// Returns true if the token looks like a state name (not a skill name, URL, or file path).
///
/// A state-like token:
/// - Contains a hyphen, or is a known enforced state without hyphen
/// - Does not contain slashes, dots (except in kebab context), or colons
/// - Does not end in -ing (skill names)
/// - Is not a URL or file path
pub fn is_state_like(s: &str, enforced: &[EnforcedState]) -> bool {
    if s.is_empty() || s.len() > 128 {
        return false;
    }
    // Reject URLs and file paths
    if s.contains("://") || s.contains('/') || s.contains('\\') {
        return false;
    }
    // Known enforced state without hyphen
    if enforced.iter().any(|es| es.name == s) && !s.contains('-') {
        return true;
    }
    // Must contain hyphen (kebab-case)
    if !s.contains('-') {
        return false;
    }
    // Reject gerund-form skill names
    if s.split('-').any(|part| part.ends_with("ing")) {
        return false;
    }
    // Only allowed chars
    s.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn enforced() -> Vec<EnforcedState> {
        vec![
            EnforcedState { name: "done".into(), agent: "".into(), description: "".into() },
            EnforcedState { name: "in-dev".into(), agent: "".into(), description: "".into() },
            EnforcedState { name: "in-qa".into(), agent: "".into(), description: "".into() },
            EnforcedState { name: "halted-stall".into(), agent: "".into(), description: "".into() },
            EnforcedState { name: "in-deskcheck".into(), agent: "".into(), description: "".into() },
        ]
    }

    #[test]
    fn valid_state_names() {
        let e = enforced();
        assert!(validate_state_name("in-dev", &e).is_ok());
        assert!(validate_state_name("in-qa", &e).is_ok());
        assert!(validate_state_name("halted-stall", &e).is_ok());
        assert!(validate_state_name("done", &e).is_ok());
        assert!(validate_state_name("in-deskcheck", &e).is_ok());
    }

    #[test]
    fn rejects_skill_names() {
        let e = enforced();
        assert!(validate_state_name("running-tdd-loops", &e).is_err());
        assert!(validate_state_name("facilitating-inception", &e).is_err());
    }

    #[test]
    fn rejects_empty_and_too_long() {
        let e = enforced();
        assert!(validate_state_name("", &e).is_err());
        assert!(validate_state_name(&"a".repeat(129), &e).is_err());
    }

    #[test]
    fn rejects_no_hyphen() {
        let e = enforced();
        assert!(validate_state_name("backlog", &e).is_err());
        assert!(validate_state_name("idle", &e).is_err());
    }

    #[test]
    fn rejects_uppercase_and_special_chars() {
        let e = enforced();
        assert!(validate_state_name("In-Dev", &e).is_err());
        assert!(validate_state_name("in.dev", &e).is_err());
        assert!(validate_state_name("in dev", &e).is_err());
    }

    #[test]
    fn is_state_like_filters_correctly() {
        let e = enforced();
        assert!(is_state_like("in-dev", &e));
        assert!(is_state_like("done", &e));
        assert!(!is_state_like("running-tdd-loops", &e));  // gerund
        assert!(!is_state_like("skills/meta/skill", &e));   // path
        assert!(!is_state_like("https://example.com", &e)); // URL
        assert!(!is_state_like("helper", &e));               // no hyphen
        assert!(!is_state_like("", &e));                     // empty
    }
}
