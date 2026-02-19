---
title: Resolve Merge Conflicts with Agent
parent: How-To Guides
nav_order: 3
---

# How to Resolve Merge Conflicts with an Agent

Use this flow when `m` (merge selected worktree into parent) reports conflicts.

## Trigger the Merge

- Select the source worktree.
- Press `m`.
- If conflicts are found, OpenSwarm opens a conflict-resolution confirm flow.

## Launch Resolution

- Confirm to launch agent-assisted resolution.
- OpenSwarm opens the parent worktree terminal popup.
- It can launch your agent and paste a structured conflict prompt.

## Complete Resolution

In the parent worktree:

1. Resolve conflict markers in all files.
2. Ensure unresolved conflict list is empty:
   - `git diff --name-only --diff-filter=U`
3. Stage resolved files:
   - `git add <files>`

## Notes

- The default conflict prompt is configurable.
- OpenSwarm does not push automatically after conflict resolution.
