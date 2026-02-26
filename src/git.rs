use std::process::Command;

pub fn is_git_repository() -> bool {
    Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn get_staged_changes() -> Result<String, String> {
    let output = Command::new("git")
        .args(["diff", "--staged"])
        .output()
        .map_err(|e| format!("Failed to run git diff: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git diff --staged failed: {stderr}"));
    }

    let diff = String::from_utf8_lossy(&output.stdout).to_string();
    if diff.trim().is_empty() {
        return Err("No staged changes found. Stage your changes with `git add` first.".into());
    }

    Ok(diff)
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
