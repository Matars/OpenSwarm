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
    collections::{BTreeMap, BTreeSet, HashSet, VecDeque},
    fs,
};

use base64::{prelude::BASE64_STANDARD, Engine};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};
use ratatui::backend::CrosstermBackend;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Margin, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph};
use ratatui::Terminal;
use ratatui_image::{picker::Picker, protocol::StatefulProtocol, StatefulImage};
use tachyonfx::{fx, EffectManager, Interpolation};

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
    WorktreeOrchestrateInput,
    WorktreeOrchestratePreview,
    WorktreeOrchestratePromptEdit,
    WorktreeBranchConflictConfirm,
    WorktreeConflictResolveConfirm,
    WorktreeRemoveDirtyConfirm,
    WorktreeGitLogPopup,
    LegacyWorkspaceMigrateConfirm,
    QuitWithSessionsConfirm,
    AgentSelectPopup,
    AgentPopup,
    NotesPopup,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NotesContext {
    Notes,
    ConflictPrompt,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NotesEditMode {
    Normal,
    Insert,
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
    worktree_orchestrator_enabled: bool,
    worktree_orchestrator_prompt_path: String,
    worktree_orchestrator_max_nodes: usize,
    worktree_graph_art: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ViewMode {
    Changes,
    Worktrees,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WorktreeGraphBuilder {
    TopDownBalanced,
    Layered,
    LeftToRight,
    Trunk,
    Swimlanes,
    Indented,
}

impl WorktreeGraphBuilder {
    fn label(self) -> &'static str {
        match self {
            WorktreeGraphBuilder::TopDownBalanced => "top-down",
            WorktreeGraphBuilder::Layered => "layered",
            WorktreeGraphBuilder::LeftToRight => "left-right",
            WorktreeGraphBuilder::Trunk => "trunk",
            WorktreeGraphBuilder::Swimlanes => "swimlanes",
            WorktreeGraphBuilder::Indented => "indented",
        }
    }

    fn next(self) -> Self {
        match self {
            WorktreeGraphBuilder::TopDownBalanced => WorktreeGraphBuilder::Layered,
            WorktreeGraphBuilder::Layered => WorktreeGraphBuilder::LeftToRight,
            WorktreeGraphBuilder::LeftToRight => WorktreeGraphBuilder::Trunk,
            WorktreeGraphBuilder::Trunk => WorktreeGraphBuilder::Swimlanes,
            WorktreeGraphBuilder::Swimlanes => WorktreeGraphBuilder::Indented,
            WorktreeGraphBuilder::Indented => WorktreeGraphBuilder::TopDownBalanced,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CanvasBackgroundMode {
    GlitterStars,
    Crosshatch,
    Rainfall,
}

impl CanvasBackgroundMode {
    fn next(self) -> Self {
        match self {
            CanvasBackgroundMode::GlitterStars => CanvasBackgroundMode::Crosshatch,
            CanvasBackgroundMode::Crosshatch => CanvasBackgroundMode::Rainfall,
            CanvasBackgroundMode::Rainfall => CanvasBackgroundMode::GlitterStars,
        }
    }

    fn short_label(self) -> &'static str {
        match self {
            CanvasBackgroundMode::GlitterStars => "stars",
            CanvasBackgroundMode::Crosshatch => "crosshatch",
            CanvasBackgroundMode::Rainfall => "rain",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WorktreeArtMode {
    ConfigArt,
    SpotifyConnector,
}

impl WorktreeArtMode {
    fn next(self) -> Self {
        match self {
            WorktreeArtMode::ConfigArt => WorktreeArtMode::SpotifyConnector,
            WorktreeArtMode::SpotifyConnector => WorktreeArtMode::ConfigArt,
        }
    }

    fn label(self) -> &'static str {
        match self {
            WorktreeArtMode::ConfigArt => "config art",
            WorktreeArtMode::SpotifyConnector => "spotify connector",
        }
    }
}

#[derive(Clone, Debug)]
struct SpotifyNowPlaying {
    track: String,
    artist: String,
    art_url: Option<String>,
}

struct SpotifyCoverArt {
    source_url: String,
    image: StatefulProtocol,
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
    overview_method_index: usize,
    overview_method_expanded: bool,
    status_line: String,
    worktree_load_error: Option<String>,
    mode: Mode,
    view_mode: ViewMode,
    changes_worktree_path: Option<String>,
    commit_input: String,
    worktrees: Vec<WorktreeEntry>,
    selected_worktree: usize,
    worktree_focus: WorktreePane,
    worktree_canvas_zoom: f64,
    worktree_canvas_pan_x: f64,
    worktree_canvas_pan_y: f64,
    worktree_graph_builder: WorktreeGraphBuilder,
    worktree_canvas_bg_mode: CanvasBackgroundMode,
    worktree_art_mode: WorktreeArtMode,
    spotify_now_playing: Option<SpotifyNowPlaying>,
    spotify_cover_art: Option<SpotifyCoverArt>,
    spotify_image_picker: Option<Picker>,
    spotify_last_refresh: Instant,
    spotify_refresh_error: Option<String>,
    canvas_bg_effects: EffectManager<&'static str>,
    canvas_bg_last_tick: Instant,
    canvas_selected_border_effects: EffectManager<&'static str>,
    canvas_selected_border_last_tick: Instant,
    canvas_node_animations: Vec<CanvasNodeAnimation>,
    last_worktree_node_points: BTreeMap<String, (f64, f64)>,
    worktree_animations_ready: bool,
    show_panel_help: bool,
    new_worktree_branch: String,
    new_worktree_base: WorktreeCreateBase,
    pending_create_branch: String,
    orchestrator_requirement_input: String,
    orchestrator_planned_requirement: String,
    orchestrator_planner_source: String,
    orchestrator_prompt_nodes: Vec<OrchestratorPromptNode>,
    orchestrator_prompt_selected: usize,
    orchestrator_prompt_edit_input: String,
    confirm_delete_branch_yes: bool,
    pending_remove_worktree_path: String,
    confirm_remove_worktree_yes: bool,
    confirm_conflict_resolve_yes: bool,
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
    notes_context: NotesContext,
    notes_edit_mode: NotesEditMode,
    notes_pending_op: Option<char>,
    agent_tx: Sender<AgentEvent>,
    agent_rx: Receiver<AgentEvent>,
    git_task: Option<GitTaskState>,
    git_task_queue: VecDeque<QueuedGitTask>,
    git_task_tx: Sender<GitTaskEvent>,
    git_task_rx: Receiver<GitTaskEvent>,
    status_refresh_in_flight: bool,
    status_refresh_tx: Sender<StatusRefreshEvent>,
    status_refresh_rx: Receiver<StatusRefreshEvent>,
    perf_debug: PerfDebugState,
}

#[derive(Clone, Debug)]
struct OrchestratorPromptNode {
    branch: String,
    parent: String,
    goal: String,
    prompt: String,
    accepted: bool,
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
    agent_kind: Option<ExternalAgent>,
    parser: vt100::Parser,
    master: Option<Box<dyn MasterPty + Send>>,
    writer: Option<Box<dyn Write + Send>>,
    child: Option<Box<dyn portable_pty::Child + Send>>,
    last_size: (u16, u16),
    launched_at: Instant,
    last_io_at: Instant,
    bytes_from_agent: u64,
    bytes_to_agent: u64,
    io_samples: VecDeque<IoSample>,
    opencode_session_id: Option<String>,
    opencode_usage: Option<OpenCodeUsage>,
}

#[derive(Clone, Copy, Debug, Default)]
struct OpenCodeUsage {
    input_tokens: u64,
    output_tokens: u64,
    input_tokens_per_second: u64,
    output_tokens_per_second: u64,
}

struct IoSample {
    at: Instant,
    bytes_from_agent: u64,
    bytes_to_agent: u64,
}

enum AgentEvent {
    Output { path: String, bytes: Vec<u8> },
}

struct GitTaskState {
    label: String,
    started_at: Instant,
}

struct GitTaskEvent {
    label: String,
    outcome: String,
    refresh_worktrees: bool,
    refresh_status: bool,
}

struct QueuedGitTask {
    label: String,
    refresh_worktrees: bool,
    refresh_status: bool,
    task: Box<dyn FnOnce() -> String + Send + 'static>,
}

struct StatusSnapshot {
    branch: String,
    ahead: usize,
    behind: usize,
    files: Vec<FileEntry>,
    tree_items: Vec<TreeItem>,
}

struct StatusRefreshEvent {
    snapshot: Option<StatusSnapshot>,
    error: Option<String>,
}

#[derive(Clone, Copy, Debug, Default)]
struct FramePhaseDurations {
    drain_agent_events: Duration,
    drain_git_task_events: Duration,
    refresh_agent_sessions: Duration,
    refresh_opencode_usage: Duration,
    resize_popup: Duration,
    draw: Duration,
    event_poll: Duration,
    event_handle: Duration,
    refresh_status: Duration,
    refresh_worktrees: Duration,
    total_loop: Duration,
}

#[derive(Clone, Copy, Debug)]
struct FrameHitch {
    at: Instant,
    phases: FramePhaseDurations,
}

struct PerfDebugState {
    enabled: bool,
    frame_intervals: VecDeque<Duration>,
    worst_frame: Duration,
    last_frame_at: Instant,
    last_hitch: Option<FrameHitch>,
    hitch_log_path: PathBuf,
}

impl PerfDebugState {
    const FRAME_WINDOW: usize = 240;
    const HITCH_THRESHOLD: Duration = Duration::from_millis(90);

    fn new() -> Self {
        Self {
            enabled: false,
            frame_intervals: VecDeque::new(),
            worst_frame: Duration::ZERO,
            last_frame_at: Instant::now(),
            last_hitch: None,
            hitch_log_path: std::env::temp_dir().join("openswarm-hitches.log"),
        }
    }

    fn record_frame_interval(&mut self, frame_interval: Duration) {
        self.frame_intervals.push_back(frame_interval);
        if self.frame_intervals.len() > Self::FRAME_WINDOW {
            self.frame_intervals.pop_front();
        }
        if frame_interval > self.worst_frame {
            self.worst_frame = frame_interval;
        }
    }

    fn avg_frame_ms(&self) -> f64 {
        if self.frame_intervals.is_empty() {
            return 0.0;
        }
        let total_secs: f64 = self.frame_intervals.iter().map(Duration::as_secs_f64).sum();
        (total_secs * 1000.0) / (self.frame_intervals.len() as f64)
    }

    fn p95_frame_ms(&self) -> f64 {
        if self.frame_intervals.is_empty() {
            return 0.0;
        }
        let mut values: Vec<f64> = self
            .frame_intervals
            .iter()
            .map(|d| d.as_secs_f64() * 1000.0)
            .collect();
        values.sort_by(|a, b| a.total_cmp(b));
        let idx = ((values.len() as f64) * 0.95).floor() as usize;
        values[idx.min(values.len().saturating_sub(1))]
    }

    fn fps(&self) -> f64 {
        let avg_ms = self.avg_frame_ms();
        if avg_ms <= f64::EPSILON {
            0.0
        } else {
            1000.0 / avg_ms
        }
    }

    fn worst_frame_ms(&self) -> f64 {
        self.worst_frame.as_secs_f64() * 1000.0
    }

    fn record_loop_phases(&mut self, phases: FramePhaseDurations) {
        let blocking_adjusted = phases.total_loop.saturating_sub(phases.event_poll);
        if blocking_adjusted < Self::HITCH_THRESHOLD {
            return;
        }

        let hitch = FrameHitch {
            at: Instant::now(),
            phases,
        };
        self.last_hitch = Some(hitch);
        if self.enabled {
            self.log_hitch(hitch);
        }
    }

    fn log_hitch(&self, hitch: FrameHitch) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let measured_no_poll = hitch
            .phases
            .drain_agent_events
            .saturating_add(hitch.phases.drain_git_task_events)
            .saturating_add(hitch.phases.refresh_agent_sessions)
            .saturating_add(hitch.phases.refresh_opencode_usage)
            .saturating_add(hitch.phases.resize_popup)
            .saturating_add(hitch.phases.draw)
            .saturating_add(hitch.phases.event_handle)
            .saturating_add(hitch.phases.refresh_status)
            .saturating_add(hitch.phases.refresh_worktrees);
        let blocking_adjusted = hitch
            .phases
            .total_loop
            .saturating_sub(hitch.phases.event_poll);
        let unattributed = blocking_adjusted.saturating_sub(measured_no_poll);

        let line = format!(
            "ts={} total={:.1}ms draw={:.1}ms drain_agent={:.1}ms opencode={:.1}ms refresh_status={:.1}ms refresh_worktrees={:.1}ms poll={:.1}ms handle={:.1}ms unattributed={:.1}ms\n",
            now,
            hitch.phases.total_loop.as_secs_f64() * 1000.0,
            hitch.phases.draw.as_secs_f64() * 1000.0,
            hitch.phases.drain_agent_events.as_secs_f64() * 1000.0,
            hitch.phases.refresh_opencode_usage.as_secs_f64() * 1000.0,
            hitch.phases.refresh_status.as_secs_f64() * 1000.0,
            hitch.phases.refresh_worktrees.as_secs_f64() * 1000.0,
            hitch.phases.event_poll.as_secs_f64() * 1000.0,
            hitch.phases.event_handle.as_secs_f64() * 1000.0,
            unattributed.as_secs_f64() * 1000.0,
        );

        if let Ok(mut file) = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.hitch_log_path)
        {
            let _ = file.write_all(line.as_bytes());
        }
    }
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
    has_upstream: bool,
    merged_with_parent: bool,
    behind_parent: bool,
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
    method_changes: Vec<MethodChange>,
    traditional_diff: Vec<DiffPreviewLine>,
    use_traditional_overview: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum MethodChangeKind {
    Added,
    Modified,
    Deleted,
}

#[derive(Clone, Debug)]
struct MethodChange {
    kind: MethodChangeKind,
    name: String,
    diff_lines: Vec<DiffPreviewLine>,
}

#[derive(Clone, Debug)]
struct DiffPreviewLine {
    kind: DiffPreviewKind,
    text: String,
}

#[derive(Clone)]
enum CanvasNodeAnimationTarget {
    Path(String),
    Point((f64, f64)),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CanvasNodeAnimationKind {
    Created,
    Deleted,
}

struct CanvasNodeAnimation {
    target: CanvasNodeAnimationTarget,
    kind: CanvasNodeAnimationKind,
    effects: EffectManager<&'static str>,
    last_tick: Instant,
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
        let (git_task_tx, git_task_rx) = mpsc::channel();
        let (status_refresh_tx, status_refresh_rx) = mpsc::channel();
        let config = load_openswarm_config();
        let mut canvas_bg_effects = EffectManager::default();
        canvas_bg_effects.add_unique_effect("bg-polish", build_canvas_bg_effect());
        let mut canvas_selected_border_effects = EffectManager::default();
        canvas_selected_border_effects
            .add_unique_effect("selected-border", build_selected_node_border_effect());
        let spotify_image_picker = Picker::from_query_stdio().ok();
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
            overview_method_index: 0,
            overview_method_expanded: false,
            status_line: "Ready".to_string(),
            worktree_load_error: None,
            mode: Mode::Normal,
            view_mode: ViewMode::Worktrees,
            changes_worktree_path: None,
            commit_input: String::new(),
            worktrees: Vec::new(),
            selected_worktree: 0,
            worktree_focus: WorktreePane::Canvas,
            worktree_canvas_zoom: 1.0,
            worktree_canvas_pan_x: 0.0,
            worktree_canvas_pan_y: 0.0,
            worktree_canvas_bg_mode: CanvasBackgroundMode::GlitterStars,
            worktree_art_mode: WorktreeArtMode::ConfigArt,
            spotify_now_playing: None,
            spotify_cover_art: None,
            spotify_image_picker,
            spotify_last_refresh: Instant::now(),
            spotify_refresh_error: None,
            worktree_graph_builder: WorktreeGraphBuilder::Layered,
            canvas_bg_effects,
            canvas_bg_last_tick: Instant::now(),
            canvas_selected_border_effects,
            canvas_selected_border_last_tick: Instant::now(),
            canvas_node_animations: Vec::new(),
            last_worktree_node_points: BTreeMap::new(),
            worktree_animations_ready: false,
            show_panel_help: false,
            new_worktree_branch: String::new(),
            new_worktree_base: WorktreeCreateBase::Selected,
            pending_create_branch: String::new(),
            orchestrator_requirement_input: String::new(),
            orchestrator_planned_requirement: String::new(),
            orchestrator_planner_source: String::new(),
            orchestrator_prompt_nodes: Vec::new(),
            orchestrator_prompt_selected: 0,
            orchestrator_prompt_edit_input: String::new(),
            confirm_delete_branch_yes: false,
            pending_remove_worktree_path: String::new(),
            confirm_remove_worktree_yes: false,
            confirm_conflict_resolve_yes: false,
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
            notes_context: NotesContext::Notes,
            notes_edit_mode: NotesEditMode::Normal,
            notes_pending_op: None,
            agent_tx,
            agent_rx,
            git_task: None,
            git_task_queue: VecDeque::new(),
            git_task_tx,
            git_task_rx,
            status_refresh_in_flight: false,
            status_refresh_tx,
            status_refresh_rx,
            perf_debug: PerfDebugState::new(),
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

    fn cycle_worktree_graph_builder(&mut self) {
        self.worktree_graph_builder = self.worktree_graph_builder.next();
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

    fn queue_created_node_animation(&mut self, path: String) {
        let mut effects = EffectManager::default();
        effects.add_effect(build_node_create_effect());
        self.canvas_node_animations.push(CanvasNodeAnimation {
            target: CanvasNodeAnimationTarget::Path(path),
            kind: CanvasNodeAnimationKind::Created,
            effects,
            last_tick: Instant::now(),
        });
        if self.canvas_node_animations.len() > 48 {
            let keep_from = self.canvas_node_animations.len().saturating_sub(48);
            self.canvas_node_animations.drain(0..keep_from);
        }
    }

    fn queue_deleted_node_animation(&mut self, path: &str) {
        let Some(point) = self.last_worktree_node_points.get(path).copied() else {
            return;
        };
        let mut effects = EffectManager::default();
        effects.add_effect(build_node_delete_effect());
        self.canvas_node_animations.push(CanvasNodeAnimation {
            target: CanvasNodeAnimationTarget::Point(point),
            kind: CanvasNodeAnimationKind::Deleted,
            effects,
            last_tick: Instant::now(),
        });
        if self.canvas_node_animations.len() > 48 {
            let keep_from = self.canvas_node_animations.len().saturating_sub(48);
            self.canvas_node_animations.drain(0..keep_from);
        }
    }

    fn sync_worktree_animations(&mut self, new_paths: &BTreeSet<String>) {
        if !self.worktree_animations_ready {
            self.worktree_animations_ready = true;
            return;
        }

        let old_paths: BTreeSet<String> = self.worktrees.iter().map(|wt| wt.path.clone()).collect();

        for path in new_paths.difference(&old_paths) {
            self.queue_created_node_animation(path.clone());
        }
        for path in old_paths.difference(new_paths) {
            self.queue_deleted_node_animation(path.as_str());
        }
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
    run_startup_checks(&mut app);
    start_status_refresh_task(&mut app);
    refresh_worktrees(&mut app);

    let ui_tick_rate_fast = Duration::from_millis(16);
    let ui_tick_rate_normal = Duration::from_millis(33);
    let status_tick_rate = Duration::from_millis(1200);
    let worktree_tick_rate = Duration::from_millis(8000);
    let opencode_usage_tick_rate = Duration::from_millis(4000);
    let mut last_ui_tick = Instant::now();
    let mut last_status_tick = Instant::now();
    let mut last_worktree_tick = Instant::now();
    let mut last_opencode_usage_tick = Instant::now();
    let mut should_quit = false;

    while !should_quit {
        let loop_started = Instant::now();
        let frame_interval = loop_started.saturating_duration_since(app.perf_debug.last_frame_at);
        app.perf_debug.last_frame_at = loop_started;
        app.perf_debug.record_frame_interval(frame_interval);
        let mut loop_phases = FramePhaseDurations::default();

        let phase_started = Instant::now();
        drain_agent_events(&mut app);
        loop_phases.drain_agent_events = phase_started.elapsed();

        let phase_started = Instant::now();
        drain_git_task_events(&mut app);
        loop_phases.drain_git_task_events = phase_started.elapsed();

        drain_status_refresh_events(&mut app);

        let phase_started = Instant::now();
        refresh_agent_sessions(&mut app);
        loop_phases.refresh_agent_sessions = phase_started.elapsed();
        let can_refresh_opencode = app.view_mode == ViewMode::Worktrees
            && !matches!(app.mode, Mode::AgentPopup)
            && last_opencode_usage_tick.elapsed() >= opencode_usage_tick_rate;
        if can_refresh_opencode {
            let phase_started = Instant::now();
            refresh_opencode_usage(&mut app);
            loop_phases.refresh_opencode_usage = phase_started.elapsed();
            last_opencode_usage_tick = Instant::now();
        }

        if app.view_mode == ViewMode::Worktrees {
            refresh_spotify_now_playing(&mut app);
        }

        // Resize terminal session to match actual popup dimensions
        if matches!(app.mode, Mode::AgentPopup) {
            if let Some(path) = app.agent_popup_path.clone() {
                let phase_started = Instant::now();
                let size = terminal.size()?;
                let frame_area = Rect::new(0, 0, size.width, size.height);
                let (rows, cols) = calc_terminal_popup_size(frame_area);
                resize_terminal_session(&mut app, &path, rows, cols);
                loop_phases.resize_popup = phase_started.elapsed();
            }
        }

        let phase_started = Instant::now();
        terminal.draw(|frame| draw_ui(frame, &mut app))?;
        loop_phases.draw = phase_started.elapsed();

        let ui_tick_rate = if matches!(app.mode, Mode::AgentPopup) {
            ui_tick_rate_fast
        } else {
            ui_tick_rate_normal
        };
        let ui_timeout = ui_tick_rate.saturating_sub(last_ui_tick.elapsed());
        let status_timeout = status_tick_rate.saturating_sub(last_status_tick.elapsed());
        let worktree_timeout = worktree_tick_rate.saturating_sub(last_worktree_tick.elapsed());
        let timeout = ui_timeout.min(status_timeout).min(worktree_timeout);
        let poll_started = Instant::now();
        let has_event = event::poll(timeout)?;
        loop_phases.event_poll = poll_started.elapsed();
        if has_event {
            let handle_started = Instant::now();
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if !matches!(app.mode, Mode::AgentPopup)
                        && key.modifiers.contains(KeyModifiers::CONTROL)
                        && key.code == KeyCode::Char('c')
                    {
                        should_quit = true;
                        continue;
                    }

                    if matches!(app.mode, Mode::Normal)
                        && key.modifiers.contains(KeyModifiers::CONTROL)
                        && key.code == KeyCode::Char('l')
                    {
                        toggle_perf_debug(&mut app);
                        continue;
                    }

                    if matches!(app.mode, Mode::Normal)
                        && app.view_mode == ViewMode::Worktrees
                        && key.modifiers.contains(KeyModifiers::CONTROL)
                        && key.code == KeyCode::Char('b')
                    {
                        cycle_worktree_canvas_background(&mut app);
                        continue;
                    }

                    match app.mode {
                        Mode::Normal => {
                            should_quit = handle_normal_mode_key(&mut app, key)?;
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
                        Mode::WorktreeOrchestrateInput => {
                            handle_worktree_orchestrate_mode_key(&mut app, key.code);
                        }
                        Mode::WorktreeOrchestratePreview => {
                            handle_worktree_orchestrate_preview_mode_key(&mut app, key.code);
                        }
                        Mode::WorktreeOrchestratePromptEdit => {
                            handle_worktree_orchestrate_prompt_edit_mode_key(&mut app, key.code);
                        }
                        Mode::WorktreeBranchConflictConfirm => {
                            handle_branch_conflict_confirm_mode_key(&mut app, key.code)?;
                        }
                        Mode::WorktreeConflictResolveConfirm => {
                            handle_conflict_resolve_confirm_mode_key(&mut app, key.code)?;
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
            loop_phases.event_handle = handle_started.elapsed();
        }

        if last_ui_tick.elapsed() >= ui_tick_rate {
            last_ui_tick = Instant::now();
        }

        let refresh_git = !matches!(app.mode, Mode::AgentPopup)
            && app.git_task.is_none()
            && app.git_task_queue.is_empty();
        if refresh_git && last_status_tick.elapsed() >= status_tick_rate {
            let phase_started = Instant::now();
            if !app.status_refresh_in_flight {
                start_status_refresh_task(&mut app);
            }
            loop_phases.refresh_status = phase_started.elapsed();
            last_status_tick = Instant::now();
        }

        let refresh_worktree_state =
            refresh_git && matches!(app.mode, Mode::Normal) && app.view_mode == ViewMode::Worktrees;
        if refresh_worktree_state && last_worktree_tick.elapsed() >= worktree_tick_rate {
            let phase_started = Instant::now();
            refresh_worktrees(&mut app);
            loop_phases.refresh_worktrees = phase_started.elapsed();
            last_worktree_tick = Instant::now();
        }

        loop_phases.total_loop = loop_started.elapsed();
        app.perf_debug.record_loop_phases(loop_phases);
    }

    terminal.show_cursor()?;
    Ok(())
}

include!("sections/input.rs");
include!("sections/worktree_git.rs");
include!("sections/ui.rs");
