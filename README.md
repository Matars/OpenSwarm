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

For the simplest local dev loop:

```bash
make dev
```

This builds and runs OpenSwarm in one step.

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

## Documentation (Divio)

This repo includes a Divio-style docs structure under `docs/`:

- `tutorials/`
- `how-to/`
- `reference/`
- `explanation/`

To run docs locally (auto-creates `.venv` and installs docs deps):

```bash
make docs
```

To build docs strictly:

```bash
make docs-build
```

## Core workflow keybindings

### Navigation

- `w`: switch between changes and worktree views
- `Tab`: cycle panes in worktree view
- Arrow keys / `h` `j` `k` `l`: move selection
- `+` / `-` / `0`: zoom worktree canvas in/out/reset
- `W` `A` `S` `D`: pan worktree canvas

### Worktree orchestration

- `a`: create a worktree branch
- `o`: open terminal popup in selected worktree
- `O`: open agent picker popup (if installed)
- `c`: stage all and commit in selected worktree (message prompt)
- `p`: push selected worktree branch (with upstream handling)
- `f`: update connected parent
- `m`: merge selected worktree into connected parent
- `d`: remove selected worktree
- `x`: prune worktrees

### Change management

- `Enter` or `Space`: stage/unstage selected file
- `c`: commit mode
- `n`: open notes editor (`notes.md` in repo root)
- `p`: push branch (with upstream handling)
- `s`: stash (including untracked)
- `S`: stash pop

### Notes editor

- `n`: open the notes popup from changes/worktree views
- Opens in vim-style `NORMAL` mode by default
- `i` / `a` / `o` / `O`: enter `INSERT` mode
- `h` `j` `k` `l`: move cursor in `NORMAL` mode
- `dd`: delete current line in `NORMAL` mode
- `Ctrl+S`: save notes without closing
- `q` (from `NORMAL`): save and close
- `Esc` (from `INSERT`): return to `NORMAL`

### Terminal popup controls

- `Ctrl+g` (or `Cmd+g` on macOS): toggle input and control mode
- `Esc`: close popup and keep session in background
- `q`: terminate session
- `r`: restart session

## Notes

- OpenSwarm uses your current shell from `SHELL`.
- Worktree operations rely on native `git worktree` commands.
- If launched outside a Git repo, commands will fail until you run it inside one.
- Agent defaults and prompt templates live in `~/.config/openswarm`.

## Project origin

OpenSwarm started as the `gitfetch` TUI and now lives as its own project focused on high-throughput, parallel worktree workflows.
