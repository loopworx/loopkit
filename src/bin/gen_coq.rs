//! `gen_coq` — CLI wrapper around the Coq generator logic.
//!
//! Auto-discovers states, transitions, entry points, and terminal states
//! from the skill files. Emits `Generated.v` and an OCaml checker.

use std::path::PathBuf;
use std::process::ExitCode;

use skill_loop_verifier::config::load_config;
use skill_loop_verifier::generator::{emit_checker_ml, emit_generated};
use skill_loop_verifier::types::Repo;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "-h" || a == "--help") {
        print_help();
        return ExitCode::SUCCESS;
    }

    let (root, skills_dir, check, output_dir) = parse_args(&args);

    let config = load_config(&root);
    let skills_dir_name = skills_dir.unwrap_or_else(|| config.skills_dir.clone());

    let repo = match Repo::from_root(root.clone(), &skills_dir_name) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("gen_coq: failed to load repo: {e}");
            return ExitCode::FAILURE;
        }
    };

    let generated = emit_generated(&repo);
    let checker = emit_checker_ml(&repo);

    let proof_dir = output_dir.unwrap_or_else(|| root.join("proofs"));
    let out_generated = proof_dir.join("Generated.v");
    let out_checker = proof_dir.join("extraction/checker.ml");

    if check {
        match std::fs::read_to_string(&out_generated) {
            Ok(existing) if existing == generated => {}
            Ok(_) => {
                eprintln!("gen_coq: Generated.v is STALE — regenerate with `gen-coq`");
                return ExitCode::FAILURE;
            }
            Err(_) => {
                eprintln!("gen_coq: Generated.v is MISSING — generate with `gen-coq`");
                return ExitCode::FAILURE;
            }
        }
        match std::fs::read_to_string(&out_checker) {
            Ok(existing) if existing == checker => {}
            Ok(_) => {
                eprintln!("gen_coq: checker.ml is STALE — regenerate with `gen-coq`");
                return ExitCode::FAILURE;
            }
            Err(_) => {
                eprintln!("gen_coq: checker.ml is MISSING — generate with `gen-coq`");
                return ExitCode::FAILURE;
            }
        }
        println!("gen_coq: Generated.v and checker.ml are up to date.");
        return ExitCode::SUCCESS;
    }

    // Create directories
    let _ = std::fs::create_dir_all(&proof_dir);
    let _ = std::fs::create_dir_all(proof_dir.join("extraction"));

    if let Err(e) = std::fs::write(&out_generated, &generated) {
        eprintln!("gen_coq: failed to write {}: {e}", out_generated.display());
        return ExitCode::FAILURE;
    }
    println!("gen_coq: wrote {}", out_generated.display());
    if let Err(e) = std::fs::write(&out_checker, &checker) {
        eprintln!("gen_coq: failed to write {}: {e}", out_checker.display());
        return ExitCode::FAILURE;
    }
    println!("gen_coq: wrote {}", out_checker.display());
    ExitCode::SUCCESS
}

fn parse_args(args: &[String]) -> (PathBuf, Option<String>, bool, Option<PathBuf>) {
    let mut root: Option<PathBuf> = None;
    let mut skills_dir: Option<String> = None;
    let mut check = false;
    let mut output_dir: Option<PathBuf> = None;
    let mut iter = args.iter().skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--root" => root = iter.next().map(PathBuf::from),
            "--skills-dir" => skills_dir = iter.next().cloned(),
            "--check" => check = true,
            "--output-dir" => output_dir = iter.next().map(PathBuf::from),
            _ => {}
        }
    }
    let root = root.unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    (root, skills_dir, check, output_dir)
}

fn print_help() {
    eprintln!("Usage: gen_coq [--root <path>] [--skills-dir <dir>] [--output-dir <dir>] [--check]");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn parse_args_no_flags() {
        let (root, skills_dir, check, output_dir) = parse_args(&["gen_coq".into()]);
        assert!(root.to_string_lossy().len() > 0);
        assert_eq!(skills_dir, None);
        assert!(!check);
        assert_eq!(output_dir, None);
    }

    #[test]
    fn parse_args_all_flags() {
        let args: Vec<String> = [
            "gen_coq", "--root", "/tmp", "--skills-dir", "my-skills",
            "--check", "--output-dir", "/out",
        ].iter().map(|s| s.to_string()).collect();
        let (root, skills_dir, check, output_dir) = parse_args(&args);
        assert_eq!(root, PathBuf::from("/tmp"));
        assert_eq!(skills_dir, Some("my-skills".to_string()));
        assert!(check);
        assert_eq!(output_dir, Some(PathBuf::from("/out")));
    }

    #[test]
    fn parse_args_unknown_flags_ignored() {
        let (root, _, check, _) = parse_args(&["gen_coq".into(), "--unknown".into(), "val".into()]);
        assert!(!check);
        assert!(root.to_string_lossy().len() > 0);
    }
}
