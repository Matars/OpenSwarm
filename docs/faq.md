---
title: FAQ
nav_order: 6
---

# FAQ

## Why Git worktrees?

Worktrees give each agent its own complete working directory. Unlike branches (which share one directory), worktrees let multiple agents read and write files simultaneously without conflicts. Each worktree has its own index, working tree, and HEAD. This is what makes parallel agent execution practical.

## What agents does OpenSwarm support?

OpenSwarm auto-detects `claude` and `opencode` on your PATH. It launches them inside an embedded PTY -- any CLI agent that runs in a terminal works. You can also just open a plain shell with `o`.

## Does OpenSwarm modify my repository?

OpenSwarm uses standard git commands: `git worktree add/remove`, `git add`, `git commit`, `git push`, `git merge`, `git status`, `git stash`. It stores parent-child worktree relationships in a `.parent-hints` file in the workspaces directory. Worktrees are created in a `.<repo>-workspaces/` sibling directory. No git hooks are modified, no global config is touched.

## Can I use OpenSwarm on an existing repo with worktrees?

Yes. OpenSwarm reads `git worktree list` on startup and displays whatever worktrees already exist. It will offer to migrate from the legacy `.gitfetch-worktrees/` layout if detected.

## What are the key dependencies?

- **ratatui** + **crossterm** -- TUI framework and terminal backend
- **portable-pty** + **vt100** -- PTY spawning and terminal emulation
- **tachyonfx** -- Animation effects
