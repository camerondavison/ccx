mod tmux;

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{Shell, generate};
use std::env;

#[derive(Parser)]
#[command(name = "ccx")]
#[command(about = "Manage Claude Code sessions in tmux")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start a new Claude Code session with the given prompt
    Start {
        /// The prompt to send to Claude
        prompt: String,
        /// Working directory for the Claude Code session
        #[arg(long)]
        cwd: Option<String>,
    },
    /// Show status of sessions (list all, or detail for a specific session)
    Status {
        /// Optional session name to show detailed output
        session: Option<String>,
        /// Number of lines to show (default: 10)
        #[arg(long, default_value = "10")]
        lines: i32,
    },
    /// List all sessions
    List,
    /// Stop a specific session
    Stop {
        /// The session name to stop
        session: String,
    },
    /// Attach to an existing session
    Attach {
        /// The session name to attach to
        session: String,
    },
    /// Generate shell completions
    Completions {
        /// The shell to generate completions for
        #[arg(value_enum, default_value = "bash")]
        shell: Shell,
    },
    /// Send a message to an existing session
    Send {
        /// The session name to send to
        session: String,
        /// The message to send
        message: String,
    },
    /// Watch a session until it completes
    Watch {
        /// The session name to watch
        session: String,
        /// Check interval in seconds (default: 2)
        #[arg(long, default_value = "2")]
        interval: u64,
    },
    /// Print the version
    Version,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start { prompt, cwd } => cmd_start(&prompt, cwd.as_deref()),
        Commands::Status { session, lines } => cmd_status(session.as_deref(), lines),
        Commands::List => cmd_list(),
        Commands::Stop { session } => cmd_stop(&session),
        Commands::Attach { session } => cmd_attach(&session),
        Commands::Completions { shell } => cmd_completions(shell),
        Commands::Send { session, message } => cmd_send(&session, &message),
        Commands::Watch { session, interval } => cmd_watch(&session, interval),
        Commands::Version => cmd_version(),
    }
}

/// Shorten a path for display by replacing $HOME with ~
fn shorten_path(path: &str) -> String {
    if let Some(home) = env::var_os("HOME") {
        let home_str = home.to_string_lossy();
        if path.starts_with(home_str.as_ref()) {
            return format!("~{}", &path[home_str.len()..]);
        }
    }
    path.to_string()
}

fn cmd_start(prompt: &str, cwd: Option<&str>) -> Result<()> {
    let session_name = tmux::generate_session_name();
    tmux::create_session(&session_name, prompt, cwd)?;
    println!("Started session: {}", session_name);
    println!("Attach with: ccx attach {}", session_name);
    Ok(())
}

fn cmd_status(session: Option<&str>, num_lines: i32) -> Result<()> {
    match session {
        Some(name) => {
            // Show detailed output for a specific session
            if !tmux::session_exists(name) {
                anyhow::bail!("Session '{}' does not exist", name);
            }
            match tmux::capture_pane(name, num_lines) {
                Ok(content) => {
                    // Take last N non-empty lines
                    let lines: Vec<&str> =
                        content.lines().filter(|l| !l.trim().is_empty()).collect();
                    let last_n: Vec<&str> = lines
                        .iter()
                        .rev()
                        .take(num_lines as usize)
                        .rev()
                        .cloned()
                        .collect();
                    for line in last_n {
                        println!("{}", line);
                    }
                }
                Err(e) => println!("Could not capture output: {}", e),
            }
        }
        None => {
            // List all sessions with just name and title
            let sessions = tmux::list_sessions()?;

            if sessions.is_empty() {
                println!("No active ccx sessions");
                return Ok(());
            }

            for session in sessions {
                let title = tmux::get_pane_title(&session.name).unwrap_or_default();
                let status = tmux::parse_status_from_title(&title);
                let status_display = match status {
                    tmux::SessionStatus::Unknown => String::new(),
                    _ => format!(" *{}*", status),
                };
                let title_display = if title.is_empty() {
                    String::new()
                } else {
                    format!(" [{}]", title)
                };
                let cwd_display = tmux::get_pane_cwd(&session.name)
                    .map(|p| format!(" {}", shorten_path(&p)))
                    .unwrap_or_default();
                println!(
                    "{}{}{}{}",
                    session.name, status_display, title_display, cwd_display
                );
            }
        }
    }

    Ok(())
}

fn cmd_list() -> Result<()> {
    let sessions = tmux::list_sessions()?;

    if sessions.is_empty() {
        println!("No active ccx sessions");
        return Ok(());
    }

    println!("{:<20} {:<10}", "SESSION", "ATTACHED");
    println!("{:-<20} {:-<10}", "", "");
    for session in sessions {
        println!(
            "{:<20} {:<10}",
            session.name,
            if session.attached { "yes" } else { "no" }
        );
    }

    Ok(())
}

fn cmd_stop(session: &str) -> Result<()> {
    if !tmux::session_exists(session) {
        anyhow::bail!("Session '{}' does not exist", session);
    }

    tmux::kill_session(session)?;
    println!("Stopped session: {}", session);
    Ok(())
}

fn cmd_attach(session: &str) -> Result<()> {
    if !tmux::session_exists(session) {
        anyhow::bail!("Session '{}' does not exist", session);
    }

    tmux::attach_session(session)
}

fn cmd_completions(shell: Shell) -> Result<()> {
    let mut cmd = Cli::command();
    generate(shell, &mut cmd, "ccx", &mut std::io::stdout());
    Ok(())
}

fn cmd_send(session: &str, message: &str) -> Result<()> {
    if !tmux::session_exists(session) {
        anyhow::bail!("Session '{}' does not exist", session);
    }

    tmux::send_keys(session, message)?;
    println!("Sent message to session: {}", session);
    Ok(())
}

fn cmd_watch(session: &str, interval: u64) -> Result<()> {
    use std::io::{self, Write};
    use std::thread;
    use std::time::Duration;

    if !tmux::session_exists(session) {
        anyhow::bail!("Session '{}' does not exist", session);
    }

    println!("Watching session: {} (Ctrl+C to stop)", session);
    println!();

    loop {
        // Check if session still exists
        if !tmux::session_exists(session) {
            println!("\nSession '{}' no longer exists", session);
            break;
        }

        // Get current status
        let title = tmux::get_pane_title(session).unwrap_or_default();
        let status = tmux::parse_status_from_title(&title);

        // Clear screen and show status
        print!("\x1B[2J\x1B[1;1H");
        io::stdout().flush()?;

        println!("Session: {}", session);
        println!("Status: {}", status);
        println!();

        // Show recent output
        if let Ok(content) = tmux::capture_pane(session, 20) {
            let lines: Vec<&str> = content.lines().filter(|l| !l.trim().is_empty()).collect();
            let last_n: Vec<&str> = lines.iter().rev().take(15).rev().cloned().collect();
            for line in last_n {
                println!("{}", line);
            }
        }

        // Check if done
        if status == tmux::SessionStatus::Done {
            println!("\nSession completed.");
            break;
        }

        thread::sleep(Duration::from_secs(interval));
    }

    Ok(())
}

fn cmd_version() -> Result<()> {
    println!("ccx {}", env!("CARGO_PKG_VERSION"));
    Ok(())
}
