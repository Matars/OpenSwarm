---
title: First Parallel Workflow
parent: Tutorials
nav_order: 1
---

# Tutorial: First Parallel Workflow

This tutorial walks you through one complete OpenSwarm loop: create a worktree, make changes, commit, and push.

## Goal

By the end, you will have one isolated branch/worktree running independently from your main branch.

## Prerequisites

- You are inside a Git repository.
- `openswarm` runs from your shell.

## Steps

1. Start OpenSwarm
   - Run `make dev` or `openswarm`.
2. Create a worktree
   - In worktree view, press `a`.
   - Choose a base branch with left/right arrows.
   - Enter a new branch name and press Enter.
3. Open a terminal in that worktree
   - Select your new node.
   - Press `o`.
   - Make your code changes from that shell.
4. Commit from OpenSwarm
   - Return to OpenSwarm.
   - Press `c` in worktree view.
   - Type a commit message and press Enter.
5. Push your branch
   - Press `p`.
   - OpenSwarm will handle upstream setup when needed.

## Verify Success

- Worktree node shows your branch.
- Commit appears in that worktree branch history.
- Push completes without errors.

## Next

Continue with `tutorials/ship-feature-with-3-worktrees.md` to run multiple streams in parallel.
