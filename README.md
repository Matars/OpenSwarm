# OpenSwarm

![OpenSwarm demo](media/demo.gif)

A keyboard-first Rust TUI for **parallel AI agent deployment across Git worktrees**.

Run 5-10 AI agents in parallel, each in its own isolated worktree, managed from a single visual interface. No terminal juggling, no `cd` between directories, no lost context.

## Why OpenSwarm

AI agents like Claude Code and OpenCode can handle longer tasks without supervision. The bottleneck isn't the agent -- it's managing the parallel execution environment. Each agent needs its own working directory, and orchestrating worktree creation, agent launching, status monitoring, staging, committing, pushing, and merging across 5+ branches from separate terminals is painful.

OpenSwarm gives you one screen:

- **Worktree graph** -- see all worktrees as an interactive graph with parent-child relationships, dirty/committed/pushed/merged badges, ahead/behind counts, and live agent activity
- **Embedded terminals** -- launch shells and agents in PTY sessions directly inside the TUI, with sessions persisting in the background
- **Inline diffs** -- switch to changes view for file staging with method-level diff analysis (Python, Rust, JS/TS, Go)
- **One-key git operations** -- create worktrees (`a`), commit (`c`), push (`p`), merge (`m`), delete (`d`) without leaving the TUI
- **Agent-assisted conflict resolution** -- when merges conflict, launch an agent with a templated prompt to help resolve them

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
4. Press `c` to commit, `p` to push, `m` to merge back

## Core keybindings

### Navigation

| Key | Action |
|-----|--------|
| `w` | Toggle Changes / Worktrees views |
| Arrow keys / `h` `j` `k` `l` | Move selection |
| `Tab` | Cycle panes |
| `+` / `-` / `0` | Zoom in / out / reset |
| `W` `A` `S` `D` | Pan canvas |

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
| `Space` / `Enter` / `a` | Stage / unstage file |
| `c` | Commit staged |
| `p` | Push |
| `s` / `S` | Stash push / pop |

### Terminal popup

| Key | Action |
|-----|--------|
| `Ctrl+G` | Toggle input / control mode |
| `Esc` | Close popup (session stays alive) |
| `q` | Terminate session |

See the [full keybindings reference](https://matars.github.io/OpenSwarm/keybindings.html) for all shortcuts including the notes editor, modals, and git reflog viewer.

## Documentation

Docs are at [matars.github.io/OpenSwarm](https://matars.github.io/OpenSwarm/) and live under `docs/`.

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

Agent defaults and prompt templates live in `~/.config/openswarm/`. See the [configuration reference](https://matars.github.io/OpenSwarm/configuration.html).

## Notes

- Uses your shell from `$SHELL` for terminal sessions
- Worktree operations use native `git worktree` commands
- Long-running git operations run in the background with a live progress indicator
- Auto-detects `claude` and `opencode` on PATH
- Worktrees are placed in `.<repo>-workspaces/` sibling directory

## Current status

OpenSwarm is actively evolving. Expect rapid iteration -- keybindings and UI behavior may change. Feedback and bug reports are welcome.
