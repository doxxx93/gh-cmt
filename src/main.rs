mod git;
mod llm;

use std::io::{self, Write};
use std::process::Command;

use clap::Parser;

#[derive(Parser)]
#[command(name = "gh-cmt", about = "AI-powered commit message generator")]
struct Cli {
    /// Commit message language
    #[arg(short, long, default_value = "english")]
    language: String,

    /// GitHub Models model to use
    #[arg(short, long, default_value = "openai/gpt-4o")]
    model: String,

    /// Number of previous commits to use as context
    #[arg(short, long, default_value_t = 3)]
    examples: u32,

    /// Auto-commit without confirmation
    #[arg(short, long)]
    yes: bool,
}

fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(&cli) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn run(cli: &Cli) -> Result<(), String> {
    if !git::is_git_repository() {
        return Err("Not a git repository.".into());
    }

    let changes = git::get_staged_changes()?;

    let examples = if cli.examples > 0 {
        git::get_commit_messages(cli.examples)?
    } else {
        String::new()
    };

    eprintln!("Generating commit message with {}...", cli.model);

    let message = llm::generate_commit_message(&changes, &cli.language, &cli.model, &examples)?;

    println!("\n{message}\n");

    if cli.yes {
        git::commit(&message)?;
        return Ok(());
    }

    loop {
        eprint!("[c]ommit / [e]dit / [a]bort: ");
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
            "a" | "abort" => {
                eprintln!("Aborted.");
                return Ok(());
            }
            _ => {
                eprintln!("Invalid choice. Please enter c, e, or a.");
            }
        }
    }
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
