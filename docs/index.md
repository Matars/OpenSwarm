---
title: Home
nav_order: 1
---

# OpenSwarm

### A keyboard-first TUI for running parallel AI agents across Git worktrees

AI agents like Claude Code and OpenCode can handle longer tasks without supervision. Run 5-10 in parallel and your throughput multiplies. But each agent needs its own isolated working directory, and managing that with raw `git worktree` commands, separate terminal windows, and manual context switching is painful.

OpenSwarm gives you one screen to manage all of it: a visual worktree graph, embedded terminal sessions, inline staging and diffs, and one-key commits, pushes, and merges -- without ever leaving the TUI.

---

## The problem

Running parallel AI agents today requires juggling:

- **Worktree lifecycle** -- `git worktree add -b feat ../repo.feat`, then `cd ../repo.feat`, then remember to clean up later
- **Multiple terminals** -- one per agent, arranged across tmux panes or OS windows, constantly switching focus
- **Status awareness** -- which agent is active? which branch has uncommitted changes? which is ahead of remote?
- **Git operations** -- staging, committing, pushing, merging scattered across different shell sessions

This works for 2 agents. At 5+ it becomes unmanageable.

## How OpenSwarm solves it

OpenSwarm replaces the terminal juggling with a single integrated TUI:

| Task | Plain git + terminals | OpenSwarm |
|---|---|---|
| Create worktree | `git worktree add -b feat ../repo.feat && cd ../repo.feat` | Press `a`, type branch name |
| Launch agent | Open new terminal, cd to worktree, run `claude` | Press `O` on the node |
| Check all status | `cd` to each worktree, run `git status` | Visible on the graph -- dirty, ahead/behind, agent activity |
| Stage + commit | `cd` to worktree, `git add`, `git commit` | Press `c`, type message |
| Push branch | `cd` to worktree, `git push -u origin HEAD` | Press `p` |
| Merge to parent | `cd` to parent, `git merge`, resolve conflicts manually | Press `m`, agent-assisted conflict resolution |
| Monitor agents | Switch between terminal windows | See spinners and activity badges on every node |
| Review diffs | `git diff` per worktree | Press `w` for inline diff with method-level analysis |

Everything stays in one screen. No `cd`. No window switching. No lost context.

---

## Features

### Worktree graph visualization

Worktrees are displayed as an interactive graph with parent-child relationships. Navigate with arrow keys or `h`/`j`/`k`/`l`. Zoom and pan the canvas. Each node shows branch name, dirty state, ahead/behind counts, and live agent activity. Edges are drawn with Unicode box-drawing characters and color-coded by branch.

### Embedded PTY terminals

Press `o` to open a shell or `O` to launch an AI agent directly inside OpenSwarm. Sessions run in a real PTY with full ANSI color support. Close the popup and the session keeps running in the background -- reopen it anytime. Toggle between input mode (keys go to the terminal) and control mode (`Ctrl+G`) to manage sessions.

### Inline staging and diffs

Press `w` to switch to the changes view. See all staged and unstaged files in a tree layout. Stage/unstage with `Space`. The overview panel shows per-file diffs with smart method-level analysis -- it detects added, modified, and deleted functions for Python, Rust, JavaScript, TypeScript, and Go.

### One-key git operations

- `a` -- create worktree (choose base branch, type name)
- `c` -- commit (stages all changes in worktree view, or staged changes in changes view)
- `p` -- push (auto-sets upstream for new branches)
- `f` -- fetch and pull parent branch
- `m` -- merge worktree into parent
- `d` -- delete worktree (with safety confirmations)
- `s` / `S` -- stash push / stash pop

### Agent-assisted conflict resolution

When a merge produces conflicts, OpenSwarm shows the conflicted files and a customizable prompt template. Confirm and it launches your agent in the parent worktree with context about what needs resolving. Edit the prompt template inline with `e`.

### Agent activity monitoring

Each graph node shows live status: a spinning indicator when the agent is actively writing, idle duration when quiet, and a done/failed badge when the process exits. You can see at a glance which of your 5-10 agents are still working.

### Built-in notes editor

Press `n` to open a vim-style markdown editor for `notes.md` in the repo root. Supports normal/insert modes, `dd` to delete lines, `gg`/`G` to jump, `Ctrl+S` to save. Useful for tracking what each worktree is working on.

### Git reflog viewer

Press `L` on any worktree to see its recent reflog in a scrollable popup.

---

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

6. **Merge back** -- select a child worktree, press `m` to merge into its parent. If there are conflicts, OpenSwarm offers to launch an agent to help resolve them.

7. **Clean up** -- press `d` to delete a worktree, `x` to prune stale entries.

---

## How it compares

### vs. plain `git worktree`

Git worktrees provide the isolation but none of the management. You type the branch name three times to create one, manually `cd` between directories, and have no unified view of status across worktrees. OpenSwarm wraps the full lifecycle in single keystrokes with visual feedback.

### vs. Worktrunk

[Worktrunk](https://worktrunk.dev) is a CLI that simplifies worktree commands -- `wt switch`, `wt list`, `wt merge`. It makes the commands shorter but you still manage agents in separate terminals (tmux, zellij) and context-switch between them. OpenSwarm is a TUI that embeds everything in one screen: the worktree graph, the terminals, the diffs, the git operations. If you want a composable CLI, use Worktrunk. If you want an integrated visual workspace, use OpenSwarm.

### vs. lazygit / other Git TUIs

Git TUIs operate on a single repository directory. They're excellent for staging, committing, and branch management within one worktree. OpenSwarm manages multiple worktrees simultaneously, embeds terminal sessions for running agents, and provides cross-worktree status awareness. You could use lazygit inside an OpenSwarm terminal popup for detailed single-repo work.

### vs. tmux/zellij + manual setup

You can achieve parallel agents with a terminal multiplexer: one pane per worktree, each running an agent. This works but scales poorly -- you lose visual overview of which agents are active, you manually manage worktree creation/deletion, and git operations require switching panes. OpenSwarm gives you the multiplexer-like embedded terminals plus the worktree management and status aggregation in one tool.

---

## FAQ

### Why Git worktrees?

Worktrees give each agent its own complete working directory. Unlike branches (which share one directory), worktrees let multiple agents read and write files simultaneously without conflicts. Each worktree has its own index, working tree, and HEAD. This is what makes parallel agent execution practical.

### What agents does OpenSwarm support?

OpenSwarm auto-detects `claude` and `opencode` on your PATH. It launches them inside an embedded PTY -- any CLI agent that runs in a terminal works. You can also just open a plain shell with `o`.

### Does OpenSwarm modify my repository?

OpenSwarm uses standard git commands: `git worktree add/remove`, `git add`, `git commit`, `git push`, `git merge`, `git status`, `git stash`. It stores parent-child worktree relationships in a `.parent-hints` file in the workspaces directory. Worktrees are created in a `.<repo>-workspaces/` sibling directory. No git hooks are modified, no global config is touched.

### Can I use OpenSwarm on an existing repo with worktrees?

Yes. OpenSwarm reads `git worktree list` on startup and displays whatever worktrees already exist. It will offer to migrate from the legacy `.gitfetch-worktrees/` layout if detected.

### What are the key dependencies?

- **ratatui** + **crossterm** -- TUI framework and terminal backend
- **portable-pty** + **vt100** -- PTY spawning and terminal emulation
- **tachyonfx** -- Animation effects

---

## Reference

- [Keybindings](keybindings.html) -- Complete keyboard shortcut reference by view
- [Configuration](configuration.html) -- Config file location, keys, and prompt templates
