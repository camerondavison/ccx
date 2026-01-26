mod tmux;

use anyhow::Result;
use clap::{Parser, Subcommand};

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
    },
    /// List all sessions
    List,
    /// Stop a specific session
    Stop {
        /// The session name to stop
        session: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start { prompt, cwd } => cmd_start(&prompt, cwd.as_deref()),
        Commands::Status { session } => cmd_status(session.as_deref()),
        Commands::List => cmd_list(),
        Commands::Stop { session } => cmd_stop(&session),
    }
}

fn cmd_start(prompt: &str, cwd: Option<&str>) -> Result<()> {
    let session_name = tmux::generate_session_name();
    tmux::create_session(&session_name, prompt, cwd)?;
    println!("Started session: {}", session_name);
    println!("Attach with: tmux attach -t {}", session_name);
    Ok(())
}

fn cmd_status(session: Option<&str>) -> Result<()> {
    match session {
        Some(name) => {
            // Show detailed output for a specific session
            if !tmux::session_exists(name) {
                anyhow::bail!("Session '{}' does not exist", name);
            }
            match tmux::capture_pane(name, 10) {
                Ok(content) => {
                    // Take last 10 non-empty lines
                    let lines: Vec<&str> =
                        content.lines().filter(|l| !l.trim().is_empty()).collect();
                    let last_10: Vec<&str> = lines.iter().rev().take(10).rev().cloned().collect();
                    for line in last_10 {
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
                let title_display = if title.is_empty() {
                    String::new()
                } else {
                    format!(" [{}]", title)
                };
                println!("{}{}", session.name, title_display);
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
