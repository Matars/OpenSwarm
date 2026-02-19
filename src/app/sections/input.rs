fn handle_normal_mode_key(app: &mut App, code: KeyCode) -> Result<bool, Box<dyn Error>> {
    if app.view_mode == ViewMode::Worktrees {
        return handle_worktree_mode_key(app, code);
    }

    match code {
        KeyCode::Char('q') => return Ok(request_quit(app)),
        KeyCode::Char('w') => {
            app.view_mode = ViewMode::Worktrees;
            app.worktree_focus = WorktreePane::Canvas;
            app.show_panel_help = false;
            app.status_line = "Switched to worktree navigator".to_string();
            refresh_worktrees(app);
        }
        KeyCode::Left | KeyCode::Char('h') => app.focus_left(),
        KeyCode::Right | KeyCode::Char('l') => app.focus_right(),
        KeyCode::Down | KeyCode::Char('j') => match app.active_pane {
            ActivePane::Files => {
                app.select_next();
                app.overview_scroll = 0;
                refresh_selected_overview(app);
            }
            ActivePane::Overview => {
                app.overview_scroll = app.overview_scroll.saturating_add(1);
            }
        },
        KeyCode::Up | KeyCode::Char('k') => match app.active_pane {
            ActivePane::Files => {
                app.select_prev();
                app.overview_scroll = 0;
                refresh_selected_overview(app);
            }
            ActivePane::Overview => {
                app.overview_scroll = app.overview_scroll.saturating_sub(1);
            }
        },
        KeyCode::Char('r') => refresh_status(app),
        KeyCode::Enter | KeyCode::Char(' ') => {
            toggle_stage(app)?;
            refresh_status(app);
        }
        KeyCode::Char('c') => {
            app.mode = Mode::CommitInput;
            app.commit_input.clear();
            app.status_line = "Commit mode: type a message and press Enter".to_string();
        }
        KeyCode::Char('n') => {
            open_notes_popup(app)?;
        }
        KeyCode::Char('p') => {
            let output = if let Some(path) = app.changes_worktree_path.as_deref() {
                push_with_upstream_at(path)?
            } else {
                push_with_upstream()?
            };
            app.status_line = output;
            refresh_status(app);
        }
        KeyCode::Char('s') => {
            app.status_line = run_git_in(
                app.changes_worktree_path.as_deref(),
                &["stash", "push", "--include-untracked"],
            )?;
            refresh_status(app);
        }
        KeyCode::Char('S') => {
            app.status_line = run_git_in(app.changes_worktree_path.as_deref(), &["stash", "pop"])?;
            refresh_status(app);
        }
        _ => {}
    }

    Ok(false)
}

fn handle_worktree_mode_key(app: &mut App, code: KeyCode) -> Result<bool, Box<dyn Error>> {
    match code {
        KeyCode::Char('q') => return Ok(request_quit(app)),
        KeyCode::Char('w') => {
            app.changes_worktree_path = app
                .selected_worktree()
                .map(|worktree| worktree.path.clone());
            app.view_mode = ViewMode::Changes;
            app.show_panel_help = false;
            if let Some(path) = app.changes_worktree_path.as_deref() {
                app.status_line = format!("Switched to changed files view for {}", path);
            } else {
                app.status_line = "Switched to changed files view".to_string();
            }
            refresh_status(app);
        }
        KeyCode::Tab => {
            app.next_worktree_pane();
            app.show_panel_help = false;
        }
        KeyCode::Char('?') => {
            app.show_panel_help = !app.show_panel_help;
        }
        KeyCode::Left => move_worktree_selection(app, NavDirection::Left),
        KeyCode::Right => move_worktree_selection(app, NavDirection::Right),
        KeyCode::Up => move_worktree_selection(app, NavDirection::Up),
        KeyCode::Down => move_worktree_selection(app, NavDirection::Down),
        KeyCode::Char('+') | KeyCode::Char('=') => zoom_worktree_canvas(app, true),
        KeyCode::Char('-') => zoom_worktree_canvas(app, false),
        KeyCode::Char('0') => reset_worktree_canvas_view(app),
        KeyCode::Char('W') => pan_worktree_canvas(app, 0.0, 1.0),
        KeyCode::Char('A') => pan_worktree_canvas(app, -1.0, 0.0),
        KeyCode::Char('S') => pan_worktree_canvas(app, 0.0, -1.0),
        KeyCode::Char('D') => pan_worktree_canvas(app, 1.0, 0.0),
        KeyCode::Char('h') => move_worktree_level_siblings(app, false),
        KeyCode::Char('l') => move_worktree_level_siblings(app, true),
        KeyCode::Char('L') => {
            open_worktree_git_log_popup(app)?;
        }
        KeyCode::Char('j') => move_worktree_level_vertical(app, false),
        KeyCode::Char('k') => move_worktree_level_vertical(app, true),
        KeyCode::Char('r') => {
            refresh_worktrees(app);
            app.status_line = "Refreshed worktree list".to_string();
        }
        KeyCode::Char('a') => {
            app.mode = Mode::WorktreeCreateInput;
            app.new_worktree_branch.clear();
            app.new_worktree_base = WorktreeCreateBase::Selected;
            app.status_line =
                "Create worktree: choose base with ←/→, then type branch name".to_string();
        }
        KeyCode::Char('o') => {
            open_terminal_popup_for_selected_worktree(app)?;
        }
        KeyCode::Char('O') => {
            open_agent_selector_for_selected_worktree(app, true)?;
        }
        KeyCode::Char('c') => {
            if let Some(path) = app.selected_worktree().map(|wt| wt.path.clone()) {
                app.mode = Mode::WorktreeCommitPushInput;
                app.worktree_commit_input.clear();
                app.worktree_commit_path = Some(path);
                app.status_line =
                    "Worktree commit mode: commit message, Enter to add/commit".to_string();
            }
        }
        KeyCode::Char('p') => {
            if let Some(path) = app.selected_worktree().map(|wt| wt.path.clone()) {
                app.status_line = push_with_upstream_at(path.as_str())?;
                refresh_worktrees(app);
                refresh_status(app);
            }
        }
        KeyCode::Char('f') => {
            app.status_line = update_connected_parent(app)?;
            refresh_worktrees(app);
            refresh_status(app);
        }
        KeyCode::Char('x') => {
            app.status_line = run_git(&["worktree", "prune"])?;
            refresh_worktrees(app);
        }
        KeyCode::Char('d') => {
            request_remove_selected_worktree(app)?;
        }
        KeyCode::Char('m') => {
            app.status_line = merge_selected_into_parent(app)?;
            refresh_worktrees(app);
            refresh_status(app);
        }
        KeyCode::Char('n') => {
            open_notes_popup(app)?;
        }
        _ => {}
    }

    Ok(false)
}

