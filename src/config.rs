use std::io::{self, Write};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default)]
pub struct Config {
    pub language: Option<String>,
    pub model: Option<String>,
    pub examples: Option<u32>,
    pub auto_commit: Option<bool>,
}

impl Config {
    pub fn load() -> Self {
        let path = config_path();
        let Ok(content) = std::fs::read_to_string(&path) else {
            return Self::default();
        };
        serde_yaml::from_str(&content).unwrap_or_default()
    }

    pub fn save(&self) -> Result<(), String> {
        let dir = config_dir();
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create config directory: {e}"))?;
        let content = serde_yaml::to_string(self)
            .map_err(|e| format!("Failed to serialize config: {e}"))?;
        std::fs::write(config_path(), content)
            .map_err(|e| format!("Failed to write config: {e}"))?;
        Ok(())
    }
}

pub fn run_config_interactive() -> Result<(), String> {
    let current = Config::load();

    println!("gh-cmt config (press Enter to keep current value)\n");

    let language = prompt_input(
        "Language",
        current.language.as_deref().unwrap_or("english"),
    )?;
    let model = prompt_input(
        "Model",
        current.model.as_deref().unwrap_or("openai/gpt-4o"),
    )?;
    let examples = prompt_input(
        "Examples (number of previous commits)",
        &current.examples.unwrap_or(3).to_string(),
    )?;
    let auto_commit = prompt_input(
        "Auto commit (true/false)",
        &current.auto_commit.unwrap_or(false).to_string(),
    )?;

    let config = Config {
        language: Some(language),
        model: Some(model),
        examples: Some(
            examples
                .parse()
                .map_err(|_| "Invalid number for examples")?,
        ),
        auto_commit: Some(auto_commit == "true"),
    };

    config.save()?;
    println!("\nConfig saved to {}", config_path().display());
    Ok(())
}

fn prompt_input(label: &str, default: &str) -> Result<String, String> {
    eprint!("  {label} [{default}]: ");
    io::stderr().flush().unwrap();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| format!("Failed to read input: {e}"))?;

    let input = input.trim();
    if input.is_empty() {
        Ok(default.to_string())
    } else {
        Ok(input.to_string())
    }
}

pub fn show_config() -> Result<(), String> {
    let path = config_path();
    if !path.exists() {
        println!("No config file found. Run `gh cmt config` to create one.");
        return Ok(());
    }

    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read config: {e}"))?;

    println!("# {}\n", path.display());
    print!("{content}");
    Ok(())
}

pub fn reset_config() -> Result<(), String> {
    let path = config_path();
    if path.exists() {
        std::fs::remove_file(&path)
            .map_err(|e| format!("Failed to remove config: {e}"))?;
        println!("Config reset. Removed {}", path.display());
    } else {
        println!("No config file to reset.");
    }
    Ok(())
}

fn config_path() -> PathBuf {
    config_dir().join("config.yml")
}

fn config_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".config").join("gh-cmt")
}
