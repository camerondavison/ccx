# Development Guidelines

## Before Committing

Always run `just fix` before committing to format code and auto-fix linting issues.

## Plugin Structure

This project includes a Claude Code plugin under `plugins/`:

- `plugins/commands/` - User-invocable slash commands (e.g., `/worktree-itack`)
- `plugins/skills/` - Skills that provide tools and context to Claude
