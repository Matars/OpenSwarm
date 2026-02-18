# OpenSwarm

![OpenSwarm demo](media/demo.gif)

OpenSwarm is a keyboard-first Rust TUI built for **parallel agent deployment across Git worktrees**.

Instead of running one long linear loop in a single branch, you can spin up isolated worktrees, run tasks in parallel, and merge back with control.

## Why OpenSwarm

- Deploy multiple agent sessions in separate worktrees at the same time.
- Keep branch state isolated, reviewable, and merge-friendly.
- Stage, commit, push, and merge from one terminal UI.
- Jump between worktree graph and file changes without leaving flow.

## Parallel Worktree Model

```text
main
 ├─ wt/agent-auth     -> agent session A (auth changes)
 ├─ wt/agent-ui       -> agent session B (UI changes)
 └─ wt/agent-tests    -> agent session C (test hardening)

Each worktree is isolated, active, and mergeable back to its parent.
```

This lets you run concurrent implementation streams while preserving clean Git boundaries.

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

## Core workflow keybindings

### Navigation

- `w`: switch between changes and worktree views
- `Tab`: cycle panes in worktree view
- Arrow keys / `h` `j` `k` `l`: move selection
- `+` / `-` / `0`: zoom worktree canvas in/out/reset
- `W` `A` `S` `D`: pan worktree canvas

### Worktree orchestration

- `a`: create a worktree branch
- `o` or `z`: open terminal popup in selected worktree
- `f`: update connected parent
- `m`: merge selected worktree into connected parent
- `d`: remove selected worktree
- `x`: prune worktrees

### Change management

- `Enter` or `Space`: stage/unstage selected file
- `c`: commit mode
- `p`: push branch (with upstream handling)
- `s`: stash (including untracked)
- `S`: stash pop

### Terminal popup controls

- `Ctrl+g` or `:`: toggle input and control mode
- `Esc`: close popup and keep session in background
- `q`: terminate session
- `r`: restart session
- `i`: return to input mode

## Notes

- OpenSwarm uses your current shell from `SHELL`.
- Worktree operations rely on native `git worktree` commands.
- If launched outside a Git repo, commands will fail until you run it inside one.

## Project origin

OpenSwarm started as the `gitfetch` TUI and now lives as its own project focused on high-throughput, parallel worktree workflows.
