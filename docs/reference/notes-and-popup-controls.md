# Reference: Notes and Popup Controls

## Notes File

- Project notes path is resolved to `<repo-root>/notes.md`.
- If it does not exist, OpenSwarm creates it.

## Notes Popup Behavior

- Open with `n` from changes or worktree view.
- Starts in vim-style `NORMAL` mode.
- Enter `INSERT` with `i`, `a`, `o`, or `O`.
- Use `h` / `j` / `k` / `l` and `dd` in `NORMAL` mode.
- Save in place with `Ctrl+S`.
- `q` (from `NORMAL`) saves and exits.
- `Esc` (from `INSERT`) returns to `NORMAL`.
- Supports multiline editing and cursor movement keys.

## Conflict Prompt Editor

- During conflict confirm mode, press `e`.
- Opens the configured prompt file in the same editor widget.
- Saving updates future conflict-resolution launches.

## Terminal Popup Modes

- Input mode: sends keys directly to shell/agent process.
- Control mode: manage popup/session state.
- Toggle with `Ctrl+g` (or `Cmd+g` on macOS).
