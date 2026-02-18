# OpenSwarm

OpenSwarm is a keyboard-first Rust TUI for day-to-day Git work.

It started as the `gitfetch` TUI, and now lives as its own project.

## What it does

- Shows a live, navigable view of changed files.
- Lets you stage/unstage files quickly.
- Supports commit and push flows from inside the TUI.
- Includes a worktree navigator for creating, opening, merging, and pruning worktrees.
- Provides an embedded terminal popup per worktree session.

## Current status

OpenSwarm is actively evolving and still experimental.

- Expect rapid iteration.
- Keybindings and UI behavior may change.
- Feedback and bug reports are welcome.

## Requirements

- Rust toolchain (stable)
- Git installed and available on `PATH`
- A Git repository (OpenSwarm runs against your current repo)

## Run

```bash
cargo run --bin openswarm
```

Install globally (so `openswarm` works on `PATH`):

```bash
cargo install --path . --bin openswarm --force
openswarm
```

Build once and print the CLI path automatically:

```bash
cargo build --release --bin openswarm
printf '%s\n' "$(pwd)/target/release/openswarm"
```

`cargo build --release` creates `./target/release/openswarm`; it does not install a global command.

## Core keybindings

### Global

- `q`: quit
- `r`: refresh

### Changes view

- `w`: switch to worktree view
- `h` / `l` or Left / Right: focus file list vs overview pane
- `j` / `k` or Up / Down: move selection or scroll overview
- `Enter` or `Space`: stage/unstage selected file
- `c`: commit mode
- `p`: push branch (with upstream handling)
- `s`: stash (including untracked)
- `S`: stash pop

### Worktree view

- `w`: switch back to changes view
- `Tab`: cycle worktree panes
- `?`: toggle panel help
- Arrow keys / `h` `j` `k` `l`: move in worktree graph/list
- `+` / `-` / `0`: zoom in, zoom out, reset canvas
- `W` `A` `S` `D`: pan canvas
- `a`: create worktree branch
- `o` or `z`: open terminal popup in selected worktree
- `p`: add/commit/push selected worktree
- `L`: open reflog popup
- `f`: update connected parent
- `m`: merge selected worktree into connected parent
- `d`: remove selected worktree
- `x`: prune worktrees

### Terminal popup

- `Ctrl+g` or `:`: toggle input mode and control mode
- In control mode:
  - `Esc`: close popup and keep session in background
  - `q`: terminate session
  - `r`: restart session
  - `i`: return to input mode

## Notes

- OpenSwarm uses your current shell from `SHELL`.
- Worktree operations rely on native `git worktree` commands.
- If launched outside a Git repo, commands will fail until you run it inside one.

## Project origin

This project was split out from the `gitfetch` repository to evolve independently as a dedicated Git workflow TUI.
