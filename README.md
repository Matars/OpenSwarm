# OpenSwarm

An interactive TUI for managing worktrees and parallel AI agents.
[![Docs](https://img.shields.io/badge/docs-matars.github.io%2FOpenSwarm-blue)](https://matars.github.io/OpenSwarm/)

![OpenSwarm screenshot](img/screenshot.png)

Run multiple AI agents in parallel, each in its own isolated Git worktree, managed from a single visual interface. No terminal juggling, no `cd` between directories, no lost context.

## The problem

AI agents work best with their own working directory. When you're running 5+ agents across different branches, you end up juggling terminals, manually creating worktrees, tracking which agent is where, and copy-pasting git commands across tabs.

OpenSwarm puts everything on one screen.

## Features

- **Worktree graph** -- visual, interactive graph with status badges, ahead/behind counts, and live agent activity
- **Embedded terminals** -- launch shells and agents in PTY sessions inside the TUI; sessions persist in the background
- **Inline diffs** -- file staging with method-level diff analysis
- **One-key git** -- create, commit, push, merge, rebase, and delete worktrees without leaving the TUI
- **Feature orchestration** -- describe a feature, review per-worktree prompts, execute across multiple branches
- **Agent conflict solver** -- merges that conflict can be handed off to an agent with a prefilled resolution prompt
- **Token telemetry** -- live tok/s leaderboard across all running agents

## Quick start

```bash
cargo install --path . --bin openswarm --force
openswarm
```

Then:
1. Press `a` to create a worktree (it is auto-selected)
2. Press `O` to launch an agent in it
3. Repeat for parallel streams
4. Press `c` to commit, `p` to push, `m` to merge back (with agent conflict solver if needed)

If you close OpenSwarm and reopen it later, default OpenCode launches can reconnect to that worktree's recent session context.

## Core keybindings

### Navigation

| Key | Action |
|-----|--------|
| `w` | Toggle Changes / Worktrees views |
| Arrow keys / `h` `j` `k` `l` | Move selection (`h/l` for sibling step) |
| `Ctrl+K` | Cycle graph builder (top-down, layered, left-right, trunk, swimlanes, indented) |
| `Tab` | Cycle panes |
| `H` / `?` | Panel help / full keybindings popup |
| `v` | Toggle details panel compact / verbose |
| `+` / `-` / `0` | Zoom in / out / reset |
| `W` `A` `S` `D` | Pan canvas |
| `M` | Toggle worktree art panel mode (config art / Spotify connector) |
| `B` | Cycle canvas background effect |
| `Ctrl+L` | Toggle frame-lag debug stats + hitch/JSONL perf logging |

### Worktree actions

| Key | Action |
|-----|--------|
| `a` | Create worktree (auto-select new node) |
| `b` | Open branch switcher (type to filter, Enter to switch/create) |
| `g` | Orchestrate feature, review per-leaf prompts, execute accepted nodes |
| `o` | Open shell |
| `O` | Open agent picker |
| `c` | Commit |
| `p` | Push |
| `f` | Fetch/pull parent (or selected root branch if behind head) |
| `F` | Rebase selected onto parent |
| `m` | Merge into parent |
| `d` / `dd` | Delete worktree (type `yes`/`no` confirm / instant force-delete) |

### Changes view

| Key | Action |
|-----|--------|
| `a` | Smart stage/unstage (stages unstaged changes first) |
| `j` / `k` (overview panel) | Select next/previous method |
| `J` / `K` (overview panel) | Scroll method details down/up |
| `Enter` / `Space` (overview panel) | Expand/collapse selected method hunk preview |
| `u` | Unstage selected file |
| `A` / `U` | Stage all / unstage all |
| `c` | Commit staged |
| `p` | Push |
| `s` / `S` | Stash push / pop |

### Terminal popup

| Key | Action |
|-----|--------|
| `Ctrl+G` | Toggle input / control mode |
| `Up` / `Down` | Scroll terminal view up/down (without sending prompt-history keys) |
| `Esc` | Close popup in CONTROL mode (session stays alive) |
| `q` | Terminate session in CONTROL mode |

This is the compact set for daily flow. See the [full keybindings reference](https://matars.github.io/OpenSwarm/keybindings.html) for advanced shortcuts (help modal, prune, reflog popup, notes editor, and confirmations).

## Documentation

Docs are at [matars.github.io/OpenSwarm](https://matars.github.io/OpenSwarm/) and live under `docs/`.

The docs follow a practical Divio-style mix (not rigidly split):

- **Tutorial-ish**: [Getting Started](https://matars.github.io/OpenSwarm/getting-started.html)
- **How-to + workflows**: [Features](https://matars.github.io/OpenSwarm/features.html)
- **Reference**: [Keybindings](https://matars.github.io/OpenSwarm/keybindings.html), [Configuration](https://matars.github.io/OpenSwarm/configuration.html)
- **Explanation**: [Comparisons](https://matars.github.io/OpenSwarm/comparisons.html), [FAQ](https://matars.github.io/OpenSwarm/faq.html)

Video demo: the GIF above is the quick preview; a longer walkthrough video can be linked from the docs home when added.

To run docs locally:

```bash
make docs
```

## Requirements

- Rust toolchain (stable)
- Git on PATH

## Documentation

Full docs, keybindings reference, configuration, and workflows: [matars.github.io/OpenSwarm](https://matars.github.io/OpenSwarm/)

## Status

Actively evolving. Feedback and bug reports welcome.
