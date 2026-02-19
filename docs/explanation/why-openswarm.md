# Explanation: Why OpenSwarm

OpenSwarm exists to remove the throughput bottleneck of one-branch-at-a-time work.

Traditional linear flow often mixes tasks, increases context switching, and creates heavy pull requests.
OpenSwarm uses Git worktrees as isolated execution lanes so multiple implementation streams can run in parallel with clean boundaries.

## Core Value

- Parallelism with branch isolation.
- Lower merge risk through focused change sets.
- Faster loops by keeping orchestration in one keyboard-first TUI.

## Intended Outcome

You ship more often, with smaller and more reviewable units of work.