fn zoom_worktree_canvas(app: &mut App, zoom_in: bool) {
    const MIN_ZOOM: f64 = 0.65;
    const MAX_ZOOM: f64 = 3.4;
    const STEP: f64 = 1.2;

    if zoom_in {
        app.worktree_canvas_zoom = (app.worktree_canvas_zoom * STEP).clamp(MIN_ZOOM, MAX_ZOOM);
    } else {
        app.worktree_canvas_zoom = (app.worktree_canvas_zoom / STEP).clamp(MIN_ZOOM, MAX_ZOOM);
    }

    app.status_line = format!("Canvas zoom: {:.2}x", app.worktree_canvas_zoom);
}

fn pan_worktree_canvas(app: &mut App, dx: f64, dy: f64) {
    let step = 0.18 / app.worktree_canvas_zoom.max(0.65);
    app.worktree_canvas_pan_x = (app.worktree_canvas_pan_x + dx * step).clamp(-1.8, 1.8);
    app.worktree_canvas_pan_y = (app.worktree_canvas_pan_y + dy * step).clamp(-1.8, 1.8);
    app.status_line = format!(
        "Canvas pan: x={:+.2} y={:+.2}",
        app.worktree_canvas_pan_x, app.worktree_canvas_pan_y
    );
}

fn reset_worktree_canvas_view(app: &mut App) {
    app.worktree_canvas_zoom = 1.0;
    app.worktree_canvas_pan_x = 0.0;
    app.worktree_canvas_pan_y = 0.0;
    app.status_line = "Canvas view reset".to_string();
}

fn open_terminal_popup_for_selected_worktree(app: &mut App) -> Result<(), Box<dyn Error>> {
    let Some(path) = app.selected_worktree().map(|wt| wt.path.clone()) else {
        app.status_line = "No worktree selected".to_string();
        return Ok(());
    };
    open_terminal_popup_for_path(app, path.as_str())?;
    Ok(())
}

fn open_terminal_popup_for_path(app: &mut App, path: &str) -> Result<(), Box<dyn Error>> {
    app.agent_popup_path = Some(path.to_string());
    app.mode = Mode::AgentPopup;
    app.terminal_popup_mode = TerminalPopupMode::Input;
    if !has_live_terminal_session(app, path) {
        launch_shell_session(app, path)?;
    } else {
        app.status_line = "Reopened terminal session".to_string();
    }
    Ok(())
}

fn open_agent_selector_for_selected_worktree(
    app: &mut App,
    allow_default_launch: bool,
) -> Result<(), Box<dyn Error>> {
    refresh_runtime_settings(app);

    let selected_path = app.selected_worktree().map(|wt| wt.path.clone());
    let conflict_path = app
        .pending_conflict_context
        .as_ref()
        .map(|ctx| ctx.parent_path.clone());
    let target_path = conflict_path.or(selected_path);

    let Some(path) = target_path else {
        app.status_line = "No worktree selected".to_string();
        return Ok(());
    };

    if app.detected_agents.is_empty() {
        app.status_line =
            "No supported agent CLI found (expected: opencode or claude). Opened shell."
                .to_string();
        open_terminal_popup_for_path(app, path.as_str())?;
        return Ok(());
    }

    if allow_default_launch {
        if let Some(default_agent) = app.config.default_agent {
            if app.detected_agents.contains(&default_agent) {
                open_terminal_popup_for_path(app, path.as_str())?;
                launch_agent_in_terminal(app, path.as_str(), default_agent)?;
                return Ok(());
            }
            app.status_line = format!(
                "Configured default agent '{}' is not installed; choose one in picker",
                default_agent.command_name()
            );
        }
    }

    app.agent_select_path = Some(path);
    app.agent_select_index = app
        .config
        .default_agent
        .and_then(|agent| app.detected_agents.iter().position(|item| *item == agent))
        .unwrap_or_else(|| {
            app.agent_select_index
                .min(app.detected_agents.len().saturating_sub(1))
        });

    app.mode = Mode::AgentSelectPopup;
    app.status_line = format!("Choose an agent (config: {})", app.config.config_path);
    Ok(())
}

fn handle_agent_select_mode_key(app: &mut App, code: KeyCode) -> Result<(), Box<dyn Error>> {
    match code {
        KeyCode::Esc => {
            app.mode = Mode::Normal;
            app.agent_select_path = None;
            app.status_line = "Agent selection cancelled".to_string();
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if app.agent_select_index > 0 {
                app.agent_select_index -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.agent_select_index + 1 < app.detected_agents.len() {
                app.agent_select_index += 1;
            }
        }
        KeyCode::Enter => {
            let Some(agent) = app.detected_agents.get(app.agent_select_index).copied() else {
                app.mode = Mode::Normal;
                app.agent_select_path = None;
                app.status_line = "No available agent selected".to_string();
                return Ok(());
            };

            let Some(path) = app.agent_select_path.clone() else {
                app.mode = Mode::Normal;
                app.status_line = "Missing target worktree for agent launch".to_string();
                return Ok(());
            };

            open_terminal_popup_for_path(app, path.as_str())?;
            launch_agent_in_terminal(app, path.as_str(), agent)?;
            app.agent_select_path = None;
        }
        _ => {}
    }

    Ok(())
}

fn launch_agent_in_terminal(
    app: &mut App,
    path: &str,
    agent: ExternalAgent,
) -> Result<(), Box<dyn Error>> {
    let launch_cmd = format!("{}\r", agent.command_name());
    write_to_agent(app, path, launch_cmd.as_str())?;

    if let Some(context) = app.pending_conflict_context.as_ref() {
        if context.parent_path == path {
            let prompt = build_conflict_resolve_prompt(app, context);
            let prompt_with_enter = format!("{}\r", normalize_terminal_newlines(prompt.as_str()));
            write_to_agent(app, path, prompt_with_enter.as_str())?;
            app.pending_conflict_context = None;
            app.status_line = format!(
                "Launched {} and pasted conflict-resolution prompt",
                agent.display_name()
            );
            return Ok(());
        }
    }

    app.status_line = format!("Launched {} in terminal", agent.display_name());
    Ok(())
}

fn shell_ansi_c_quote(text: &str) -> String {
    let mut out = String::from("$'");
    for ch in text.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '\'' => out.push_str("\\'"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => {
                out.push_str(format!("\\x{:02x}", c as u32).as_str());
            }
            c => out.push(c),
        }
    }
    out.push('\'');
    out
}

