use crate::types::TransitionRule;
use regex::Regex;
use std::collections::HashMap;
use std::path::Path;

lazy_static::lazy_static! {
    static ref TRANSITION_RE: Regex = Regex::new(
        r"^transition\s+(?P<from>[\w-]+)\s*(?:→|->|--->)\s*(?P<to>[\w-]+)\s*$"
    ).unwrap();

    static ref SUB_TRIGGER_RE: Regex = Regex::new(
        r"^\s*trigger\s+(?P<desc>.+)$"
    ).unwrap();

    static ref SUB_HANDOFF_RE: Regex = Regex::new(
        r"^\s*handoff\s+(?P<target>[\w-]+)\s+to\s+(?P<agent>[\w-]+)\s*$"
    ).unwrap();

    static ref SUB_HALT_RE: Regex = Regex::new(
        r"^\s*halt\s+(?P<reason>[\w-]+)(?:\s+after\s+(?P<after>\S+)\s*iterations?)?\s*$"
    ).unwrap();
}

/// Parse transition rules from LOOP.md content.
///
/// Syntax:
/// ```text
/// transition <from> -> <to>
///   trigger <description>
///   handoff <target> to <agent>
///   halt <reason> after <N> iterations
/// ```
///
/// `trigger`, `handoff`, and `halt` are optional keywords within a transition block.
/// Multiple transition blocks may appear in a single "## State Transition Rule" section.
pub fn parse_transition_rules(content: &str, skill_name: &str) -> Vec<TransitionRule> {
    let mut rules = Vec::new();
    let mut current: Option<TransitionRuleContext> = None;

    for line in content.lines() {
        let stripped = line.trim().replace('`', "");

        if let Some(caps) = TRANSITION_RE.captures(&stripped) {
            if let Some(ctx) = current.take() {
                if !ctx.from.is_empty() && !ctx.to.is_empty() {
                    rules.push(ctx.finish(skill_name));
                }
            }
            current = Some(TransitionRuleContext {
                from: caps["from"].to_string(),
                to: caps["to"].to_string(),
                trigger: None,
                handoff_target: None,
                handoff_agent: None,
                halt_reason: None,
                halt_after: None,
            });
        } else if let Some(ref mut ctx) = current {
            if let Some(caps) = SUB_TRIGGER_RE.captures(line.trim()) {
                ctx.trigger = Some(caps["desc"].to_string());
            } else if let Some(caps) = SUB_HANDOFF_RE.captures(line.trim()) {
                ctx.handoff_target = Some(caps["target"].to_string());
                ctx.handoff_agent = Some(caps["agent"].to_string());
            } else if let Some(caps) = SUB_HALT_RE.captures(line.trim()) {
                ctx.halt_reason = Some(caps["reason"].to_string());
                ctx.halt_after = caps.name("after").and_then(|m| m.as_str().parse().ok());
            }
        }
    }

    if let Some(ctx) = current {
        if !ctx.from.is_empty() && !ctx.to.is_empty() {
            rules.push(ctx.finish(skill_name));
        }
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

        if trimmed.starts_with('|') && trimmed.contains("---") {
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

            if header_map.is_empty() {
                for (i, col) in cols.iter().enumerate() {
                    header_map.insert(i, col.to_lowercase());
                }
                continue;
            }

            let mut from = String::new();
            let mut to = String::new();
            let mut trigger = String::new();

            for (i, col) in cols.iter().enumerate() {
                let val = col.trim().to_string();
                if val.is_empty() || val == "-" {
                    continue;
                }
                match header_map.get(&i).map(|s| s.as_str()) {
                    Some("from") => from = val,
                    Some("to") => to = val,
                    Some("trigger") => trigger = val,
                    _ => {}
                }
            }

            if !from.is_empty() && !to.is_empty() {
                rules.push(TransitionRule {
                    from,
                    to,
                    trigger: if trigger.is_empty() { None } else { Some(trigger) },
                    handoff_target: None,
                    handoff_agent: None,
                    halt_reason: None,
                    halt_after: None,
                    defined_in: skill_name.to_string(),
                });
            }

            in_table = false;
        }
    }

    if rules.is_empty() {
        rules = parse_transition_rules(content, skill_name);
    }

    rules
}

