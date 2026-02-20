# OpenSwarm

![OpenSwarm demo](media/demo.gif)

A keyboard-first Rust TUI for **parallel AI agent deployment across Git worktrees**.

Run 5-10 AI agents in parallel, each in its own isolated worktree, managed from a single visual interface. No terminal juggling, no `cd` between directories, no lost context.

## Why OpenSwarm

AI agents like Claude Code and OpenCode can handle longer tasks without supervision. The bottleneck isn't the agent -- it's managing the parallel execution environment. Each agent needs its own working directory, and orchestrating worktree creation, agent launching, status monitoring, staging, committing, pushing, and merging across 5+ branches from separate terminals is painful.

OpenSwarm gives you one screen:

- **Worktree graph** -- see all worktrees as an interactive graph with parent-child relationships, dirty/needs-pull/committed/local-only/pushed/merged-with-parent badges, ahead/behind counts, live agent activity, and text-based estimated PTY token telemetry (ctx/out + tok/s)
- **Embedded terminals + per-node session memory** -- launch shells and agents in PTY sessions directly inside the TUI; sessions persist in the background, and default OpenCode launches reconnect to the most recent session for that same worktree node after restarting OpenSwarm
- **Inline diffs** -- switch to changes view for file staging with method-level diff analysis (Python, Rust, JS/TS, Go)
- **One-key git operations** -- create worktrees (`a`), commit (`c`), push (`p`), merge (`m`), delete (`d`) without leaving the TUI
- **Agent-powered merge conflict solver** -- when merges conflict, OpenSwarm can launch OpenCode with a prefilled conflict-resolution prompt in the parent worktree, so the agent resolves/stages while you keep orchestration in one place

## Quick start

```bash
# Install
cargo install --path . --bin openswarm --force

# Or build and run directly
make dev

# Launch inside any Git repo
openswarm
```

Then:
1. Press `a` to create a worktree
2. Press `O` to launch an agent in it
3. Repeat for parallel streams
4. Press `c` to commit, `p` to push, `m` to merge back (with agent conflict solver if needed)

If you close OpenSwarm and reopen it later, default OpenCode launches can reconnect to that worktree's recent session context.

## Core keybindings

### Navigation

| Key | Action |
|-----|--------|
| `w` | Toggle Changes / Worktrees views |
| Arrow keys / `h` `j` `k` `l` | Move selection |
| `Ctrl+K` | Cycle graph builder (top-down / radial) |
| `Tab` | Cycle panes |
| `+` / `-` / `0` | Zoom in / out / reset |
| `W` `A` `S` `D` | Pan canvas |
| `Ctrl+B` | Cycle canvas background effect |

### Worktree actions

| Key | Action |
|-----|--------|
| `a` | Create worktree |
| `o` | Open shell |
| `O` | Open agent picker |
| `c` | Commit |
| `p` | Push |
| `f` | Fetch/pull parent |
| `m` | Merge into parent |
| `d` | Delete worktree |

### Changes view

| Key | Action |
|-----|--------|
| `Space` / `Enter` / `a` | Smart stage/unstage (stages unstaged changes first) |
| `u` | Unstage selected file |
| `A` / `U` | Stage all / unstage all |
| `c` | Commit staged |
| `p` | Push |
| `s` / `S` | Stash push / pop |

### Terminal popup

| Key | Action |
|-----|--------|
| `Ctrl+G` | Toggle input / control mode |
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
- A Git repository

## Build commands

```bash
make dev                                          # Build and install globally
cargo run --bin openswarm                         # Run directly
cargo build --release --bin openswarm             # Build release binary
cargo install --path . --bin openswarm --force    # Install to PATH
```

## Configuration

Agent defaults, prompt templates, and optional worktree art live in `~/.config/openswarm/` (missing `worktree_graph_art` is auto-seeded with the default ASCII block). See the [configuration reference](https://matars.github.io/OpenSwarm/configuration.html).

## Notes

- Uses your shell from `$SHELL` for terminal sessions
- Worktree operations use native `git worktree` commands
- Long-running git operations run in the background with a live progress indicator
- Auto-detects `claude` and `opencode` on PATH
- Worktrees are placed in `.<repo>-workspaces/` sibling directory

## Current status

OpenSwarm is actively evolving. Expect rapid iteration -- keybindings and UI behavior may change. Feedback and bug reports are welcome.
