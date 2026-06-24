use std::collections::HashMap;
use std::path::Path;

use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use crate::types::{Diagnostic, Section, Skill};

/// Parse YAML frontmatter from SKILL.md content.
/// Returns a map of key-value pairs and the line number where the body starts.
pub fn parse_frontmatter(content: &str) -> (HashMap<String, String>, usize) {
    if !content.starts_with("---") {
        return (HashMap::new(), 0);
    }

    let rest = &content[3..];
    match rest.find("\n---") {
        None => (HashMap::new(), 0),
        Some(end) => {
            let yaml_str = &rest[..end].trim();
            if yaml_str.is_empty() {
                return (HashMap::new(), 0);
            }

            let mut map = HashMap::new();
            for line in yaml_str.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }
                // Skip indented lines: they are sub-keys of a parent mapping (e.g., metadata:)
                if line.starts_with(' ') || line.starts_with('\t') {
                    continue;
                }
                if let Some((k, v)) = trimmed.split_once(':') {
                    let key = k.trim().to_string();
                    let val = v.trim().trim_matches('"').trim_matches('\'').to_string();
                    map.insert(key, val);
                }
            }

            let body_line = rest[..end].matches('\n').count() + 1;
            (map, body_line)
        }
    }
}

/// Parse H2 sections from markdown content.
/// Returns a vector of Section structs with name and body.
pub fn parse_sections(content: &str) -> Vec<Section> {
    let parser = Parser::new_ext(content, Options::all());
    let mut sections = Vec::new();
    let mut current_name: Option<String> = None;
    let mut current_body = String::new();
    let mut in_h2_heading = false;
    let mut heading_text = String::new();

    for event in parser {
        match event {
            Event::Start(Tag::Heading {
                level: HeadingLevel::H2,
                ..
            }) => {
                if let Some(name) = current_name.take() {
                    sections.push(Section {
                        name,
                        body: current_body.trim().to_string(),
                    });
                    current_body = String::new();
                }
                in_h2_heading = true;
                heading_text = String::new();
            }
            Event::End(TagEnd::Heading(HeadingLevel::H2)) => {
                if in_h2_heading {
                    current_name = Some(heading_text.trim().to_string());
                    in_h2_heading = false;
                }
            }
            Event::Text(text) => {
                if in_h2_heading {
                    heading_text.push_str(&text);
                } else if current_name.is_some() {
                    current_body.push_str(&text);
                }
            }
            Event::Code(code) => {
                if current_name.is_some() {
                    current_body.push('`');
                    current_body.push_str(&code);
                    current_body.push('`');
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                if current_name.is_some() {
                    current_body.push('\n');
                }
            }
            _ => {}
        }
    }

    if let Some(name) = current_name {
        sections.push(Section {
            name,
            body: current_body.trim().to_string(),
        });
    }

    sections
}

/// Extract the body of a section with the given heading name.
/// Uses pulldown_cmark event stream to handle formatted headings.
pub fn extract_section_body(content: &str, heading: &str) -> Option<String> {
    let parser = Parser::new_ext(content, Options::all());
    let mut in_target = false;
    let mut target_ended = false;
    let mut body = String::new();
    let normalized_heading = heading.trim().to_lowercase();

    for event in parser {
        match event {
            Event::Start(Tag::Heading {
                level: HeadingLevel::H2,
                ..
            }) => {
                if in_target {
                    target_ended = true;
                }
                in_target = false;
            }
            Event::End(TagEnd::Heading(HeadingLevel::H2)) => {
                // heading ended
            }
            Event::Text(text) => {
                if !in_target {
                    let text_trimmed = text.as_ref().trim();
                    if text_trimmed.to_lowercase() == normalized_heading {
                        in_target = true;
                    }
                } else if !target_ended {
                    body.push_str(&text);
                }
            }
            Event::Start(Tag::Heading { level, .. })
                if matches!(level, HeadingLevel::H1 | HeadingLevel::H3) =>
            {
                if in_target {
                    target_ended = true;
                    in_target = false;
                }
            }
            Event::Code(t) => {
                if in_target && !target_ended {
                    body.push('`');
                    body.push_str(&t);
                    body.push('`');
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                if in_target && !target_ended {
                    body.push('\n');
                }
            }
            _ => {}
        }
    }

    let trimmed = body.trim().to_string();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

/// Parse a single skill directory containing SKILL.md.
/// Returns Ok(Some(skill)) on success, Ok(None) if no SKILL.md, or Err(diagnostics) on parse errors.
pub fn parse_skill_dir(dir: &Path) -> Result<Option<Skill>, Vec<Diagnostic>> {
    let skill_md = dir.join("SKILL.md");
    if !skill_md.exists() {
        return Ok(None);
    }

    let content = std::fs::read_to_string(&skill_md).map_err(|e| {
        vec![Diagnostic::error(
            "skill-read-error",
            format!("Failed to read {}: {}", skill_md.display(), e),
            skill_md.clone(),
        )]
    })?;

    let (frontmatter, _) = parse_frontmatter(&content);

    let name = frontmatter.get("name").cloned().unwrap_or_default();
    if name.is_empty() {
        return Err(vec![Diagnostic::error(
            "skill-missing-name",
            "SKILL.md is missing required 'name' in frontmatter".to_string(),
            skill_md.clone(),
        )]);
    }

    let level = frontmatter.get("level").cloned().unwrap_or_default();
    let description = frontmatter.get("description").cloned().unwrap_or_default();
    let owner = frontmatter
        .get("owner")
        .map(|o| o.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();

    let sections = parse_sections(&content);

    let category = frontmatter
        .get("category")
        .cloned()
        .unwrap_or_else(|| {
            // Legacy: derive category from parent directory for nested structure
            dir.parent()
                .and_then(|p| p.file_name())
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default()
        });

    Ok(Some(Skill {
        name,
        level,
        owner,
        description,
        category,
        path: dir.to_path_buf(),
        skill_md,
        sections,
        states: Vec::new(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_frontmatter_basic() {
        let content = "---\nname: test-skill\ndescription: A test skill\n---\n\n# Body here";
        let (fm, body_start) = parse_frontmatter(content);
        assert_eq!(fm.get("name").unwrap(), "test-skill");
        assert_eq!(fm.get("description").unwrap(), "A test skill");
        assert!(body_start > 0);
    }

    #[test]
    fn test_parse_frontmatter_no_frontmatter() {
        let content = "# Just a heading\n\nSome body";
        let (fm, body_start) = parse_frontmatter(content);
        assert!(fm.is_empty());
        assert_eq!(body_start, 0);
    }

    #[test]
    fn test_parse_frontmatter_empty_frontmatter() {
        let content = "---\n---\n\n# Body";
        let (fm, _body_start) = parse_frontmatter(content);
        assert!(fm.is_empty());
    }

    #[test]
    fn test_parse_sections_single_h2() {
        let content = "# Title\n\n## Section One\n\nBody text here.\n\n## Section Two\n\nMore body.";
        let sections = parse_sections(content);
        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0].name, "Section One");
        assert!(sections[0].body.contains("Body text"));
        assert_eq!(sections[1].name, "Section Two");
        assert!(sections[1].body.contains("More body"));
    }

    #[test]
    fn test_parse_sections_no_sections() {
        let content = "# Just a title\n\nSome paragraph text.";
        let sections = parse_sections(content);
        assert!(sections.is_empty());
    }

    #[test]
    fn test_extract_section_body_basic() {
        let content = "## My Section\n\nThis is the body.\n\n## Other Section\n\nOther body.";
        let body = extract_section_body(content, "My Section");
        assert_eq!(body, Some("This is the body.".to_string()));
    }

    #[test]
    fn test_extract_section_body_not_found() {
        let content = "## My Section\n\nBody.\n\n## Other\n\nOther body.";
        let body = extract_section_body(content, "Nonexistent");
        assert_eq!(body, None);
    }

    #[test]
    fn test_extract_section_body_bold_formatted_heading() {
        let content = "## **Bold Title**\n\nBody under bold heading.\n\n## Next\n\nMore.";
        let body = extract_section_body(content, "**Bold Title**");
        assert_eq!(body, None); // pulldown_cmark Text event won't include ** markers
    }

    #[test]
    fn test_parse_skill_dir_missing_name() {
        let dir = tempfile::tempdir().unwrap();
        let skill_md = dir.path().join("SKILL.md");
        std::fs::write(
            &skill_md,
            "---\ndescription: No name here\n---\n\n# Body",
        )
        .unwrap();
        let result = parse_skill_dir(dir.path());
        assert!(result.is_err());
        let diags = result.unwrap_err();
        assert_eq!(diags[0].code, "skill-missing-name");
    }
}
