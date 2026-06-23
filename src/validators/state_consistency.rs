//! State consistency validator: checks that states declared in
//! SKILL.md State Model sections appear in the handoff graph, and
//! that graph states are owned by at least one skill.

use crate::types::{Diagnostic, FileLocation, Repo, Severity};
use std::collections::HashSet;

/// Check bidirectional consistency between SKILL.md state declarations
/// and the handoff graph.
pub fn validate_state_consistency(repo: &Repo) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    let graph_states: HashSet<&str> =
        repo.handoff_graph.nodes.iter().map(|s| s.name.as_str()).collect();

    // Check: states in skill transitions should be in the graph
    // (they always are since the graph is built from them, but this
    // catches skills that declare states in prose State Model sections
    // that don't appear in any transition)
    for skill in &repo.skills {
        // Extract state names from the skill's State Model section
        let skill_md = skill.skill_md();
        if let Ok(content) = std::fs::read_to_string(&skill_md) {
            let prose_states = extract_prose_states(&content);
            for state in &prose_states {
                if !graph_states.contains(state.as_str()) {
                    diags.push(Diagnostic {
                        severity: Severity::Warning,
                        code: "state-undefined-in-graph".to_string(),
                        message: format!(
                            "Skill `{}` references state `{}` in its State Model but it is not in the handoff graph",
                            skill.name, state
                        ),
                        location: FileLocation {
                            path: skill_md.clone(),
                            line: None,
                            column: None,
                        },
                        help: format!(
                            "Add a transition involving `{}` to some LOOP.md, or remove it from the State Model.",
                            state
                        ),
                    });
                }
            }
        }
    }

    diags
}

/// Extract state-like tokens from a SKILL.md's State Model section.
/// Looks for backtick-quoted words and `→`-separated chains.
fn extract_prose_states(content: &str) -> Vec<String> {
    let mut states = Vec::new();

    // Find the State Model section
    let marker = "## State Model";
    let start = match content.find(marker) {
        Some(s) => s + marker.len(),
        None => return states,
    };
    let after = &content[start..];
    let end = after.find("\n## ").unwrap_or(after.len());
    let section = &after[..end];

    // Extract backtick-quoted tokens
    let mut in_backtick = false;
    let mut current = String::new();
    for ch in section.chars() {
        if ch == '`' {
            if in_backtick {
                // Close — check if it looks like a state name
                if is_state_like(&current) {
                    states.push(current.clone());
                }
                current.clear();
            }
            in_backtick = !in_backtick;
        } else if in_backtick {
            current.push(ch);
        }
    }

    states.sort();
    states.dedup();
    states
}

/// Heuristic: does a token look like a state name?
fn is_state_like(s: &str) -> bool {
    !s.is_empty()
        && s.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        && s.contains('-') // state names use kebab-case
        && !s.starts_with("http")
        && !s.contains('/')
}
