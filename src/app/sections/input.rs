fn handle_normal_mode_key(app: &mut App, key: KeyEvent) -> Result<bool, Box<dyn Error>> {
    let code = key.code;
    if app.view_mode == ViewMode::Worktrees {
        return handle_worktree_mode_key(app, key);
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
        KeyCode::Down => match app.active_pane {
            ActivePane::Files => {
                app.select_next();
                app.overview_scroll = 0;
                refresh_selected_overview(app);
            }
            ActivePane::Overview => {
                app.overview_scroll = app.overview_scroll.saturating_add(1);
            }
        },
        KeyCode::Up => match app.active_pane {
            ActivePane::Files => {
                app.select_prev();
                app.overview_scroll = 0;
                refresh_selected_overview(app);
            }
            ActivePane::Overview => {
                app.overview_scroll = app.overview_scroll.saturating_sub(1);
            }
        },
        KeyCode::Char('j') => match app.active_pane {
            ActivePane::Files => {
                app.select_next();
                app.overview_scroll = 0;
                refresh_selected_overview(app);
            }
            ActivePane::Overview => move_overview_method(app, true),
        },
        KeyCode::Char('k') => match app.active_pane {
            ActivePane::Files => {
                app.select_prev();
                app.overview_scroll = 0;
                refresh_selected_overview(app);
            }
            ActivePane::Overview => move_overview_method(app, false),
        },
        KeyCode::Char('J') => {
            if app.active_pane == ActivePane::Overview {
                app.overview_scroll = app.overview_scroll.saturating_add(1);
            }
        }
        KeyCode::Char('K') => {
            if app.active_pane == ActivePane::Overview {
                app.overview_scroll = app.overview_scroll.saturating_sub(1);
            }
        }
        KeyCode::Char('r') => refresh_status(app),
        KeyCode::Enter => {
            if !toggle_overview_method_expanded(app) {
                toggle_stage(app)?;
                refresh_status(app);
            }
        }
        KeyCode::Char(' ') => {
            if !toggle_overview_method_expanded(app) {
                toggle_stage(app)?;
                refresh_status(app);
            }
        }
        KeyCode::Char('a') => {
            toggle_stage(app)?;
            refresh_status(app);
        }
        KeyCode::Char('u') => {
            unstage_selected(app)?;
            refresh_status(app);
        }
        KeyCode::Char('A') => {
            stage_all_changes(app)?;
            refresh_status(app);
        }
        KeyCode::Char('U') => {
            unstage_all_changes(app)?;
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
            if let Some(path) = app.changes_worktree_path.clone() {
                start_git_task(app, "Push (changes view)", false, true, move || {
                    git_result_text(push_with_upstream_at(path.as_str()))
                });
            } else {
                start_git_task(app, "Push (changes view)", false, true, || {
                    git_result_text(push_with_upstream())
                });
            }
        }
        KeyCode::Char('s') => {
            stash_push_changes(app)?;
            refresh_status(app);
        }
        KeyCode::Char('S') => {
            stash_pop_changes(app)?;
            refresh_status(app);
        }
        _ => {}
    }

    Ok(false)
}

fn move_overview_method(app: &mut App, forward: bool) {
    if app.active_pane != ActivePane::Overview {
        return;
    }

    let count = app
        .selected_overview
        .as_ref()
        .map(|overview| overview.method_changes.len())
        .unwrap_or(0);
    if count == 0 {
        return;
    }

    app.overview_method_expanded = false;
    if forward {
        app.overview_method_index = (app.overview_method_index + 1) % count;
    } else if app.overview_method_index == 0 {
        app.overview_method_index = count - 1;
    } else {
        app.overview_method_index -= 1;
    }

    let max_scroll = max_overview_scroll(app);
    if app.overview_scroll > max_scroll {
        app.overview_scroll = max_scroll;
    }
}

fn toggle_overview_method_expanded(app: &mut App) -> bool {
    if app.active_pane != ActivePane::Overview {
        return false;
    }

    let count = app
        .selected_overview
        .as_ref()
        .map(|overview| overview.method_changes.len())
        .unwrap_or(0);
    if count == 0 {
        return false;
    }

    app.overview_method_expanded = !app.overview_method_expanded;
    app.overview_scroll = 0;
    true
}

