---
title: Keybindings
nav_order: 3
---

# Keybindings

## Global

| Key | Action |
|-----|--------|
| `q` | Quit (confirms if terminal sessions are active) |
| `Ctrl+C` | Force quit |
| `w` | Toggle between Changes and Worktrees views |
| `n` | Open notes editor |

---

## Worktrees view

### Navigation

| Key | Action |
|-----|--------|
| Arrow keys | Directional graph navigation (nearest node in that direction) |
| `h` / `l` | Move between siblings at the same depth |
| `j` / `k` | Move to child / parent levels |
| `Tab` | Cycle focus: Canvas, Details, Actions |
| `?` | Toggle help modal |

### Canvas

| Key | Action |
|-----|--------|
| `+` / `=` | Zoom in |
| `-` | Zoom out |
| `0` | Reset zoom and pan |
| `W` `A` `S` `D` | Pan up / left / down / right |
| `Ctrl+B` | Cycle canvas background mode (stars / nebula / crosshatch) |

### Actions

| Key | Action |
|-----|--------|
| `a` | Create worktree |
| `o` | Open shell in worktree |
| `O` | Open agent picker (or launch default agent; default OpenCode attempts session resume) |
| `c` | Commit (`git add . && git commit`) for selected worktree |
| `p` | Push selected worktree branch |
| `f` | Fetch and pull parent branch |
| `m` | Merge selected worktree into parent |
| `d` | Delete selected worktree |
| `x` | Prune stale worktrees |
| `L` | Open git reflog popup |
| `r` | Refresh worktree list |

---

## Changes view

### Navigation

| Key | Action |
|-----|--------|
| `h` / `Left` | Focus files panel |
| `l` / `Right` | Focus overview panel |
| `j` / `Down` | Next file / scroll down |
| `k` / `Up` | Previous file / scroll up |

### Actions

| Key | Action |
|-----|--------|
| `Space` / `Enter` / `a` | Smart stage/unstage selected file (stages unstaged changes first) |
| `u` | Unstage selected file |
| `A` / `U` | Stage all / unstage all |
| `c` | Commit staged changes |
| `p` | Push current branch |
| `s` | Stash push (includes untracked) |
| `S` | Stash pop |
| `r` | Refresh status |

---

## Terminal popup

Toggle between INPUT and CONTROL mode with `Ctrl+G` (or `Cmd+G` on macOS).

**INPUT mode** -- all keys forwarded to the PTY process (shell or agent).

**CONTROL mode:**

| Key | Action |
|-----|--------|
| `Esc` | Close popup, keep session running in background |
| `q` | Terminate session and close |
| `r` | Restart session |

---

## Notes editor

Vim-style editor for `notes.md` (or conflict prompt templates).

**NORMAL mode:**

| Key | Action |
|-----|--------|
| `h` `j` `k` `l` | Move cursor |
| `0` / `Home` | Beginning of line |
| `$` / `End` | End of line |
| `gg` | Go to first line |
| `G` | Go to last line |
| `i` | Insert at cursor |
| `a` | Insert after cursor |
| `A` | Insert at end of line |
| `o` | New line below |
| `O` | New line above |
| `x` / `Delete` | Delete character |
| `dd` | Delete line |
| `q` | Save and close |
| `Ctrl+S` | Save |

**INSERT mode:**

| Key | Action |
|-----|--------|
| Characters | Insert text |
| `Enter` | New line |
| `Backspace` | Delete backward |
| `Tab` | Insert 4 spaces |
| `Esc` | Return to NORMAL mode |
| `Ctrl+S` | Save |

---

## Modals and confirmations

### Create worktree modal

| Key | Action |
|-----|--------|
| `Left` / `Right` | Cycle base: Main, Selected, Selected+Changes |
| Characters | Type branch name |
| `Enter` | Create |
| `Esc` | Cancel |

### Agent picker

| Key | Action |
|-----|--------|
| `j` / `k` | Move selection |
| `Enter` | Launch selected agent |
| `Esc` | Cancel |

### Git reflog popup

| Key | Action |
|-----|--------|
| `j` / `k` | Scroll |
| `PageDown` / `PageUp` | Scroll fast |
| `Home` / `End` | Jump to top / bottom |
| `Esc` / `q` / `L` | Close |

### Confirmation dialogs

All confirmation dialogs (delete dirty worktree, quit with sessions, branch conflict, merge conflict):

| Key | Action |
|-----|--------|
| `Left` / `Right` / `Tab` | Toggle Yes / No |
| `y` | Select Yes |
| `n` | Select No |
| `Enter` | Confirm |
| `Esc` | Cancel |

The conflict resolution dialog also supports `e` to edit the prompt template inline.

When you confirm conflict resolution with Enter, OpenSwarm launches OpenCode by default for that flow.
