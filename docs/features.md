---
title: Features
nav_order: 2
---

# Features

## Worktree graph visualization

Worktrees are displayed as an interactive graph with parent-child relationships. Navigate with arrow keys or `h`/`j`/`k`/`l`. Press `Ctrl+K` to cycle graph builders: top-down balanced, layered, left-right, trunk, swimlanes, and indented. Zoom and pan the canvas. Each node shows branch name, status badges (`dirty`, `behind parent`, `behind head`, `committed`, `local only`, `pushed`, `merged with parent`), ahead/behind counts, and live agent activity. Edges are color-coded by branch and rendered with terminal-safe single-width glyphs for consistent alignment.
Press `M` to switch the right-side art panel between configured static art and the Spotify connector view (album art + song + artist). The connector reads Spotify metadata through `playerctl` (MPRIS) when available and falls back to AppleScript on macOS.

## Embedded PTY terminals

Press `o` to open a shell or `O` to launch an AI agent directly inside OpenSwarm. Sessions run in a real PTY with full ANSI color support. Close the popup and the session keeps running in the background -- reopen it anytime. If your configured default agent is OpenCode, `O` attempts to reconnect to the most recent OpenCode session whose directory matches that worktree node (including after restarting OpenSwarm). Toggle between input mode (keys go to the terminal) and control mode (`Ctrl+G`) to manage sessions.

## Inline staging and diffs

Press `w` to switch to the changes view. See all staged and unstaged files in a tree layout. `Space`/`Enter`/`a` use lazygit-style smart staging (if a file still has unstaged changes, stage them first), and `u` explicitly unstages. `A`/`U` stage or unstage everything quickly. The overview panel shows per-file diffs with smart method-level analysis -- it detects added, modified, and deleted functions for Python, Rust, JavaScript, TypeScript, and Go.

## One-key git operations

- `a` -- create worktree (choose base branch, type name, auto-select new node)
- `b` -- open branch switcher (scrollable local branches + live filter, Enter switches or creates typed branch)
- `g` -- orchestrate a feature requirement, review/accept/refine per-leaf prompts, then execute accepted worktrees
- `c` -- commit (stages all changes in worktree view, or staged changes in changes view)
- `p` -- push (auto-sets upstream for new branches)
- `f` -- fetch and pull parent branch (or selected root branch if behind head)
- `F` -- rebase selected worktree branch onto parent branch
- `m` -- merge worktree into parent
- `d` -- delete worktree (with safety confirmations)
- `s` / `S` -- stash push / stash pop

All git operations run in the background. If you trigger another operation while one is in flight, it gets queued and auto-executes when the current task finishes. The status bar shows `+N queued` so you always know how many tasks are pending. This means you can press `p` on three different worktrees back-to-back without waiting for each push to complete.

## Feature-to-worktree orchestration

Press `g` in worktrees view and describe the feature (for example: `implement auth with oauth + session refresh`). OpenSwarm asks OpenCode to return a strict JSON plan of branch nodes and parent edges, then opens a preview for each leaf with a suggested execution prompt. You can accept/reject nodes, refine prompts per node, and then execute only the accepted worktree abstractions on your canvas. It does not write product code, commit, push, or auto-launch agents.

If OpenCode is unavailable or its plan is invalid, OpenSwarm falls back to a built-in heuristic planner so the flow still works offline. The planner prompt template, enable toggle, and max node cap are configurable in `~/.config/openswarm/config.toml`.

## Agent-assisted conflict resolution

When a merge produces conflicts, OpenSwarm shows the conflicted files and a customizable prompt template. Confirming launches OpenCode in the parent worktree with a prefilled resolution prompt (or opens a shell there if OpenCode is unavailable). This keeps conflict solving inside the worktree graph workflow instead of dropping to manual multi-terminal merge handling. You can still launch another agent manually with `O`. Edit the prompt template inline with `e`.

## Agent activity monitoring

Each graph node shows live status: a spinning indicator when the agent is actively writing, idle duration when quiet, and a done/failed badge when the process exits. You can see at a glance which of your 5-10 agents are still working.
The worktree canvas also includes an animated top-right tok/s leaderboard with Unicode bars normalized to the busiest stream, while idle worktrees are grouped into a single compact `idle xN` row to save space.
In the details panel, OpenCode sessions use exact token usage from OpenCode's session database (shown as `tokens:`). Non-OpenCode sessions fall back to PTY text-based estimates (shown as `tokens~`).

## Built-in notes editor

Press `n` to open a vim-style markdown editor for `notes.md` in the repo root. Supports normal/insert modes, `dd` to delete lines, `gg`/`G` to jump, `Ctrl+S` to save. Useful for tracking what each worktree is working on.

## Git reflog viewer

Press `L` on any worktree to see its recent reflog in a scrollable popup.