fn handle_worktree_mode_key(app: &mut App, key: KeyEvent) -> Result<bool, Box<dyn Error>> {
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('k') {
        app.cycle_worktree_graph_builder();
        app.status_line = format!("Graph builder: {}", app.worktree_graph_builder.label());
        return Ok(false);
    }

    let code = key.code;

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
        KeyCode::Char('+') | KeyCode::Char('=') => resize_worktree_section(app, true),
        KeyCode::Char('-') => resize_worktree_section(app, false),
        KeyCode::Char(']') => zoom_worktree_canvas(app, true),
        KeyCode::Char('[') => zoom_worktree_canvas(app, false),
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
            refresh_runtime_settings(app);
            refresh_worktrees(app);
            app.status_line = "Refreshed worktree list + config".to_string();
        }
        KeyCode::Char('M') => {
            toggle_worktree_art_mode(app);
        }
        KeyCode::Char('a') => {
            app.mode = Mode::WorktreeCreateInput;
            app.new_worktree_branch.clear();
            app.new_worktree_base = WorktreeCreateBase::Selected;
            app.status_line =
                "Create worktree: choose base with ←/→, then type branch name".to_string();
        }
        KeyCode::Char('g') => {
            app.mode = Mode::WorktreeOrchestrateInput;
            app.orchestrator_requirement_input.clear();
            app.orchestrator_plan_state = OrchestratorPlanState::Idle;
            app.pending_orchestrator_launch = None;
            app.status_line =
                "Orchestrate worktrees: describe the feature, Enter to preview prompts".to_string();
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
                start_git_task(app, "Push selected worktree", false, true, move || {
                    git_result_text(push_with_upstream_at(path.as_str()))
                });
            }
        }
        KeyCode::Char('f') => {
            if let Some(parent) = connected_parent_worktree(app) {
                let path = parent.path;
                let branch = parent.branch;
                start_git_task(app, "Fetch + pull parent", false, true, move || {
                    git_result_text(update_parent_at(path.as_str(), branch.as_str()))
                });
            } else if let Some(selected) = app.selected_worktree().cloned() {
                if worktree_is_main_behind_head(&selected) {
                    let path = selected.path;
                    let branch = selected.branch;
                    start_git_task(app, "Fetch + pull selected head", false, true, move || {
                        git_result_text(update_worktree_head_at(path.as_str(), branch.as_str()))
                    });
                } else if is_main_branch_name(selected.branch.as_str()) && selected.has_upstream {
                    app.status_line =
                        "Selected main branch is already in sync with head".to_string();
                } else {
                    app.status_line =
                        "No connected parent node found for selected worktree".to_string();
                }
            } else {
                app.status_line =
                    "No connected parent node found for selected worktree".to_string();
            }
        }
        KeyCode::Char('F') => {
            let selected = app.selected_worktree().cloned();
            let parent = connected_parent_worktree(app);
            if let (Some(selected), Some(parent)) = (selected, parent) {
                let child_path = selected.path;
                let child_branch = selected.branch;
                let parent_path = parent.path;
                let parent_branch = parent.branch;
                start_git_task(app, "Rebase selected onto parent", false, true, move || {
                    git_result_text(rebase_onto_parent_at(
                        child_path.as_str(),
                        child_branch.as_str(),
                        parent_path.as_str(),
                        parent_branch.as_str(),
                    ))
                });
            } else {
                app.status_line =
                    "No connected parent node found for selected worktree".to_string();
            }
        }
        KeyCode::Char('x') => {
            start_git_task(app, "Prune stale worktrees", true, false, || {
                git_result_text(run_git(&["worktree", "prune"]))
            });
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

fn resize_worktree_section(app: &mut App, grow: bool) {
    const CANVAS_STEP: i16 = 4;
    const ART_STEP: i16 = 2;
    const DETAILS_STEP: i16 = 2;

    match app.worktree_focus {
        WorktreePane::Canvas => {
            let delta = if grow { CANVAS_STEP } else { -CANVAS_STEP };
            let next = (app.worktree_canvas_width_percent as i16 + delta).clamp(52, 86);
            app.worktree_canvas_width_percent = next as u16;
            app.status_line = format!(
                "Canvas width: {}% (right stack {}%)",
                app.worktree_canvas_width_percent,
                100u16.saturating_sub(app.worktree_canvas_width_percent)
            );
        }
        WorktreePane::Art => {
            let delta = if grow { ART_STEP } else { -ART_STEP };
            app.worktree_art_height_delta = (app.worktree_art_height_delta + delta).clamp(-28, 28);
            app.status_line = "Art section height adjusted".to_string();
        }
        WorktreePane::Details => {
            let delta = if grow { DETAILS_STEP } else { -DETAILS_STEP };
            app.worktree_details_height_delta =
                (app.worktree_details_height_delta + delta).clamp(-28, 28);
            app.status_line = "Details section height adjusted".to_string();
        }
        WorktreePane::Actions => {
            let delta = if grow { DETAILS_STEP } else { -DETAILS_STEP };
            app.worktree_details_height_delta =
                (app.worktree_details_height_delta - delta).clamp(-28, 28);
            app.status_line = "Actions section height adjusted".to_string();
        }
    }
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

fn cycle_worktree_canvas_background(app: &mut App) {
    app.worktree_canvas_bg_mode = app.worktree_canvas_bg_mode.next();
    app.status_line = format!(
        "Canvas background: {} (Ctrl+B to cycle)",
        app.worktree_canvas_bg_mode.short_label()
    );
}

fn toggle_worktree_art_mode(app: &mut App) {
    app.worktree_art_mode = app.worktree_art_mode.next();
    if app.worktree_art_mode == WorktreeArtMode::SpotifyConnector {
        app.spotify_last_refresh = Instant::now()
            .checked_sub(Duration::from_secs(5))
            .unwrap_or_else(Instant::now);
        refresh_spotify_now_playing(app);
    }
    app.status_line = format!(
        "Worktree art mode: {} (M to toggle)",
        app.worktree_art_mode.label()
    );
}

fn refresh_spotify_now_playing(app: &mut App) {
    if app.worktree_art_mode != WorktreeArtMode::SpotifyConnector {
        return;
    }

    if app.spotify_last_refresh.elapsed() < Duration::from_millis(900) {
        return;
    }

    app.spotify_last_refresh = Instant::now();

    if let Ok(Some(now_playing)) = fetch_spotify_now_playing_playerctl() {
        sync_spotify_cover_art(app, &now_playing);
        app.spotify_now_playing = Some(now_playing);
        app.spotify_refresh_error = None;
        return;
    }

    #[cfg(target_os = "macos")]
    {
        match fetch_spotify_now_playing_macos() {
            Ok(Some(now_playing)) => {
                sync_spotify_cover_art(app, &now_playing);
                app.spotify_now_playing = Some(now_playing);
                app.spotify_refresh_error = None;
            }
            Ok(None) => {
                app.spotify_now_playing = None;
                app.spotify_cover_art = None;
                app.spotify_refresh_error = None;
            }
            Err(err) => {
                app.spotify_now_playing = None;
                app.spotify_cover_art = None;
                app.spotify_refresh_error = Some(err);
            }
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        app.spotify_now_playing = None;
        app.spotify_cover_art = None;
        app.spotify_refresh_error = Some(
            "No MPRIS Spotify metadata found (try installing playerctl and starting playback)"
                .to_string(),
        );
    }
}

fn sync_spotify_cover_art(app: &mut App, now_playing: &SpotifyNowPlaying) {
    let Some(url) = now_playing.art_url.as_deref() else {
        app.spotify_cover_art = None;
        return;
    };

    if app
        .spotify_cover_art
        .as_ref()
        .map(|art| art.source_url.as_str() == url)
        .unwrap_or(false)
    {
        return;
    }

    let Some(picker) = app.spotify_image_picker.as_ref() else {
        app.spotify_cover_art = None;
        return;
    };

    app.spotify_cover_art = load_spotify_cover_art(url, picker).ok();
}

fn load_spotify_cover_art(url: &str, picker: &Picker) -> Result<SpotifyCoverArt, String> {
    let bytes = if let Some(file_path) = url.strip_prefix("file://") {
        fs::read(file_path).map_err(|err| format!("Failed to read cover art file: {}", err))?
    } else if url.starts_with("data:") {
        let encoded = url
            .split_once("base64,")
            .map(|(_, data)| data)
            .ok_or_else(|| "Invalid base64 art URL".to_string())?;
        BASE64_STANDARD
            .decode(encoded)
            .map_err(|err| format!("Failed to decode base64 cover art: {}", err))?
    } else {
        reqwest::blocking::Client::new()
            .get(url)
            .send()
            .and_then(|resp| resp.bytes())
            .map(|bytes| bytes.to_vec())
            .map_err(|err| format!("Failed to download cover art: {}", err))?
    };

    let image = image::ImageReader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|err| format!("Could not detect cover art format: {}", err))?
        .decode()
        .map_err(|err| format!("Could not decode cover art image: {}", err))?;

    Ok(SpotifyCoverArt {
        source_url: url.to_string(),
        image: picker.new_resize_protocol(image),
    })
}

fn fetch_spotify_now_playing_playerctl() -> Result<Option<SpotifyNowPlaying>, String> {
    // Inspired by qxb3/fum's MPRIS metadata flow (MIT).
    let metadata_output = Command::new("playerctl")
        .args([
            "--player=spotify",
            "metadata",
            "--format",
            "{{title}}||{{artist}}||{{mpris:artUrl}}",
        ])
        .output()
        .map_err(|err| format!("Failed to run playerctl: {}", err))?;

    if !metadata_output.status.success() {
        let stderr = String::from_utf8_lossy(&metadata_output.stderr)
            .trim()
            .to_string();
        return Err(if stderr.is_empty() {
            "playerctl returned a non-zero exit status".to_string()
        } else {
            stderr
        });
    }

    let payload = String::from_utf8_lossy(&metadata_output.stdout)
        .trim()
        .to_string();
    if payload.is_empty() {
        return Ok(None);
    }

    let mut parts = payload.splitn(3, "||");
    let track = parts.next().unwrap_or_default().trim().to_string();
    let artist = parts.next().unwrap_or_default().trim().to_string();
    let art_raw = parts.next().unwrap_or_default().trim();

    if track.is_empty() {
        return Ok(None);
    }

    Ok(Some(SpotifyNowPlaying {
        track,
        artist,
        art_url: if art_raw.is_empty() {
            None
        } else {
            Some(art_raw.to_string())
        },
    }))
}

#[cfg(target_os = "macos")]
fn fetch_spotify_now_playing_macos() -> Result<Option<SpotifyNowPlaying>, String> {
    let script = concat!(
        "if not application \"Spotify\" is running then return \"\"\n",
        "tell application \"Spotify\"\n",
        "set ps to player state as string\n",
        "if ps is not \"playing\" and ps is not \"paused\" then return \"\"\n",
        "set track_name to name of current track\n",
        "set track_artist to artist of current track\n",
        "set art_url to \"\"\n",
        "try\n",
        "set art_url to artwork url of current track\n",
        "end try\n",
        "return track_name & \"||\" & track_artist & \"||\" & art_url\n",
        "end tell"
    );

    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .map_err(|err| format!("Failed to run osascript: {}", err))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            "Spotify AppleScript returned a non-zero exit status".to_string()
        } else {
            stderr
        });
    }

    let payload = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if payload.is_empty() {
        return Ok(None);
    }

    let mut parts = payload.splitn(3, "||");
    let track = parts.next().unwrap_or_default().trim().to_string();
    let artist = parts.next().unwrap_or_default().trim().to_string();
    let art_raw = parts.next().unwrap_or_default().trim();

    if track.is_empty() {
        return Ok(None);
    }

    Ok(Some(SpotifyNowPlaying {
        track,
        artist,
        art_url: if art_raw.is_empty() {
            None
        } else {
            Some(art_raw.to_string())
        },
    }))
}

