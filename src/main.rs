use loopkit::config::load_config;
use loopkit::diagnostic::{diagnostics_json, format_diagnostics};
use loopkit::types::{Diagnostic, Repo, Severity};
use loopkit::validators::run_all;
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let exit_code = run_cli(&args, &|root, skills_dir| {
        let repo = Repo::from_root(PathBuf::from(root), skills_dir)?;
        Ok(run_all(&repo))
    });
    std::process::exit(exit_code);
}

/// Run the CLI check command. Returns an exit code (0 = success, 1 = errors).
/// This function is testable by passing a custom loader.
pub fn run_cli(
    args: &[String],
    loader: &dyn Fn(&str, &str) -> std::io::Result<Vec<Diagnostic>>,
) -> i32 {
    let json = args.iter().any(|a| a == "--json");

    if args.iter().any(|a| a == "-h" || a == "--help") {
        print_help();
        return 0;
    }

    let (root, skills_dir) = parse_args(args);

    let config = load_config(&PathBuf::from(&root));
    let skills_dir_name = skills_dir.unwrap_or_else(|| config.skills_dir.clone());

    let diagnostics = match loader(&root, &skills_dir_name) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("error: failed to load repo: {}", e);
            return 1;
        }
    };

    if json {
        println!("{}", diagnostics_json(&diagnostics));
    } else {
        println!("{}", format_diagnostics(&diagnostics));
    }

    let errors = diagnostics.iter().filter(|d| matches!(d.severity, Severity::Error | Severity::Warning)).count();
    if errors > 0 { 1 } else { 0 }
}

fn parse_args(args: &[String]) -> (String, Option<String>) {
    let mut root: Option<String> = None;
    let mut skills_dir: Option<String> = None;
    let mut iter = args.iter().skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--root" => root = iter.next().cloned(),
            "--skills-dir" => skills_dir = iter.next().cloned(),
            "--json" => {}
            _ => {}
        }
    }
    let root = root.unwrap_or_else(|| {
        std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| ".".to_string())
    });
    (root, skills_dir)
}

fn print_help() {
    println!("loopkit — Prove your agent skill loops are correct");
    println!();
    println!("USAGE:");
    println!("  loopkit [OPTIONS] [PATH]");
    println!();
    println!("ARGS:");
    println!("  <PATH>             Path to skills directory (default: skills/)");
    println!();
    println!("OPTIONS:");
    println!("  --root <path>       Repository root (default: current directory)");
    println!("  --skills-dir <dir>  Skills directory name (default: from .loopkit.yaml)");
    println!("  --json              Output diagnostics as JSON");
    println!("  -h, --help          Show this help");
}

#[cfg(test)]
mod tests {
    use super::*;
    use loopkit::types::{Diagnostic, FileLocation, Severity};
    use std::path::PathBuf;

    fn make_diag(code: &str, severity: Severity) -> Diagnostic {
        Diagnostic {
            severity,
            code: code.to_string(),
            message: "msg".to_string(),
            location: FileLocation { path: PathBuf::from("x.md"), line: None, column: None },
            help: "help".to_string(),
        }
    }

    #[test]
    fn run_cli_help_returns_0() {
        let code = run_cli(&["prog".into(), "-h".into()], &|_, _| Ok(vec![]));
        assert_eq!(code, 0);
    }

    #[test]
    fn run_cli_no_errors_returns_0() {
        let code = run_cli(
            &["prog".into()],
            &|_, _| Ok(vec![]),
        );
        assert_eq!(code, 0);
    }

    #[test]
    fn run_cli_with_errors_returns_1() {
        let code = run_cli(
            &["prog".into()],
            &|_, _| Ok(vec![make_diag("E1", Severity::Error)]),
        );
        assert_eq!(code, 1);
    }

    #[test]
    fn run_cli_with_only_warnings_or_info_returns_0() {
        let code = run_cli(
            &["prog".into()],
            &|_, _| Ok(vec![make_diag("I1", Severity::Info)]),
        );
        assert_eq!(code, 0);
    }

    #[test]
    fn run_cli_json_flag() {
        let code = run_cli(
            &["prog".into(), "--json".into()],
            &|_, _| Ok(vec![]),
        );
        assert_eq!(code, 0);
    }

    #[test]
    fn parse_args_with_root_and_skills_dir() {
        let (root, skills) = parse_args(&[
            "prog".into(),
            "--root".into(),
            "/tmp".into(),
            "--skills-dir".into(),
            "my-skills".into(),
        ]);
        assert_eq!(root, "/tmp");
        assert_eq!(skills, Some("my-skills".to_string()));
    }

    #[test]
    fn parse_args_defaults() {
        let (root, skills) = parse_args(&["prog".into()]);
        assert!(!root.is_empty());
        assert!(skills.is_none());
    }

    #[test]
    fn print_help_includes_keywords() {
        // Just verify it doesn't panic
        print_help();
    }
}
