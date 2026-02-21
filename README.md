# OpenSwarm

A keyboard-first TUI for parallel AI agent deployment across Git worktrees.

## Install

```bash
cargo install --path . --bin openswarm --force
# or: make dev
```

## Usage

```bash
openswarm  # run inside a Git repo
```

- `a` - create worktree
- `O` - launch agent
- `c` - commit
- `p` - push
- `m` - merge into parent
- `d` - delete worktree
- `w` - toggle Changes/Worktrees view
- `h/j/k/l` or arrows - navigate

## Keybindings

| Key | Action |
|-----|--------|
| `a` | Create worktree |
| `o` | Open shell |
| `O` | Open agent |
| `c` | Commit |
| `p` | Push |
| `m` | Merge |
| `d` | Delete |
| `w` | Toggle view |
| `Tab` | Cycle panes |
| `Esc` | Close popup |

## Docs

[matars.github.io/OpenSwarm](https://matars.github.io/OpenSwarm/)
