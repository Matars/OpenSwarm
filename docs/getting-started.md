---
title: Getting Started
nav_order: 1
---

# Getting Started

## Install

```bash
cargo install --path . --bin openswarm --force
```

Or build from source:

```bash
git clone https://github.com/Matars/OpenSwarm.git
cd OpenSwarm
make dev
```

## Quick start

1. **Launch** inside any Git repository:
   ```
   openswarm
   ```

2. **Create a worktree** -- press `a`, select a base branch with left/right arrows, type a branch name, press Enter.

3. **Launch an agent** -- select the new node, press `O` to pick an agent (Claude, OpenCode) or `o` for a plain shell.

4. **Work in parallel** -- create more worktrees, launch more agents. The graph shows all of them with live status.

5. **Commit and push** -- select a worktree, press `c` to commit, `p` to push.

6. **Merge back** -- select a child worktree, press `m` to merge into its parent. If there are conflicts, OpenSwarm offers to launch OpenCode with a prefilled conflict-resolution prompt (or you can launch another agent via `O`).

7. **Clean up** -- press `d` to delete a worktree, `x` to prune stale entries.

8. **Reconnect context later** -- if you close and reopen OpenSwarm, pressing `O` on the same worktree can reconnect the recent OpenCode session for that node (when OpenCode is your default agent).

## Requirements

- Rust toolchain (stable)
- Git on PATH
- A Git repository
