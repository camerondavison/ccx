---
name: worktree-itack
description: >
  Automate working on itack issues using git worktrees.
  (1) Creates a git worktree for the issue in $WORKTREE directory
  (2) Starts a ccx session in the worktree to work on the issue
  (3) Provides cleanup commands after the work is merged
allowed-tools: Bash(git *), Bash(ccx *), Bash(itack *)
---

# Worktree Itack

Automates the workflow for working on itack issues using git worktrees and ccx sessions.

## Usage

```
/worktree-itack <issue-number>
```

## Workflow

When invoked with an issue number, perform these steps:

### 1. Get the issue details

```bash
itack show <issue-number>
```

Extract the issue title to create a descriptive branch name.

### 2. Create the worktree

Get the current project name from the basename of the current working directory.

Create a branch name from the issue: `<project>-issue-<number>-<slugified-title>`

For example, in project "ccx", issue 15 with title "Add user authentication" becomes `ccx-issue-15-add-user-authentication`.

```bash
git worktree add "${WORKTREE}/<project>-issue-<number>-<slug>" -b <project>-issue-<number>-<slug>
```

The `$WORKTREE` environment variable must be set to the directory where worktrees should be created.

### 3. Start a ccx session

```bash
ccx start --cwd "${WORKTREE}/<project>-issue-<number>-<slug>" "Work on itack issue <number>. When you have completed the work, mark the issue as done with 'itack done <number>' and then commit your changes with a descriptive commit message."
```

### 4. Provide next steps

After starting the session, tell the user:

1. Attach to the session: `ccx attach <session-name>`
2. Work on the issue in the tmux session
3. When done, commit and merge into main
4. Clean up with:
   ```bash
   git worktree remove "${WORKTREE}/<project>-issue-<number>-<slug>"
   git branch -d <project>-issue-<number>-<slug>
   ```

## Example

```
/worktree-itack 15
```

In a project named "myapp", this will:
1. Fetch issue 15 details from itack
2. Create worktree at `$WORKTREE/myapp-issue-15-<title-slug>`
3. Start a ccx session to work on the issue
4. Print instructions for attaching and cleanup
