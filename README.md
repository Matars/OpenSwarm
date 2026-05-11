# OpenSwarm

Keyboard-first TUI for running parallel AI agents across Git worktrees.

[![Docs](https://img.shields.io/badge/docs-matars.github.io%2FOpenSwarm-blue)](https://matars.github.io/OpenSwarm/)

![OpenSwarm screenshot](img/screenshot.png)

OpenSwarm is for the point where one agent is no longer enough.
Instead of juggling terminal tabs and manual worktree commands, you get one visual command center.

## Why it feels different

- **One screen for everything**: worktree graph, agent sessions, diffs, and git actions
- **Built for parallel flow**: run multiple isolated streams without context thrash
- **Fast keyboard loop**: create, commit, push, merge, and clean up without leaving the TUI
- **Configurable keybinds**: remap most Worktrees/Changes actions in `~/.config/openswarm/config.toml`, then inspect them live with `Ctrl+H`

## Quick start

```bash
cargo install --path . --bin openswarm --force
openswarm
```

Inside OpenSwarm:

1. `a` create worktree
2. `O` launch agent
3. `c` commit, `p` push, `m` merge

## Docs

Full docs: [matars.github.io/OpenSwarm](https://matars.github.io/OpenSwarm/)

- [Getting Started](https://matars.github.io/OpenSwarm/getting-started.html)
- [Features](https://matars.github.io/OpenSwarm/features.html)
- [Keybindings](https://matars.github.io/OpenSwarm/keybindings.html)
- [Configuration](https://matars.github.io/OpenSwarm/configuration.html)
- [Comparisons](https://matars.github.io/OpenSwarm/comparisons.html)
- [FAQ](https://matars.github.io/OpenSwarm/faq.html)

Run docs locally:

```bash
make docs
```

## Status

Actively evolving. Feedback and bug reports are welcome.