/// Parse all LOOP.md files for known skills and return LoopContract map.
/// This is the primary entry point for the validator orchestrator.
pub fn parse_all_handoffs(
    skills_dir: &str,
    skills: &[loopkit_core::types::Skill],
) -> HashMap<String, crate::types::LoopContract> {
    let mut result = HashMap::new();
    let skills_path = Path::new(skills_dir);

    for skill in skills {
        let loop_md = skill.loop_md();
        if loop_md.exists() {
            if let Ok(content) = std::fs::read_to_string(&loop_md) {
                let rules = parse_transition_rules(&content, &skill.name);
                if let Some(contract) =
                    crate::parser::loop_::parse_loop_contract(&loop_md, &skill.name)
                {
                    let mut contract = contract;
                    contract.transitions = rules;
                    result.insert(skill.name.clone(), contract);
                    continue;
                }
            }
        }

        let handoffs_md = skill.handoffs_md();
        if handoffs_md.exists() {
            if let Ok(content) = std::fs::read_to_string(&handoffs_md) {
                let rules = parse_handoff_table(&content, &skill.name);
                if !rules.is_empty() {
                    let contract = crate::types::LoopContract {
                        skill: skill.name.clone(),
                        sections: Vec::new(),
                        section_order_valid: true,
                        transitions: rules,
                        loop_md_path: handoffs_md.clone(),
                    };
                    result.insert(skill.name.clone(), contract);
                }
            }
        }
    }

    // Also discover skills from directory (for backwards compatibility)
    let (discovered, _) = loopkit_core::discovery::discover_skills(skills_path);
    for skill in &discovered {
        if result.contains_key(&skill.name) {
            continue;
        }
        let loop_md = skill.loop_md();
        if loop_md.exists() {
            if let Ok(content) = std::fs::read_to_string(&loop_md) {
                let rules = parse_transition_rules(&content, &skill.name);
                if let Some(contract) =
                    crate::parser::loop_::parse_loop_contract(&loop_md, &skill.name)
                {
                    let mut contract = contract;
                    let has_transitions = !rules.is_empty();
                    contract.transitions = rules;
                    if has_transitions || !contract.sections.is_empty() {
                        result.insert(skill.name.clone(), contract);
                    }
                }
            }
        }
    }

    let _ = skills_path;

    result
}

