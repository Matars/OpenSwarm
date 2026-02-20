---
title: Comparisons
nav_order: 5
---

# How it compares

## vs. plain `git worktree`

Git worktrees provide the isolation but none of the management. You type the branch name three times to create one, manually `cd` between directories, and have no unified view of status across worktrees. OpenSwarm wraps the full lifecycle in single keystrokes with visual feedback.

## vs. Worktrunk

[Worktrunk](https://worktrunk.dev) is a CLI that simplifies worktree commands -- `wt switch`, `wt list`, `wt merge`. It makes the commands shorter but you still manage agents in separate terminals (tmux, zellij) and context-switch between them. OpenSwarm is a TUI that embeds everything in one screen: the worktree graph, the terminals, the diffs, the git operations. If you want a composable CLI, use Worktrunk. If you want an integrated visual workspace, use OpenSwarm.

## vs. lazygit / other Git TUIs

Git TUIs operate on a single repository directory. They're excellent for staging, committing, and branch management within one worktree. OpenSwarm manages multiple worktrees simultaneously, embeds terminal sessions for running agents, and provides cross-worktree status awareness. You could use lazygit inside an OpenSwarm terminal popup for detailed single-repo work.

## vs. tmux/zellij + manual setup

You can achieve parallel agents with a terminal multiplexer: one pane per worktree, each running an agent. This works but scales poorly -- you lose visual overview of which agents are active, you manually manage worktree creation/deletion, and git operations require switching panes. OpenSwarm gives you the multiplexer-like embedded terminals plus the worktree management and status aggregation in one tool.
