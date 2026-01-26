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
    },
    /// Show status of all running sessions with content preview
    Status,
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
        Commands::Start { prompt } => cmd_start(&prompt),
        Commands::Status => cmd_status(),
        Commands::List => cmd_list(),
        Commands::Stop { session } => cmd_stop(&session),
    }
}

fn cmd_start(prompt: &str) -> Result<()> {
    let session_name = tmux::generate_session_name();
    tmux::create_session(&session_name, prompt)?;
    println!("Started session: {}", session_name);
    println!("Attach with: tmux attach -t {}", session_name);
    Ok(())
}

fn cmd_status() -> Result<()> {
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
        println!("=== {}{} ===", session.name, title_display);
        println!(
            "  Attached: {}",
            if session.attached { "yes" } else { "no" }
        );

        match tmux::capture_pane(&session.name, 10) {
            Ok(content) => {
                let preview: String = content
                    .lines()
                    .filter(|l| !l.trim().is_empty())
                    .take(5)
                    .map(|l| format!("  | {}", l))
                    .collect::<Vec<_>>()
                    .join("\n");
                if !preview.is_empty() {
                    println!("  Recent output:");
                    println!("{}", preview);
                }
            }
            Err(e) => println!("  Could not capture output: {}", e),
        }
        println!();
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
