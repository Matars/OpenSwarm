# Reference: Keybindings

This page lists core keybindings by view.

## Global

- `q`: quit (or open quit-confirm when terminal sessions are active)
- `w`: switch between changes and worktree views
- `n`: open notes popup

## Worktree View

- Navigation
  - Arrow keys: directional graph movement
  - `h` / `l`: sibling movement on same depth
  - `j` / `k`: vertical movement across levels
  - `Tab`: cycle panes
- Canvas controls
  - `+` / `-` / `0`: zoom in/out/reset
  - `W` / `A` / `S` / `D`: pan canvas
  - `?`: toggle help modal
- Actions
  - `a`: create worktree
  - `o`: open terminal popup
  - `O`: open agent picker
  - `c`: commit mode for selected worktree
  - `p`: push selected worktree
  - `f`: fetch/pull connected parent
  - `m`: merge selected worktree into parent
  - `d`: remove selected worktree
  - `x`: prune worktrees
  - `L`: open git reflog popup
  - `r`: refresh worktree list

## Changes View

- Navigation
  - `h` / `l`: focus files or overview pane
  - `j` / `k`: move selection or scroll overview
- File operations
  - `Space` or `Enter`: stage/unstage selected item
  - `c`: commit mode
  - `p`: push branch
  - `s`: stash push (include untracked)
  - `S`: stash pop
  - `r`: refresh status

## Terminal Popup

- `Ctrl+g` (or `Cmd+g` on macOS): toggle input/control mode
- Input mode: keys pass through to PTY
- Control mode:
  - `Esc`: close popup and keep session alive
  - `q`: terminate session
  - `r`: restart session

## Notes Popup

- `Ctrl+S`: save
- `Esc`: save and close
- Arrow keys, Home/End, PageUp/PageDown: cursor/navigation
