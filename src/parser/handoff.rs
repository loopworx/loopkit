use crate::types::TransitionRule;
use regex::Regex;
use std::collections::HashMap;
use std::path::Path;

/// Parse transition rules from LOOP.md content.
///
/// Syntax:
/// ```text
/// transition <from> -> <to>
///   trigger <description>
///   handoff <skill-name> to <agent>
///   halt <reason> after <N> iterations
/// ```
///
/// `trigger`, `handoff`, and `halt` are optional keywords within a transition block.
/// Multiple transition blocks may appear in a single "## State Transition Rule" section.
pub fn parse_transition_rules(content: &str, skill_name: &str) -> Vec<TransitionRule> {
    let transition_re = Regex::new(
        r"^transition\s+(?P<from>[\w-]+)\s*→\s*(?P<to>[\w-]+)\s*$"
    )
    .expect("hardcoded regex");

    let handoff_re = Regex::new(
        r"^\s*handoff\s+(?P<skill>[\w-]+)\s+to\s+(?P<agent>[\w-]+)\s*$"
    )
    .expect("hardcoded regex");

    let halt_re = Regex::new(
        r"^\s*halt\s+(?P<reason>[\w-]+)(?:\s+after\s+(?P<after>\d+)\s+iterations?)?\s*$"
    )
    .expect("hardcoded regex");

    let trigger_re = Regex::new(r"^\s*trigger\s+(?P<desc>.+)$").expect("hardcoded regex");

    let mut rules = Vec::new();
    let mut current: Option<TransitionRuleContext> = None;

    for line in content.lines() {
        if let Some(caps) = transition_re.captures(line) {
            // Finish previous rule
            if let Some(ctx) = current.take() {
                rules.push(ctx.finish(skill_name));
            }
            current = Some(TransitionRuleContext {
                from: caps["from"].to_string(),
                to: caps["to"].to_string(),
                trigger: None,
                handoff_skill: None,
                handoff_agent: None,
                halt_reason: None,
                halt_after: None,
            });
        } else if let Some(caps) = handoff_re.captures(line) {
            if let Some(ref mut ctx) = current {
                ctx.handoff_skill = Some(caps["skill"].to_string());
                ctx.handoff_agent = Some(caps["agent"].to_string());
            }
        } else if let Some(caps) = halt_re.captures(line) {
            if let Some(ref mut ctx) = current {
                ctx.halt_reason = Some(caps["reason"].to_string());
                ctx.halt_after = caps.name("after").map(|m| {
                    m.as_str().parse::<u32>().unwrap_or(0)
                });
            }
        } else if let Some(caps) = trigger_re.captures(line) {
            if let Some(ref mut ctx) = current {
                ctx.trigger = Some(caps["desc"].to_string());
            }
        }
    }

    // Finish last rule
    if let Some(ctx) = current {
        rules.push(ctx.finish(skill_name));
    }

    rules
}

/// Parse transition rules from HANDOFFS.md tables (backwards compatibility).
/// Expects markdown tables with columns: from, to, trigger, condition
pub fn parse_handoff_table(content: &str, skill_name: &str) -> Vec<TransitionRule> {
    let mut rules = Vec::new();
    let mut in_table = false;
    let mut header_map: HashMap<usize, String> = HashMap::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // Detect table separator line
        if trimmed.starts_with('|') && trimmed.contains("---") {
            // Parse the preceding header line from a buffer pattern
            // We handle this differently: look for lines with | col1 | col2 | ... |
            in_table = true;
            continue;
        }

        if in_table && trimmed.starts_with('|') {
            let cols: Vec<&str> = trimmed
                .split('|')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();

            if cols.is_empty() {
                continue;
            }

            // First non-separator row is the header
            if header_map.is_empty() {
                for (i, col) in cols.iter().enumerate() {
                    header_map.insert(i, col.to_lowercase());
                }
                continue;
            }

            let mut from = String::new();
            let mut to = String::new();
            let mut trigger = String::new();
            let mut condition: Option<String> = None;

            for (i, col) in cols.iter().enumerate() {
                let val = col.trim().to_string();
                if val.is_empty() || val == "-" {
                    continue;
                }
                match header_map.get(&i).map(|s| s.as_str()) {
                    Some("from") => from = val,
                    Some("to") => to = val,
                    Some("trigger") => trigger = val,
                    Some("condition") => condition = Some(val),
                    _ => {}
                }
            }

            if !from.is_empty() && !to.is_empty() {
                rules.push(TransitionRule {
                    from,
                    to,
                    trigger: if trigger.is_empty() { None } else { Some(trigger) },
                    handoff_skill: None,
                    handoff_agent: None,
                    halt_reason: None,
                    halt_after: None,
                    defined_in: skill_name.to_string(),
                });
            }

            // After parsing one data row in handoff table, stop (we don't expect more
            // table rows to be relevant)
            in_table = false;
        }
    }

    // If we didn't find a table, look for "transition" style rules too
    if rules.is_empty() {
        rules = parse_transition_rules(content, skill_name);
    }

    rules
}