fn launch_opencode_conflict_resolution(app: &mut App) -> Result<(), Box<dyn Error>> {
    refresh_runtime_settings(app);

    let Some(context) = app.pending_conflict_context.clone() else {
        app.mode = Mode::Normal;
        app.confirm_conflict_resolve_yes = false;
        app.status_line = "No pending merge-conflict context found".to_string();
        return Ok(());
    };

    if !command_exists_on_path("opencode") {
        open_terminal_popup_for_path(app, context.parent_path.as_str())?;
        app.status_line =
            "OpenCode CLI not found on PATH; opened shell in conflicted parent worktree"
                .to_string();
        return Ok(());
    }

    open_terminal_popup_for_path(app, context.parent_path.as_str())?;
    let prompt = build_conflict_resolve_prompt(app, &context);
    let cmd = format!(
        "opencode --prompt {}\r",
        shell_ansi_c_quote(prompt.as_str())
    );
    write_to_agent(app, context.parent_path.as_str(), cmd.as_str())?;
    app.pending_conflict_context = None;
    app.confirm_conflict_resolve_yes = false;
    app.status_line = "Launched OpenCode with configured conflict prompt".to_string();
    Ok(())
}

fn normalize_terminal_newlines(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\n', "\r")
}

fn refresh_runtime_settings(app: &mut App) {
    app.config = load_openswarm_config();
    app.detected_agents = detect_available_agents();
    app.agent_select_index = app
        .agent_select_index
        .min(app.detected_agents.len().saturating_sub(1));
}

fn detect_available_agents() -> Vec<ExternalAgent> {
    let mut out = Vec::new();
    if command_exists_on_path("opencode") {
        out.push(ExternalAgent::Opencode);
    }
    if command_exists_on_path("claude") {
        out.push(ExternalAgent::Claude);
    }
    out
}

fn command_exists_on_path(command: &str) -> bool {
    let Some(paths) = std::env::var_os("PATH") else {
        return false;
    };

    for dir in std::env::split_paths(paths.as_os_str()) {
        let candidate = dir.join(command);
        if candidate.is_file() {
            return true;
        }
    }

    false
}

fn build_conflict_resolve_prompt(app: &App, context: &ConflictResolveContext) -> String {
    let template =
        load_conflict_prompt_template(app).unwrap_or_else(default_conflict_prompt_template);
    let files = if context.conflicted_files.is_empty() {
        "(none reported)".to_string()
    } else {
        context.conflicted_files.join("\n")
    };

    template
        .replace("{parent_path}", context.parent_path.as_str())
        .replace("{source_branch}", context.source_branch.as_str())
        .replace("{target_branch}", context.target_branch.as_str())
        .replace("{conflicted_files}", files.as_str())
}

fn load_conflict_prompt_template(app: &App) -> Option<String> {
    let prompt_path = app.config.conflict_resolve_prompt_path.as_str();

    if !Path::new(prompt_path).exists() {
        if let Some(parent) = Path::new(prompt_path).parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(prompt_path, default_conflict_prompt_template());
    }

    fs::read_to_string(prompt_path).ok()
}

fn load_openswarm_config() -> OpenSwarmConfig {
    let config_dir = openswarm_config_dir();
    let prompts_dir = config_dir.join("prompts");
    let config_path = config_dir.join("config.toml");

    let _ = fs::create_dir_all(prompts_dir.as_path());
    if !config_path.exists() {
        let _ = fs::write(config_path.as_path(), default_openswarm_config_text());
    }

    let mut default_agent: Option<ExternalAgent> = None;
    let mut conflict_prompt_path = prompts_dir.join("conflict-resolve-prompt.md");

    if let Ok(raw) = fs::read_to_string(config_path.as_path()) {
        for line in raw.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let Some((key, value)) = trimmed.split_once('=') else {
                continue;
            };
            let key = key.trim();
            let value = parse_config_string(value.trim());
            match key {
                "default_agent" => {
                    default_agent = value
                        .as_deref()
                        .and_then(parse_external_agent)
                        .or(default_agent);
                }
                "conflict_resolve_prompt" => {
                    if let Some(v) = value {
                        let candidate = PathBuf::from(v.as_str());
                        conflict_prompt_path = if candidate.is_absolute() {
                            candidate
                        } else {
                            config_dir.join(candidate)
                        };
                    }
                }
                _ => {}
            }
        }
    }

    if !conflict_prompt_path.exists() {
        if let Some(parent) = conflict_prompt_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(
            conflict_prompt_path.as_path(),
            default_conflict_prompt_template(),
        );
    }

    OpenSwarmConfig {
        config_path: config_path.to_string_lossy().to_string(),
        default_agent,
        conflict_resolve_prompt_path: conflict_prompt_path.to_string_lossy().to_string(),
    }
}

fn openswarm_config_dir() -> PathBuf {
    if let Some(home) = std::env::var_os("HOME") {
        PathBuf::from(home).join(".config").join("openswarm")
    } else {
        PathBuf::from(".").join(".config").join("openswarm")
    }
}

fn default_openswarm_config_text() -> String {
    "# OpenSwarm config\n# default_agent accepts: \"\", \"opencode\", \"claude\"\n# empty default_agent means always show the picker on Shift+O\ndefault_agent = \"\"\n\n# relative paths are resolved from ~/.config/openswarm\nconflict_resolve_prompt = \"prompts/conflict-resolve-prompt.md\"\n"
        .to_string()
}

fn parse_config_string(value: &str) -> Option<String> {
    let head = value.split('#').next().unwrap_or(value).trim();
    if head.is_empty() {
        return None;
    }
    if let Some(stripped) = head.strip_prefix('"').and_then(|v| v.strip_suffix('"')) {
        return Some(stripped.to_string());
    }
    Some(head.to_string())
}

fn parse_external_agent(value: &str) -> Option<ExternalAgent> {
    match value.trim().to_ascii_lowercase().as_str() {
        "opencode" => Some(ExternalAgent::Opencode),
        "claude" => Some(ExternalAgent::Claude),
        _ => None,
    }
}

fn default_conflict_prompt_template() -> String {
    "Resolve the current Git merge conflict in this worktree.\n\nContext:\n- Parent worktree path: {parent_path}\n- Merge source branch: {source_branch}\n- Merge target branch: {target_branch}\n- Conflicted files:\n{conflicted_files}\n\nInstructions:\n1) Inspect conflict markers and resolve carefully; prefer minimal safe edits.\n2) Keep intended behavior from both branches when possible.\n3) Run `git diff --name-only --diff-filter=U` and ensure it is empty.\n4) Stage resolved files with `git add`.\n5) Summarize what was resolved and any risks.\n6) Do not push. Stop after conflicts are resolved and staged."
        .to_string()
}

