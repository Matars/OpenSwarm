# Explanation: Worktree Graph Model

OpenSwarm visualizes worktrees as a graph instead of a flat list.

## Why a Graph

Worktrees are usually created from a source branch. That relationship matters for safe update/merge operations.
The graph makes parent-child intent visible:

- Child branch: active change stream.
- Connected parent: expected integration target.

## How It Helps

- Navigation follows branch relationships, not just alphabetical order.
- Merge (`m`) and update-parent (`f`) become contextual actions.
- You can reason about integration flow at a glance.

## Mental Model

Treat each node as an isolated lane with a clear upstream parent. Keep lanes small and merge frequently.
