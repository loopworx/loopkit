#![allow(dead_code)]

use std::path::PathBuf;
use tempfile::TempDir;

/// Create a SKILL.md for a skill in a temp directory.
pub fn write_skill(dir: &TempDir, category: &str, name: &str, content: &str) -> PathBuf {
    let skill_dir = dir.path().join("skills").join(category).join(name);
    std::fs::create_dir_all(&skill_dir).unwrap();
    let path = skill_dir.join("SKILL.md");
    std::fs::write(&path, content).unwrap();
    path
}

/// Create a LOOP.md for a skill in a temp directory.
pub fn write_loop(dir: &TempDir, category: &str, name: &str, content: &str) -> PathBuf {
    let skill_dir = dir.path().join("skills").join(category).join(name);
    std::fs::create_dir_all(&skill_dir).unwrap();
    let path = skill_dir.join("LOOP.md");
    std::fs::write(&path, content).unwrap();
    path
}

/// Create a HANDOFFS.md for a skill in a temp directory.
pub fn write_handoffs(dir: &TempDir, category: &str, name: &str, content: &str) -> PathBuf {
    let skill_dir = dir.path().join("skills").join(category).join(name);
    std::fs::create_dir_all(&skill_dir).unwrap();
    let path = skill_dir.join("HANDOFFS.md");
    std::fs::write(&path, content).unwrap();
    path
}

/// Minimal valid SKILL.md content for a given name and level.
pub fn minimal_skill(name: &str, level: &str) -> String {
    format!(
        "---\nname: {name}\nlevel: {level}\nowner: dev-agent\n---\n\n\
         ## Description\nA skill.\n\n## Rules\n- Rule.\n\n## State Model\nStates.\n"
    )
}

/// Minimal valid LOOP.md with a transition rule and all 7 sections.
pub fn minimal_loop_with_transition(from: &str, to: &str, handoff_skill: &str) -> String {
    format!(
        "## Entry Conditions\nready\n\n\
         ## Loop State Schema\n| field | type |\n|-------|------|\n| s | str |\n\n\
         ## Single Iteration Step\n1. verify entry\n\n\
         ## Proof of Progress\n`test`\n\n\
         ## State Transition Rule\ntransition {from} → {to}\n  trigger t\n  handoff {handoff_skill} to agent\n\n\
         ## Halt Conditions\nhalt stall after 5 iterations\n\n\
         ## Handoff Target\nhandoff {handoff_skill} to agent\n"
    )
}

/// L1-RIGID SKILL.md with Entry Conditions and Halt Conditions.
pub fn minimal_l1_skill(name: &str) -> String {
    format!(
        "---\nname: {name}\nlevel: L1-RIGID\nowner: dev-agent\n---\n\n\
         ## Description\nA skill.\n\n## Rules\n- Rule.\n\n## State Model\nStates.\n\n\
         ## Entry Conditions\nReady.\n\n## Halt Conditions\nOn stall.\n"
    )
}

/// Create a Repo from a temp directory.
pub fn make_repo(dir: &TempDir) -> skill_loop_verifier::types::Repo {
    skill_loop_verifier::types::Repo::from_root(dir.path().to_path_buf(), "skills").unwrap()
}