fn request_quit(app: &mut App) -> bool {
    if live_terminal_session_count(app) == 0 {
        return true;
    }

    app.mode = Mode::QuitWithSessionsConfirm;
    app.confirm_quit_with_sessions_yes = false;
    app.status_line = "Active terminal sessions detected".to_string();
    false
}

fn live_terminal_session_count(app: &App) -> usize {
    app.agent_sessions
        .values()
        .filter(|session| session.child.is_some())
        .count()
}

fn handle_quit_with_sessions_mode_key(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Esc => {
            app.mode = Mode::Normal;
            app.confirm_quit_with_sessions_yes = false;
            app.status_line = "Quit cancelled".to_string();
        }
        KeyCode::Left | KeyCode::Right | KeyCode::Tab => {
            app.confirm_quit_with_sessions_yes = !app.confirm_quit_with_sessions_yes;
        }
        KeyCode::Char('y') => app.confirm_quit_with_sessions_yes = true,
        KeyCode::Char('n') => app.confirm_quit_with_sessions_yes = false,
        KeyCode::Enter => {
            if app.confirm_quit_with_sessions_yes {
                app.quit_now = true;
            } else {
                app.mode = Mode::Normal;
                app.status_line = "Quit cancelled".to_string();
            }
        }
        _ => {}
    }
}

fn handle_legacy_workspace_migrate_mode_key(
    app: &mut App,
    code: KeyCode,
) -> Result<(), Box<dyn Error>> {
    match code {
        KeyCode::Esc => {
            app.mode = Mode::Normal;
            app.confirm_legacy_workspace_migrate_yes = false;
            app.legacy_workspace_prompt_dismissed = true;
            app.status_line = "Legacy workspace migration skipped".to_string();
        }
        KeyCode::Left | KeyCode::Right | KeyCode::Tab => {
            app.confirm_legacy_workspace_migrate_yes = !app.confirm_legacy_workspace_migrate_yes;
        }
        KeyCode::Char('y') => app.confirm_legacy_workspace_migrate_yes = true,
        KeyCode::Char('n') => app.confirm_legacy_workspace_migrate_yes = false,
        KeyCode::Enter => {
            if app.confirm_legacy_workspace_migrate_yes {
                let root = app.pending_legacy_workspace_root.clone();
                app.status_line = migrate_legacy_workspace_layout(root.as_str())?;
                app.legacy_workspace_prompt_dismissed = true;
                refresh_worktrees(app);
                refresh_status(app);
            } else {
                app.status_line = "Legacy workspace migration skipped".to_string();
                app.legacy_workspace_prompt_dismissed = true;
            }

            app.mode = Mode::Normal;
            app.confirm_legacy_workspace_migrate_yes = false;
            app.pending_legacy_workspace_root.clear();
            app.pending_legacy_workspace_path.clear();
            app.pending_new_workspace_path.clear();
        }
        _ => {}
    }

    Ok(())
}

fn handle_worktree_create_mode_key(app: &mut App, code: KeyCode) -> Result<(), Box<dyn Error>> {
    match code {
        KeyCode::Esc => {
            app.mode = Mode::Normal;
            app.status_line = "Create worktree cancelled".to_string();
        }
        KeyCode::Left => {
            app.cycle_worktree_base_left();
        }
        KeyCode::Right => {
            app.cycle_worktree_base_right();
        }
        KeyCode::Enter => {
            let branch = app.new_worktree_branch.trim();
            if branch.is_empty() {
                app.status_line = "Branch name is required".to_string();
            } else {
                let root = create_root_for_app(app);
                if branch_exists(root.as_str(), branch) {
                    app.pending_create_branch = branch.to_string();
                    app.confirm_delete_branch_yes = false;
                    app.mode = Mode::WorktreeBranchConflictConfirm;
                    app.status_line = format!(
                        "Branch '{}' already exists. Confirm delete and recreate.",
                        branch
                    );
                    return Ok(());
                }

                app.status_line = create_worktree(app, branch)?;
                refresh_worktrees(app);
                refresh_status(app);
            }
            app.mode = Mode::Normal;
            app.new_worktree_branch.clear();
        }
        KeyCode::Backspace => {
            app.new_worktree_branch.pop();
        }
        KeyCode::Char(c) => {
            app.new_worktree_branch.push(c);
        }
        _ => {}
    }

    Ok(())
}

fn handle_branch_conflict_confirm_mode_key(
    app: &mut App,
    code: KeyCode,
) -> Result<(), Box<dyn Error>> {
    match code {
        KeyCode::Esc => {
            app.mode = Mode::Normal;
            app.confirm_delete_branch_yes = false;
            app.pending_create_branch.clear();
            app.status_line = "Create worktree cancelled".to_string();
        }
        KeyCode::Left | KeyCode::Right | KeyCode::Tab => {
            app.confirm_delete_branch_yes = !app.confirm_delete_branch_yes;
        }
        KeyCode::Char('y') => app.confirm_delete_branch_yes = true,
        KeyCode::Char('n') => app.confirm_delete_branch_yes = false,
        KeyCode::Enter => {
            if app.confirm_delete_branch_yes {
                let branch = app.pending_create_branch.clone();
                let root = create_root_for_app(app);
                app.status_line =
                    delete_branch_and_create_worktree(app, root.as_str(), branch.as_str())?;
                refresh_worktrees(app);
                refresh_status(app);
            } else {
                app.status_line = "Create worktree cancelled (kept existing branch)".to_string();
            }

            app.mode = Mode::Normal;
            app.confirm_delete_branch_yes = false;
            app.pending_create_branch.clear();
            app.new_worktree_branch.clear();
        }
        _ => {}
    }

    Ok(())
}

fn handle_conflict_resolve_confirm_mode_key(
    app: &mut App,
    code: KeyCode,
) -> Result<(), Box<dyn Error>> {
    match code {
        KeyCode::Esc => {
            app.mode = Mode::Normal;
            app.confirm_conflict_resolve_yes = false;
            app.status_line =
                "Agent conflict resolution cancelled (you can still resolve manually)".to_string();
        }
        KeyCode::Left | KeyCode::Right | KeyCode::Tab => {
            app.confirm_conflict_resolve_yes = !app.confirm_conflict_resolve_yes;
        }
        KeyCode::Char('y') => app.confirm_conflict_resolve_yes = true,
        KeyCode::Char('n') => app.confirm_conflict_resolve_yes = false,
        KeyCode::Char('e') | KeyCode::Char('E') => {
            open_conflict_prompt_editor(app)?;
        }
        KeyCode::Enter => {
            if app.confirm_conflict_resolve_yes {
                launch_opencode_conflict_resolution(app)?;
            } else {
                app.mode = Mode::Normal;
                app.confirm_conflict_resolve_yes = false;
                app.status_line =
                    "Agent conflict resolution skipped (resolve manually or press Shift+O)"
                        .to_string();
            }
        }
        _ => {}
    }

    Ok(())
}

