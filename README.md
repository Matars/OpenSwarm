# OpenSwarm

![OpenSwarm demo](media/demo.gif)

A keyboard-first Rust TUI for **parallel AI agent deployment across Git worktrees**.

Run 5-10 AI agents in parallel, each in its own isolated worktree, managed from a single visual interface. No terminal juggling, no `cd` between directories, no lost context.

## Quick Start

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

## Essential Keybindings

| Key | Action |
|-----|--------|
| `w` | Toggle Changes / Worktrees views |
| `h` `j` `k` `l` | Navigate graph |
| `a` | Create worktree |
| `o` | Open shell |
| `O` | Launch agent |
| `c` | Commit |
| `p` | Push |
| `m` | Merge into parent |
| `d` | Delete worktree |

See [full keybindings reference](https://matars.github.io/OpenSwarm/keybindings.html) for advanced shortcuts.

## Documentation

Full docs at [matars.github.io/OpenSwarm](https://matars.github.io/OpenSwarm/):

- [Getting Started](https://matars.github.io/OpenSwarm/getting-started.html) -- tutorial
- [Features](https://matars.github.io/OpenSwarm/features.html) -- workflow guide
- [Keybindings](https://matars.github.io/OpenSwarm/keybindings.html) -- complete reference
- [Configuration](https://matars.github.io/OpenSwarm/configuration.html) -- setup guide

Run docs locally: `make docs`

## Requirements

- Rust toolchain (stable)
- Git on PATH
- A Git repository

## Build

```bash
make dev              # Build and install globally
cargo run --bin openswarm    # Run directly
cargo build --release --bin openswarm   # Build release
```

## Configuration

Agent defaults and templates in `~/.config/openswarm/`. See [configuration reference](https://matars.github.io/OpenSwarm/configuration.html).

---

OpenSwarm is actively evolving. Feedback and bug reports welcome.
