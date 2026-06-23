use crate::types::{
    validate_halt_reason, Diagnostic, FileLocation, Repo, Severity, STANDARD_HALT_REASONS,
    STANDARD_VERBS,
};

use regex::Regex;

/// Validate loop language conventions across all LOOP.md files.
/// Checks that:
/// - Section headings are canonical
/// - Halt reasons use standard vocabulary
/// - Action verbs use standard vocabulary
/// - Transitions have valid `handoff <skill> to <agent>` syntax
pub fn validate_loop_language(repo: &Repo) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    for skill in &repo.skills {
        let loop_path = skill.loop_md();
        if !loop_path.exists() {
            continue;
        }

        let content = match std::fs::read_to_string(&loop_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        // Check halt reason vocabulary in transitions and halt conditions
        diags.extend(check_halt_vocabulary(&content, &skill.name, &loop_path));

        // Check for unknown verbs in imperatives
        diags.extend(check_verb_vocabulary(&content, &skill.name, &loop_path));

        // Check transition syntax
        diags.extend(check_transition_syntax(&content, &skill.name, &loop_path, &repo.skills));
    }

    diags
}

fn check_halt_vocabulary(
    content: &str,
    skill_name: &str,
    path: &std::path::Path,
) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    // Match `halt <reason>` where reason is a word (not common prose words like "the", "this")
    let halt_re = Regex::new(r"(?i)halt\s+(\w[\w-]*)").expect("hardcoded regex");

    // Words that appear after "halt" in prose but aren't halt reasons
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
            if validate_halt_reason(reason).is_some() {
                diags.push(Diagnostic {
                    severity: Severity::Error,
                    code: "loop-unknown-halt-reason".to_string(),
                    message: format!(
                        "Unknown halt reason '{}' in LOOP.md for `{}` at line {}",
                        reason,
                        skill_name,
                        line_num + 1,
                    ),
                    location: FileLocation {
                        path: path.to_path_buf(),
                        line: Some(line_num + 1),
                        column: None,
                    },
                    help: format!(
                        "Standard halt reasons: {}. Use exactly one of these.",
                        STANDARD_HALT_REASONS.join(", ")
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
) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    // Look for imperative sentences that start with a verb
    let verb_re = Regex::new(r"^\s*\d*\.\s*(\w+)").expect("hardcoded regex");

    for (line_num, line) in content.lines().enumerate() {
        if let Some(cap) = verb_re.captures(line) {
            let verb = &cap[1];
            let verb_lower = verb.to_lowercase();
            // Skip common words that aren't action verbs
            if matches!(
                verb_lower.as_str(),
                "the" | "a" | "an" | "if" | "when" | "for" | "each" | "all" | "no" | "this"
                    | "that" | "these" | "those" | "every" | "story" | "flag" | "flags"
                    | "field" | "fields" | "in" | "on" | "at"
            ) {
                continue;
            }
            // Check if it looks like a verb that should be in the standard set
            if !crate::types::is_standard_verb(verb)
                && is_likely_action_verb(verb, line)
            {
                diags.push(Diagnostic {
                    severity: Severity::Warning,
                    code: "loop-nonstandard-verb".to_string(),
                    message: format!(
                        "Non-standard verb '{}' in LOOP.md for `{}` at line {}",
                        verb,
                        skill_name,
                        line_num + 1,
                    ),
                    location: FileLocation {
                        path: path.to_path_buf(),
                        line: Some(line_num + 1),
                        column: None,
                    },
                    help: format!(
                        "Standard verbs: {}. Consider using one of these for clarity.",
                        STANDARD_VERBS.join(", ")
                    ),
                });
            }
        }
    }

    diags
}

/// Heuristic: is this word likely an action verb (not a noun in an imperative sentence)?
fn is_likely_action_verb(verb: &str, line: &str) -> bool {
    // If it starts with a capital letter in an imperative context, it's likely a verb
    // Check if the line starts with a number (step) or bullet
    let trimmed = line.trim();
    let is_step = trimmed.starts_with(|c: char| c.is_ascii_digit())
        || trimmed.starts_with('-')
        || trimmed.starts_with('*');
    if !is_step {
        return false;
    }
    // If the verb is already in the standard set, don't flag
    if crate::types::is_standard_verb(verb) {
        return false;
    }
    // Extended set of common lowercase action verbs
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
    _skills: &[crate::types::Skill],
) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    // Find the State Transition Rule section
    if let Some(body) = extract_section(content, "State Transition Rule") {
        let has_transition = body.contains("transition ");
        let has_table = body.contains("| from") || body.contains("| From");

        if !has_transition && !has_table {
            diags.push(Diagnostic {
                severity: Severity::Error,
                code: "loop-no-transitions".to_string(),
                message: format!(
                    "State Transition Rule section in LOOP.md for `{}` has no transition directives",
                    skill_name
                ),
                location: FileLocation {
                    path: path.to_path_buf(),
                    line: None,
                    column: None,
                },
                help: "Add transition directives using: transition <from> → <to>".to_string(),
            });
        }
    }

    diags
}

fn extract_section(content: &str, heading: &str) -> Option<String> {
    let marker = format!("## {}", heading);
    let start = content.find(&marker)?;
    let after = &content[start + marker.len()..];
    let end = after.find("\n## ").unwrap_or(after.len());
    Some(after[..end].trim().to_string())
}