fn request_remove_selected_worktree(app: &mut App) -> Result<(), Box<dyn Error>> {
    let Some(selected) = app.selected_worktree().cloned() else {
        app.status_line = "No worktree selected".to_string();
        return Ok(());
    };

    if selected.is_current {
        app.status_line = "Refusing to remove current worktree".to_string();
        return Ok(());
    }

    if selected.dirty {
        app.pending_remove_worktree_path = selected.path;
        app.confirm_remove_worktree_yes = false;
        app.mode = Mode::WorktreeRemoveDirtyConfirm;
        app.status_line = "Selected worktree has uncommitted changes".to_string();
        return Ok(());
    }

    app.status_line = remove_selected_worktree(app)?;
    refresh_worktrees(app);
    refresh_status(app);
    Ok(())
}

fn handle_worktree_remove_dirty_confirm_mode_key(
    app: &mut App,
    code: KeyCode,
) -> Result<(), Box<dyn Error>> {
    match code {
        KeyCode::Esc => {
            app.mode = Mode::Normal;
            app.confirm_remove_worktree_yes = false;
            app.pending_remove_worktree_path.clear();
            app.status_line = "Delete cancelled".to_string();
        }
        KeyCode::Left | KeyCode::Right | KeyCode::Tab => {
            app.confirm_remove_worktree_yes = !app.confirm_remove_worktree_yes;
        }
        KeyCode::Char('y') => app.confirm_remove_worktree_yes = true,
        KeyCode::Char('n') => app.confirm_remove_worktree_yes = false,
        KeyCode::Enter => {
            if app.confirm_remove_worktree_yes {
                let path = app.pending_remove_worktree_path.clone();
                if path.is_empty() {
                    app.status_line = "Delete cancelled (missing worktree path)".to_string();
                } else {
                    app.status_line = remove_worktree_by_path(app, path.as_str(), true)?;
                    refresh_worktrees(app);
                    refresh_status(app);
                }
            } else {
                app.status_line = "Delete cancelled".to_string();
            }

            app.mode = Mode::Normal;
            app.confirm_remove_worktree_yes = false;
            app.pending_remove_worktree_path.clear();
        }
        _ => {}
    }

    Ok(())
}

fn open_worktree_git_log_popup(app: &mut App) -> Result<(), Box<dyn Error>> {
    let Some(path) = app.selected_worktree().map(|wt| wt.path.clone()) else {
        app.status_line = "No worktree selected".to_string();
        return Ok(());
    };

    app.git_log_popup_path = Some(path.clone());
    app.git_log_lines = load_worktree_git_log(path.as_str())?;
    app.git_log_scroll = 0;
    app.show_panel_help = false;
    app.mode = Mode::WorktreeGitLogPopup;
    app.status_line = format!("Opened git log popup for {}", path);
    Ok(())
}

fn load_worktree_git_log(path: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let output = Command::new("git")
        .args([
            "-C",
            path,
            "reflog",
            "--date=relative",
            "--decorate",
            "--max-count",
            "80",
            "--pretty=format:%h %gd (%cr) %gs",
        ])
        .output()?;

    if output.status.success() {
        let text = sanitize_for_tui(String::from_utf8_lossy(&output.stdout).as_ref())
            .lines()
            .map(|line| line.trim_end().to_string())
            .filter(|line| !line.is_empty())
            .collect::<Vec<String>>();

        if text.is_empty() {
            Ok(vec![
                "(no reflog entries found for this worktree)".to_string()
            ])
        } else {
            Ok(text)
        }
    } else {
        let stderr = sanitize_for_tui(String::from_utf8_lossy(&output.stderr).as_ref())
            .trim()
            .to_string();
        let stdout = sanitize_for_tui(String::from_utf8_lossy(&output.stdout).as_ref())
            .trim()
            .to_string();
        let reason = if !stderr.is_empty() { stderr } else { stdout };
        Ok(vec![format!("Failed to load git reflog: {}", reason)])
    }
}

fn handle_worktree_git_log_mode_key(app: &mut App, code: KeyCode) {
    let max_scroll = app.git_log_lines.len().saturating_sub(1) as u16;
    match code {
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('L') => {
            app.mode = Mode::Normal;
            app.git_log_popup_path = None;
            app.git_log_lines.clear();
            app.git_log_scroll = 0;
            app.status_line = "Closed git log popup".to_string();
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.git_log_scroll = app.git_log_scroll.saturating_add(1).min(max_scroll);
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.git_log_scroll = app.git_log_scroll.saturating_sub(1);
        }
        KeyCode::PageDown => {
            app.git_log_scroll = app.git_log_scroll.saturating_add(8).min(max_scroll);
        }
        KeyCode::PageUp => {
            app.git_log_scroll = app.git_log_scroll.saturating_sub(8);
        }
        KeyCode::Home => {
            app.git_log_scroll = 0;
        }
        KeyCode::End => {
            app.git_log_scroll = max_scroll;
        }
        _ => {}
    }
}

