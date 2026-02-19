use std::error::Error;
use std::io;
use std::io::Read;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};
use std::{
    collections::{BTreeMap, HashSet},
    fs,
};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};
use ratatui::backend::CrosstermBackend;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph};
use ratatui::Terminal;

#[derive(Clone, Debug)]
struct FileEntry {
    path: String,
    staged: bool,
    unstaged: bool,
    untracked: bool,
}

#[derive(Clone, Debug)]
enum Mode {
    Normal,
    CommitInput,
    WorktreeCommitPushInput,
    WorktreeCreateInput,
    WorktreeBranchConflictConfirm,
    WorktreeRemoveDirtyConfirm,
    WorktreeGitLogPopup,
    LegacyWorkspaceMigrateConfirm,
    QuitWithSessionsConfirm,
    AgentSelectPopup,
    AgentPopup,
    NotesPopup,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ExternalAgent {
    Opencode,
    Claude,
}

impl ExternalAgent {
    fn display_name(self) -> &'static str {
        match self {
            ExternalAgent::Opencode => "OpenCode",
            ExternalAgent::Claude => "Claude",
        }
    }

    fn command_name(self) -> &'static str {
        match self {
            ExternalAgent::Opencode => "opencode",
            ExternalAgent::Claude => "claude",
        }
    }
}

#[derive(Clone, Debug)]
struct ConflictResolveContext {
    parent_path: String,
    source_branch: String,
    target_branch: String,
    conflicted_files: Vec<String>,
}

