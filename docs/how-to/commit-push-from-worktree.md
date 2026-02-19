# How to Commit and Push from a Worktree

Use this guide to stage changes and publish a branch from a selected worktree.

## Stage and Review

- Switch to changed files view with `w`.
- Select a file and press `Space` or `Enter` to stage/unstage.
- Review file overview in the middle panel.

## Commit

- In changes view: press `c` and enter your commit message.
- In worktree view: press `c` on selected worktree for commit mode there.

## Push

- Press `p`.
- OpenSwarm handles upstream setup if the branch is new.

## Common Issues

- No selected file: move selection with `j`/`k`.
- Empty commit message: commit mode will not complete meaningfully.
- Not on expected worktree: verify selected node before pushing.