/// Parse all HANDOFFS.md files in the skills directory.
/// Returns a map of skill name -> transition rules.
pub fn parse_all_handoffs(skills_dir: &Path) -> HashMap<String, Vec<TransitionRule>> {
    let mut result = HashMap::new();
    if !skills_dir.exists() {
        return result;
    }

    let skills = crate::parser::skill::discover_skills(skills_dir).unwrap_or_default();
    for skill in &skills {
        // Prefer LOOP.md transition rules
        if skill.has_loop_md {
            let loop_md = skill.loop_md();
            if let Ok(content) = std::fs::read_to_string(&loop_md) {
                let rules = parse_transition_rules(&content, &skill.name);
                if !rules.is_empty() {
                    result.insert(skill.name.clone(), rules);
                    continue;
                }
            }
        }

        // Fall back to HANDOFFS.md table
        if skill.has_handoffs_md {
            let handoffs_md = skill.handoffs_md();
            if let Ok(content) = std::fs::read_to_string(&handoffs_md) {
                let rules = parse_handoff_table(&content, &skill.name);
                if !rules.is_empty() {
                    result.insert(skill.name.clone(), rules);
                }
            }
        }
    }

    result
}

// ── Internal ─────────────────────────────────────────────────────────

struct TransitionRuleContext {
    from: String,
    to: String,
    trigger: Option<String>,
    handoff_skill: Option<String>,
    handoff_agent: Option<String>,
    halt_reason: Option<String>,
    halt_after: Option<u32>,
}

impl TransitionRuleContext {
    fn finish(self, skill_name: &str) -> TransitionRule {
        TransitionRule {
            from: self.from,
            to: self.to,
            trigger: self.trigger,
            handoff_skill: self.handoff_skill,
            handoff_agent: self.handoff_agent,
            halt_reason: self.halt_reason,
            halt_after: self.halt_after,
            defined_in: skill_name.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_single_transition() {
        let content = "\
## State Transition Rule

transition in-dev → ready-for-deskcheck
  trigger all acceptance tests green
  handoff running-desk-checks to qa-agent
";
        let rules = parse_transition_rules(content, "test-skill");
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].from, "in-dev");
        assert_eq!(rules[0].to, "ready-for-deskcheck");
        assert_eq!(rules[0].trigger.as_deref(), Some("all acceptance tests green"));
        assert_eq!(rules[0].handoff_skill.as_deref(), Some("running-desk-checks"));
        assert_eq!(rules[0].handoff_agent.as_deref(), Some("qa-agent"));
    }

    #[test]
    fn parse_transition_with_halt() {
        let content = "\
## State Transition Rule

transition in-dev → halted-stall
  halt stall after 5 iterations
";
        let rules = parse_transition_rules(content, "test-skill");
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].from, "in-dev");
        assert_eq!(rules[0].to, "halted-stall");
        assert_eq!(rules[0].halt_reason.as_deref(), Some("stall"));
        assert_eq!(rules[0].halt_after, Some(5));
    }

    #[test]
    fn parse_multiple_transitions() {
        let content = "\
## State Transition Rule

transition in-dev → ready-for-deskcheck
  trigger all-ACs-green
  handoff running-desk-checks to qa-agent

transition in-dev → ready-for-qa
  trigger manual-QA-pull
  handoff running-regression-suite to qa-agent

transition in-dev → halted-stall
  halt stall after 5 iterations
";
        let rules = parse_transition_rules(content, "test-skill");
        assert_eq!(rules.len(), 3);
    }
}
