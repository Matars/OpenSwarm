---
title: Git Behavior and Safety
parent: Reference
nav_order: 4
---

# Reference: Git Behavior and Safety

This page captures notable Git behavior in OpenSwarm.

## Worktree Discovery

- Uses `git worktree list --porcelain`.
- Computes per-worktree dirty/ahead/behind state using `git status --porcelain=1 -b -uall`.

## Create Worktree

- Uses `git worktree add -b <branch> <path> <start-point>`.
- Can carry uncommitted tracked changes when using selected-with-changes base.

## Stage and Commit

- Stage/unstage in changes view maps to `git add` and `git restore --staged`.
- Commit modes are interactive through OpenSwarm prompts.

## Push

- Push commands include upstream handling for new branches.

## Merge

- Merge into connected parent uses `git merge --no-edit <target-ref>` in parent worktree.
- On conflict, OpenSwarm can start an agent-assisted resolution flow.

## Removal Safety

- Refuses removal of current worktree.
- Dirty worktrees require confirmation before force remove.
