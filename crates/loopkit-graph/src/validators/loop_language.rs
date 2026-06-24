use crate::types::LoopContract;
use loopkit_core::parser::skill::extract_section_body;
use loopkit_core::types::{Config, Diagnostic, FileLocation, Severity, Skill};
use regex::Regex;
use std::collections::{HashMap, HashSet};

/// Validate loop language conventions across all LOOP.md files.
pub fn validate(
    skills: &[Skill],
    all_handoffs: &HashMap<String, LoopContract>,
    config: &Config,
) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    // Build a set of known skill names for verb checking context
    let known_skills: HashSet<&str> = skills.iter().map(|s| s.name.as_str()).collect();

    for skill in skills {
        let loop_path = skill.loop_md();
        if !loop_path.exists() {
            continue;
        }

        let content = match std::fs::read_to_string(&loop_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        // Check halt reason vocabulary
        diags.extend(check_halt_vocabulary(&content, &skill.name, &loop_path, config));

        // Check verb vocabulary
        diags.extend(check_verb_vocabulary(&content, &skill.name, &loop_path, config));

        // Check transition syntax
        diags.extend(check_transition_syntax(
            &content,
            &skill.name,
            &loop_path,
            &known_skills,
        ));
    }

    diags
}

fn check_halt_vocabulary(
    content: &str,
    skill_name: &str,
    path: &std::path::Path,
    config: &Config,
) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let halt_re = Regex::new(r"(?i)halt\s+(\w[\w-]*)").expect("hardcoded regex");

    let skip_words: &[&str] = &[
        "the", "this", "that", "a", "an", "when", "if", "after", "iteration",
        "and", "or", "all", "any", "current", "at", "in", "on", "to", "for",
        "conditions", "condition", "is", "are", "as", "by", "with", "without",
    ];

    for (line_num, line) in content.lines().enumerate() {
        for cap in halt_re.captures_iter(line) {
            let reason = &cap[1];
            let reason_lower = reason.to_lowercase();
            if skip_words.contains(&reason_lower.as_str()) {
                continue;
            }
            if config.halt_reasons.iter().all(|r| r != reason_lower) {
                diags.push(Diagnostic {
                    severity: Severity::Error,
                    code: "loop-unknown-halt-reason".to_string(),
                    message: format!(
                        "Unknown halt reason '{}' in LOOP.md for `{}` at line {}",
                        reason,
                        skill_name,
                        line_num + 1,
                    ),
                    location: FileLocation::new(path.to_path_buf()).at_line((line_num + 1) as u32),
                    help: format!(
                        "Standard halt reasons: {}. Use exactly one of these.",
                        config.halt_reasons.join(", ")
                    ),
                });
            }
        }
    }

    diags
}

fn check_verb_vocabulary(
    content: &str,
    skill_name: &str,
    path: &std::path::Path,
    config: &Config,
) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    let verb_re = Regex::new(r"^\s*\d*\.\s*(\w+)").expect("hardcoded regex");

    for (line_num, line) in content.lines().enumerate() {
        if let Some(cap) = verb_re.captures(line) {
            let verb = &cap[1];
            let verb_lower = verb.to_lowercase();

            // Merge compound verbs: "Hand off" -> "handoff"
            let normalized = merge_compound_verbs(&verb_lower);

            // Skip temporal conjunctions
            if is_temporal_conjunction(&verb_lower) {
                continue;
            }

            // Skip common non-verb words
            if matches!(
                verb_lower.as_str(),
                "the" | "a" | "an" | "if" | "when" | "for" | "each" | "all" | "no" | "this"
                    | "that" | "these" | "those" | "every" | "story" | "flag" | "flags"
                    | "field" | "fields" | "in" | "on" | "at"
            ) {
                continue;
            }

            // Check against configured verbs
            let is_standard = config.standard_verbs.iter().any(|v| v == &normalized)
                || config.standard_verbs.iter().any(|v| v == &verb_lower);

            if !is_standard && is_likely_action_verb(verb, line) {
                diags.push(Diagnostic {
                    severity: Severity::Warning,
                    code: "loop-nonstandard-verb".to_string(),
                    message: format!(
                        "Non-standard verb '{}' in LOOP.md for `{}` at line {}",
                        verb,
                        skill_name,
                        line_num + 1,
                    ),
                    location: FileLocation::new(path.to_path_buf()).at_line((line_num + 1) as u32),
                    help: format!(
                        "Standard verbs: {}. Consider using one of these for clarity.",
                        config.standard_verbs.join(", ")
                    ),
                });
            }
        }
    }

    diags
}