fn handle_agent_popup_key(app: &mut App, key: KeyEvent) -> Result<(), Box<dyn Error>> {
    let code = key.code;
    let Some(path) = app.agent_popup_path.clone() else {
        app.mode = Mode::Normal;
        return Ok(());
    };

    if !has_live_terminal_session(app, path.as_str()) {
        launch_shell_session(app, path.as_str())?;
    }

    if is_terminal_mode_toggle(key) {
        app.terminal_popup_mode = match app.terminal_popup_mode {
            TerminalPopupMode::Input => TerminalPopupMode::Control,
            TerminalPopupMode::Control => TerminalPopupMode::Input,
        };
        return Ok(());
    }

    if app.terminal_popup_mode == TerminalPopupMode::Control {
        match code {
            KeyCode::Esc => {
                app.mode = Mode::Normal;
                app.agent_popup_path = None;
                app.status_line = "Terminal session moved to background".to_string();
            }
            KeyCode::Char('q') => {
                terminate_terminal_session(app, path.as_str());
                app.mode = Mode::Normal;
                app.agent_popup_path = None;
                app.status_line = "Terminal session closed".to_string();
            }
            KeyCode::Char('r') => {
                app.agent_sessions.remove(path.as_str());
                launch_shell_session(app, path.as_str())?;
                app.status_line = "Terminal restarted".to_string();
            }
            _ => {}
        }
        return Ok(());
    }

    match code {
        KeyCode::Tab => {
            write_to_agent(app, path.as_str(), "\t")?;
        }
        KeyCode::Left => {
            write_to_agent(app, path.as_str(), "\x1b[D")?;
        }
        KeyCode::Right => {
            write_to_agent(app, path.as_str(), "\x1b[C")?;
        }
        KeyCode::Up => {
            write_to_agent(app, path.as_str(), "\x1b[A")?;
        }
        KeyCode::Down => {
            write_to_agent(app, path.as_str(), "\x1b[B")?;
        }
        KeyCode::Home => {
            write_to_agent(app, path.as_str(), "\x1b[H")?;
        }
        KeyCode::End => {
            write_to_agent(app, path.as_str(), "\x1b[F")?;
        }
        KeyCode::PageUp => {
            write_to_agent(app, path.as_str(), "\x1b[5~")?;
        }
        KeyCode::PageDown => {
            write_to_agent(app, path.as_str(), "\x1b[6~")?;
        }
        KeyCode::Delete => {
            write_to_agent(app, path.as_str(), "\x1b[3~")?;
        }
        KeyCode::Backspace => {
            write_to_agent(app, path.as_str(), "\x7f")?;
        }
        KeyCode::Enter => {
            write_to_agent(app, path.as_str(), "\r")?;
        }
        KeyCode::Char(c) => {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                if let Some(seq) = control_seq(c) {
                    write_to_agent(app, path.as_str(), seq)?;
                }
            } else {
                let mut s = String::new();
                s.push(c);
                write_to_agent(app, path.as_str(), s.as_str())?;
            }
        }
        _ => {}
    }

    Ok(())
}

fn is_terminal_mode_toggle(key: KeyEvent) -> bool {
    matches!(key.code, KeyCode::Char('g') | KeyCode::Char('G'))
        && key
            .modifiers
            .intersects(KeyModifiers::CONTROL | KeyModifiers::SUPER)
}

fn open_notes_popup(app: &mut App) -> Result<(), Box<dyn Error>> {
    let path = notes_file_path();
    let notes_file = Path::new(path.as_str());
    if !notes_file.exists() {
        fs::write(notes_file, "# Notes\n")?;
    }

    let content = fs::read_to_string(notes_file)?;
    let mut lines = content
        .split('\n')
        .map(|line| line.to_string())
        .collect::<Vec<String>>();
    if lines.is_empty() {
        lines.push(String::new());
    }

    app.notes_path = path;
    app.notes_lines = lines;
    app.notes_cursor_row = app.notes_lines.len().saturating_sub(1);
    app.notes_cursor_col = line_char_len(app.notes_lines[app.notes_cursor_row].as_str());
    app.notes_scroll = 0;
    app.notes_context = NotesContext::Notes;
    app.mode = Mode::NotesPopup;
    app.status_line = format!("Opened {}", app.notes_path);
    Ok(())
}

fn open_conflict_prompt_editor(app: &mut App) -> Result<(), Box<dyn Error>> {
    refresh_runtime_settings(app);
    let prompt_path = app.config.conflict_resolve_prompt_path.clone();

    if !Path::new(prompt_path.as_str()).exists() {
        if let Some(parent) = Path::new(prompt_path.as_str()).parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(prompt_path.as_str(), default_conflict_prompt_template())?;
    }

    let content = fs::read_to_string(prompt_path.as_str())?;
    let mut lines = content
        .split('\n')
        .map(|line| line.to_string())
        .collect::<Vec<String>>();
    if lines.is_empty() {
        lines.push(String::new());
    }

    app.notes_path = prompt_path;
    app.notes_lines = lines;
    app.notes_cursor_row = app.notes_lines.len().saturating_sub(1);
    app.notes_cursor_col = line_char_len(app.notes_lines[app.notes_cursor_row].as_str());
    app.notes_scroll = 0;
    app.notes_context = NotesContext::ConflictPrompt;
    app.mode = Mode::NotesPopup;
    app.status_line = format!("Editing conflict prompt: {}", app.notes_path);
    Ok(())
}

fn save_notes_popup(app: &mut App) -> Result<(), Box<dyn Error>> {
    if app.notes_lines.is_empty() {
        app.notes_lines.push(String::new());
    }
    let body = app.notes_lines.join("\n");
    fs::write(app.notes_path.as_str(), body)?;
    app.status_line = match app.notes_context {
        NotesContext::Notes => format!("Saved {}", app.notes_path),
        NotesContext::ConflictPrompt => {
            format!("Saved conflict resolve prompt to {}", app.notes_path)
        }
    };
    Ok(())
}

