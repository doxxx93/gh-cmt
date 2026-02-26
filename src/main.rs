mod config;
mod git;
mod llm;

use std::io::{self, Write};
use std::process::Command;

use clap::{Parser, Subcommand};

use config::Config;

#[derive(Parser)]
#[command(name = "gh-cmt", about = "AI-powered commit message generator")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Commit message language
    #[arg(short, long)]
    language: Option<String>,

    /// GitHub Models model to use
    #[arg(short, long)]
    model: Option<String>,

    /// Number of previous commits to use as context
    #[arg(short, long)]
    examples: Option<u32>,

    /// Auto-commit without confirmation
    #[arg(short, long)]
    yes: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage configuration
    Config {
        /// Show current config and exit
        #[arg(short, long)]
        show: bool,

        /// Reset config to defaults
        #[arg(short, long)]
        reset: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match &cli.command {
        Some(Commands::Config { show, reset }) => {
            if *reset {
                config::reset_config()
            } else if *show {
                config::show_config()
            } else {
                config::run_config_interactive()
            }
        }
        None => {
            let cfg = Config::load();
            run_generate(&cli, &cfg)
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn run_generate(cli: &Cli, cfg: &Config) -> Result<(), String> {
    if !git::is_git_repository() {
        return Err("Not a git repository.".into());
    }

    let language = cli.language.as_deref()
        .or(cfg.language.as_deref())
        .unwrap_or("english");
    let model = cli.model.as_deref()
        .or(cfg.model.as_deref())
        .unwrap_or("openai/gpt-4o");
    let examples_count = cli.examples
        .or(cfg.examples)
        .unwrap_or(3);
    let auto_commit = cli.yes || cfg.auto_commit.unwrap_or(false);

    let changes = git::get_staged_changes()?;

    let examples = if examples_count > 0 {
        git::get_commit_messages(examples_count)?
    } else {
        String::new()
    };

    let mut message = generate(&changes, language, model, &examples)?;

    if auto_commit {
        git::commit(&message)?;
        return Ok(());
    }

    loop {
        eprint!("[c]ommit / [e]dit / [r]egenerate / [a]bort: ");
        io::stderr().flush().unwrap();

        let mut input = String::new();
        let bytes = io::stdin()
            .read_line(&mut input)
            .map_err(|e| format!("Failed to read input: {e}"))?;

        if bytes == 0 {
            eprintln!("\nAborted (no TTY).");
            return Ok(());
        }

        match input.trim().to_lowercase().as_str() {
            "c" | "commit" => {
                git::commit(&message)?;
                return Ok(());
            }
            "e" | "edit" => {
                let edited = edit_message(&message)?;
                git::commit(&edited)?;
                return Ok(());
            }
            "r" | "regenerate" => {
                message = generate(&changes, language, model, &examples)?;
            }
            "a" | "abort" => {
                eprintln!("Aborted.");
                return Ok(());
            }
            _ => {
                eprintln!("Invalid choice. Please enter c, e, r, or a.");
            }
        }
    }
}

fn generate(changes: &str, language: &str, model: &str, examples: &str) -> Result<String, String> {
    eprintln!("Generating commit message with {model}...");
    let message = llm::generate_commit_message(changes, language, model, examples)?;
    println!("\n{message}\n");
    Ok(message)
}

fn edit_message(message: &str) -> Result<String, String> {
    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| "vi".into());

    let tmp_dir = std::env::temp_dir();
    let tmp_path = tmp_dir.join("gh-cmt-message.txt");

    std::fs::write(&tmp_path, message)
        .map_err(|e| format!("Failed to write temp file: {e}"))?;

    let status = Command::new(&editor)
        .arg(&tmp_path)
        .status()
        .map_err(|e| format!("Failed to open editor '{editor}': {e}"))?;

    if !status.success() {
        return Err("Editor exited with error.".into());
    }

    let edited = std::fs::read_to_string(&tmp_path)
        .map_err(|e| format!("Failed to read edited message: {e}"))?;

    let _ = std::fs::remove_file(&tmp_path);

    let edited = edited.trim().to_string();
    if edited.is_empty() {
        return Err("Commit message is empty. Aborting.".into());
    }

    Ok(edited)
}
