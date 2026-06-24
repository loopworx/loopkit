use clap::Parser;
use loopkit_core::config::load_config;
use loopkit_core::diagnostic::{diagnostics_json, format_diagnostics, format_summary};
use loopkit_core::discovery::discover_skills;
use loopkit_core::types::Severity;
use loopkit_graph::validators;
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(name = "loopkit", about = "Prove your agent skill loops are correct")]
struct Cli {
    /// Path to skills directory
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Output JSON instead of text
    #[arg(long)]
    json: bool,
}

fn main() {
    let cli = Cli::parse();

    let config = load_config(&cli.path);
    let skills_dir = cli.path.join(&config.skills_dir);

    let (skills, discovery_diags) = discover_skills(&skills_dir);
    let mut diagnostics = discovery_diags;

    diagnostics.extend(validators::run_all(&config, &skills));
    diagnostics.extend(loopkit::best_practices::check_all(&skills));

    let error_count = diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .count();

    if cli.json {
        println!("{}", diagnostics_json(&diagnostics, skills.len()));
    } else {
        println!("{}", format_diagnostics(&diagnostics));
        println!("{}", format_summary(&diagnostics, skills.len()));
    }

    if error_count > 0 {
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }

    #[test]
    fn cli_default_path() {
        let cli = Cli::parse_from(["loopkit"]);
        assert_eq!(cli.path, PathBuf::from("."));
        assert!(!cli.json);
    }

    #[test]
    fn cli_json_flag() {
        let cli = Cli::parse_from(["loopkit", "--json"]);
        assert!(cli.json);
        assert_eq!(cli.path, PathBuf::from("."));
    }

    #[test]
    fn cli_custom_path() {
        let cli = Cli::parse_from(["loopkit", "/custom/path"]);
        assert_eq!(cli.path, PathBuf::from("/custom/path"));
        assert!(!cli.json);
    }
}
