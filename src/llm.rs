use std::process::Command;

use serde::{Deserialize, Serialize};

const PROMPT_YAML: &str = include_str!("prompt.yml");
const API_ENDPOINT: &str = "https://models.github.ai/inference/chat/completions";

#[derive(Deserialize)]
struct PromptConfig {
    messages: Vec<PromptMessage>,
    model: ModelConfig,
}

#[derive(Deserialize)]
struct ModelConfig {
    temperature: f64,
    top_p: f64,
}

#[derive(Deserialize)]
struct PromptMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f64,
    top_p: f64,
    stream: bool,
}

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: String,
}

pub fn get_github_token() -> Result<String, String> {
    // Try environment variables first
    if let Ok(token) = std::env::var("GH_TOKEN") {
        if !token.is_empty() {
            return Ok(token);
        }
    }
    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        if !token.is_empty() {
            return Ok(token);
        }
    }

    // Fall back to gh auth token
    let output = Command::new("gh")
        .args(["auth", "token"])
        .output()
        .map_err(|e| format!("Failed to run `gh auth token`: {e}. Is gh CLI installed?"))?;

    if !output.status.success() {
        return Err("Failed to get GitHub token. Run `gh auth login` first.".into());
    }

    let token = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if token.is_empty() {
        return Err("GitHub token is empty. Run `gh auth login` first.".into());
    }

    Ok(token)
}

pub fn generate_commit_message(
    changes: &str,
    language: &str,
    model: &str,
    examples: &str,
) -> Result<String, String> {
    let config: PromptConfig =
        serde_yaml::from_str(PROMPT_YAML).map_err(|e| format!("Failed to parse prompt.yml: {e}"))?;

    let token = get_github_token()?;

    let examples_section = if examples.is_empty() {
        String::new()
    } else {
        format!(
            "Here are recent commit messages for style reference:\n\n{examples}"
        )
    };

    let messages: Vec<ChatMessage> = config
        .messages
        .iter()
        .map(|m| {
            let content = m
                .content
                .replace("{{changes}}", changes)
                .replace("{{language}}", language)
                .replace("{{examples}}", &examples_section);
            ChatMessage {
                role: m.role.clone(),
                content,
            }
        })
        .collect();

    let request = ChatRequest {
        model: model.to_string(),
        messages,
        temperature: config.model.temperature,
        top_p: config.model.top_p,
        stream: false,
    };

    let client = reqwest::blocking::Client::new();
    let response = client
        .post(API_ENDPOINT)
        .header("Authorization", format!("Bearer {token}"))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .map_err(|e| format!("API request failed: {e}"))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().unwrap_or_default();
        return Err(format!("API returned {status}: {body}"));
    }

    let chat_response: ChatResponse = response
        .json()
        .map_err(|e| format!("Failed to parse API response: {e}"))?;

    chat_response
        .choices
        .first()
        .map(|c| strip_code_block(c.message.content.trim()))
        .ok_or_else(|| "No response from model".into())
}

fn strip_code_block(s: &str) -> String {
    let s = s.strip_prefix("```").unwrap_or(s);
    let s = s.strip_suffix("```").unwrap_or(s);
    s.trim().to_string()
}
