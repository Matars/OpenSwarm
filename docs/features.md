---
title: Features
nav_order: 2
---

# Features

## Worktree graph visualization

Worktrees are displayed as an interactive graph with parent-child relationships. Navigate with arrow keys or `h`/`j`/`k`/`l`. Zoom and pan the canvas. Each node shows branch name, dirty state, ahead/behind counts, and live agent activity. Edges are drawn with Unicode box-drawing characters and color-coded by branch.

## Embedded PTY terminals

Press `o` to open a shell or `O` to launch an AI agent directly inside OpenSwarm. Sessions run in a real PTY with full ANSI color support. Close the popup and the session keeps running in the background -- reopen it anytime. Toggle between input mode (keys go to the terminal) and control mode (`Ctrl+G`) to manage sessions.

## Inline staging and diffs

Press `w` to switch to the changes view. See all staged and unstaged files in a tree layout. `Space`/`Enter`/`a` use lazygit-style smart staging (if a file still has unstaged changes, stage them first), and `u` explicitly unstages. `A`/`U` stage or unstage everything quickly. The overview panel shows per-file diffs with smart method-level analysis -- it detects added, modified, and deleted functions for Python, Rust, JavaScript, TypeScript, and Go.

## One-key git operations

- `a` -- create worktree (choose base branch, type name)
- `c` -- commit (stages all changes in worktree view, or staged changes in changes view)
- `p` -- push (auto-sets upstream for new branches)
- `f` -- fetch and pull parent branch
- `m` -- merge worktree into parent
- `d` -- delete worktree (with safety confirmations)
- `s` / `S` -- stash push / stash pop

## Agent-assisted conflict resolution

When a merge produces conflicts, OpenSwarm shows the conflicted files and a customizable prompt template. Confirming launches OpenCode in the parent worktree with context about what needs resolving (or opens a shell there if OpenCode is unavailable). You can still launch another agent manually with `O`. Edit the prompt template inline with `e`.

## Agent activity monitoring

Each graph node shows live status: a spinning indicator when the agent is actively writing, idle duration when quiet, and a done/failed badge when the process exits. You can see at a glance which of your 5-10 agents are still working.

## Built-in notes editor

Press `n` to open a vim-style markdown editor for `notes.md` in the repo root. Supports normal/insert modes, `dd` to delete lines, `gg`/`G` to jump, `Ctrl+S` to save. Useful for tracking what each worktree is working on.

## Git reflog viewer

Press `L` on any worktree to see its recent reflog in a scrollable popup.