/// Parse all HANDOFFS.md files in the skills directory (legacy).
/// Returns a map of skill name -> transition rules.
pub fn parse_all_handoffs_legacy(skills_dir: &Path) -> HashMap<String, Vec<TransitionRule>> {
    let mut result = HashMap::new();
    if !skills_dir.exists() {
        return result;
    }

    let (skills, _diagnostics) = loopkit_core::discovery::discover_skills(skills_dir);
    for skill in &skills {
        let loop_md = skill.loop_md();
        if loop_md.exists() {
            if let Ok(content) = std::fs::read_to_string(&loop_md) {
                let rules = parse_transition_rules(&content, &skill.name);
                if !rules.is_empty() {
                    result.insert(skill.name.clone(), rules);
                    continue;
                }
            }
        }

        let handoffs_md = skill.handoffs_md();
        if handoffs_md.exists() {
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

struct TransitionRuleContext {
    from: String,
    to: String,
    trigger: Option<String>,
    handoff_target: Option<String>,
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
            handoff_target: self.handoff_target,
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

transition in-dev → in-deskcheck
  trigger all acceptance tests green
  handoff running-desk-checks to qa-agent
";
        let rules = parse_transition_rules(content, "test-skill");
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].from, "in-dev");
        assert_eq!(rules[0].to, "in-deskcheck");
        assert_eq!(rules[0].trigger.as_deref(), Some("all acceptance tests green"));
        assert_eq!(rules[0].handoff_target.as_deref(), Some("running-desk-checks"));
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

transition in-dev → in-deskcheck
  trigger all-ACs-green
  handoff running-desk-checks to qa-agent

transition in-dev → in-qa
  trigger manual-QA-pull
  handoff running-regression-suite to qa-agent

transition in-dev → halted-stall
  halt stall after 5 iterations
";
        let rules = parse_transition_rules(content, "test-skill");
        assert_eq!(rules.len(), 3);
    }

    #[test]
    fn parse_ascii_arrow() {
        let content = "\
## State Transition Rule

transition in-dev -> in-deskcheck
  trigger all-ACs-green
";
        let rules = parse_transition_rules(content, "test-skill");
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].from, "in-dev");
        assert_eq!(rules[0].to, "in-deskcheck");
    }

    #[test]
    fn parse_long_ascii_arrow() {
        let content = "\
## State Transition Rule

transition in-dev ---> in-deskcheck
  trigger all-ACs-green
";
        let rules = parse_transition_rules(content, "test-skill");
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].from, "in-dev");
        assert_eq!(rules[0].to, "in-deskcheck");
    }

    #[test]
    fn strip_backtick_states() {
        let content = "\
## State Transition Rule

transition `in-dev` → `in-deskcheck`
  trigger all-ACs-green
";
        let rules = parse_transition_rules(content, "test-skill");
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].from, "in-dev");
        assert_eq!(rules[0].to, "in-deskcheck");
    }

    #[test]
    fn halt_without_iterations() {
        let content = "\
## State Transition Rule

transition in-dev → halted-stall
  halt stall
";
        let rules = parse_transition_rules(content, "test-skill");
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].halt_reason.as_deref(), Some("stall"));
        assert_eq!(rules[0].halt_after, None);
    }

    #[test]
    fn halt_invalid_after_value() {
        let content = "\
## State Transition Rule

transition in-dev → halted-stall
  halt stall after abc iterations
";
        let rules = parse_transition_rules(content, "test-skill");
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].halt_reason.as_deref(), Some("stall"));
        assert_eq!(rules[0].halt_after, None);
    }

    #[test]
    fn parse_handoff_table_with_valid_table() {
        // Note: parse_handoff_table expects separator before header.
        // The first line with `---` enables table mode, the next line is header,
        // then data rows.
        let content = "\
|--------|-----------|------------------|
| from   | to        | trigger          |
| in-dev | in-qa     | all-ACs-green    |
";
        let rules = parse_handoff_table(content, "test-skill");
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].from, "in-dev");
        assert_eq!(rules[0].to, "in-qa");
        assert_eq!(rules[0].trigger.as_deref(), Some("all-ACs-green"));
    }

    #[test]
    fn parse_handoff_table_fallback_to_transition() {
        let content = "\
## State Transition Rule

transition in-dev → in-qa
  trigger all green
";
        let rules = parse_handoff_table(content, "test-skill");
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].from, "in-dev");
        assert_eq!(rules[0].to, "in-qa");
    }

    #[test]
    fn parse_handoff_table_empty_content() {
        let rules = parse_handoff_table("", "test-skill");
        assert!(rules.is_empty());
    }

    #[test]
    fn parse_all_handoffs_discovers_skills() {
        let dir = tempfile::TempDir::new().unwrap();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir(&skills_dir).unwrap();

        let skill_dir = skills_dir.join("test-skill");
        std::fs::create_dir(&skill_dir).unwrap();
        std::fs::write(skill_dir.join("SKILL.md"), "").unwrap();
        std::fs::write(
            skill_dir.join("LOOP.md"),
            "\
## Entry Conditions

## Loop State Schema

## Single Iteration Step

## Proof of Progress

## State Transition Rule
transition in-dev → in-qa

## Halt Conditions

## Handoff Target
",
        )
        .unwrap();

        use loopkit_core::types::Skill;
        let skills = vec![Skill {
            name: "test-skill".into(),
            level: "L3".into(),
            owner: vec![],
            description: "".into(),
            category: "".into(),
            path: skill_dir.clone(),
            skill_md: skill_dir.join("SKILL.md"),
            sections: vec![],
            states: vec![],
        }];

        let handoffs = parse_all_handoffs(&skills_dir.to_string_lossy(), &skills);
        assert!(handoffs.contains_key("test-skill"));
    }

    #[test]
    fn parse_all_handoffs_legacy_works() {
        let dir = tempfile::TempDir::new().unwrap();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir(&skills_dir).unwrap();

        // discover_skills requires depth 2: skills_dir/category/skill-name/
        let category_dir = skills_dir.join("general");
        let skill_dir = category_dir.join("test-skill");
        std::fs::create_dir_all(&skill_dir).unwrap();
        std::fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: test-skill\ndescription: A test\nlevel: L3\n---\n",
        )
        .unwrap();
        std::fs::write(
            skill_dir.join("LOOP.md"),
            "\
## State Transition Rule
transition in-dev → in-qa
",
        )
        .unwrap();

        let result = parse_all_handoffs_legacy(&skills_dir);
        assert!(result.contains_key("test-skill"));
        assert_eq!(result["test-skill"].len(), 1);
    }

    #[test]
    fn parse_all_handoffs_legacy_empty_dir() {
        let dir = tempfile::TempDir::new().unwrap();
        let result = parse_all_handoffs_legacy(dir.path());
        assert!(result.is_empty());
    }

    #[test]
    fn parse_all_handoffs_legacy_nonexistent_dir() {
        let result = parse_all_handoffs_legacy(std::path::Path::new("/nonexistent/path/xyz"));
        assert!(result.is_empty());
    }

    #[test]
    fn parse_handoff_table_with_empty_cols_and_extra_header() {
        let content = "\
|--------|-----------|------------------|---------------|
| from   | to        | trigger          | condition     |
| in-dev | in-qa     | all-ACs-green    | -             |
";
        let rules = parse_handoff_table(content, "test-skill");
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].from, "in-dev");
        assert_eq!(rules[0].to, "in-qa");
        assert_eq!(rules[0].trigger.as_deref(), Some("all-ACs-green"));
    }

    #[test]
    fn parse_all_handoffs_with_handoffs_md_fallback() {
        let dir = tempfile::TempDir::new().unwrap();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir(&skills_dir).unwrap();

        let skill_dir = skills_dir.join("test-skill");
        std::fs::create_dir(&skill_dir).unwrap();
        std::fs::write(skill_dir.join("SKILL.md"), "").unwrap();
        // No LOOP.md — but has HANDOFFS.md
        std::fs::write(
            skill_dir.join("HANDOFFS.md"),
            "\
## State Transition Rule
transition in-dev → in-qa
",
        )
        .unwrap();

        use loopkit_core::types::Skill;
        let skills = vec![Skill {
            name: "test-skill".into(),
            level: "L3".into(),
            owner: vec![],
            description: "".into(),
            category: "".into(),
            path: skill_dir.clone(),
            skill_md: skill_dir.join("SKILL.md"),
            sections: vec![],
            states: vec![],
        }];

        let handoffs = parse_all_handoffs(&skills_dir.to_string_lossy(), &skills);
        assert!(handoffs.contains_key("test-skill"));
    }

    #[test]
    fn parse_all_handoffs_discovery_adds_new_skills() {
        let dir = tempfile::TempDir::new().unwrap();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir(&skills_dir).unwrap();

        // A skill discovered only via directory scan (not in skills list)
        let category_dir = skills_dir.join("general");
        let skill_dir = category_dir.join("discovered-skill");
        std::fs::create_dir_all(&skill_dir).unwrap();
        std::fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: discovered-skill\ndescription: A test\nlevel: L3\n---\n",
        )
        .unwrap();
        std::fs::write(
            skill_dir.join("LOOP.md"),
            "\
## Entry Conditions

## Loop State Schema

## Single Iteration Step

## Proof of Progress

## State Transition Rule
transition a → b

## Halt Conditions

## Handoff Target
",
        )
        .unwrap();

        // Pass empty skills — discovery should find it
        use loopkit_core::types::Skill;
        let skills: Vec<Skill> = vec![];

        let handoffs = parse_all_handoffs(&skills_dir.to_string_lossy(), &skills);
        assert!(handoffs.contains_key("discovered-skill"));
    }

    #[test]
    fn parse_all_handoffs_legacy_with_handoffs_md() {
        let dir = tempfile::TempDir::new().unwrap();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir(&skills_dir).unwrap();

        let category_dir = skills_dir.join("general");
        let skill_dir = category_dir.join("test-skill");
        std::fs::create_dir_all(&skill_dir).unwrap();
        std::fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: test-skill\ndescription: A test\nlevel: L3\n---\n",
        )
        .unwrap();
        // No LOOP.md — fallback to HANDOFFS.md
        std::fs::write(
            skill_dir.join("HANDOFFS.md"),
            "\
## State Transition Rule
transition in-dev → in-qa
",
        )
        .unwrap();

        let result = parse_all_handoffs_legacy(&skills_dir);
        assert!(result.contains_key("test-skill"));
        assert_eq!(result["test-skill"].len(), 1);
    }
}
