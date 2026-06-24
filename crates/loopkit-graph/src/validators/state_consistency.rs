use crate::types::Transition;
use loopkit_core::parser::skill::extract_section_body;
use loopkit_core::state_name::is_state_like;
use loopkit_core::types::{Config, Diagnostic, FileLocation, Severity, Skill};
use std::collections::HashSet;

/// Check bidirectional consistency between SKILL.md state declarations
/// and the handoff graph.
///
/// Forward: every state declared in a SKILL.md State Model (or alias) must
/// appear in the graph.
/// Reverse: every graph node must be declared in at least one State Model section.
pub fn validate(skills: &[Skill], transitions: &[Transition], config: &Config) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    let graph_states: HashSet<&str> = {
        let mut s = HashSet::new();
        for t in transitions {
            s.insert(t.from.as_str());
            s.insert(t.to.as_str());
        }
        s
    };

    // --- Forward check: skill-declared states must be in the graph ---
    for skill in skills {
        if let Ok(content) = std::fs::read_to_string(&skill.skill_md) {
            let mut prose_states = Vec::new();

            for alias in &config.state_model_aliases {
                if let Some(body) = extract_section_body(&content, alias) {
                    prose_states.extend(extract_prose_states(&body));
                }
            }

            prose_states.sort();
            prose_states.dedup();

            for state in &prose_states {
                if !graph_states.contains(state.as_str()) {
                    diags.push(Diagnostic {
                        severity: Severity::Warning,
                        code: "state-undefined-in-graph".to_string(),
                        message: format!(
                            "Skill `{}` references state `{}` in its State Model but it is not in the handoff graph",
                            skill.name, state
                        ),
                        location: FileLocation::new(skill.skill_md.clone()),
                        help: format!(
                            "Add a transition involving `{}` to some LOOP.md, or remove it from the State Model.",
                            state
                        ),
                    });
                }
            }
        }
    }

    // --- Reverse check: graph nodes must be declared in some State Model ---
    let mut graph_declared: HashSet<String> = HashSet::new();

    for skill in skills {
        if let Ok(content) = std::fs::read_to_string(&skill.skill_md) {
            for alias in &config.state_model_aliases {
                if let Some(body) = extract_section_body(&content, alias) {
                    for state in extract_prose_states(&body) {
                        graph_declared.insert(state);
                    }
                }
            }
        }
    }

    for state in &graph_states {
        if !graph_declared.contains(*state) {
            diags.push(Diagnostic {
                severity: Severity::Warning,
                code: "state-undeclared-in-skill".to_string(),
                message: format!(
                    "State '{}' appears in the handoff graph but is not declared in any SKILL.md State Model section",
                    state
                ),
                location: FileLocation::new(skills.first().map_or_else(
                    || std::path::PathBuf::from("skills"),
                    |s| s.path.parent().unwrap_or(&s.path).to_path_buf(),
                )),
                help: format!(
                    "Declare '{}' in a SKILL.md under one of: {}.",
                    state,
                    config.state_model_aliases.join(", ")
                ),
            });
        }
    }

    diags
}

/// Extract state-like tokens from section body text.
/// Looks for backtick-quoted words.
fn extract_prose_states(section_body: &str) -> Vec<String> {
    if section_body.trim().is_empty() {
        return Vec::new();
    }

    let mut states: Vec<String> = Vec::new();

    let mut in_backtick = false;
    let mut current_start = 0usize;

    for (i, ch) in section_body.char_indices() {
        if ch == '`' {
            if in_backtick {
                let token = &section_body[current_start..i];
                if is_state_like(token) {
                    states.push(token.to_string());
                }
            }
            in_backtick = !in_backtick;
            current_start = i + 1;
        }
    }

    states.sort();
    states.dedup();
    states
}
