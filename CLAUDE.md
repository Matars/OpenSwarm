# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Documentation

When changing features, always check that README.md and docs/ is up to date at the end of your work. This includes keybindings, workflow descriptions, and any user-facing behavior changes.

## Build Commands

```bash
make dev              # Build release and install globally
cargo run --bin openswarm    # Run directly without installing
cargo build --release --bin openswarm   # Build release binary only
cargo install --path . --bin openswarm --force   # Install to PATH
```

## Architecture

OpenSwarm is a keyboard-first Rust TUI for parallel agent deployment across Git worktrees. It uses ratatui/crossterm for rendering and portable-pty/vt100 for embedded terminal sessions.

### Code Organization

The codebase uses `include!` macros to split a monolithic App module into logical sections:

- `src/main.rs` - Entry point, delegates to `app::run()`
- `src/app/mod.rs` - Core App struct, state management, and main event loop
- `src/app/sections/input.rs` - Keyboard input handlers for each Mode
- `src/app/sections/worktree_git.rs` - Git operations and worktree parsing
- `src/app/sections/ui.rs` - All ratatui rendering functions

### State Model

Single `App` struct holds all application state. Key state includes:
- `mode: Mode` - Current UI mode (Normal, CommitInput, AgentPopup, etc.)
- `view_mode: ViewMode` - Either Changes or Worktrees view
- `worktrees: Vec<WorktreeEntry>` - Parsed git worktree list
- `agent_sessions: BTreeMap<String, AgentSession>` - Active PTY sessions by path

### Event Loop

The main loop in `run()` handles:
1. Agent PTY output draining via channel (`agent_rx`)
2. Terminal resize for popup sessions
3. UI rendering with mode-specific drawing
4. Keyboard input dispatch based on current `Mode`
5. Periodic git status refresh (1.2s interval)

### UI Modes

Mode enum controls input handling and popup rendering:
- `Normal` - Main navigation in either Changes or Worktrees view
- `AgentPopup` - Fullscreen PTY terminal for external agents
- `NotesPopup` - Simple text editor for notes.md
- Various confirm dialogs and input modals

### External Agent Integration

Supports launching external CLI agents (opencode, claude) in worktrees via PTY. Detection at startup checks PATH availability. Sessions persist in background when popup is dismissed.

## Key Dependencies

- `ratatui` + `crossterm` - TUI framework and terminal backend
- `portable-pty` + `vt100` - PTY spawning and terminal emulation
- `tachyonfx` - Animation effects for canvas nodes