#[derive(Clone, Debug)]
struct OpenSwarmConfig {
    config_path: String,
    default_agent: Option<ExternalAgent>,
    conflict_resolve_prompt_path: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ViewMode {
    Changes,
    Worktrees,
}

struct App {
    branch: String,
    ahead: usize,
    behind: usize,
    files: Vec<FileEntry>,
    tree_items: Vec<TreeItem>,
    selected: usize,
    selected_overview: Option<FileOverview>,
    active_pane: ActivePane,
    overview_scroll: u16,
    status_line: String,
    mode: Mode,
    view_mode: ViewMode,
    commit_input: String,
    worktrees: Vec<WorktreeEntry>,
    selected_worktree: usize,
    worktree_focus: WorktreePane,
    worktree_canvas_zoom: f64,
    worktree_canvas_pan_x: f64,
    worktree_canvas_pan_y: f64,
    show_panel_help: bool,
    new_worktree_branch: String,
    new_worktree_base: WorktreeCreateBase,
    pending_create_branch: String,
    confirm_delete_branch_yes: bool,
    pending_remove_worktree_path: String,
    confirm_remove_worktree_yes: bool,
    worktree_commit_input: String,
    worktree_commit_path: Option<String>,
    git_log_popup_path: Option<String>,
    git_log_lines: Vec<String>,
    git_log_scroll: u16,
    confirm_quit_with_sessions_yes: bool,
    confirm_legacy_workspace_migrate_yes: bool,
    pending_legacy_workspace_root: String,
    pending_legacy_workspace_path: String,
    pending_new_workspace_path: String,
    legacy_workspace_prompt_dismissed: bool,
    quit_now: bool,
    agent_sessions: BTreeMap<String, AgentSession>,
    detected_agents: Vec<ExternalAgent>,
    agent_select_index: usize,
    agent_select_path: Option<String>,
    pending_conflict_context: Option<ConflictResolveContext>,
    config: OpenSwarmConfig,
    agent_popup_path: Option<String>,
    terminal_popup_mode: TerminalPopupMode,
    notes_path: String,
    notes_lines: Vec<String>,
    notes_cursor_row: usize,
    notes_cursor_col: usize,
    notes_scroll: u16,
    agent_tx: Sender<AgentEvent>,
    agent_rx: Receiver<AgentEvent>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TerminalPopupMode {
    Input,
    Control,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AgentState {
    Launching,
    Running,
    Done,
    Failed,
}

struct AgentSession {
    state: AgentState,
    parser: vt100::Parser,
    master: Option<Box<dyn MasterPty + Send>>,
    writer: Option<Box<dyn Write + Send>>,
    child: Option<Box<dyn portable_pty::Child + Send>>,
    last_size: (u16, u16),
    launched_at: Instant,
    last_io_at: Instant,
    bytes_from_agent: u64,
    bytes_to_agent: u64,
}

enum AgentEvent {
    Output { path: String, bytes: Vec<u8> },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WorktreePane {
    Canvas,
    Details,
    Actions,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WorktreeCreateBase {
    Main,
    Selected,
    SelectedWithChanges,
}

#[derive(Clone, Debug, Default)]
struct WorktreeEntry {
    path: String,
    head: String,
    branch: String,
    bare: bool,
    detached: bool,
    locked: bool,
    prunable: bool,
    is_current: bool,
    dirty: bool,
    ahead: usize,
    behind: usize,
    parent_hint: Option<String>,
}

#[derive(Clone, Debug)]
struct TreeItem {
    path: String,
    label: String,
    kind: TreeKind,
    staged: bool,
    unstaged: bool,
    untracked: bool,
    added_lines: usize,
    removed_lines: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum TreeKind {
    Folder,
    File,
}

#[derive(Clone, Copy, Debug, Default)]
struct PathStatus {
    staged: bool,
    unstaged: bool,
    untracked: bool,
}

#[derive(Clone, Copy, Debug, Default)]
struct PathDelta {
    added_lines: usize,
    removed_lines: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ActivePane {
    Files,
    Overview,
}

#[derive(Clone, Debug)]
struct FileOverview {
    file: String,
    state: String,
    added_lines: usize,
    removed_lines: usize,
    methods_added: Vec<String>,
    methods_modified: Vec<String>,
    methods_deleted: Vec<String>,
    traditional_diff: Vec<DiffPreviewLine>,
    use_traditional_overview: bool,
}

#[derive(Clone, Debug)]
struct DiffPreviewLine {
    kind: DiffPreviewKind,
    text: String,
}

#[derive(Clone, Debug)]
enum DiffPreviewKind {
    Added,
    Removed,
    Meta,
    Context,
}

impl App {
    fn new() -> Self {
        let (agent_tx, agent_rx) = mpsc::channel();
        let config = load_openswarm_config();
        Self {
            branch: "unknown".to_string(),
            ahead: 0,
            behind: 0,
            files: Vec::new(),
            tree_items: Vec::new(),
            selected: 0,
            selected_overview: None,
            active_pane: ActivePane::Files,
            overview_scroll: 0,
            status_line: "Ready".to_string(),
            mode: Mode::Normal,
            view_mode: ViewMode::Worktrees,
            commit_input: String::new(),
            worktrees: Vec::new(),
            selected_worktree: 0,
            worktree_focus: WorktreePane::Canvas,
            worktree_canvas_zoom: 1.0,
            worktree_canvas_pan_x: 0.0,
            worktree_canvas_pan_y: 0.0,
            show_panel_help: false,
            new_worktree_branch: String::new(),
            new_worktree_base: WorktreeCreateBase::Selected,
            pending_create_branch: String::new(),
            confirm_delete_branch_yes: false,
            pending_remove_worktree_path: String::new(),
            confirm_remove_worktree_yes: false,
            worktree_commit_input: String::new(),
            worktree_commit_path: None,
            git_log_popup_path: None,
            git_log_lines: Vec::new(),
            git_log_scroll: 0,
            confirm_quit_with_sessions_yes: false,
            confirm_legacy_workspace_migrate_yes: false,
            pending_legacy_workspace_root: String::new(),
            pending_legacy_workspace_path: String::new(),
            pending_new_workspace_path: String::new(),
            legacy_workspace_prompt_dismissed: false,
            quit_now: false,
            agent_sessions: BTreeMap::new(),
            detected_agents: detect_available_agents(),
            agent_select_index: 0,
            agent_select_path: None,
            pending_conflict_context: None,
            config,
            agent_popup_path: None,
            terminal_popup_mode: TerminalPopupMode::Input,
            notes_path: String::new(),
            notes_lines: vec![String::new()],
            notes_cursor_row: 0,
            notes_cursor_col: 0,
            notes_scroll: 0,
            agent_tx,
            agent_rx,
        }
    }

    fn selected_item(&self) -> Option<&TreeItem> {
        self.tree_items.get(self.selected)
    }

    fn select_next(&mut self) {
        if self.tree_items.is_empty() {
            self.selected = 0;
        } else {
            self.selected = (self.selected + 1) % self.tree_items.len();
        }
    }

    fn select_prev(&mut self) {
        if self.tree_items.is_empty() {
            self.selected = 0;
        } else if self.selected == 0 {
            self.selected = self.tree_items.len() - 1;
        } else {
            self.selected -= 1;
        }
    }

    fn focus_left(&mut self) {
        self.active_pane = ActivePane::Files;
    }

    fn focus_right(&mut self) {
        self.active_pane = ActivePane::Overview;
    }

    fn selected_worktree(&self) -> Option<&WorktreeEntry> {
        self.worktrees.get(self.selected_worktree)
    }

    fn next_worktree_pane(&mut self) {
        self.worktree_focus = match self.worktree_focus {
            WorktreePane::Canvas => WorktreePane::Details,
            WorktreePane::Details => WorktreePane::Actions,
            WorktreePane::Actions => WorktreePane::Canvas,
        };
    }

    fn cycle_worktree_base_left(&mut self) {
        self.new_worktree_base = match self.new_worktree_base {
            WorktreeCreateBase::Main => WorktreeCreateBase::SelectedWithChanges,
            WorktreeCreateBase::Selected => WorktreeCreateBase::Main,
            WorktreeCreateBase::SelectedWithChanges => WorktreeCreateBase::Selected,
        };
    }

    fn cycle_worktree_base_right(&mut self) {
        self.new_worktree_base = match self.new_worktree_base {
            WorktreeCreateBase::Main => WorktreeCreateBase::Selected,
            WorktreeCreateBase::Selected => WorktreeCreateBase::SelectedWithChanges,
            WorktreeCreateBase::SelectedWithChanges => WorktreeCreateBase::Main,
        };
    }
}

struct TuiGuard;

impl Drop for TuiGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let mut stdout = io::stdout();
        let _ = execute!(stdout, LeaveAlternateScreen);
    }
}

pub fn run() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let _guard = TuiGuard;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    refresh_status(&mut app);

    let ui_tick_rate_fast = Duration::from_millis(16);
    let ui_tick_rate_normal = Duration::from_millis(33);
    let status_tick_rate = Duration::from_millis(1200);
    let mut last_ui_tick = Instant::now();
    let mut last_status_tick = Instant::now();
    let mut should_quit = false;

    while !should_quit {
        drain_agent_events(&mut app);
        refresh_agent_sessions(&mut app);

        // Resize terminal session to match actual popup dimensions
        if matches!(app.mode, Mode::AgentPopup) {
            if let Some(path) = app.agent_popup_path.clone() {
                let size = terminal.size()?;
                let frame_area = Rect::new(0, 0, size.width, size.height);
                let (rows, cols) = calc_terminal_popup_size(frame_area);
                resize_terminal_session(&mut app, &path, rows, cols);
            }
        }

        terminal.draw(|frame| draw_ui(frame, &app))?;

        let ui_tick_rate = if matches!(app.mode, Mode::AgentPopup) {
            ui_tick_rate_fast
        } else {
            ui_tick_rate_normal
        };
        let ui_timeout = ui_tick_rate.saturating_sub(last_ui_tick.elapsed());
        let status_timeout = status_tick_rate.saturating_sub(last_status_tick.elapsed());
        let timeout = if ui_timeout < status_timeout {
            ui_timeout
        } else {
            status_timeout
        };
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if !matches!(app.mode, Mode::AgentPopup)
                        && key.modifiers.contains(KeyModifiers::CONTROL)
                        && key.code == KeyCode::Char('c')
                    {
                        should_quit = true;
                        continue;
                    }
                    match app.mode {
                        Mode::Normal => {
                            should_quit = handle_normal_mode_key(&mut app, key.code)?;
                        }
                        Mode::CommitInput => {
                            handle_commit_mode_key(&mut app, key.code)?;
                        }
                        Mode::WorktreeCommitPushInput => {
                            handle_worktree_commit_push_mode_key(&mut app, key.code)?;
                        }
                        Mode::WorktreeCreateInput => {
                            handle_worktree_create_mode_key(&mut app, key.code)?;
                        }
                        Mode::WorktreeBranchConflictConfirm => {
                            handle_branch_conflict_confirm_mode_key(&mut app, key.code)?;
                        }
                        Mode::WorktreeRemoveDirtyConfirm => {
                            handle_worktree_remove_dirty_confirm_mode_key(&mut app, key.code)?;
                        }
                        Mode::WorktreeGitLogPopup => {
                            handle_worktree_git_log_mode_key(&mut app, key.code);
                        }
                        Mode::LegacyWorkspaceMigrateConfirm => {
                            handle_legacy_workspace_migrate_mode_key(&mut app, key.code)?;
                        }
                        Mode::QuitWithSessionsConfirm => {
                            handle_quit_with_sessions_mode_key(&mut app, key.code);
                        }
                        Mode::AgentSelectPopup => {
                            handle_agent_select_mode_key(&mut app, key.code)?;
                        }
                        Mode::AgentPopup => {
                            handle_agent_popup_key(&mut app, key)?;
                        }
                        Mode::NotesPopup => {
                            handle_notes_popup_key(&mut app, key)?;
                        }
                    }

                    if app.quit_now {
                        should_quit = true;
                    }
                }
            }
        }

        if last_ui_tick.elapsed() >= ui_tick_rate {
            last_ui_tick = Instant::now();
        }

        let refresh_git = !matches!(app.mode, Mode::AgentPopup);
        if refresh_git && last_status_tick.elapsed() >= status_tick_rate {
            refresh_status(&mut app);
            last_status_tick = Instant::now();
        }
    }

    terminal.show_cursor()?;
    Ok(())
}

include!("sections/input.rs");
include!("sections/worktree_git.rs");
include!("sections/ui.rs");
