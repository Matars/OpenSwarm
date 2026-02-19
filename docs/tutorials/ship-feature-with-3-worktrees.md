---
title: Ship Feature with 3 Worktrees
parent: Tutorials
nav_order: 2
---

# Tutorial: Ship One Feature with 3 Worktrees

This tutorial shows how to split one feature into parallel streams (core logic, UI, tests) and merge safely.

## Scenario

You want faster delivery without mixing unrelated changes in one long branch.

## Plan

- Worktree A: implementation
- Worktree B: UX/polish
- Worktree C: tests/hardening

## Steps

1. Create three worktrees
   - Press `a` three times and create three branch names.
2. Launch shells or agents per worktree
   - Press `o` or `O` on each selected worktree node.
3. Work in parallel
   - Keep each branch focused on one concern.
4. Commit and push each stream
   - Use `c` then `p` per worktree.
5. Update the connected parent
   - On a selected worktree, press `f` before merging.
6. Merge each branch back
   - Select a child worktree and press `m`.

## Conflict Handling

If a merge conflict appears, OpenSwarm can prompt agent-assisted resolution from the parent worktree flow.

## Result

You ship a feature with cleaner boundaries, lower review overhead, and better throughput.