fn handle_notes_popup_key(app: &mut App, key: KeyEvent) -> Result<(), Box<dyn Error>> {
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        match key.code {
            KeyCode::Char('s') => {
                save_notes_popup(app)?;
            }
            _ => {}
        }
        return Ok(());
    }

    if app.notes_lines.is_empty() {
        app.notes_lines.push(String::new());
    }

    match key.code {
        KeyCode::Esc => {
            save_notes_popup(app)?;
            match app.notes_context {
                NotesContext::Notes => {
                    app.mode = Mode::Normal;
                    app.status_line = format!("Saved notes to {}", app.notes_path);
                }
                NotesContext::ConflictPrompt => {
                    refresh_runtime_settings(app);
                    app.mode = if app.pending_conflict_context.is_some() {
                        Mode::WorktreeConflictResolveConfirm
                    } else {
                        Mode::Normal
                    };
                    app.status_line =
                        "Saved conflict prompt. Press Enter to launch OpenCode".to_string();
                }
            }
        }
        KeyCode::Up => {
            if app.notes_cursor_row > 0 {
                app.notes_cursor_row -= 1;
            }
            clamp_notes_cursor(app);
        }
        KeyCode::Down => {
            if app.notes_cursor_row + 1 < app.notes_lines.len() {
                app.notes_cursor_row += 1;
            }
            clamp_notes_cursor(app);
        }
        KeyCode::Left => {
            if app.notes_cursor_col > 0 {
                app.notes_cursor_col -= 1;
            } else if app.notes_cursor_row > 0 {
                app.notes_cursor_row -= 1;
                app.notes_cursor_col =
                    line_char_len(app.notes_lines[app.notes_cursor_row].as_str());
            }
        }
        KeyCode::Right => {
            let line_len = line_char_len(app.notes_lines[app.notes_cursor_row].as_str());
            if app.notes_cursor_col < line_len {
                app.notes_cursor_col += 1;
            } else if app.notes_cursor_row + 1 < app.notes_lines.len() {
                app.notes_cursor_row += 1;
                app.notes_cursor_col = 0;
            }
        }
        KeyCode::Home => {
            app.notes_cursor_col = 0;
        }
        KeyCode::End => {
            app.notes_cursor_col = line_char_len(app.notes_lines[app.notes_cursor_row].as_str());
        }
        KeyCode::PageUp => {
            app.notes_cursor_row = app.notes_cursor_row.saturating_sub(10);
            clamp_notes_cursor(app);
        }
        KeyCode::PageDown => {
            let max_row = app.notes_lines.len().saturating_sub(1);
            app.notes_cursor_row = (app.notes_cursor_row + 10).min(max_row);
            clamp_notes_cursor(app);
        }
        KeyCode::Enter => {
            let current = app.notes_lines[app.notes_cursor_row].clone();
            let split_idx = char_to_byte_idx(current.as_str(), app.notes_cursor_col);
            let before = current[..split_idx].to_string();
            let after = current[split_idx..].to_string();
            app.notes_lines[app.notes_cursor_row] = before;
            app.notes_lines.insert(app.notes_cursor_row + 1, after);
            app.notes_cursor_row += 1;
            app.notes_cursor_col = 0;
        }
        KeyCode::Backspace => {
            if app.notes_cursor_col > 0 {
                let line = &mut app.notes_lines[app.notes_cursor_row];
                let end = char_to_byte_idx(line.as_str(), app.notes_cursor_col);
                let start = char_to_byte_idx(line.as_str(), app.notes_cursor_col - 1);
                line.replace_range(start..end, "");
                app.notes_cursor_col -= 1;
            } else if app.notes_cursor_row > 0 {
                let current = app.notes_lines.remove(app.notes_cursor_row);
                app.notes_cursor_row -= 1;
                app.notes_cursor_col =
                    line_char_len(app.notes_lines[app.notes_cursor_row].as_str());
                app.notes_lines[app.notes_cursor_row].push_str(current.as_str());
            }
        }
        KeyCode::Delete => {
            let row = app.notes_cursor_row;
            let col = app.notes_cursor_col;
            let line_len = line_char_len(app.notes_lines[row].as_str());
            if col < line_len {
                let line = &mut app.notes_lines[row];
                let start = char_to_byte_idx(line.as_str(), col);
                let end = char_to_byte_idx(line.as_str(), col + 1);
                line.replace_range(start..end, "");
            } else if row + 1 < app.notes_lines.len() {
                let next = app.notes_lines.remove(row + 1);
                app.notes_lines[row].push_str(next.as_str());
            }
        }
        KeyCode::Tab => {
            let line = &mut app.notes_lines[app.notes_cursor_row];
            let idx = char_to_byte_idx(line.as_str(), app.notes_cursor_col);
            line.insert_str(idx, "    ");
            app.notes_cursor_col += 4;
        }
        KeyCode::Char(c) => {
            let line = &mut app.notes_lines[app.notes_cursor_row];
            let idx = char_to_byte_idx(line.as_str(), app.notes_cursor_col);
            line.insert(idx, c);
            app.notes_cursor_col += 1;
        }
        _ => {}
    }

    if app.notes_lines.is_empty() {
        app.notes_lines.push(String::new());
        app.notes_cursor_row = 0;
        app.notes_cursor_col = 0;
    }

    if app.notes_cursor_row < app.notes_scroll as usize {
        app.notes_scroll = app.notes_cursor_row as u16;
    }

    Ok(())
}

fn clamp_notes_cursor(app: &mut App) {
    if app.notes_lines.is_empty() {
        app.notes_lines.push(String::new());
        app.notes_cursor_row = 0;
        app.notes_cursor_col = 0;
        return;
    }
    let max_row = app.notes_lines.len().saturating_sub(1);
    app.notes_cursor_row = app.notes_cursor_row.min(max_row);
    let max_col = line_char_len(app.notes_lines[app.notes_cursor_row].as_str());
    app.notes_cursor_col = app.notes_cursor_col.min(max_col);
}

fn line_char_len(text: &str) -> usize {
    text.chars().count()
}

fn char_to_byte_idx(text: &str, char_idx: usize) -> usize {
    text.char_indices()
        .nth(char_idx)
        .map(|(idx, _)| idx)
        .unwrap_or(text.len())
}

fn notes_file_path() -> String {
    let root = repo_container_from_path(".")
        .or_else(repo_root)
        .unwrap_or_else(|| ".".to_string());
    format!("{}/notes.md", root)
}

fn terminate_terminal_session(app: &mut App, path: &str) {
    if let Some(mut session) = app.agent_sessions.remove(path) {
        if let Some(mut child) = session.child.take() {
            let _ = child.kill();
        }
    }
}

fn control_seq(c: char) -> Option<&'static str> {
    match c.to_ascii_lowercase() {
        'a' => Some("\x01"),
        'b' => Some("\x02"),
        'c' => Some("\x03"),
        'd' => Some("\x04"),
        'e' => Some("\x05"),
        'f' => Some("\x06"),
        'g' => Some("\x07"),
        'h' => Some("\x08"),
        'i' => Some("\x09"),
        'j' => Some("\x0A"),
        'k' => Some("\x0B"),
        'l' => Some("\x0C"),
        'm' => Some("\x0D"),
        'n' => Some("\x0E"),
        'o' => Some("\x0F"),
        'p' => Some("\x10"),
        'q' => Some("\x11"),
        'r' => Some("\x12"),
        's' => Some("\x13"),
        't' => Some("\x14"),
        'u' => Some("\x15"),
        'v' => Some("\x16"),
        'w' => Some("\x17"),
        'x' => Some("\x18"),
        'y' => Some("\x19"),
        'z' => Some("\x1A"),
        _ => None,
    }
}

fn has_live_terminal_session(app: &App, path: &str) -> bool {
    app.agent_sessions
        .get(path)
        .map(|session| session.child.is_some() && session.writer.is_some())
        .unwrap_or(false)
}

