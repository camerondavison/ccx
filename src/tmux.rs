use anyhow::{Context, Result};
use std::process::Command;

const SESSION_PREFIX: &str = "ccx-";

/// Generate a unique session name with the ccx- prefix
pub fn generate_session_name() -> String {
    let id: u32 = rand_id();
    format!("{}{:08x}", SESSION_PREFIX, id)
}

/// Simple random ID generator using process ID and timestamp
fn rand_id() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u32;
    let pid = std::process::id();
    time.wrapping_add(pid)
}

/// Create a new detached tmux session running claude with the given prompt
pub fn create_session(session_name: &str, prompt: &str) -> Result<()> {
    let claude_cmd = format!("claude \"{}\"", prompt.replace('"', "\\\""));

    let status = Command::new("tmux")
        .args(["new-session", "-d", "-s", session_name, &claude_cmd])
        .status()
        .context("Failed to execute tmux")?;

    if !status.success() {
        anyhow::bail!("Failed to create tmux session");
    }

    // Enable title updates so Claude Code can set pane title with status icon
    let _ = Command::new("tmux")
        .args(["set-option", "-t", session_name, "allow-rename", "on"])
        .status();

    Ok(())
}

/// List all ccx sessions
pub fn list_sessions() -> Result<Vec<Session>> {
    let output = Command::new("tmux")
        .args(["list-sessions", "-F", "#{session_name}:#{session_attached}"])
        .output()
        .context("Failed to execute tmux")?;

    if !output.status.success() {
        // No sessions exist
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let sessions: Vec<Session> = stdout
        .lines()
        .filter(|line| line.starts_with(SESSION_PREFIX))
        .filter_map(|line| {
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() >= 2 {
                Some(Session {
                    name: parts[0].to_string(),
                    attached: parts[1] == "1",
                })
            } else {
                None
            }
        })
        .collect();

    Ok(sessions)
}

/// Get the pane title for a session (contains Claude Code status icon)
pub fn get_pane_title(session_name: &str) -> Result<String> {
    let output = Command::new("tmux")
        .args(["display-message", "-t", session_name, "-p", "#{pane_title}"])
        .output()
        .context("Failed to get pane title")?;

    if !output.status.success() {
        anyhow::bail!("Failed to get pane title for session {}", session_name);
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Capture recent content from a session's pane
pub fn capture_pane(session_name: &str, lines: i32) -> Result<String> {
    let output = Command::new("tmux")
        .args([
            "capture-pane",
            "-t",
            session_name,
            "-p",
            "-S",
            &format!("-{}", lines),
        ])
        .output()
        .context("Failed to capture pane")?;

    if !output.status.success() {
        anyhow::bail!("Failed to capture pane for session {}", session_name);
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Kill a tmux session by name
pub fn kill_session(session_name: &str) -> Result<()> {
    let status = Command::new("tmux")
        .args(["kill-session", "-t", session_name])
        .status()
        .context("Failed to execute tmux")?;

    if !status.success() {
        anyhow::bail!("Failed to kill session {}", session_name);
    }

    Ok(())
}

/// Check if a session exists
pub fn session_exists(session_name: &str) -> bool {
    Command::new("tmux")
        .args(["has-session", "-t", session_name])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[derive(Debug)]
pub struct Session {
    pub name: String,
    pub attached: bool,
}
