use skill_loop_verifier::config::load_config;
use skill_loop_verifier::diagnostic::{diagnostics_json, format_diagnostics};
use skill_loop_verifier::types::Repo;
use skill_loop_verifier::validators::run_all;
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let json = args.iter().any(|a| a == "--json");

    if args.iter().any(|a| a == "-h" || a == "--help") {
        print_help();
        return;
    }

    let (root, skills_dir) = parse_args(&args);

    let config = load_config(&root);
    let skills_dir_name = skills_dir.unwrap_or_else(|| config.skills_dir.clone());

    let repo = match Repo::from_root(root.clone(), &skills_dir_name) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: failed to load repo: {}", e);
            std::process::exit(1);
        }
    };

    let diagnostics = run_all(&repo);

    if json {
        println!("{}", diagnostics_json(&diagnostics));
    } else {
        println!("{}", format_diagnostics(&diagnostics));
    }

    let errors = diagnostics
        .iter()
        .filter(|d| d.severity == skill_loop_verifier::types::Severity::Error)
        .count();
    if errors > 0 {
        std::process::exit(1);
    }
}

fn parse_args(args: &[String]) -> (PathBuf, Option<String>) {
    let mut root: Option<PathBuf> = None;
    let mut skills_dir: Option<String> = None;
    let mut iter = args.iter().skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--root" => root = iter.next().map(PathBuf::from),
            "--skills-dir" => skills_dir = iter.next().cloned(),
            "--json" => {} // handled above
            _ => {}
        }
    }

    let root = root.unwrap_or_else(|| {
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    });

    (root, skills_dir)
}

fn print_help() {
    println!("skill-loop-verifier — Verify skill-based agent loops");
    println!();
    println!("USAGE:");
    println!("  skill-loop-verifier [OPTIONS]");
    println!("  skill-loop-verifier init [--root <path>]");
    println!("  skill-loop-verifier gen-coq [--root <path>]");
    println!();
    println!("OPTIONS:");
    println!("  --root <path>       Repository root (default: current directory)");
    println!("  --skills-dir <dir>  Skills directory name (default: from .loop-verifier.yaml)");
    println!("  --json              Output diagnostics as JSON");
    println!("  -h, --help          Show this help");
    println!();
    println!("COMMANDS:");
    println!("  check    Validate all skills, loop contracts, and the handoff graph (default)");
    println!("  init     Create .loop-verifier.yaml and copy Coq theories into the project");
    println!("  gen-coq  Generate Coq formalization from the current skill graph");
}
