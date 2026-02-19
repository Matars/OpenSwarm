---
title: Agent Session Lifecycle
parent: Explanation
nav_order: 3
---

# Explanation: Agent Session Lifecycle

OpenSwarm terminal sessions are long-lived PTY processes attached to worktree paths.

## Lifecycle

1. Launching: shell process starts in selected worktree.
2. Running: user input and output stream through popup.
3. Background: popup closes but session remains alive.
4. Done/Failed: process exits and state is tracked.

## Why This Model

- Preserves context between interactions.
- Lets you switch nodes without restarting tool state.
- Supports quick reopen for ongoing agent/shell work.

## Control vs Input Modes

- Input mode sends keys to terminal.
- Control mode manages popup/session behavior.