fn toggle_perf_debug(app: &mut App) {
    app.perf_debug.enabled = !app.perf_debug.enabled;
    if app.perf_debug.enabled {
        app.status_line = format!(
            "Perf debug enabled (Ctrl+L to toggle). Hitch log: {}",
            app.perf_debug.hitch_log_path.display()
        );
    } else {
        app.status_line = "Perf debug disabled (Ctrl+L to toggle)".to_string();
    }
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
    reset_terminal_popup_scrollback(app, path);
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
                launch_agent_in_terminal(
                    app,
                    path.as_str(),
                    default_agent,
                    default_agent == ExternalAgent::Opencode,
                )?;
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
            launch_agent_in_terminal(app, path.as_str(), agent, false)?;
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
    allow_opencode_session_resume: bool,
) -> Result<(), Box<dyn Error>> {
    wait_for_terminal_ready(app, path);

    if let Some(context) = app.pending_conflict_context.as_ref() {
        if context.parent_path == path {
            let prompt = build_conflict_resolve_prompt(app, context);

            if agent == ExternalAgent::Opencode {
                let launch = build_opencode_launch_command(
                    path,
                    Some(prompt.as_str()),
                    allow_opencode_session_resume,
                );
                write_to_agent(app, path, launch.command.as_str())?;
                attach_session_agent(app, path, agent, launch.session_id);
                app.pending_conflict_context = None;
                app.status_line = if launch.resumed {
                    "Reconnected OpenCode session with conflict-resolution prompt (--prompt)"
                        .to_string()
                } else {
                    "Launched OpenCode with conflict-resolution prompt (--prompt)".to_string()
                };
                return Ok(());
            }

            let launch_cmd = format!("{}\r", agent.command_name());
            write_to_agent(app, path, launch_cmd.as_str())?;
            attach_session_agent(app, path, agent, None);
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

    let (launch_cmd, resumed_session, opencode_session_id) = if agent == ExternalAgent::Opencode {
        build_opencode_launch_command(path, None, allow_opencode_session_resume).into_parts()
    } else {
        (format!("{}\r", agent.command_name()), false, None)
    };
    write_to_agent(app, path, launch_cmd.as_str())?;
    attach_session_agent(app, path, agent, opencode_session_id);

    app.status_line = if resumed_session {
        "Reconnected OpenCode session in terminal".to_string()
    } else {
        format!("Launched {} in terminal", agent.display_name())
    };
    Ok(())
}

fn attach_session_agent(
    app: &mut App,
    path: &str,
    agent: ExternalAgent,
    opencode_session_id: Option<String>,
) {
    if let Some(session) = app.agent_sessions.get_mut(path) {
        session.agent_kind = Some(agent);
        if agent == ExternalAgent::Opencode {
            session.opencode_session_id = opencode_session_id;
            session.opencode_usage = None;
        } else {
            session.opencode_session_id = None;
            session.opencode_usage = None;
        }
    }
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
    launch_agent_in_terminal(
        app,
        context.parent_path.as_str(),
        ExternalAgent::Opencode,
        false,
    )?;
    app.confirm_conflict_resolve_yes = false;
    Ok(())
}

fn build_opencode_launch_command(
    worktree_path: &str,
    prompt: Option<&str>,
    allow_resume: bool,
) -> OpenCodeLaunchCommand {
    let resumed_session = if allow_resume {
        resolve_recent_opencode_session_id_for_worktree(worktree_path)
    } else {
        None
    };

    let mut cmd = String::from("opencode");
    let resumed = if let Some(session_id) = resumed_session.as_ref() {
        cmd.push_str(" --session ");
        cmd.push_str(shell_quote_for_command(session_id.as_str()).as_str());
        true
    } else {
        false
    };

    if let Some(text) = prompt {
        cmd.push_str(" --prompt ");
        cmd.push_str(shell_quote_for_command(text).as_str());
    }
    cmd.push('\r');
    OpenCodeLaunchCommand {
        command: cmd,
        resumed,
        session_id: resumed_session,
    }
}

struct OpenCodeLaunchCommand {
    command: String,
    resumed: bool,
    session_id: Option<String>,
}

impl OpenCodeLaunchCommand {
    fn into_parts(self) -> (String, bool, Option<String>) {
        (self.command, self.resumed, self.session_id)
    }
}

fn resolve_recent_opencode_session_id_for_worktree(worktree_path: &str) -> Option<String> {
    let output = Command::new("opencode")
        .args(["session", "list", "--format", "json", "-n", "120"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let json = String::from_utf8_lossy(&output.stdout);
    let sessions = parse_opencode_session_rows(json.as_ref());
    if sessions.is_empty() {
        return None;
    }

    let target = normalize_path_for_session_match(worktree_path);
    sessions
        .into_iter()
        .find(|(_, directory)| normalize_path_for_session_match(directory.as_str()) == target)
        .map(|(id, _)| id)
}

fn normalize_path_for_session_match(path: &str) -> String {
    let candidate = PathBuf::from(path);
    let absolute = if candidate.is_absolute() {
        candidate
    } else {
        std::env::current_dir()
            .map(|cwd| cwd.join(candidate.as_path()))
            .unwrap_or(candidate)
    };
    fs::canonicalize(absolute.as_path())
        .unwrap_or(absolute)
        .to_string_lossy()
        .to_string()
}

fn parse_opencode_session_rows(json: &str) -> Vec<(String, String)> {
    parse_top_level_json_objects(json)
        .into_iter()
        .filter_map(|object| {
            let id = parse_json_string_field(object.as_str(), "id")?;
            let directory = parse_json_string_field(object.as_str(), "directory")?;
            Some((id, directory))
        })
        .collect()
}

fn parse_top_level_json_objects(json: &str) -> Vec<String> {
    let bytes = json.as_bytes();
    let mut out = Vec::new();
    let mut i = 0usize;
    let mut in_string = false;
    let mut escaping = false;
    let mut depth = 0usize;
    let mut start = None;

    while i < bytes.len() {
        let c = bytes[i];
        if in_string {
            if escaping {
                escaping = false;
            } else if c == b'\\' {
                escaping = true;
            } else if c == b'"' {
                in_string = false;
            }
            i += 1;
            continue;
        }

        match c {
            b'"' => in_string = true,
            b'{' => {
                if depth == 0 {
                    start = Some(i);
                }
                depth = depth.saturating_add(1);
            }
            b'}' => {
                if depth > 0 {
                    depth -= 1;
                    if depth == 0 {
                        if let Some(begin) = start.take() {
                            out.push(json[begin..=i].to_string());
                        }
                    }
                }
            }
            _ => {}
        }
        i += 1;
    }

    out
}

fn parse_json_string_field(object: &str, field_name: &str) -> Option<String> {
    let key = format!("\"{}\"", field_name);
    let mut cursor = 0usize;
    while let Some(found) = object[cursor..].find(key.as_str()) {
        let idx = cursor + found;
        let mut i = idx + key.len();
        skip_json_ws(object, &mut i);
        if object.as_bytes().get(i).copied() != Some(b':') {
            cursor = i;
            continue;
        }
        i += 1;
        skip_json_ws(object, &mut i);
        if object.as_bytes().get(i).copied() != Some(b'"') {
            cursor = i;
            continue;
        }
        let (value, _) = parse_json_string_literal(object, i)?;
        return Some(value);
    }
    None
}

fn skip_json_ws(text: &str, idx: &mut usize) {
    while let Some(ch) = text.as_bytes().get(*idx) {
        if !matches!(*ch, b' ' | b'\n' | b'\r' | b'\t') {
            break;
        }
        *idx += 1;
    }
}

fn parse_json_string_literal(text: &str, quote_idx: usize) -> Option<(String, usize)> {
    let bytes = text.as_bytes();
    if bytes.get(quote_idx).copied() != Some(b'"') {
        return None;
    }

    let mut i = quote_idx + 1;
    let mut out = String::new();
    while i < bytes.len() {
        let ch = bytes[i];
        if ch == b'"' {
            return Some((out, i + 1));
        }
        if ch != b'\\' {
            out.push(ch as char);
            i += 1;
            continue;
        }

        i += 1;
        let esc = *bytes.get(i)?;
        match esc {
            b'"' => out.push('"'),
            b'\\' => out.push('\\'),
            b'/' => out.push('/'),
            b'b' => out.push('\u{0008}'),
            b'f' => out.push('\u{000C}'),
            b'n' => out.push('\n'),
            b'r' => out.push('\r'),
            b't' => out.push('\t'),
            b'u' => {
                let end = i + 5;
                let hex = text.get(i + 1..end)?;
                if let Ok(value) = u16::from_str_radix(hex, 16) {
                    if let Some(decoded) = char::from_u32(value as u32) {
                        out.push(decoded);
                    }
                }
                i = end - 1;
            }
            _ => return None,
        }
        i += 1;
    }

    None
}

fn normalize_terminal_newlines(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\n', "\r")
}

fn wait_for_terminal_ready(app: &mut App, path: &str) {
    let needs_bootstrap_delay = app
        .agent_sessions
        .get(path)
        .map(|session| session.state == AgentState::Launching && session.bytes_from_agent == 0)
        .unwrap_or(false);

    if needs_bootstrap_delay {
        thread::sleep(Duration::from_millis(120));
        drain_agent_events(app);
    }
}

fn shell_ansi_c_quote(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 8);
    out.push_str("$'");
    for ch in text.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '\'' => out.push_str("\\'"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(ch),
        }
    }
    out.push('\'');
    out
}

fn shell_powershell_single_quote(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 8);
    out.push('\'');
    for ch in text.chars() {
        if ch == '\'' {
            out.push_str("''");
        } else {
            out.push(ch);
        }
    }
    out.push('\'');
    out
}

fn shell_quote_for_command(text: &str) -> String {
    if cfg!(windows) {
        shell_powershell_single_quote(text)
    } else {
        shell_ansi_c_quote(text)
    }
}

fn interactive_shell_command() -> (String, Vec<&'static str>) {
    if cfg!(windows) {
        if command_exists_on_path("pwsh") {
            return ("pwsh".to_string(), vec!["-NoLogo", "-NoExit"]);
        }
        return ("powershell.exe".to_string(), vec!["-NoLogo", "-NoExit"]);
    }

    let shell = std::env::var("SHELL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "/bin/sh".to_string());
    (shell, vec!["-i", "-l"])
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

    #[cfg(windows)]
    let path_exts: Vec<String> = std::env::var("PATHEXT")
        .ok()
        .map(|raw| {
            raw.split(';')
                .filter_map(|ext| {
                    let trimmed = ext.trim();
                    if trimmed.is_empty() {
                        None
                    } else {
                        Some(trimmed.to_string())
                    }
                })
                .collect::<Vec<String>>()
        })
        .filter(|items| !items.is_empty())
        .unwrap_or_else(|| {
            vec![
                ".COM".to_string(),
                ".EXE".to_string(),
                ".BAT".to_string(),
                ".CMD".to_string(),
            ]
        });

    #[cfg(windows)]
    let has_extension = command.rsplit_once('.').is_some();

    for dir in std::env::split_paths(paths.as_os_str()) {
        let candidate = dir.join(command);
        if candidate.is_file() {
            return true;
        }

        #[cfg(windows)]
        if !has_extension {
            for ext in &path_exts {
                let with_ext = dir.join(format!("{}{}", command, ext));
                if with_ext.is_file() {
                    return true;
                }
            }
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
    let mut worktree_orchestrator_enabled = true;
    let mut worktree_orchestrator_prompt_path = prompts_dir.join("worktree-orchestrator-prompt.md");
    let mut worktree_orchestrator_max_nodes = 8usize;
    let mut worktree_graph_art = default_worktree_graph_art_lines();
    let mut has_worktree_graph_art = false;

    if let Ok(mut raw) = fs::read_to_string(config_path.as_path()) {
        let lines: Vec<&str> = raw.lines().collect();
        let mut idx = 0usize;
        while idx < lines.len() {
            let line = lines[idx];
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                idx += 1;
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                idx += 1;
                continue;
            };
            let key = key.trim();
            let value = value.trim();
            match key {
                "default_agent" => {
                    default_agent = parse_config_string(value)
                        .as_deref()
                        .and_then(parse_external_agent)
                        .or(default_agent);
                }
                "conflict_resolve_prompt" => {
                    if let Some(v) = parse_config_string(value) {
                        let candidate = PathBuf::from(v.as_str());
                        conflict_prompt_path = if candidate.is_absolute() {
                            candidate
                        } else {
                            config_dir.join(candidate)
                        };
                    }
                }
                "worktree_orchestrator_enabled" => {
                    if let Some(v) = parse_config_bool(value) {
                        worktree_orchestrator_enabled = v;
                    }
                }
                "worktree_orchestrator_prompt" => {
                    if let Some(v) = parse_config_string(value) {
                        let candidate = PathBuf::from(v.as_str());
                        worktree_orchestrator_prompt_path = if candidate.is_absolute() {
                            candidate
                        } else {
                            config_dir.join(candidate)
                        };
                    }
                }
                "worktree_orchestrator_max_nodes" => {
                    if let Some(v) = parse_config_usize(value) {
                        worktree_orchestrator_max_nodes = v.clamp(1, 24);
                    }
                }
                "worktree_graph_art" => {
                    has_worktree_graph_art = true;
                    if value.starts_with("\"\"\"") {
                        if let Some((parsed, consumed)) =
                            parse_config_multiline_string(lines.as_slice(), idx, value)
                        {
                            worktree_graph_art =
                                parsed.lines().map(|line| line.to_string()).collect();
                            idx += consumed;
                            continue;
                        }
                    } else if let Some(v) = parse_config_string(value) {
                        let expanded = v.replace("\\n", "\n");
                        worktree_graph_art =
                            expanded.lines().map(|line| line.to_string()).collect();
                    }
                }
                _ => {}
            }

            idx += 1;
        }

        if !has_worktree_graph_art {
            if !raw.ends_with('\n') {
                raw.push('\n');
            }
            raw.push('\n');
            raw.push_str(default_worktree_graph_art_config_block().as_str());
            let _ = fs::write(config_path.as_path(), raw);
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

    if !worktree_orchestrator_prompt_path.exists() {
        if let Some(parent) = worktree_orchestrator_prompt_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(
            worktree_orchestrator_prompt_path.as_path(),
            default_worktree_orchestrator_prompt_template(),
        );
    }

    OpenSwarmConfig {
        config_path: config_path.to_string_lossy().to_string(),
        default_agent,
        conflict_resolve_prompt_path: conflict_prompt_path.to_string_lossy().to_string(),
        worktree_orchestrator_enabled,
        worktree_orchestrator_prompt_path: worktree_orchestrator_prompt_path
            .to_string_lossy()
            .to_string(),
        worktree_orchestrator_max_nodes,
        worktree_graph_art,
    }
}

fn openswarm_config_dir() -> PathBuf {
    fn env_path(var: &str) -> Option<PathBuf> {
        let value = std::env::var_os(var)?;
        if value.is_empty() {
            return None;
        }
        Some(PathBuf::from(value))
    }

    let home = if cfg!(windows) {
        env_path("USERPROFILE")
            .or_else(|| {
                let home_drive = env_path("HOMEDRIVE")?;
                let home_path = env_path("HOMEPATH")?;
                let mut combined = home_drive;
                combined.push(home_path);
                Some(combined)
            })
            .or_else(|| env_path("HOME"))
            .or_else(|| {
                std::env::var("USERNAME")
                    .ok()
                    .filter(|name| !name.trim().is_empty())
                    .map(|name| PathBuf::from("C:\\Users").join(name))
            })
            .unwrap_or_else(|| PathBuf::from("C:\\Users\\Default"))
    } else {
        env_path("HOME")
            .or_else(|| env_path("USERPROFILE"))
            .unwrap_or_else(|| PathBuf::from("."))
    };

    home.join(".config").join("openswarm")
}

fn default_openswarm_config_text() -> String {
    let mut text =
        "# OpenSwarm config\n# default_agent accepts: \"\", \"opencode\", \"claude\"\n# empty default_agent means always show the picker on Shift+O\ndefault_agent = \"\"\n\n# relative paths are resolved from the OpenSwarm config dir\n# (~/.config/openswarm, or %USERPROFILE%\\.config\\openswarm on Windows)\nconflict_resolve_prompt = \"prompts/conflict-resolve-prompt.md\"\n\n# orchestrate feature requirements into a worktree plan using OpenCode\nworktree_orchestrator_enabled = true\nworktree_orchestrator_prompt = \"prompts/worktree-orchestrator-prompt.md\"\nworktree_orchestrator_max_nodes = 8\n\n"
            .to_string();
    text.push_str(default_worktree_graph_art_config_block().as_str());
    text
}

fn default_worktree_orchestrator_prompt_template() -> String {
    "You are planning Git worktree abstractions only. Do not write code, do not run tests, do not suggest commits.\n\nRequirement:\n{requirement}\n\nRepository defaults:\n- root branch: {root_branch}\n- selected branch: {selected_branch}\n- max nodes: {max_nodes}\n- existing branches:\n{existing_branches}\n\nReturn STRICT JSON only (no prose, no markdown fence):\n{\n  \"layout\": \"feature-parent\" | \"stacked-domains\",\n  \"nodes\": [\n    {\n      \"branch\": \"feature/example\",\n      \"parent\": \"main\",\n      \"goal\": \"short why\"\n    }\n  ]\n}\n\nRules:\n- Prefer 2-6 nodes.\n- Branch names must be lowercase and git-safe; slashes allowed.\n- Parent must be an existing branch or another node in this plan.\n- Include one top-level feature branch when useful, then split children beneath it.\n- Keep nodes focused so a developer can prompt each worktree independently."
        .to_string()
}

fn default_worktree_graph_art_config_block() -> String {
    let mut block = "# optional art shown above details in worktree view\n# supports ASCII or Unicode. Keep it compact for narrow terminals.\nworktree_graph_art = \"\"\"\n"
        .to_string();

    for line in default_worktree_graph_art_lines() {
        block.push_str(line.as_str());
        block.push('\n');
    }

    block.push_str("\"\"\"\n");
    block
}

fn default_worktree_graph_art_lines() -> Vec<String> {
    vec![
        "  .-\"\"\"\"-.   .-\"\"\"\"-.".to_string(),
        " /  .--.  \\ /  .--.  \\".to_string(),
        "|  /    \\_/\\_/    \\  |".to_string(),
        "| |  o    . .    o  | |".to_string(),
        "| |      ( ^ )      | |".to_string(),
        " \\ \\   .`-.-`.   / /".to_string(),
        "  `._`--(_____)--`_.`".to_string(),
    ]
}

fn parse_config_multiline_string(
    lines: &[&str],
    start_idx: usize,
    current_value: &str,
) -> Option<(String, usize)> {
    let mut text = String::new();
    let mut consumed = 1usize;
    let opening_remainder = current_value.strip_prefix("\"\"\"")?;

    if let Some(end_idx) = opening_remainder.find("\"\"\"") {
        text.push_str(&opening_remainder[..end_idx]);
        return Some((text, consumed));
    }

    if !opening_remainder.is_empty() {
        text.push_str(opening_remainder);
    }

    while start_idx + consumed < lines.len() {
        let line = lines[start_idx + consumed];
        if line.trim() == "\"\"\"" {
            return Some((text, consumed + 1));
        }

        if !text.is_empty() {
            text.push('\n');
        }
        text.push_str(line);
        consumed += 1;
    }

    None
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

fn parse_config_bool(value: &str) -> Option<bool> {
    match parse_config_string(value)?.to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Some(true),
        "false" | "0" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn parse_config_usize(value: &str) -> Option<usize> {
    parse_config_string(value)?.parse::<usize>().ok()
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

fn handle_worktree_orchestrate_mode_key(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Esc => {
            if matches!(
                app.orchestrator_plan_state,
                OrchestratorPlanState::Loading { .. }
            ) {
                app.status_line =
                    "Planner is still running; wait for completion before closing".to_string();
                return;
            }
            app.mode = Mode::Normal;
            app.status_line = "Worktree orchestration cancelled".to_string();
        }
        KeyCode::Enter => {
            if matches!(
                app.orchestrator_plan_state,
                OrchestratorPlanState::Loading { .. }
            ) {
                app.status_line = "Planner already running for this requirement".to_string();
                return;
            }

            refresh_runtime_settings(app);
            if !app.config.worktree_orchestrator_enabled {
                app.mode = Mode::Normal;
                app.status_line =
                    "Worktree orchestrator is disabled in config.toml (set worktree_orchestrator_enabled = true)"
                        .to_string();
                app.orchestrator_requirement_input.clear();
                return;
            }

            let requirement = app.orchestrator_requirement_input.trim();
            if requirement.is_empty() {
                app.status_line = "Feature requirement is required".to_string();
                return;
            }

            let root = create_root_for_app(app);
            let selected_branch = selected_branch_name(app);
            let root_branch = resolve_main_branch();
            let existing_branches = app
                .worktrees
                .iter()
                .filter(|wt| !wt.detached && !wt.branch.trim().is_empty())
                .map(|wt| wt.branch.clone())
                .collect::<Vec<_>>();
            let requirement_text = requirement.to_string();
            let prompt_path = app.config.worktree_orchestrator_prompt_path.clone();
            let max_nodes = app.config.worktree_orchestrator_max_nodes;

            app.orchestrator_plan_state = OrchestratorPlanState::Loading {
                started_at: Instant::now(),
            };
            app.status_line = "Planning worktree graph with OpenCode...".to_string();
            start_orchestrator_plan_task(
                app,
                root,
                requirement_text,
                root_branch,
                selected_branch,
                existing_branches,
                prompt_path,
                max_nodes,
            );
        }
        KeyCode::Backspace => {
            if matches!(
                app.orchestrator_plan_state,
                OrchestratorPlanState::Loading { .. }
            ) {
                return;
            }
            app.orchestrator_requirement_input.pop();
        }
        KeyCode::Char(c) => {
            if matches!(
                app.orchestrator_plan_state,
                OrchestratorPlanState::Loading { .. }
            ) {
                return;
            }
            app.orchestrator_requirement_input.push(c);
        }
        _ => {}
    }
}

fn handle_worktree_orchestrate_preview_mode_key(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Esc => {
            clear_orchestrator_prompt_preview(app);
            app.pending_orchestrator_launch = None;
            app.mode = Mode::Normal;
            app.status_line = "Worktree orchestration cancelled".to_string();
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if app.orchestrator_prompt_nodes.is_empty() {
                app.orchestrator_prompt_selected = 0;
            } else if app.orchestrator_prompt_selected == 0 {
                app.orchestrator_prompt_selected = app.orchestrator_prompt_nodes.len() - 1;
            } else {
                app.orchestrator_prompt_selected -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.orchestrator_prompt_nodes.is_empty() {
                app.orchestrator_prompt_selected = 0;
            } else {
                app.orchestrator_prompt_selected =
                    (app.orchestrator_prompt_selected + 1) % app.orchestrator_prompt_nodes.len();
            }
        }
        KeyCode::Char(' ') => {
            if let Some(node) = app
                .orchestrator_prompt_nodes
                .get_mut(app.orchestrator_prompt_selected)
            {
                node.accepted = !node.accepted;
            }
        }
        KeyCode::Char('a') | KeyCode::Char('A') => {
            for node in &mut app.orchestrator_prompt_nodes {
                node.accepted = true;
            }
        }
        KeyCode::Char('r') | KeyCode::Char('R') => {
            if let Some(node) = app
                .orchestrator_prompt_nodes
                .get(app.orchestrator_prompt_selected)
            {
                app.orchestrator_prompt_edit_input = node.prompt.clone();
                app.mode = Mode::WorktreeOrchestratePromptEdit;
                app.status_line = "Refine prompt: edit text, Enter save, Esc cancel".to_string();
            }
        }
        KeyCode::Enter => {
            let accepted_nodes_raw = app
                .orchestrator_prompt_nodes
                .iter()
                .filter(|node| node.accepted)
                .map(|node| ProposedWorktreeNode {
                    branch: node.branch.clone(),
                    parent: node.parent.clone(),
                    goal: node.goal.clone(),
                })
                .collect::<Vec<_>>();
            let launch_nodes = app
                .orchestrator_prompt_nodes
                .iter()
                .filter(|node| node.accepted)
                .map(|node| OrchestratorLaunchNode {
                    branch: node.branch.clone(),
                    prompt: node.prompt.clone(),
                })
                .collect::<Vec<_>>();

            if accepted_nodes_raw.is_empty() {
                app.status_line = "No accepted nodes selected for execution".to_string();
                return;
            }

            let root_branch = resolve_main_branch();
            let selected_branch = selected_branch_name(app);
            let existing_branches = app
                .worktrees
                .iter()
                .filter(|wt| !wt.detached && !wt.branch.trim().is_empty())
                .map(|wt| wt.branch.clone())
                .collect::<Vec<_>>();
            let accepted_nodes = normalize_orchestrated_nodes(
                accepted_nodes_raw,
                app.orchestrator_planned_requirement.as_str(),
                root_branch.as_str(),
                selected_branch.as_str(),
                existing_branches.as_slice(),
                app.config.worktree_orchestrator_max_nodes,
            );

            let root = create_root_for_app(app);
            let requirement = app.orchestrator_planned_requirement.clone();
            let planner_source = app.orchestrator_planner_source.clone();
            app.pending_orchestrator_launch = Some(PendingOrchestratorLaunch {
                requirement: requirement.clone(),
                nodes: launch_nodes,
            });
            start_git_task(
                app,
                "Execute orchestrated worktrees",
                true,
                true,
                move || {
                    create_worktrees_from_orchestrated_nodes(
                        root.as_str(),
                        requirement.as_str(),
                        planner_source.as_str(),
                        accepted_nodes,
                    )
                },
            );

            clear_orchestrator_prompt_preview(app);
            app.mode = Mode::Normal;
        }
        _ => {}
    }
}

fn handle_worktree_orchestrate_prompt_edit_mode_key(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Esc => {
            app.mode = Mode::WorktreeOrchestratePreview;
            app.orchestrator_prompt_edit_input.clear();
            app.status_line = "Prompt refine cancelled".to_string();
        }
        KeyCode::Enter => {
            if let Some(node) = app
                .orchestrator_prompt_nodes
                .get_mut(app.orchestrator_prompt_selected)
            {
                let refined = app.orchestrator_prompt_edit_input.trim();
                if !refined.is_empty() {
                    node.prompt = refined.to_string();
                }
            }
            app.mode = Mode::WorktreeOrchestratePreview;
            app.orchestrator_prompt_edit_input.clear();
            app.status_line = "Prompt refined for selected leaf".to_string();
        }
        KeyCode::Backspace => {
            app.orchestrator_prompt_edit_input.pop();
        }
        KeyCode::Char(c) => {
            app.orchestrator_prompt_edit_input.push(c);
        }
        _ => {}
    }
}

fn clear_orchestrator_prompt_preview(app: &mut App) {
    app.orchestrator_planned_requirement.clear();
    app.orchestrator_planner_source.clear();
    app.orchestrator_prompt_nodes.clear();
    app.orchestrator_prompt_selected = 0;
    app.orchestrator_prompt_edit_input.clear();
    app.pending_orchestrator_launch = None;
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

    let children = removable_descendant_paths(app, selected.path.as_str());
    if !children.is_empty() {
        app.pending_remove_worktree_path = selected.path;
        app.pending_remove_worktree_children = children;
        app.confirm_remove_worktree_yes = false;
        app.mode = Mode::WorktreeRemoveChildrenConfirm;
        app.status_line = format!(
            "Selected worktree has {} child worktree(s)",
            app.pending_remove_worktree_children.len()
        );
        return Ok(());
    }

    if selected.dirty {
        app.pending_remove_worktree_path = selected.path;
        app.pending_remove_worktree_children.clear();
        app.confirm_remove_worktree_yes = false;
        app.mode = Mode::WorktreeRemoveDirtyConfirm;
        app.status_line = "Selected worktree has uncommitted changes".to_string();
        return Ok(());
    }

    start_remove_worktree_task(app, selected.path, false);
    Ok(())
}

fn handle_worktree_remove_children_confirm_mode_key(
    app: &mut App,
    code: KeyCode,
) -> Result<(), Box<dyn Error>> {
    match code {
        KeyCode::Esc => {
            app.mode = Mode::Normal;
            app.confirm_remove_worktree_yes = false;
            app.pending_remove_worktree_path.clear();
            app.pending_remove_worktree_children.clear();
            app.status_line = "Delete cancelled".to_string();
        }
        KeyCode::Left | KeyCode::Right | KeyCode::Tab => {
            app.confirm_remove_worktree_yes = !app.confirm_remove_worktree_yes;
        }
        KeyCode::Char('y') => app.confirm_remove_worktree_yes = true,
        KeyCode::Char('n') => app.confirm_remove_worktree_yes = false,
        KeyCode::Enter => {
            if app.confirm_remove_worktree_yes {
                let paths = pending_remove_worktree_targets(app);
                if paths.is_empty() {
                    app.status_line = "Delete cancelled (missing worktree path)".to_string();
                } else {
                    let has_dirty = paths.iter().any(|path| {
                        app.worktrees
                            .iter()
                            .find(|worktree| worktree.path == *path)
                            .map(|worktree| worktree.dirty)
                            .unwrap_or(false)
                    });

                    if has_dirty {
                        app.mode = Mode::WorktreeRemoveDirtyConfirm;
                        app.confirm_remove_worktree_yes = false;
                        app.status_line =
                            "One or more selected worktrees have uncommitted changes".to_string();
                        return Ok(());
                    }

                    start_remove_worktrees_task(app, paths, false);
                }
            } else {
                app.status_line = "Delete cancelled".to_string();
            }

            app.mode = Mode::Normal;
            app.confirm_remove_worktree_yes = false;
            app.pending_remove_worktree_path.clear();
            app.pending_remove_worktree_children.clear();
        }
        _ => {}
    }

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
            app.pending_remove_worktree_children.clear();
            app.status_line = "Delete cancelled".to_string();
        }
        KeyCode::Left | KeyCode::Right | KeyCode::Tab => {
            app.confirm_remove_worktree_yes = !app.confirm_remove_worktree_yes;
        }
        KeyCode::Char('y') => app.confirm_remove_worktree_yes = true,
        KeyCode::Char('n') => app.confirm_remove_worktree_yes = false,
        KeyCode::Enter => {
            if app.confirm_remove_worktree_yes {
                let paths = pending_remove_worktree_targets(app);
                if paths.is_empty() {
                    app.status_line = "Delete cancelled (missing worktree path)".to_string();
                } else {
                    start_remove_worktrees_task(app, paths, true);
                }
            } else {
                app.status_line = "Delete cancelled".to_string();
            }

            app.mode = Mode::Normal;
            app.confirm_remove_worktree_yes = false;
            app.pending_remove_worktree_path.clear();
            app.pending_remove_worktree_children.clear();
        }
        _ => {}
    }

    Ok(())
}

fn start_remove_worktree_task(app: &mut App, worktree_path: String, force: bool) {
    start_remove_worktrees_task(app, vec![worktree_path], force);
}

fn pending_remove_worktree_targets(app: &App) -> Vec<String> {
    if app.pending_remove_worktree_path.is_empty() {
        return Vec::new();
    }

    let mut targets = app.pending_remove_worktree_children.clone();
    if !targets
        .iter()
        .any(|path| path == app.pending_remove_worktree_path.as_str())
    {
        targets.push(app.pending_remove_worktree_path.clone());
    }
    targets
}

fn removable_descendant_paths(app: &App, parent_path: &str) -> Vec<String> {
    if app.worktrees.is_empty() {
        return Vec::new();
    }

    let root_branch = current_session_branch(app);
    let parents = worktree_parent_map(&app.worktrees, root_branch.as_str());
    let Some(parent_idx) = app
        .worktrees
        .iter()
        .position(|worktree| worktree.path == parent_path)
    else {
        return Vec::new();
    };

    let mut children: Vec<Vec<usize>> = vec![Vec::new(); app.worktrees.len()];
    for (idx, parent) in parents.iter().enumerate() {
        if let Some(parent_idx_for_node) = parent {
            if *parent_idx_for_node != idx && *parent_idx_for_node < children.len() {
                children[*parent_idx_for_node].push(idx);
            }
        }
    }

    fn collect_postorder(idx: usize, children: &[Vec<usize>], out: &mut Vec<usize>) {
        for child in &children[idx] {
            collect_postorder(*child, children, out);
            out.push(*child);
        }
    }

    let mut ordered_idxs: Vec<usize> = Vec::new();
    collect_postorder(parent_idx, &children, &mut ordered_idxs);
    ordered_idxs
        .into_iter()
        .filter_map(|idx| app.worktrees.get(idx).map(|worktree| worktree.path.clone()))
        .collect()
}

fn start_remove_worktrees_task(app: &mut App, worktree_paths: Vec<String>, force: bool) {
    if worktree_paths.is_empty() {
        app.status_line = "Delete cancelled (no worktrees selected)".to_string();
        return;
    }

    let mut targets: Vec<WorktreeEntry> = Vec::new();
    for path in &worktree_paths {
        let Some(worktree) = app
            .worktrees
            .iter()
            .find(|worktree| worktree.path == *path)
            .cloned()
        else {
            app.status_line = format!("Worktree not found: {}", path);
            return;
        };

        if worktree.is_current {
            app.status_line = format!("Refusing to remove current worktree: {}", worktree.path);
            return;
        }

        if worktree.dirty && !force {
            app.status_line = format!(
                "Refusing to remove dirty worktree without force: {}",
                worktree.path
            );
            return;
        }

        targets.push(worktree);
    }

    let mut closed_sessions = 0usize;
    for target in &targets {
        if has_live_terminal_session(app, target.path.as_str()) {
            closed_sessions = closed_sessions.saturating_add(1);
        }
        terminate_terminal_session(app, target.path.as_str());

        if app.agent_popup_path.as_deref() == Some(target.path.as_str()) {
            app.agent_popup_path = None;
            if matches!(app.mode, Mode::AgentPopup) {
                app.mode = Mode::Normal;
            }
        }
    }

    let target_paths = targets
        .into_iter()
        .map(|worktree| worktree.path)
        .collect::<Vec<_>>();
    let target_count = target_paths.len();
    let label = match (force, target_count > 1) {
        (true, true) => "Force-remove selected worktree tree",
        (true, false) => "Force-remove selected worktree",
        (false, true) => "Remove selected worktree tree",
        (false, false) => "Remove selected worktree",
    };

    start_git_task(app, label, true, true, move || {
        let mut outcomes: Vec<String> = Vec::new();
        for path in &target_paths {
            let result = if force {
                run_git(&["worktree", "remove", "--force", path.as_str()])
            } else {
                run_git(&["worktree", "remove", path.as_str()])
            };

            outcomes.push(format!("{}: {}", path, git_result_text(result)));
        }

        let mut combined = outcomes.join(" | ");
        if closed_sessions > 0 {
            combined
                .push_str(format!(" (closed {} terminal session(s))", closed_sessions).as_str());
        }
        combined
    });
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

    if let Some(scroll_delta) = terminal_popup_scroll_delta(key) {
        adjust_terminal_popup_scrollback(app, path.as_str(), scroll_delta);
        return Ok(());
    }

    reset_terminal_popup_scrollback(app, path.as_str());

    match code {
        KeyCode::Esc => {
            write_to_agent(app, path.as_str(), "\x1b")?;
        }
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

fn terminal_popup_scroll_delta(key: KeyEvent) -> Option<isize> {
    if !key.modifiers.contains(KeyModifiers::SHIFT)
        || key
            .modifiers
            .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SUPER)
    {
        return None;
    }

    match key.code {
        KeyCode::Up => Some(1),
        KeyCode::Down => Some(-1),
        _ => None,
    }
}

fn adjust_terminal_popup_scrollback(app: &mut App, path: &str, delta: isize) {
    let Some(session) = app.agent_sessions.get_mut(path) else {
        return;
    };

    let scrollback = session.parser.screen().scrollback();
    let next = if delta.is_negative() {
        scrollback.saturating_sub(delta.unsigned_abs())
    } else {
        scrollback.saturating_add(delta as usize)
    };

    session.parser.set_scrollback(next);
}

fn reset_terminal_popup_scrollback(app: &mut App, path: &str) {
    if let Some(session) = app.agent_sessions.get_mut(path) {
        if session.parser.screen().scrollback() != 0 {
            session.parser.set_scrollback(0);
        }
    }
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
    open_notes_popup_inline(app, path.as_str(), NotesContext::Notes)?;
    app.status_line = format!("Opened {} (vim-style mode)", app.notes_path);
    Ok(())
}

fn open_notes_popup_inline(
    app: &mut App,
    path: &str,
    context: NotesContext,
) -> Result<(), Box<dyn Error>> {
    let content = fs::read_to_string(path)?;
    let mut lines = content
        .split('\n')
        .map(|line| line.to_string())
        .collect::<Vec<String>>();
    if lines.is_empty() {
        lines.push(String::new());
    }

    app.notes_path = path.to_string();
    app.notes_lines = lines;
    app.notes_cursor_row = app.notes_lines.len().saturating_sub(1);
    app.notes_cursor_col = line_char_len(app.notes_lines[app.notes_cursor_row].as_str());
    app.notes_scroll = 0;
    app.notes_context = context;
    app.notes_edit_mode = NotesEditMode::Normal;
    app.notes_pending_op = None;
    app.mode = Mode::NotesPopup;
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

    open_notes_popup_inline(app, prompt_path.as_str(), NotesContext::ConflictPrompt)?;
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

    if app.notes_edit_mode == NotesEditMode::Insert {
        match key.code {
            KeyCode::Esc => {
                app.notes_edit_mode = NotesEditMode::Normal;
                app.notes_pending_op = None;
            }
            KeyCode::Up => move_notes_up(app),
            KeyCode::Down => move_notes_down(app),
            KeyCode::Left => move_notes_left(app),
            KeyCode::Right => move_notes_right(app),
            KeyCode::Home => app.notes_cursor_col = 0,
            KeyCode::End => {
                app.notes_cursor_col =
                    line_char_len(app.notes_lines[app.notes_cursor_row].as_str());
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
            KeyCode::Enter => insert_newline_at_cursor(app),
            KeyCode::Backspace => backspace_in_notes(app),
            KeyCode::Delete => delete_at_cursor(app),
            KeyCode::Tab => insert_text_at_cursor(app, "    "),
            KeyCode::Char(c) => insert_char_at_cursor(app, c),
            _ => {}
        }
    } else {
        match key.code {
            KeyCode::Esc => {
                app.notes_pending_op = None;
            }
            KeyCode::Up | KeyCode::Char('k') => move_notes_up(app),
            KeyCode::Down | KeyCode::Char('j') => move_notes_down(app),
            KeyCode::Left | KeyCode::Char('h') => move_notes_left(app),
            KeyCode::Right | KeyCode::Char('l') => move_notes_right(app),
            KeyCode::Home | KeyCode::Char('0') => app.notes_cursor_col = 0,
            KeyCode::End | KeyCode::Char('$') => {
                app.notes_cursor_col =
                    line_char_len(app.notes_lines[app.notes_cursor_row].as_str());
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
            KeyCode::Char('q') => {
                close_notes_popup_after_save(app)?;
            }
            KeyCode::Char('i') => {
                app.notes_edit_mode = NotesEditMode::Insert;
                app.notes_pending_op = None;
            }
            KeyCode::Char('a') => {
                let line_len = line_char_len(app.notes_lines[app.notes_cursor_row].as_str());
                if app.notes_cursor_col < line_len {
                    app.notes_cursor_col += 1;
                }
                app.notes_edit_mode = NotesEditMode::Insert;
                app.notes_pending_op = None;
            }
            KeyCode::Char('A') => {
                app.notes_cursor_col =
                    line_char_len(app.notes_lines[app.notes_cursor_row].as_str());
                app.notes_edit_mode = NotesEditMode::Insert;
                app.notes_pending_op = None;
            }
            KeyCode::Char('o') => {
                let insert_at = app.notes_cursor_row.saturating_add(1);
                app.notes_lines.insert(insert_at, String::new());
                app.notes_cursor_row = insert_at;
                app.notes_cursor_col = 0;
                app.notes_edit_mode = NotesEditMode::Insert;
                app.notes_pending_op = None;
            }
            KeyCode::Char('O') => {
                let insert_at = app.notes_cursor_row;
                app.notes_lines.insert(insert_at, String::new());
                app.notes_cursor_col = 0;
                app.notes_edit_mode = NotesEditMode::Insert;
                app.notes_pending_op = None;
            }
            KeyCode::Char('x') | KeyCode::Delete => {
                delete_at_cursor(app);
                app.notes_pending_op = None;
            }
            KeyCode::Char('G') => {
                app.notes_cursor_row = app.notes_lines.len().saturating_sub(1);
                clamp_notes_cursor(app);
                app.notes_pending_op = None;
            }
            KeyCode::Char('d') => {
                if app.notes_pending_op == Some('d') {
                    delete_current_line(app);
                    app.notes_pending_op = None;
                } else {
                    app.notes_pending_op = Some('d');
                }
            }
            KeyCode::Char('g') => {
                if app.notes_pending_op == Some('g') {
                    app.notes_cursor_row = 0;
                    clamp_notes_cursor(app);
                    app.notes_pending_op = None;
                } else {
                    app.notes_pending_op = Some('g');
                }
            }
            _ => {
                app.notes_pending_op = None;
            }
        }
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

fn close_notes_popup_after_save(app: &mut App) -> Result<(), Box<dyn Error>> {
    save_notes_popup(app)?;
    app.notes_pending_op = None;
    app.notes_edit_mode = NotesEditMode::Normal;
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
            app.status_line = "Saved conflict prompt. Press Enter to launch OpenCode".to_string();
        }
    }

    Ok(())
}

fn move_notes_up(app: &mut App) {
    if app.notes_cursor_row > 0 {
        app.notes_cursor_row -= 1;
    }
    clamp_notes_cursor(app);
}

fn move_notes_down(app: &mut App) {
    if app.notes_cursor_row + 1 < app.notes_lines.len() {
        app.notes_cursor_row += 1;
    }
    clamp_notes_cursor(app);
}

fn move_notes_left(app: &mut App) {
    if app.notes_cursor_col > 0 {
        app.notes_cursor_col -= 1;
    } else if app.notes_cursor_row > 0 {
        app.notes_cursor_row -= 1;
        app.notes_cursor_col = line_char_len(app.notes_lines[app.notes_cursor_row].as_str());
    }
}

fn move_notes_right(app: &mut App) {
    let line_len = line_char_len(app.notes_lines[app.notes_cursor_row].as_str());
    if app.notes_cursor_col < line_len {
        app.notes_cursor_col += 1;
    } else if app.notes_cursor_row + 1 < app.notes_lines.len() {
        app.notes_cursor_row += 1;
        app.notes_cursor_col = 0;
    }
}

fn insert_newline_at_cursor(app: &mut App) {
    let current = app.notes_lines[app.notes_cursor_row].clone();
    let split_idx = char_to_byte_idx(current.as_str(), app.notes_cursor_col);
    let before = current[..split_idx].to_string();
    let after = current[split_idx..].to_string();
    app.notes_lines[app.notes_cursor_row] = before;
    app.notes_lines.insert(app.notes_cursor_row + 1, after);
    app.notes_cursor_row += 1;
    app.notes_cursor_col = 0;
}

fn backspace_in_notes(app: &mut App) {
    if app.notes_cursor_col > 0 {
        let line = &mut app.notes_lines[app.notes_cursor_row];
        let end = char_to_byte_idx(line.as_str(), app.notes_cursor_col);
        let start = char_to_byte_idx(line.as_str(), app.notes_cursor_col - 1);
        line.replace_range(start..end, "");
        app.notes_cursor_col -= 1;
    } else if app.notes_cursor_row > 0 {
        let current = app.notes_lines.remove(app.notes_cursor_row);
        app.notes_cursor_row -= 1;
        app.notes_cursor_col = line_char_len(app.notes_lines[app.notes_cursor_row].as_str());
        app.notes_lines[app.notes_cursor_row].push_str(current.as_str());
    }
}

fn delete_at_cursor(app: &mut App) {
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

fn delete_current_line(app: &mut App) {
    if app.notes_lines.len() == 1 {
        app.notes_lines[0].clear();
        app.notes_cursor_row = 0;
        app.notes_cursor_col = 0;
        return;
    }

    app.notes_lines.remove(app.notes_cursor_row);
    if app.notes_cursor_row >= app.notes_lines.len() {
        app.notes_cursor_row = app.notes_lines.len().saturating_sub(1);
    }
    clamp_notes_cursor(app);
}

fn insert_text_at_cursor(app: &mut App, text: &str) {
    let line = &mut app.notes_lines[app.notes_cursor_row];
    let idx = char_to_byte_idx(line.as_str(), app.notes_cursor_col);
    line.insert_str(idx, text);
    app.notes_cursor_col += text.chars().count();
}

fn insert_char_at_cursor(app: &mut App, c: char) {
    let line = &mut app.notes_lines[app.notes_cursor_row];
    let idx = char_to_byte_idx(line.as_str(), app.notes_cursor_col);
    line.insert(idx, c);
    app.notes_cursor_col += 1;
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

    let (shell, shell_args) = interactive_shell_command();
    let mut cmd = CommandBuilder::new(shell.as_str());
    for arg in shell_args {
        cmd.arg(arg);
    }
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
        agent_kind: None,
        parser: vt100::Parser::new(TERM_ROWS, TERM_COLS, 2000),
        master: Some(pair.master),
        writer: Some(writer),
        child: Some(child),
        last_size: (TERM_ROWS, TERM_COLS),
        launched_at: now,
        last_io_at: now,
        bytes_from_agent: 0,
        bytes_to_agent: 0,
        io_samples: VecDeque::new(),
        opencode_session_id: None,
        opencode_usage: None,
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
            let now = Instant::now();
            let bytes = text.len() as u64;
            session.bytes_to_agent = session.bytes_to_agent.saturating_add(bytes);
            session.last_io_at = now;
            record_agent_io_sample(session, now, 0, bytes);
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
                    let now = Instant::now();
                    let byte_count = bytes.len() as u64;
                    session.state = AgentState::Running;
                    session.bytes_from_agent = session.bytes_from_agent.saturating_add(byte_count);
                    session.last_io_at = now;
                    record_agent_io_sample(session, now, byte_count, 0);
                    session.parser.process(bytes.as_slice());
                }
            }
        }
    }
}

const TOKEN_RATE_WINDOW: Duration = Duration::from_secs(4);
const MAX_IO_SAMPLES: usize = 1024;

fn record_agent_io_sample(
    session: &mut AgentSession,
    now: Instant,
    bytes_from_agent: u64,
    bytes_to_agent: u64,
) {
    session.io_samples.push_back(IoSample {
        at: now,
        bytes_from_agent,
        bytes_to_agent,
    });

    while session.io_samples.len() > MAX_IO_SAMPLES {
        session.io_samples.pop_front();
    }

    while let Some(sample) = session.io_samples.front() {
        if now.saturating_duration_since(sample.at) <= TOKEN_RATE_WINDOW {
            break;
        }
        session.io_samples.pop_front();
    }
}

fn drain_git_task_events(app: &mut App) {
    while let Ok(event) = app.git_task_rx.try_recv() {
        app.git_task = None;
        app.status_line = format!("{}: {}", event.label, single_line(event.outcome.as_str()));

        if event.refresh_worktrees {
            refresh_worktrees(app);
        }
        if event.refresh_status {
            refresh_status(app);
        }

        if event.label == "Execute orchestrated worktrees" {
            launch_pending_orchestrator_agents(app);
        }

        // Start the next queued task, if any
        pop_next_git_task(app);
    }
}

fn launch_pending_orchestrator_agents(app: &mut App) {
    let Some(pending) = app.pending_orchestrator_launch.take() else {
        return;
    };

    let agent = preferred_orchestrator_agent(app);
    if agent.is_none() {
        app.status_line = format!(
            "Executed orchestration for '{}', but no agent CLI found to run prompts",
            truncate_text(single_line(pending.requirement.as_str()).as_str(), 80)
        );
        return;
    }
    let agent = agent.unwrap_or(ExternalAgent::Opencode);

    let mut launched = 0usize;
    let mut skipped = 0usize;
    let mut failed = 0usize;

    for node in pending.nodes {
        let Some(path) = app
            .worktrees
            .iter()
            .find(|wt| !wt.detached && wt.branch == node.branch)
            .map(|wt| wt.path.clone())
        else {
            failed = failed.saturating_add(1);
            continue;
        };

        match launch_orchestrated_prompt_in_background(
            app,
            path.as_str(),
            agent,
            node.prompt.as_str(),
        ) {
            Ok(LaunchPromptResult::Launched) => {
                launched = launched.saturating_add(1);
            }
            Ok(LaunchPromptResult::SkippedAlreadyRunning) => {
                skipped = skipped.saturating_add(1);
            }
            Err(_) => {
                failed = failed.saturating_add(1);
            }
        }
    }

    app.status_line = format!(
        "Orchestrator prompts started: {} launched, {} skipped, {} failed (agent: {})",
        launched,
        skipped,
        failed,
        agent.command_name()
    );
}

fn preferred_orchestrator_agent(app: &App) -> Option<ExternalAgent> {
    if let Some(default_agent) = app.config.default_agent {
        if app.detected_agents.contains(&default_agent) {
            return Some(default_agent);
        }
    }

    if app.detected_agents.contains(&ExternalAgent::Opencode) {
        return Some(ExternalAgent::Opencode);
    }

    app.detected_agents.first().copied()
}

enum LaunchPromptResult {
    Launched,
    SkippedAlreadyRunning,
}

fn launch_orchestrated_prompt_in_background(
    app: &mut App,
    path: &str,
    agent: ExternalAgent,
    prompt: &str,
) -> Result<LaunchPromptResult, Box<dyn Error>> {
    if !has_live_terminal_session(app, path) {
        launch_shell_session(app, path)?;
    }

    if app
        .agent_sessions
        .get(path)
        .map(|session| session.agent_kind.is_some() && session.state == AgentState::Running)
        .unwrap_or(false)
    {
        return Ok(LaunchPromptResult::SkippedAlreadyRunning);
    }

    wait_for_terminal_ready(app, path);

    if let Some(session) = app.agent_sessions.get_mut(path) {
        session
            .parser
            .process(b"\r\n[orchestrator launching background prompt]\r\n");
    }

    if agent == ExternalAgent::Opencode {
        let launch = build_opencode_launch_command(path, Some(prompt), false);
        write_to_agent(app, path, launch.command.as_str())?;
        attach_session_agent(app, path, agent, launch.session_id);
    } else {
        let launch_cmd = format!("{}\r", agent.command_name());
        write_to_agent(app, path, launch_cmd.as_str())?;
        attach_session_agent(app, path, agent, None);
        let prompt_with_enter = format!("{}\r", normalize_terminal_newlines(prompt));
        write_to_agent(app, path, prompt_with_enter.as_str())?;
    }

    Ok(LaunchPromptResult::Launched)
}

fn start_orchestrator_plan_task(
    app: &mut App,
    root: String,
    requirement_text: String,
    root_branch: String,
    selected_branch: String,
    existing_branches: Vec<String>,
    prompt_path: String,
    max_nodes: usize,
) {
    let tx = app.orchestrator_plan_tx.clone();
    thread::spawn(move || {
        let result = std::panic::catch_unwind(|| {
            plan_orchestrated_worktrees_from_requirement(
                root.as_str(),
                requirement_text.as_str(),
                root_branch.as_str(),
                selected_branch.as_str(),
                existing_branches,
                prompt_path.as_str(),
                max_nodes,
            )
        })
        .map_err(|_| "Planner crashed unexpectedly".to_string());
        let _ = tx.send(OrchestratorPlanEvent {
            requirement: requirement_text,
            result,
        });
    });
}

fn drain_orchestrator_plan_events(app: &mut App) {
    while let Ok(event) = app.orchestrator_plan_rx.try_recv() {
        match event.result {
            Ok(plan) => {
                if plan.nodes.is_empty() {
                    let message = "Orchestrator produced no valid worktree nodes".to_string();
                    app.orchestrator_plan_state = OrchestratorPlanState::Failed {
                        message: message.clone(),
                    };
                    app.status_line = message;
                    continue;
                }

                app.orchestrator_planned_requirement = event.requirement.clone();
                app.orchestrator_planner_source = plan.planner_source.to_string();
                app.orchestrator_prompt_nodes = plan
                    .nodes
                    .into_iter()
                    .map(|node| OrchestratorPromptNode {
                        prompt: build_leaf_execution_prompt(event.requirement.as_str(), &node),
                        branch: node.branch,
                        parent: node.parent,
                        goal: node.goal,
                        accepted: true,
                    })
                    .collect();
                app.orchestrator_prompt_selected = 0;
                app.orchestrator_prompt_edit_input.clear();
                app.orchestrator_requirement_input.clear();
                app.orchestrator_plan_state = OrchestratorPlanState::Idle;
                app.mode = Mode::WorktreeOrchestratePreview;

                if let Some(err) = plan.planner_error {
                    app.status_line = format!(
                        "Planner fallback: OpenCode failed, used heuristic ({})",
                        truncate_text(single_line(err.as_str()).as_str(), 120)
                    );
                } else {
                    app.status_line =
                        "Review leaf prompts: accept/refine each node, Enter executes accepted nodes"
                            .to_string();
                }
            }
            Err(err) => {
                let message = format!(
                    "Orchestrator planner crashed: {}",
                    single_line(err.as_str())
                );
                app.orchestrator_plan_state = OrchestratorPlanState::Failed {
                    message: message.clone(),
                };
                app.status_line = message;
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

const OPENCODE_USAGE_RATE_WINDOW_MS: u64 = 4000;

fn refresh_opencode_usage(app: &mut App) {
    let mut should_query = false;
    for (path, session) in app.agent_sessions.iter_mut() {
        let tracks_opencode = session.agent_kind == Some(ExternalAgent::Opencode)
            || session.opencode_session_id.is_some();
        if !tracks_opencode {
            continue;
        }
        should_query = true;
        if session.opencode_session_id.is_none() {
            session.opencode_session_id =
                resolve_recent_opencode_session_id_for_worktree(path.as_str());
        }
    }

    if !should_query || !command_exists_on_path("opencode") {
        for session in app.agent_sessions.values_mut() {
            if session.agent_kind == Some(ExternalAgent::Opencode)
                || session.opencode_session_id.is_some()
            {
                session.opencode_usage = None;
            }
        }
        return;
    }

    let mut ids = BTreeSet::new();
    for session in app.agent_sessions.values() {
        if let Some(session_id) = session.opencode_session_id.as_ref() {
            ids.insert(session_id.clone());
        }
    }

    if ids.is_empty() {
        for session in app.agent_sessions.values_mut() {
            if session.agent_kind == Some(ExternalAgent::Opencode) {
                session.opencode_usage = None;
            }
        }
        return;
    }

    let session_ids = ids.into_iter().collect::<Vec<_>>();
    let Ok(usage_by_session) = load_opencode_usage_for_sessions(session_ids.as_slice()) else {
        return;
    };

    for session in app.agent_sessions.values_mut() {
        let Some(session_id) = session.opencode_session_id.as_ref() else {
            if session.agent_kind == Some(ExternalAgent::Opencode) {
                session.opencode_usage = None;
            }
            continue;
        };

        session.opencode_usage = usage_by_session.get(session_id).copied();
    }
}

fn load_opencode_usage_for_sessions(
    session_ids: &[String],
) -> Result<BTreeMap<String, OpenCodeUsage>, Box<dyn Error>> {
    if session_ids.is_empty() {
        return Ok(BTreeMap::new());
    }

    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    let cutoff_ms = now_ms.saturating_sub(OPENCODE_USAGE_RATE_WINDOW_MS);

    let mut in_values = String::new();
    for (idx, session_id) in session_ids.iter().enumerate() {
        if idx > 0 {
            in_values.push(',');
        }
        in_values.push('\'');
        in_values.push_str(session_id.replace('\'', "''").as_str());
        in_values.push('\'');
    }

    let sql = format!(
        "WITH totals AS (\
            SELECT session_id,\
                   SUM(COALESCE(json_extract(data, '$.tokens.input'), 0)) AS input_tokens,\
                   SUM(COALESCE(json_extract(data, '$.tokens.output'), 0)) AS output_tokens\
              FROM part\
             WHERE json_extract(data, '$.type') = 'step-finish'\
               AND session_id IN ({in_values})\
             GROUP BY session_id\
        ), recent AS (\
            SELECT session_id,\
                   SUM(COALESCE(json_extract(data, '$.tokens.input'), 0)) AS input_tokens,\
                   SUM(COALESCE(json_extract(data, '$.tokens.output'), 0)) AS output_tokens\
              FROM part\
             WHERE json_extract(data, '$.type') = 'step-finish'\
               AND time_created >= {cutoff_ms}\
               AND session_id IN ({in_values})\
             GROUP BY session_id\
        )\
        SELECT totals.session_id,\
               totals.input_tokens,\
               totals.output_tokens,\
               COALESCE(recent.input_tokens, 0),\
               COALESCE(recent.output_tokens, 0)\
          FROM totals\
          LEFT JOIN recent ON recent.session_id = totals.session_id"
    );

    let output = Command::new("opencode")
        .args(["db", sql.as_str()])
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "OpenCode usage query failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    Ok(parse_opencode_usage_rows(
        String::from_utf8_lossy(&output.stdout).as_ref(),
        OPENCODE_USAGE_RATE_WINDOW_MS,
    ))
}

fn parse_opencode_usage_rows(tsv: &str, rate_window_ms: u64) -> BTreeMap<String, OpenCodeUsage> {
    let mut usage_by_session = BTreeMap::new();

    for (idx, line) in tsv.lines().enumerate() {
        if idx == 0 || line.trim().is_empty() {
            continue;
        }

        let cols = line.split('\t').collect::<Vec<_>>();
        if cols.len() < 5 {
            continue;
        }

        let session_id = cols[0].trim();
        if session_id.is_empty() {
            continue;
        }

        let input_tokens = parse_tsv_u64(cols[1]);
        let output_tokens = parse_tsv_u64(cols[2]);
        let recent_input_tokens = parse_tsv_u64(cols[3]);
        let recent_output_tokens = parse_tsv_u64(cols[4]);
        let window_secs = (rate_window_ms as f64 / 1000.0).max(1.0);

        usage_by_session.insert(
            session_id.to_string(),
            OpenCodeUsage {
                input_tokens,
                output_tokens,
                input_tokens_per_second: (recent_input_tokens as f64 / window_secs).round() as u64,
                output_tokens_per_second: (recent_output_tokens as f64 / window_secs).round()
                    as u64,
            },
        );
    }

    usage_by_session
}

fn parse_tsv_u64(value: &str) -> u64 {
    value.trim().parse::<u64>().unwrap_or(0)
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
