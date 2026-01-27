# ccx (Claude Code eXecutor)

A CLI tool for running and managing multiple Claude Code sessions in tmux. Start background coding sessions with prompts, monitor their progress, and attach to them when needed.

## Installation

```bash
cargo install ccx
```

## Quick Start

```bash
ccx start "fix the login bug"
ccx status
ccx list
ccx stop <session>
```

## Claude Code plugins

```
claude plugin marketplace add camerondavison/ccx
claude plugin install ccx@ccx-marketplace
```
