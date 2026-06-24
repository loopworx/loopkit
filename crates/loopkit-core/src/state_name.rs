/// Validates a state name. Returns Ok(()) if valid, Err(reason) if not.
///
/// Rules:
/// - Not empty
/// - Max 128 chars
/// - Must contain at least one hyphen (kebab-case), unless it's "done"
/// - Only lowercase ASCII letters, digits, and hyphens
/// - No dots, slashes, backslashes, or whitespace
/// - Not a gerund form (ending in -ing, which indicates a skill name)
pub fn validate_state_name(s: &str) -> Result<(), String> {
    if s.is_empty() {
        return Err("empty state name".into());
    }
    if s.len() > 128 {
        return Err(format!("state name too long ({} chars, max 128)", s.len()));
    }
    if !s.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        return Err("state name must contain only lowercase letters, digits, and hyphens".into());
    }
    // Special case: "done" is a valid state name even without hyphen
    if s == "done" {
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
/// - Contains a hyphen
/// - Does not contain slashes, dots (except in kebab context), or colons
/// - Does not end in -ing (skill names)
/// - Is not a URL or file path
pub fn is_state_like(s: &str) -> bool {
    if s.is_empty() || s.len() > 128 {
        return false;
    }
    // Reject URLs and file paths
    if s.contains("://") || s.contains('/') || s.contains('\\') {
        return false;
    }
    // "done" is a valid state
    if s == "done" {
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

    #[test]
    fn valid_state_names() {
        assert!(validate_state_name("in-dev").is_ok());
        assert!(validate_state_name("ready-for-qa").is_ok());
        assert!(validate_state_name("halted-stall").is_ok());
        assert!(validate_state_name("done").is_ok());
        assert!(validate_state_name("in-deskcheck").is_ok());
    }

    #[test]
    fn rejects_skill_names() {
        assert!(validate_state_name("running-tdd-loops").is_err());
        assert!(validate_state_name("facilitating-inception").is_err());
    }

    #[test]
    fn rejects_empty_and_too_long() {
        assert!(validate_state_name("").is_err());
        assert!(validate_state_name(&"a".repeat(129)).is_err());
    }

    #[test]
    fn rejects_no_hyphen() {
        assert!(validate_state_name("backlog").is_err());
        assert!(validate_state_name("idle").is_err());
    }

    #[test]
    fn rejects_uppercase_and_special_chars() {
        assert!(validate_state_name("In-Dev").is_err());
        assert!(validate_state_name("in.dev").is_err());
        assert!(validate_state_name("in dev").is_err());
    }

    #[test]
    fn is_state_like_filters_correctly() {
        assert!(is_state_like("in-dev"));
        assert!(is_state_like("done"));
        assert!(!is_state_like("running-tdd-loops"));  // gerund
        assert!(!is_state_like("skills/meta/skill"));   // path
        assert!(!is_state_like("https://example.com")); // URL
        assert!(!is_state_like("helper"));               // no hyphen
        assert!(!is_state_like(""));                     // empty
    }
}
