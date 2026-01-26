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
pub fn create_session(session_name: &str, prompt: &str, cwd: Option<&str>) -> Result<()> {
    let claude_cmd = format!(
        "claude --dangerously-skip-permissions \"{}\"",
        prompt.replace('"', "\\\"")
    );

    let mut args = vec!["new-session", "-d", "-s", session_name];

    if let Some(dir) = cwd {
        args.push("-c");
        args.push(dir);
    }

    args.push(&claude_cmd);

    let status = Command::new("tmux")
        .args(&args)
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

/// Attach to an existing tmux session (replaces current process)
pub fn attach_session(session_name: &str) -> Result<()> {
    use std::os::unix::process::CommandExt;

    let err = Command::new("tmux")
        .args(["attach-session", "-t", session_name])
        .exec();

    // exec() only returns if it fails
    Err(anyhow::anyhow!("Failed to exec tmux: {}", err))
}

#[derive(Debug)]
pub struct Session {
    pub name: String,
    pub attached: bool,
}

/// Status of a Claude Code session based on the spinner character in the pane title
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionStatus {
    /// Session is actively working (spinner characters like ⠐⠒⠔⠕⠖⠗⠘⠙⠚⠛)
    InProgress,
    /// Session has completed (✳ character)
    Done,
    /// Status could not be determined
    Unknown,
}

impl std::fmt::Display for SessionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionStatus::InProgress => write!(f, "in-progress"),
            SessionStatus::Done => write!(f, "done"),
            SessionStatus::Unknown => write!(f, "unknown"),
        }
    }
}

/// Check if a character is a braille pattern dot (Unicode range U+2800-U+28FF)
/// These are used by Claude Code spinners to indicate in-progress status
fn is_braille_spinner(c: char) -> bool {
    let code = c as u32;
    // Braille Patterns block: U+2800 to U+28FF
    // Exclude U+2800 (blank braille pattern) as it's not a spinner
    (0x2801..=0x28FF).contains(&code)
}

/// Done indicator character
const DONE_CHAR: char = '✳';

/// Parse the session status from a pane title
pub fn parse_status_from_title(title: &str) -> SessionStatus {
    let trimmed = title.trim();
    if trimmed.is_empty() {
        return SessionStatus::Unknown;
    }

    // Check the first character of the title
    if let Some(first_char) = trimmed.chars().next() {
        if first_char == DONE_CHAR {
            return SessionStatus::Done;
        }
        if is_braille_spinner(first_char) {
            return SessionStatus::InProgress;
        }
    }

    SessionStatus::Unknown
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_braille_spinner() {
        // All braille patterns except blank should be spinners
        assert!(is_braille_spinner('⠐')); // U+2810
        assert!(is_braille_spinner('⠋')); // U+280B
        assert!(is_braille_spinner('⠂')); // U+2802
        assert!(is_braille_spinner('⣿')); // U+28FF (max braille)
        assert!(is_braille_spinner('⠁')); // U+2801 (min non-blank braille)

        // Blank braille and non-braille should not be spinners
        assert!(!is_braille_spinner('⠀')); // U+2800 (blank braille)
        assert!(!is_braille_spinner('A'));
        assert!(!is_braille_spinner('✳'));
    }

    #[test]
    fn test_parse_status_done() {
        assert_eq!(
            parse_status_from_title("✳ Stack Issue 1"),
            SessionStatus::Done
        );
        assert_eq!(parse_status_from_title("✳"), SessionStatus::Done);
        assert_eq!(
            parse_status_from_title("  ✳ with spaces"),
            SessionStatus::Done
        );
    }

    #[test]
    fn test_parse_status_in_progress() {
        assert_eq!(
            parse_status_from_title("⠐ Stack Issue 1"),
            SessionStatus::InProgress
        );
        assert_eq!(
            parse_status_from_title("⠋ Spinning"),
            SessionStatus::InProgress
        );
        assert_eq!(
            parse_status_from_title("⠹ Another spinner"),
            SessionStatus::InProgress
        );
        // Test the character that was missing
        assert_eq!(
            parse_status_from_title("⠂ Status Indicators"),
            SessionStatus::InProgress
        );
    }

    #[test]
    fn test_parse_status_unknown() {
        assert_eq!(parse_status_from_title(""), SessionStatus::Unknown);
        assert_eq!(parse_status_from_title("   "), SessionStatus::Unknown);
        assert_eq!(
            parse_status_from_title("No spinner here"),
            SessionStatus::Unknown
        );
    }

    #[test]
    fn test_session_status_display() {
        assert_eq!(format!("{}", SessionStatus::InProgress), "in-progress");
        assert_eq!(format!("{}", SessionStatus::Done), "done");
        assert_eq!(format!("{}", SessionStatus::Unknown), "unknown");
    }
}
