---
name: ccx
description: >
  CLI tool for running and managing multiple background Claude Code sessions.
  Use when you need to:
    (1) Start background Claude Code sessions with prompts
    (2) Monitor status and progress of running sessions
    (3) List all active sessions
    (4) Stop or attach to existing sessions
    (5) View session logs for debugging failed or past sessions.
allowed-tools: Bash(ccx *)
---

# ccx (Claude Code eXecutor)

CLI tool for running and managing multiple background Claude Code sessions.

## Commands

### Start a Session

```bash
# Start a new Claude Code session with a prompt
ccx start "fix the login bug"

# Start a session in a specific directory
ccx start "implement new feature" --cwd /path/to/project
```

### Check Status

```bash
# List all sessions with their titles
ccx status

# Show detailed output for a specific session (last 10 lines)
ccx status <session-name>
```

### List Sessions

```bash
# List all active sessions with attachment status
ccx list
```

### Stop a Session

```bash
# Stop a specific session
ccx stop <session-name>
```

### Attach to a Session

```bash
# Attach to an existing session interactively
ccx attach <session-name>
```

### Session Logs

Session events are logged to `~/.ccx/logs/<session-name>.log` for debugging.

```bash
# List all log files
ccx logs list

# Show log for a specific session
ccx logs show <session-name>

# Clean up logs older than 7 days (default)
ccx logs clean

# Clean up logs older than N days
ccx logs clean --days 30
```

## Workflow

1. Run `ccx start "your prompt"` to start a background Claude Code session
2. Run `ccx status` to see all sessions and their current titles
3. Run `ccx status <session>` to see recent output from a session
4. Run `ccx attach <session>` to attach and interact with the session
5. Run `ccx stop <session>` when done to clean up