fn launch_shell_session(app: &mut App, path: &str) -> Result<(), Box<dyn Error>> {
    const TERM_ROWS: u16 = 44;
    const TERM_COLS: u16 = 150;

    let pty_system = native_pty_system();
    let pair = pty_system.openpty(PtySize {
        rows: TERM_ROWS,
        cols: TERM_COLS,
        pixel_width: 0,
        pixel_height: 0,
    })?;

    let shell = std::env::var("SHELL").unwrap_or_else(|_| "zsh".to_string());
    let mut cmd = CommandBuilder::new(shell.as_str());
    cmd.arg("-i");
    cmd.arg("-l");
    cmd.cwd(path);
    let child = pair.slave.spawn_command(cmd)?;

    let tx = app.agent_tx.clone();
    let mut reader = pair.master.try_clone_reader()?;
    let writer = pair.master.take_writer()?;
    let output_path = path.to_string();
    thread::spawn(move || {
        let mut buf = [0u8; 1024];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    let _ = tx.send(AgentEvent::Output {
                        path: output_path.clone(),
                        bytes: buf[..n].to_vec(),
                    });
                }
                Err(_) => break,
            }
        }
    });

    let now = Instant::now();
    let session = AgentSession {
        state: AgentState::Launching,
        parser: vt100::Parser::new(TERM_ROWS, TERM_COLS, 2000),
        master: Some(pair.master),
        writer: Some(writer),
        child: Some(child),
        last_size: (TERM_ROWS, TERM_COLS),
        launched_at: now,
        last_io_at: now,
        bytes_from_agent: 0,
        bytes_to_agent: 0,
    };

    app.agent_sessions.insert(path.to_string(), session);
    if let Some(active) = app.agent_sessions.get_mut(path) {
        active
            .parser
            .process(b"[terminal attached - type commands and press Enter]\r\n");
    }
    app.status_line = format!("Shell started in popup for {}", path);
    Ok(())
}

fn resize_terminal_session(app: &mut App, path: &str, rows: u16, cols: u16) {
    if let Some(session) = app.agent_sessions.get_mut(path) {
        // Only resize if size actually changed
        if session.last_size == (rows, cols) {
            return;
        }
        // Resize the PTY
        if let Some(master) = session.master.as_ref() {
            let _ = master.resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            });
        }
        // Resize the vt100 parser to match
        session.parser.set_size(rows, cols);
        session.last_size = (rows, cols);
    }
}

/// Calculate terminal popup dimensions based on frame size
fn calc_terminal_popup_size(frame_area: Rect) -> (u16, u16) {
    let popup = terminal_popup_rect(frame_area);
    let inner = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(1),
        ])
        .split(popup);
    // Terminal area is inner[2], minus borders
    let rows = inner[2].height.saturating_sub(2);
    let cols = inner[2].width.saturating_sub(2);
    (rows, cols)
}

fn write_to_agent(app: &mut App, path: &str, text: &str) -> Result<(), Box<dyn Error>> {
    if let Some(session) = app.agent_sessions.get_mut(path) {
        if let Some(writer) = session.writer.as_mut() {
            writer.write_all(text.as_bytes())?;
            writer.flush()?;
            session.bytes_to_agent = session.bytes_to_agent.saturating_add(text.len() as u64);
            session.last_io_at = Instant::now();
            if session.state == AgentState::Launching {
                session.state = AgentState::Running;
            }
        }
    }
    Ok(())
}

fn drain_agent_events(app: &mut App) {
    while let Ok(event) = app.agent_rx.try_recv() {
        match event {
            AgentEvent::Output { path, bytes } => {
                if let Some(session) = app.agent_sessions.get_mut(path.as_str()) {
                    session.state = AgentState::Running;
                    session.bytes_from_agent =
                        session.bytes_from_agent.saturating_add(bytes.len() as u64);
                    session.last_io_at = Instant::now();
                    session.parser.process(bytes.as_slice());
                }
            }
        }
    }
}

const AGENT_ACTIVE_WINDOW: Duration = Duration::from_secs(3);

fn agent_session_is_live(session: &AgentSession) -> bool {
    session.child.is_some() && session.writer.is_some()
}

fn agent_session_idle_seconds(session: &AgentSession, now: Instant) -> u64 {
    now.saturating_duration_since(session.last_io_at).as_secs()
}

fn agent_session_is_active(session: &AgentSession, now: Instant) -> bool {
    agent_session_is_live(session)
        && now.saturating_duration_since(session.last_io_at) <= AGENT_ACTIVE_WINDOW
}

fn agent_session_avg_bps(session: &AgentSession, now: Instant) -> u64 {
    let seconds = now
        .saturating_duration_since(session.launched_at)
        .as_secs()
        .max(1);
    session.bytes_from_agent / seconds
}

fn session_label_from_path(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(path)
        .to_string()
}

fn refresh_agent_sessions(app: &mut App) {
    for session in app.agent_sessions.values_mut() {
        if let Some(child) = session.child.as_mut() {
            if let Ok(Some(status)) = child.try_wait() {
                if status.success() {
                    session.state = AgentState::Done;
                    session
                        .parser
                        .process(b"\r\n[terminal exited successfully]\r\n");
                } else {
                    session.state = AgentState::Failed;
                    let line = format!("\r\n[terminal exited: {}]\r\n", status);
                    session.parser.process(line.as_bytes());
                }
                session.child = None;
                session.writer = None;
            }
        }
    }
}

fn handle_commit_mode_key(app: &mut App, code: KeyCode) -> Result<(), Box<dyn Error>> {
    match code {
        KeyCode::Esc => {
            app.mode = Mode::Normal;
            app.status_line = "Commit cancelled".to_string();
        }
        KeyCode::Enter => {
            let message = app.commit_input.trim();
            if message.is_empty() {
                app.status_line = "Commit message is empty".to_string();
            } else {
                let output = run_git_in(
                    app.changes_worktree_path.as_deref(),
                    &["commit", "-m", message],
                )?;
                app.status_line = output;
                refresh_status(app);
            }
            app.mode = Mode::Normal;
            app.commit_input.clear();
        }
        KeyCode::Backspace => {
            app.commit_input.pop();
        }
        KeyCode::Char(c) => {
            app.commit_input.push(c);
        }
        _ => {}
    }

    Ok(())
}

fn handle_worktree_commit_push_mode_key(
    app: &mut App,
    code: KeyCode,
) -> Result<(), Box<dyn Error>> {
    match code {
        KeyCode::Esc => {
            app.mode = Mode::Normal;
            app.worktree_commit_input.clear();
            app.worktree_commit_path = None;
            app.status_line = "Worktree commit cancelled".to_string();
        }
        KeyCode::Enter => {
            let message = app.worktree_commit_input.trim().to_string();
            let Some(path) = app.worktree_commit_path.clone() else {
                app.status_line = "No worktree selected for commit".to_string();
                app.mode = Mode::Normal;
                return Ok(());
            };

            if message.is_empty() {
                app.status_line = "Commit message is empty".to_string();
            } else {
                app.status_line = commit_worktree(path.as_str(), message.as_str())?;
                refresh_worktrees(app);
                refresh_status(app);
            }

            app.mode = Mode::Normal;
            app.worktree_commit_input.clear();
            app.worktree_commit_path = None;
        }
        KeyCode::Backspace => {
            app.worktree_commit_input.pop();
        }
        KeyCode::Char(c) => {
            app.worktree_commit_input.push(c);
        }
        _ => {}
    }

    Ok(())
}