/// Merge compound verbs: "Hand off" is equivalent to "handoff",
/// "Cross reference" to "cross-reference"
fn merge_compound_verbs(word: &str) -> String {
    match word {
        "hand" | "off" => "handoff".to_string(),
        "cross" | "reference" => "cross-reference".to_string(),
        other => other.to_string(),
    }
}

/// Skip temporal conjunctions that start imperative-style lines.
fn is_temporal_conjunction(word: &str) -> bool {
    matches!(word, "after" | "before" | "once")
}

/// Heuristic: is this word likely an action verb (not a noun)?
fn is_likely_action_verb(verb: &str, line: &str) -> bool {
    let trimmed = line.trim();
    let is_step = trimmed.starts_with(|c: char| c.is_ascii_digit())
        || trimmed.starts_with('-')
        || trimmed.starts_with('*');
    if !is_step {
        return false;
    }
    let lower = verb.to_lowercase();
    !matches!(
        lower.as_str(),
        "if" | "when" | "for" | "each" | "all" | "no" | "this"
            | "that" | "these" | "those" | "every" | "story" | "flag" | "flags"
            | "field" | "fields" | "in" | "on" | "at" | "the" | "a" | "an"
            | "is" | "are" | "was" | "were" | "be" | "been"
    )
}

fn check_transition_syntax(
    content: &str,
    skill_name: &str,
    path: &std::path::Path,
    _known_skills: &HashSet<&str>,
) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    if let Some(body) = extract_section_body(content, "State Transition Rule") {
        let has_transition_directive = body.lines().any(|line| {
            let trimmed = line.trim();
            trimmed.starts_with("transition ")
                && (trimmed.contains('→') || trimmed.contains("->"))
        });
        let has_table = body.contains("| from") || body.contains("| From");

        if !has_transition_directive && !has_table {
            diags.push(Diagnostic {
                severity: Severity::Error,
                code: "loop-no-transitions".to_string(),
                message: format!(
                    "State Transition Rule section in LOOP.md for `{}` has no transition directives",
                    skill_name
                ),
                location: FileLocation::new(path.to_path_buf()),
                help: "Add transition directives using: transition <from> → <to>".to_string(),
            });
        }
    }

    diags
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn halt_vocabulary_recognizes_standard_reason() {
        let config = Config::default();
        let content = "halt stall after 5 iterations";
        let diags = check_halt_vocabulary(
            content,
            "test",
            std::path::Path::new("LOOP.md"),
            &config,
        );
        assert!(diags.is_empty());
    }

    #[test]
    fn halt_vocabulary_flags_unknown_reason() {
        let mut config = Config::default();
        config.halt_reasons = vec!["stall".to_string()];
        let content = "halt timeout after 3 iterations";
        let diags = check_halt_vocabulary(
            content,
            "test",
            std::path::Path::new("LOOP.md"),
            &config,
        );
        assert!(!diags.is_empty());
    }

    #[test]
    fn temporal_conjunctions_are_skipped() {
        assert!(is_temporal_conjunction("after"));
        assert!(is_temporal_conjunction("before"));
        assert!(is_temporal_conjunction("once"));
        assert!(!is_temporal_conjunction("trigger"));
    }

    #[test]
    fn compound_verbs_are_merged() {
        assert_eq!(merge_compound_verbs("hand"), "handoff");
        assert_eq!(merge_compound_verbs("off"), "handoff");
        assert_eq!(merge_compound_verbs("cross"), "cross-reference");
    }
}
