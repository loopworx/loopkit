use crate::types::Skill;
use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Parse frontmatter from a markdown file. Returns (fields, body_line_count).
pub fn parse_frontmatter(content: &str) -> (HashMap<String, String>, usize) {
    let mut fields = HashMap::new();
    let mut lines = content.lines();
    let mut line_count = 0;

    // Skip if no frontmatter
    match lines.next() {
        Some(line) if line.trim() == "---" => {
            line_count += 1;
        }
        _ => return (fields, 0),
    }

    for line in &mut lines {
        line_count += 1;
        if line.trim() == "---" {
            line_count += 1;
            return (fields, line_count);
        }
        if let Some((key, value)) = line.split_once(':') {
            fields.insert(key.trim().to_string(), value.trim().to_string());
        }
    }

    (fields, line_count)
}

/// Parse the section headings from a markdown file.
pub fn parse_sections(content: &str) -> Vec<String> {
    let mut sections = Vec::new();
    let parser = Parser::new(content);
    let mut in_heading = false;

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) if level == HeadingLevel::H2 => {
                in_heading = true;
            }
            Event::Text(text) if in_heading => {
                sections.push(text.to_string());
                in_heading = false;
            }
            Event::End(TagEnd::Heading(..)) => {
                in_heading = false;
            }
            _ => {}
        }
    }
    sections
}

/// Parse a skill directory.
pub fn parse_skill_dir(dir: &Path) -> Option<Skill> {
    let skill_md = dir.join("SKILL.md");
    if !skill_md.exists() {
        return None;
    }

    let content = std::fs::read_to_string(&skill_md).ok()?;
    let (frontmatter, _) = parse_frontmatter(&content);

    let name = frontmatter.get("name")?.clone();
    let level = frontmatter.get("level").cloned().unwrap_or_default();
    let owner: Vec<String> = frontmatter
        .get("owner")
        .map(|o| o.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();

    // Derive category from parent directory name
    let category = dir
        .parent()
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    let sections = parse_sections(&content);

    let has_loop_md = dir.join("LOOP.md").exists();
    let has_handoffs_md = dir.join("HANDOFFS.md").exists();

    Some(Skill {
        name,
        category,
        path: dir.to_path_buf(),
        level,
        owner,
        sections,
        states: Vec::new(),
        has_loop_md,
        has_handoffs_md,
        transitions: Vec::new(),
    })
}

/// Discover all skills under a skills directory: skills/<category>/<skill-name>/SKILL.md
pub fn discover_skills(skills_dir: &Path) -> std::io::Result<Vec<Skill>> {
    let mut skills = Vec::new();
    if !skills_dir.exists() {
        return Ok(skills);
    }
    for entry in std::fs::read_dir(skills_dir)? {
        let entry = entry?;
        if !entry.metadata()?.is_dir() {
            continue;
        }
        for skill_entry in std::fs::read_dir(entry.path())? {
            let skill_entry = skill_entry?;
            if !skill_entry.metadata()?.is_dir() {
                continue;
            }
            if let Some(skill) = parse_skill_dir(&skill_entry.path()) {
                skills.push(skill);
            }
        }
    }
    skills.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(skills)
}

/// Enrich skills with state information from the graph.
pub fn enrich_skill_states(
    skills: &mut [Skill],
    all_states: &HashSet<String>,
    entry_points: &HashSet<String>,
    terminal_states: &HashSet<String>,
) {
    for skill in skills.iter_mut() {
        // Extract state names from the skill's own transition rules
        let mut skill_states: Vec<String> = skill
            .transitions
            .iter()
            .flat_map(|t| [t.from.clone(), t.to.clone()])
            .collect();
        skill_states.sort();
        skill_states.dedup();

        // Filter to only canonical states
        skill_states.retain(|s| all_states.contains(s));

        skill.states = skill_states
            .into_iter()
            .map(|name| crate::types::State {
                is_entry: entry_points.contains(&name),
                is_terminal: terminal_states.contains(&name),
                defined_in: vec![skill.name.clone()],
                name,
            })
            .collect();
    }
}
