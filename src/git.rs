use std::process::Command;

pub fn is_git_repository() -> bool {
    Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Approximate max characters to send (~6000 tokens worth, leaving room for prompt)
const MAX_DIFF_CHARS: usize = 20000;

pub fn get_staged_changes() -> Result<String, String> {
    let stat = Command::new("git")
        .args(["diff", "--staged", "--stat"])
        .output()
        .map_err(|e| format!("Failed to run git diff: {e}"))?;

    let diff = Command::new("git")
        .args(["diff", "--staged"])
        .output()
        .map_err(|e| format!("Failed to run git diff: {e}"))?;

    if !diff.status.success() {
        let stderr = String::from_utf8_lossy(&diff.stderr);
        return Err(format!("git diff --staged failed: {stderr}"));
    }

    let diff_text = String::from_utf8_lossy(&diff.stdout).to_string();
    if diff_text.trim().is_empty() {
        return Err("No staged changes found. Stage your changes with `git add` first.".into());
    }

    let stat_text = String::from_utf8_lossy(&stat.stdout).to_string();

    if diff_text.len() <= MAX_DIFF_CHARS {
        return Ok(diff_text);
    }

    // Diff too large: send stat summary + truncated diff
    let truncated: String = diff_text.chars().take(MAX_DIFF_CHARS).collect();
    Ok(format!(
        "# File change summary:\n{stat_text}\n# Diff (truncated):\n{truncated}\n\n[... diff truncated due to size ...]"
    ))
}

pub fn get_commit_messages(count: u32) -> Result<String, String> {
    let output = Command::new("git")
        .args([
            "log",
            &format!("-n {count}"),
            "--format=%s",
        ])
        .output()
        .map_err(|e| format!("Failed to run git log: {e}"))?;

    if !output.status.success() {
        return Ok(String::new());
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn commit(message: &str) -> Result<(), String> {
    let output = Command::new("git")
        .args(["commit", "-m", message])
        .output()
        .map_err(|e| format!("Failed to run git commit: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git commit failed: {stderr}"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("{stdout}");
    Ok(())
}
