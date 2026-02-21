fn draw_ui(frame: &mut ratatui::Frame<'_>, app: &mut App) {
    frame.render_widget(Clear, frame.area());
    frame.render_widget(
        Block::default().style(Style::default().bg(Color::Black).fg(Color::White)),
        frame.area(),
    );

    if matches!(app.mode, Mode::AgentPopup) {
        draw_agent_popup(frame, app);
        return;
    }

    if app.view_mode == ViewMode::Changes {
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .spacing(1)
            .constraints([
                Constraint::Percentage(22),
                Constraint::Percentage(56),
                Constraint::Percentage(22),
            ])
            .split(frame.area());

        let right = Layout::default()
            .direction(Direction::Vertical)
            .spacing(1)
            .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
            .split(columns[2]);

        draw_files_panel(frame, app, columns[0]);
        draw_selected_overview_panel(frame, app, columns[1]);
        draw_pulse_panel(frame, app, right[0]);
        draw_changes_actions_panel(frame, right[1]);
    } else {
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .spacing(1)
            .constraints([Constraint::Percentage(72), Constraint::Percentage(28)])
            .split(frame.area());

        let right = Layout::default()
            .direction(Direction::Vertical)
            .spacing(1)
            .constraints(worktree_right_panel_constraints(app, columns[1]))
            .split(columns[1]);

        draw_worktree_canvas_panel(frame, app, columns[0]);
        draw_worktree_art_panel(frame, app, right[0]);
        draw_worktree_details_panel(frame, app, right[1]);
        draw_worktree_actions_panel(frame, app, right[2]);
    }

    if matches!(app.mode, Mode::CommitInput) {
        draw_commit_modal(frame, app);
    }

    if matches!(app.mode, Mode::WorktreeCommitPushInput) {
        draw_worktree_commit_push_modal(frame, app);
    }

    if matches!(app.mode, Mode::NotesPopup) {
        draw_notes_modal(frame, app);
    }

    if matches!(app.mode, Mode::WorktreeCreateInput) {
        draw_worktree_create_modal(frame, app);
    }

    if matches!(app.mode, Mode::WorktreeOrchestrateInput) {
        draw_worktree_orchestrate_modal(frame, app);
    }

    if matches!(app.mode, Mode::WorktreeBranchConflictConfirm) {
        draw_branch_conflict_confirm_modal(frame, app);
    }

    if matches!(app.mode, Mode::WorktreeConflictResolveConfirm) {
        draw_conflict_resolve_confirm_modal(frame, app);
    }

    if matches!(app.mode, Mode::WorktreeRemoveDirtyConfirm) {
        draw_worktree_remove_dirty_confirm_modal(frame, app);
    }

    if matches!(app.mode, Mode::WorktreeGitLogPopup) {
        draw_worktree_git_log_modal(frame, app);
    }

    if matches!(app.mode, Mode::QuitWithSessionsConfirm) {
        draw_quit_with_sessions_modal(frame, app);
    }

    if matches!(app.mode, Mode::LegacyWorkspaceMigrateConfirm) {
        draw_legacy_workspace_migrate_modal(frame, app);
    }

    if matches!(app.mode, Mode::AgentSelectPopup) {
        draw_agent_select_modal(frame, app);
    }

    if app.view_mode == ViewMode::Worktrees && app.show_panel_help {
        draw_worktree_help_modal(frame, app);
    }
}

fn draw_files_panel(frame: &mut ratatui::Frame<'_>, app: &App, area: Rect) {
    frame.render_widget(Clear, area);

    let content_width = area.width.saturating_sub(6) as usize;
    let mut items: Vec<ListItem<'_>> = Vec::new();
    let mut index_map: Vec<Option<usize>> = Vec::new();
    let count_width = app
        .tree_items
        .iter()
        .map(|item| {
            item.added_lines
                .max(item.removed_lines)
                .to_string()
                .chars()
                .count()
        })
        .max()
        .unwrap_or(1)
        .max(4);

    let unstaged_indices: Vec<usize> = app
        .tree_items
        .iter()
        .enumerate()
        .filter(|(_, item)| item.unstaged || item.untracked)
        .map(|(idx, _)| idx)
        .collect();
    let staged_indices: Vec<usize> = app
        .tree_items
        .iter()
        .enumerate()
        .filter(|(_, item)| item.staged && !item.unstaged && !item.untracked)
        .map(|(idx, _)| idx)
        .collect();

    push_section_header(
        &mut items,
        &mut index_map,
        "unstaged",
        unstaged_indices.len(),
    );
    if unstaged_indices.is_empty() {
        items.push(ListItem::new(Line::from(Span::styled(
            "  clean",
            Style::default().fg(Color::DarkGray),
        ))));
        index_map.push(None);
    } else {
        for idx in unstaged_indices {
            push_tree_row(
                &mut items,
                &mut index_map,
                idx,
                &app.tree_items[idx],
                content_width,
                count_width,
            );
        }
    }

    items.push(ListItem::new(Line::from("")));
    index_map.push(None);

    push_section_header(&mut items, &mut index_map, "staged", staged_indices.len());
    if staged_indices.is_empty() {
        items.push(ListItem::new(Line::from(Span::styled(
            "  clean",
            Style::default().fg(Color::DarkGray),
        ))));
        index_map.push(None);
    } else {
        for idx in staged_indices {
            push_tree_row(
                &mut items,
                &mut index_map,
                idx,
                &app.tree_items[idx],
                content_width,
                count_width,
            );
        }
    }

    let selected_render_idx = index_map
        .iter()
        .position(|mapped| *mapped == Some(app.selected));

    let mut state = ListState::default();
    if let Some(idx) = selected_render_idx {
        state.select(Some(idx));
    }

    let border_color = if app.active_pane == ActivePane::Files {
        Color::Cyan
    } else {
        Color::Gray
    };

    let list = List::new(items)
        .block(
            Block::default()
                .title("changed files")
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::Black))
                .border_style(Style::default().fg(border_color)),
        )
        .highlight_style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::UNDERLINED),
        )
        .highlight_symbol("▶ ")
        .style(Style::default().bg(Color::Black));

    frame.render_stateful_widget(list, area, &mut state);
}

fn push_section_header(
    items: &mut Vec<ListItem<'_>>,
    index_map: &mut Vec<Option<usize>>,
    title: &str,
    count: usize,
) {
    items.push(ListItem::new(Line::from(vec![Span::styled(
        format!("{} ({})", title, count),
        Style::default()
            .fg(Color::Gray)
            .add_modifier(Modifier::BOLD),
    )])));
    index_map.push(None);
}

fn push_tree_row(
    items: &mut Vec<ListItem<'_>>,
    index_map: &mut Vec<Option<usize>>,
    idx: usize,
    item: &TreeItem,
    content_width: usize,
    count_width: usize,
) {
    let mut spans: Vec<Span<'_>> = Vec::new();
    let name_color = if item.kind == TreeKind::Folder {
        Color::LightYellow
    } else {
        Color::LightCyan
    };

    let plus_text = format!("+{:>width$}", item.added_lines, width = count_width);
    let minus_text = format!("-{:>width$}", item.removed_lines, width = count_width);
    let right_len = plus_text.chars().count() + 1 + minus_text.chars().count();
    let label_col = content_width.saturating_sub(right_len).max(8);
    let label = truncate_text(item.label.as_str(), label_col);
    let padding = label_col.saturating_sub(label.chars().count());

    spans.push(Span::styled(label, Style::default().fg(name_color)));
    spans.push(Span::raw(" ".repeat(padding)));
    spans.push(Span::styled(plus_text, Style::default().fg(Color::Green)));
    spans.push(Span::raw(" "));
    spans.push(Span::styled(minus_text, Style::default().fg(Color::Red)));

    items.push(ListItem::new(Line::from(spans)));
    index_map.push(Some(idx));
}

fn draw_selected_overview_panel(frame: &mut ratatui::Frame<'_>, app: &App, area: Rect) {
    frame.render_widget(Clear, area);

    let info = app.selected_overview.as_ref();

    let mut lines: Vec<Line<'_>> = Vec::new();
    if let Some(info) = info {
        lines.push(Line::from(vec![
            Span::styled("file: ", Style::default().fg(Color::Gray)),
            Span::styled(info.file.as_str(), Style::default().fg(Color::White)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("state: ", Style::default().fg(Color::Gray)),
            Span::styled(info.state.as_str(), Style::default().fg(Color::Cyan)),
        ]));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            "files changes",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(vec![
            Span::styled(
                "+",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                info.added_lines.to_string(),
                Style::default().fg(Color::Green),
            ),
            Span::raw("  "),
            Span::styled(
                "-",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                info.removed_lines.to_string(),
                Style::default().fg(Color::Red),
            ),
        ]));
        lines.push(Line::from(""));

        if info.use_traditional_overview {
            if info.traditional_diff.is_empty() {
                lines.push(Line::from("No diff preview available"));
            } else {
                lines.push(Line::from("diff preview:"));
                for row in info.traditional_diff.iter().take(24) {
                    let color = match row.kind {
                        DiffPreviewKind::Added => Color::Green,
                        DiffPreviewKind::Removed => Color::Red,
                        DiffPreviewKind::Meta => Color::Blue,
                        DiffPreviewKind::Context => Color::Gray,
                    };
                    lines.push(Line::from(Span::styled(
                        row.text.as_str(),
                        Style::default().fg(color),
                    )));
                }
            }
        } else {
            lines.push(Line::from(vec![
                Span::styled(
                    "methods",
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    " (j/k select, J/K scroll, Enter/Space expand)",
                    Style::default().fg(Color::Gray),
                ),
            ]));

            if info.method_changes.is_empty() {
                lines.push(Line::from("No method-level entries found"));
                lines.push(Line::from(""));
            } else {
                for (idx, method) in info.method_changes.iter().enumerate() {
                    let selected = idx == app.overview_method_index;
                    let marker = if selected { "> " } else { "  " };
                    let expanded = selected && app.overview_method_expanded;
                    let fold = if expanded { "[-]" } else { "[+]" };
                    let (kind_label, kind_color) = match method.kind {
                        MethodChangeKind::Added => ("A", Color::LightGreen),
                        MethodChangeKind::Modified => ("M", Color::Yellow),
                        MethodChangeKind::Deleted => ("D", Color::LightRed),
                    };
                    let name_color = if selected { Color::Cyan } else { Color::White };

                    lines.push(Line::from(vec![
                        Span::styled(marker, Style::default().fg(Color::Gray)),
                        Span::styled(fold, Style::default().fg(Color::Gray)),
                        Span::raw(" "),
                        Span::styled(kind_label, Style::default().fg(kind_color)),
                        Span::raw(" "),
                        Span::styled(
                            truncate_text(method.name.as_str(), 54),
                            Style::default().fg(name_color),
                        ),
                    ]));

                    if expanded {
                        if method.diff_lines.is_empty() {
                            lines.push(Line::from(Span::styled(
                                "    no hunk preview available",
                                Style::default().fg(Color::DarkGray),
                            )));
                        } else {
                            for row in method.diff_lines.iter().take(40) {
                                let color = match row.kind {
                                    DiffPreviewKind::Added => Color::Green,
                                    DiffPreviewKind::Removed => Color::Red,
                                    DiffPreviewKind::Meta => Color::Blue,
                                    DiffPreviewKind::Context => Color::Gray,
                                };
                                lines.push(Line::from(vec![
                                    Span::raw("    "),
                                    Span::styled(row.text.as_str(), Style::default().fg(color)),
                                ]));
                            }
                        }
                        lines.push(Line::from(""));
                    }
                }
            }
        }
    } else {
        lines.push(Line::from("No changed file selected"));
    }

    let panel = Paragraph::new(lines)
        .scroll((app.overview_scroll, 0))
        .block(
            Block::default()
                .title("selected file overview")
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::Black))
                .border_style(
                    Style::default().fg(if app.active_pane == ActivePane::Overview {
                        Color::Cyan
                    } else {
                        Color::Gray
                    }),
                ),
        )
        .style(Style::default().bg(Color::Black).fg(Color::White))
        .alignment(Alignment::Left);

    frame.render_widget(panel, area);
}

fn draw_pulse_panel(frame: &mut ratatui::Frame<'_>, app: &App, area: Rect) {
    let staged_count = app.files.iter().filter(|f| f.staged).count();
    let unstaged_count = app
        .files
        .iter()
        .filter(|f| f.unstaged || f.untracked)
        .count();

    let status_limit = area.width.saturating_sub(12) as usize;
    let (status_text, status_color) = if let Some(task) = app.git_task.as_ref() {
        let elapsed = Instant::now().saturating_duration_since(task.started_at);
        let queued = app.git_task_queue.len();
        let queue_suffix = if queued > 0 {
            format!(" +{} queued", queued)
        } else {
            String::new()
        };
        (
            truncate_text(
                format!(
                    "[{}] {} ({:.1}s){}",
                    spinner_glyph(elapsed),
                    task.label,
                    elapsed.as_secs_f32(),
                    queue_suffix,
                )
                .as_str(),
                status_limit.max(10),
            ),
            Color::LightYellow,
        )
    } else {
        (
            truncate_text(
                single_line(app.status_line.as_str()).as_str(),
                status_limit.max(10),
            ),
            Color::White,
        )
    };

    let info = vec![
        Line::from(vec![
            Span::styled("Branch: ", Style::default().fg(Color::Gray)),
            Span::styled(
                app.branch.as_str(),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Ahead: ", Style::default().fg(Color::Gray)),
            Span::styled(app.ahead.to_string(), Style::default().fg(Color::Green)),
            Span::raw("  "),
            Span::styled("Behind: ", Style::default().fg(Color::Gray)),
            Span::styled(app.behind.to_string(), Style::default().fg(Color::Yellow)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Staged files: ", Style::default().fg(Color::Gray)),
            Span::styled(staged_count.to_string(), Style::default().fg(Color::Green)),
        ]),
        Line::from(vec![
            Span::styled("Unstaged files: ", Style::default().fg(Color::Gray)),
            Span::styled(
                unstaged_count.to_string(),
                Style::default().fg(Color::Yellow),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Status: ", Style::default().fg(Color::Gray)),
            Span::styled(status_text, Style::default().fg(status_color)),
        ]),
    ];

    let panel = Paragraph::new(info)
        .block(
            Block::default()
                .title("pulse")
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::Black))
                .border_style(Style::default().fg(Color::Gray)),
        )
        .style(Style::default().bg(Color::Black).fg(Color::White))
        .alignment(Alignment::Left);

    frame.render_widget(panel, area);
}

fn draw_changes_actions_panel(frame: &mut ratatui::Frame<'_>, area: Rect) {
    let lines = vec![
        Line::from(vec![
            Span::styled("w", Style::default().fg(Color::LightBlue)),
            Span::raw(" worktree canvas"),
        ]),
        Line::from(vec![
            Span::styled("h/l", Style::default().fg(Color::LightBlue)),
            Span::raw(" focus files/overview"),
        ]),
        Line::from(vec![
            Span::styled("j/k", Style::default().fg(Color::LightBlue)),
            Span::raw(" move selection/method"),
        ]),
        Line::from(vec![
            Span::styled("J/K", Style::default().fg(Color::LightBlue)),
            Span::raw(" scroll overview"),
        ]),
        Line::from(vec![
            Span::styled("a", Style::default().fg(Color::LightGreen)),
            Span::raw(" smart stage / unstage"),
        ]),
        Line::from(vec![
            Span::styled(
                "enter|space (overview)",
                Style::default().fg(Color::LightGreen),
            ),
            Span::raw(" expand selected method"),
        ]),
        Line::from(vec![
            Span::styled("u", Style::default().fg(Color::LightGreen)),
            Span::raw(" unstage selected"),
        ]),
        Line::from(vec![
            Span::styled("A / U", Style::default().fg(Color::LightGreen)),
            Span::raw(" stage all / unstage all"),
        ]),
        Line::from(vec![
            Span::styled("c", Style::default().fg(Color::Yellow)),
            Span::raw(" commit"),
        ]),
        Line::from(vec![
            Span::styled("n", Style::default().fg(Color::LightCyan)),
            Span::raw(" notes (vim-style)"),
        ]),
        Line::from(vec![
            Span::styled("p", Style::default().fg(Color::Magenta)),
            Span::raw(" push"),
        ]),
        Line::from(vec![
            Span::styled("s", Style::default().fg(Color::LightYellow)),
            Span::raw(" stash changes"),
        ]),
        Line::from(vec![
            Span::styled("S", Style::default().fg(Color::Yellow)),
            Span::raw(" stash pop"),
        ]),
        Line::from(vec![
            Span::styled("r", Style::default().fg(Color::Cyan)),
            Span::raw(" refresh"),
        ]),
        Line::from(vec![
            Span::styled("q", Style::default().fg(Color::Red)),
            Span::raw(" quit"),
        ]),
    ];

    let panel = Paragraph::new(lines)
        .block(
            Block::default()
                .title("actions")
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::Black))
                .border_style(Style::default().fg(Color::Gray)),
        )
        .style(Style::default().bg(Color::Black).fg(Color::White))
        .alignment(Alignment::Left);

    frame.render_widget(panel, area);
}

fn draw_worktree_canvas_panel(frame: &mut ratatui::Frame<'_>, app: &mut App, area: Rect) {
    let border_color = if app.worktree_focus == WorktreePane::Canvas {
        Color::Cyan
    } else {
        Color::Gray
    };
    let perf_badge = if app.perf_debug.enabled {
        "  perf:on"
    } else {
        ""
    };
    let title = if app.worktree_canvas_zoom != 1.0
        || app.worktree_canvas_pan_x != 0.0
        || app.worktree_canvas_pan_y != 0.0
    {
        format!(
            "worktree graph [?]  {}  z:{:.1}x  bg:{}{}",
            app.worktree_graph_builder.label(),
            app.worktree_canvas_zoom,
            app.worktree_canvas_bg_mode.short_label(),
            perf_badge
        )
    } else {
        format!(
            "worktree graph [?]  {}  bg:{}{}",
            app.worktree_graph_builder.label(),
            app.worktree_canvas_bg_mode.short_label(),
            perf_badge
        )
    };
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .style(Style::default());

    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.width < 10 || inner.height < 6 {
        return;
    }

    let root_branch = current_session_branch(app);

    if app.worktrees.is_empty() {
        let empty_text = app
            .worktree_load_error
            .as_ref()
            .map(|reason| {
                format!(
                    "Unable to load worktrees\n{}\n\nRun from a git repo/worktree path and verify `git worktree list --porcelain`",
                    truncate_text(single_line(reason.as_str()).as_str(), 120)
                )
            })
            .unwrap_or_else(|| "No worktrees. Press 'a' to create one.".to_string());
        frame.render_widget(
            Paragraph::new(empty_text)
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::DarkGray)),
            inner,
        );
        return;
    }

    let parents = worktree_parent_map(&app.worktrees, root_branch.as_str());
    let collapsed_root_idx = None;
    let logical = graph_layout(&parents, app.worktree_graph_builder, &app.worktrees);
    let node_points: Vec<(f64, f64)> = logical
        .iter()
        .map(|point| logical_to_canvas_point(*point))
        .collect();
    let root_point = None;
    let bounds = worktree_canvas_bounds(app);
    let selected_idx = app
        .selected_worktree
        .min(app.worktrees.len().saturating_sub(1));
    let canvas_selected_idx = if Some(selected_idx) == collapsed_root_idx {
        app.worktrees
            .iter()
            .enumerate()
            .find_map(|(idx, _)| (Some(idx) != collapsed_root_idx).then_some(idx))
            .unwrap_or(selected_idx)
    } else {
        selected_idx
    };
    let mut screen_points: Vec<(u16, u16)> = Vec::with_capacity(node_points.len());
    for point in &node_points {
        if let Some((sx, sy)) = canvas_point_to_screen(inner, bounds, *point) {
            screen_points.push((sx, sy));
        } else {
            screen_points.push((inner.x, inner.y));
        }
    }

    let buf = frame.buffer_mut();
    draw_worktree_canvas_background(buf, inner, app);
    draw_unicode_worktree_graph(
        buf,
        inner,
        &parents,
        &screen_points,
        root_point,
        app,
        canvas_selected_idx,
        collapsed_root_idx,
    );

    let mut label_areas: BTreeMap<String, Rect> = BTreeMap::new();
    let mut node_centers: BTreeMap<String, (u16, u16)> = BTreeMap::new();
    let mut logical_points_by_path: BTreeMap<String, (f64, f64)> = BTreeMap::new();
    let mut selected_label_area: Option<Rect> = None;

    for (idx, entry) in app.worktrees.iter().enumerate() {
        if Some(idx) == collapsed_root_idx {
            continue;
        }
        let selected = idx == selected_idx;
        let label = canvas_node_label(app, entry, selected);
        let (badge_text, badge_style) = node_merge_badge(entry);
        let style = if selected {
            Style::default()
                .fg(Color::LightCyan)
                .add_modifier(Modifier::BOLD)
        } else if entry.dirty {
            Style::default().fg(Color::Red)
        } else if entry.behind_parent {
            Style::default().fg(Color::Yellow)
        } else if entry.is_current {
            Style::default()
                .fg(Color::LightBlue)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        logical_points_by_path.insert(entry.path.clone(), node_points[idx]);

        if let Some((sx, sy)) = canvas_point_to_screen(inner, bounds, node_points[idx]) {
            node_centers.insert(entry.path.clone(), (sx, sy));
        }

        if let Some(label_area) = draw_canvas_label(
            frame,
            inner,
            bounds,
            node_points[idx],
            label.as_str(),
            style,
            badge_text,
            badge_style,
        ) {
            label_areas.insert(entry.path.clone(), label_area);
            if selected {
                selected_label_area = Some(label_area);
            }
        }
    }

    app.last_worktree_node_points = logical_points_by_path;

    if let Some(selected_area) = selected_label_area {
        let elapsed = app.canvas_selected_border_last_tick.elapsed();
        app.canvas_selected_border_last_tick = Instant::now();
        app.canvas_selected_border_effects.process_effects(
            elapsed.into(),
            frame.buffer_mut(),
            selected_area,
        );
        draw_spinning_border_shine(frame.buffer_mut(), selected_area);
        if let Some(selected) = app.selected_worktree() {
            let selected_label = canvas_node_label(app, selected, true);
            let (selected_badge, selected_badge_style) = node_merge_badge(selected);
            let text_x = selected_area.x.saturating_add(2);
            let text_y = selected_area.y.saturating_add(1);
            frame.render_widget(
                Paragraph::new(selected_label).style(
                    Style::default()
                        .fg(Color::LightCyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Rect::new(text_x, text_y, selected_area.width.saturating_sub(4), 1),
            );
            frame.render_widget(
                Paragraph::new(selected_badge)
                    .style(selected_badge_style.add_modifier(Modifier::DIM)),
                Rect::new(
                    text_x,
                    text_y.saturating_add(1),
                    selected_area.width.saturating_sub(4),
                    1,
                ),
            );
        }
    }

    app.canvas_node_animations.retain_mut(|animation| {
        let target_area = match &animation.target {
            CanvasNodeAnimationTarget::Path(path) => label_areas.get(path).copied().or_else(|| {
                node_centers
                    .get(path)
                    .copied()
                    .map(|point| pulse_rect(inner, point))
            }),
            CanvasNodeAnimationTarget::Point(point) => {
                canvas_point_to_screen(inner, bounds, *point)
                    .map(|screen| pulse_rect(inner, screen))
            }
        };

        let Some(effect_area) = target_area else {
            return animation.effects.is_running();
        };

        if animation.kind == CanvasNodeAnimationKind::Deleted {
            let center_x = effect_area.x + effect_area.width / 2;
            let center_y = effect_area.y + effect_area.height / 2;
            if let Some(cell) = frame.buffer_mut().cell_mut((center_x, center_y)) {
                cell.set_symbol("◌");
                cell.set_style(Style::default().fg(Color::DarkGray));
            }
        }

        let elapsed = animation.last_tick.elapsed();
        animation.last_tick = Instant::now();
        animation
            .effects
            .process_effects(elapsed.into(), frame.buffer_mut(), effect_area);
        animation.effects.is_running()
    });

    draw_worktree_token_leaderboard(frame, app, inner);
}

#[derive(Clone)]
struct WorktreeTokenLeaderboardRow {
    label: String,
    tokens_per_second: u64,
    is_selected: bool,
    is_idle_cluster: bool,
}

fn draw_worktree_token_leaderboard(frame: &mut ratatui::Frame<'_>, app: &App, canvas_area: Rect) {
    if canvas_area.width < 34 || canvas_area.height < 8 {
        return;
    }

    let now = Instant::now();
    let mut active_rows: Vec<WorktreeTokenLeaderboardRow> = Vec::new();
    let mut idle_count = 0usize;

    for (idx, entry) in app.worktrees.iter().enumerate() {
        let tokens_per_second = app
            .agent_sessions
            .get(entry.path.as_str())
            .map(|session| {
                agent_session_context_tokens_per_second(session, now)
                    .saturating_add(agent_session_output_tokens_per_second(session, now))
            })
            .unwrap_or(0);

        if tokens_per_second == 0 {
            idle_count = idle_count.saturating_add(1);
            continue;
        }

        let label = if entry.is_current {
            "current".to_string()
        } else {
            entry.branch.clone()
        };
        active_rows.push(WorktreeTokenLeaderboardRow {
            label,
            tokens_per_second,
            is_selected: idx == app.selected_worktree,
            is_idle_cluster: false,
        });
    }

    active_rows.sort_by(|left, right| {
        right
            .tokens_per_second
            .cmp(&left.tokens_per_second)
            .then_with(|| left.label.cmp(&right.label))
    });

    let mut rows = active_rows;
    if idle_count > 0 {
        rows.push(WorktreeTokenLeaderboardRow {
            label: format!("idle x{}", idle_count),
            tokens_per_second: 0,
            is_selected: false,
            is_idle_cluster: true,
        });
    }

    if rows.is_empty() {
        return;
    }

    let max_rows = canvas_area.height.saturating_sub(3) as usize;
    if max_rows == 0 {
        return;
    }

    if rows.len() > max_rows {
        rows.truncate(max_rows);
    }

    let panel_width = canvas_area.width.min(44).max(34);
    let panel_height = rows.len().saturating_add(2) as u16;
    let area = Rect::new(
        canvas_area.right().saturating_sub(panel_width),
        canvas_area.y,
        panel_width,
        panel_height,
    );

    let block = Block::default()
        .title(" tok/s leaderboard ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .style(Style::default().bg(Color::Black));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width < 23 || inner.height == 0 {
        return;
    }

    let max_tps = rows
        .iter()
        .map(|row| row.tokens_per_second)
        .max()
        .unwrap_or(0)
        .max(1);
    let available = inner.width as usize;
    let label_width = available.saturating_sub(23).clamp(6, 16);
    let bar_width = available
        .saturating_sub(label_width)
        .saturating_sub(19)
        .max(4);

    let mut lines: Vec<Line<'_>> = Vec::with_capacity(rows.len());
    for (rank, row) in rows.into_iter().enumerate() {
        let ratio = if row.is_idle_cluster {
            0.0
        } else {
            row.tokens_per_second as f64 / max_tps as f64
        };
        let bar = unicode_bar(ratio, bar_width, !row.is_idle_cluster);

        let name = truncate_text(row.label.as_str(), label_width);
        let marker = if row.is_idle_cluster {
            "⋯"
        } else if rank == 0 {
            "◈"
        } else if row.is_selected {
            "◎"
        } else {
            "•"
        };
        let row_style = if row.is_idle_cluster {
            Style::default().fg(Color::DarkGray)
        } else if row.is_selected {
            Style::default()
                .fg(Color::LightCyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        let bar_style = if row.is_idle_cluster {
            Style::default().fg(Color::DarkGray)
        } else if row.tokens_per_second == max_tps {
            Style::default().fg(Color::LightCyan)
        } else {
            Style::default().fg(Color::Green)
        };

        lines.push(Line::from(vec![
            Span::styled(marker, row_style),
            Span::raw(" "),
            Span::styled(format!("{name:label_width$}"), row_style),
            Span::raw(" "),
            Span::styled(
                format!("{:>6}/s", format_compact_metric(row.tokens_per_second)),
                row_style,
            ),
            Span::raw(" "),
            Span::styled(
                if row.is_idle_cluster {
                    "   -  ".to_string()
                } else {
                    format!("{:>5.2}x", ratio.max(0.01))
                },
                if row.is_idle_cluster {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default().fg(Color::Gray)
                },
            ),
            Span::raw(" "),
            Span::styled(bar, bar_style),
        ]));
    }

    frame.render_widget(Paragraph::new(lines), inner);
}

fn unicode_bar(ratio: f64, width: usize, keep_min_fill: bool) -> String {
    if width == 0 {
        return String::new();
    }

    let filled_units = (ratio.clamp(0.0, 1.0) * (width as f64 * 8.0)).round() as usize;
    let min_units = if keep_min_fill { 1 } else { 0 };
    let total_units = filled_units.max(min_units).min(width * 8);

    let full_cells = total_units / 8;
    let partial = total_units % 8;
    let mut out = String::with_capacity(width * 3);
    out.push_str(&"█".repeat(full_cells.min(width)));
    if full_cells < width && partial > 0 {
        let glyph = match partial {
            1 => '▏',
            2 => '▎',
            3 => '▍',
            4 => '▌',
            5 => '▋',
            6 => '▊',
            _ => '▉',
        };
        out.push(glyph);
    }

    let used_cells = out.chars().count().min(width);
    out.push_str(&"░".repeat(width.saturating_sub(used_cells)));
    out
}

fn pulse_rect(area: Rect, center: (u16, u16)) -> Rect {
    let width = 5u16.min(area.width.max(1));
    let height = 3u16.min(area.height.max(1));
    let half_w = width / 2;
    let half_h = height / 2;

    let min_x = area.x;
    let min_y = area.y;
    let max_x = area.right().saturating_sub(width);
    let max_y = area.bottom().saturating_sub(height);

    let x = center.0.saturating_sub(half_w).clamp(min_x, max_x);
    let y = center.1.saturating_sub(half_h).clamp(min_y, max_y);
    Rect::new(x, y, width, height)
}

#[derive(Clone, Copy)]
struct GraphCell {
    links: u8,
    style: Style,
    priority: u8,
}

impl GraphCell {
    fn new() -> Self {
        Self {
            links: 0,
            style: Style::default().fg(Color::DarkGray),
            priority: 0,
        }
    }
}

const DIR_UP: u8 = 1;
const DIR_RIGHT: u8 = 2;
const DIR_DOWN: u8 = 4;
const DIR_LEFT: u8 = 8;

fn draw_unicode_worktree_graph(
    buf: &mut Buffer,
    area: Rect,
    parents: &[Option<usize>],
    node_points: &[(u16, u16)],
    root_point: Option<(u16, u16)>,
    app: &App,
    selected_idx: usize,
    collapsed_root_idx: Option<usize>,
) {
    if area.width < 3 || area.height < 3 {
        return;
    }

    let width = area.width as usize;
    let height = area.height as usize;
    let mut cells = vec![GraphCell::new(); width * height];
    let root_branch = current_session_branch(app);

    for (idx, parent) in parents.iter().enumerate() {
        if Some(idx) == collapsed_root_idx {
            continue;
        }
        let Some(&(to_x, to_y)) = node_points.get(idx) else {
            continue;
        };
        let from = parent
            .and_then(|p| node_points.get(p).copied())
            .or(root_point)
            .unwrap_or((to_x, to_y));
        let is_head_to_main_edge = parent.is_none()
            && app
                .worktrees
                .get(idx)
                .map(|entry| !entry.detached && entry.branch == root_branch)
                .unwrap_or(false);
        let edge_from = if is_head_to_main_edge {
            shorten_edge_from_start(from, (to_x, to_y), 0.5)
        } else {
            from
        };

        let is_selected_edge =
            idx == selected_idx || parent.map(|p| p == selected_idx).unwrap_or(false);
        let mut edge_color = graph_palette_color(idx);
        if is_selected_edge {
            edge_color = Color::White;
        }
        let edge_style = Style::default().fg(edge_color);
        let priority = if is_selected_edge { 3 } else { 1 };
        draw_smooth_branch_path(
            &mut cells,
            area,
            edge_from,
            (to_x, to_y),
            edge_style,
            priority,
        );
    }

    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            let cell = cells[idx];
            if cell.links == 0 {
                continue;
            }
            let ch = graph_link_glyph(cell.links);
            paint_graph_char(buf, area, x as u16, y as u16, ch, cell.style);
        }
    }

    if let Some((rx, ry)) = root_point {
        paint_graph_char(
            buf,
            area,
            rx.saturating_sub(area.x),
            ry.saturating_sub(area.y),
            'O',
            Style::default().fg(Color::LightBlue),
        );
    }

    for (idx, entry) in app.worktrees.iter().enumerate() {
        if Some(idx) == collapsed_root_idx {
            continue;
        }
        let Some(&(sx, sy)) = node_points.get(idx) else {
            continue;
        };
        let local_x = sx.saturating_sub(area.x);
        let local_y = sy.saturating_sub(area.y);
        let selected = idx == selected_idx;
        let node_color = if selected {
            Color::LightCyan
        } else if entry.dirty {
            Color::Red
        } else if entry.behind_parent {
            Color::Yellow
        } else if entry.is_current {
            Color::LightBlue
        } else {
            graph_palette_color(idx)
        };
        let glyph = if selected {
            '@'
        } else if entry.is_current {
            'O'
        } else {
            'o'
        };

        paint_graph_char(
            buf,
            area,
            local_x,
            local_y,
            glyph,
            Style::default().fg(node_color),
        );
    }
}

fn shorten_edge_from_start(from: (u16, u16), to: (u16, u16), length_scale: f32) -> (u16, u16) {
    let clamped = length_scale.clamp(0.0, 1.0);
    let move_toward_end = 1.0 - clamped;
    let nx = from.0 as f32 + (to.0 as f32 - from.0 as f32) * move_toward_end;
    let ny = from.1 as f32 + (to.1 as f32 - from.1 as f32) * move_toward_end;
    (nx.round() as u16, ny.round() as u16)
}

fn draw_smooth_branch_path(
    cells: &mut [GraphCell],
    area: Rect,
    from: (u16, u16),
    to: (u16, u16),
    style: Style,
    priority: u8,
) {
    let fx = from.0.saturating_sub(area.x);
    let fy = from.1.saturating_sub(area.y);
    let tx = to.0.saturating_sub(area.x);
    let ty = to.1.saturating_sub(area.y);

    let width = area.width.saturating_sub(1);
    let height = area.height.saturating_sub(1);
    let fx = fx.min(width);
    let fy = fy.min(height);
    let tx = tx.min(width);
    let ty = ty.min(height);

    let bend_y = if ty > fy {
        ty.saturating_sub(1).max(fy)
    } else {
        ty.saturating_add(1).min(fy)
    };

    draw_axis_path(
        cells,
        area.width,
        area.height,
        (fx, fy),
        (fx, bend_y),
        style,
        priority,
    );
    draw_axis_path(
        cells,
        area.width,
        area.height,
        (fx, bend_y),
        (tx, bend_y),
        style,
        priority,
    );
    draw_axis_path(
        cells,
        area.width,
        area.height,
        (tx, bend_y),
        (tx, ty),
        style,
        priority,
    );
}

fn draw_axis_path(
    cells: &mut [GraphCell],
    width: u16,
    height: u16,
    start: (u16, u16),
    end: (u16, u16),
    style: Style,
    priority: u8,
) {
    let mut x = start.0 as i32;
    let mut y = start.1 as i32;
    let end_x = end.0 as i32;
    let end_y = end.1 as i32;

    if x == end_x {
        let step = if end_y >= y { 1 } else { -1 };
        while y != end_y {
            let next_y = y + step;
            link_graph_cells(cells, width, height, (x, y), (x, next_y), style, priority);
            y = next_y;
        }
        return;
    }

    if y == end_y {
        let step = if end_x >= x { 1 } else { -1 };
        while x != end_x {
            let next_x = x + step;
            link_graph_cells(cells, width, height, (x, y), (next_x, y), style, priority);
            x = next_x;
        }
    }
}

fn link_graph_cells(
    cells: &mut [GraphCell],
    width: u16,
    height: u16,
    from: (i32, i32),
    to: (i32, i32),
    style: Style,
    priority: u8,
) {
    let inside =
        |x: i32, y: i32| -> bool { x >= 0 && y >= 0 && x < width as i32 && y < height as i32 };
    if !inside(from.0, from.1) || !inside(to.0, to.1) {
        return;
    }

    let dx = to.0 - from.0;
    let dy = to.1 - from.1;
    let (from_dir, to_dir) = match (dx, dy) {
        (1, 0) => (DIR_RIGHT, DIR_LEFT),
        (-1, 0) => (DIR_LEFT, DIR_RIGHT),
        (0, 1) => (DIR_DOWN, DIR_UP),
        (0, -1) => (DIR_UP, DIR_DOWN),
        _ => return,
    };

    let from_idx = from.1 as usize * width as usize + from.0 as usize;
    let to_idx = to.1 as usize * width as usize + to.0 as usize;
    apply_graph_link(&mut cells[from_idx], from_dir, style, priority);
    apply_graph_link(&mut cells[to_idx], to_dir, style, priority);
}

fn apply_graph_link(cell: &mut GraphCell, direction: u8, style: Style, priority: u8) {
    cell.links |= direction;
    if priority >= cell.priority {
        cell.priority = priority;
        cell.style = style;
    }
}

fn graph_link_glyph(mask: u8) -> char {
    const UD: u8 = DIR_UP | DIR_DOWN;
    const LR: u8 = DIR_LEFT | DIR_RIGHT;
    const DR: u8 = DIR_DOWN | DIR_RIGHT;
    const DL: u8 = DIR_DOWN | DIR_LEFT;
    const UL: u8 = DIR_UP | DIR_LEFT;
    const UR: u8 = DIR_UP | DIR_RIGHT;
    const UDR: u8 = DIR_UP | DIR_DOWN | DIR_RIGHT;
    const UDL: u8 = DIR_UP | DIR_DOWN | DIR_LEFT;
    const LRD: u8 = DIR_LEFT | DIR_RIGHT | DIR_DOWN;
    const LRU: u8 = DIR_LEFT | DIR_RIGHT | DIR_UP;
    const ALL: u8 = DIR_UP | DIR_RIGHT | DIR_DOWN | DIR_LEFT;
    match mask {
        UD => '│',
        LR => '─',
        DR => '┌',
        DL => '┐',
        UL => '┘',
        UR => '└',
        UDR => '├',
        UDL => '┤',
        LRD => '┬',
        LRU => '┴',
        ALL => '┼',
        DIR_UP => '│',
        DIR_DOWN => '│',
        DIR_LEFT => '─',
        DIR_RIGHT => '─',
        _ => ' ',
    }
}

fn paint_graph_char(buf: &mut Buffer, area: Rect, x: u16, y: u16, ch: char, style: Style) {
    if x >= area.width || y >= area.height {
        return;
    }
    let sx = area.x + x;
    let sy = area.y + y;
    if let Some(cell) = buf.cell_mut((sx, sy)) {
        let mut encoded = [0; 4];
        cell.set_symbol(ch.encode_utf8(&mut encoded));
        cell.set_style(style);
    }
}

fn graph_palette_color(idx: usize) -> Color {
    const GRAPH_PALETTE: [Color; 12] = [
        Color::LightGreen,
        Color::Green,
        Color::Cyan,
        Color::LightCyan,
        Color::LightBlue,
        Color::Blue,
        Color::Magenta,
        Color::LightMagenta,
        Color::LightRed,
        Color::Red,
        Color::Yellow,
        Color::LightYellow,
    ];
    GRAPH_PALETTE[idx % GRAPH_PALETTE.len()]
}

fn draw_worktree_canvas_background(buf: &mut Buffer, area: Rect, app: &mut App) {
    if area.width < 3 || area.height < 3 {
        return;
    }

    let now_millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis() as u64)
        .unwrap_or(0);
    let time_seconds = now_millis as f64 / 1000.0;
    let pan_x = (app.worktree_canvas_pan_x * 200.0).round() as i64;
    let pan_y = (app.worktree_canvas_pan_y * 140.0).round() as i64;

    match app.worktree_canvas_bg_mode {
        CanvasBackgroundMode::GlitterStars => {
            draw_canvas_bg_glitter_stars(buf, area, pan_x, pan_y, time_seconds)
        }
        CanvasBackgroundMode::Crosshatch => {
            draw_canvas_bg_crosshatch(buf, area, pan_x, pan_y, time_seconds)
        }
        CanvasBackgroundMode::Rainfall => {
            draw_canvas_bg_rainfall(buf, area, pan_x, pan_y, time_seconds)
        }
    }

    let elapsed = app.canvas_bg_last_tick.elapsed();
    app.canvas_bg_last_tick = Instant::now();
    app.canvas_bg_effects
        .process_effects(elapsed.into(), buf, area);
}

fn draw_canvas_bg_glitter_stars(
    buf: &mut Buffer,
    area: Rect,
    pan_x: i64,
    pan_y: i64,
    time_seconds: f64,
) {
    let tau = std::f64::consts::TAU;

    for y in 0..area.height {
        for x in 0..area.width {
            let wx = x as i64 + pan_x;
            let wy = y as i64 + pan_y;
            let seed = star_seed(wx, wy);

            let dust_roll = seed & 0xFF;
            if dust_roll < 16 {
                paint_graph_char(
                    buf,
                    area,
                    x,
                    y,
                    '·',
                    Style::default().fg(Color::Rgb(29, 33, 51)),
                );
            }

            let phase_seed = star_seed(wx ^ 0x4F2A, wy ^ 0x9813);
            let phase = star_seed_unit(phase_seed, 16) * tau;
            let drift_scale_x = 0.05 + star_seed_unit(phase_seed, 40) * 0.07;
            let drift_x = ((time_seconds * drift_scale_x + phase).sin() * 2.2
                + (time_seconds * (drift_scale_x * 0.53) + phase * 0.31).cos() * 1.4)
                .round() as i64;
            let fall_rate = 0.35 + star_seed_unit(phase_seed, 22) * 0.75;
            let fall_offset = (time_seconds * fall_rate).floor() as i64;
            let drift_y = ((time_seconds * (0.02 + star_seed_unit(phase_seed, 54) * 0.03)
                + phase * 0.77)
                .sin()
                * 1.2)
                .round() as i64;

            let glitter_seed = star_seed(wx + drift_x, wy - fall_offset + drift_y);
            if glitter_seed % 1000 >= 8 {
                continue;
            }

            let twinkle_speed = 0.0007 + star_seed_unit(glitter_seed, 20) * 0.0014;
            let twinkle_phase = star_seed_unit(glitter_seed, 36) * tau;
            let twinkle = ((time_seconds * twinkle_speed + twinkle_phase).sin() + 1.0) * 0.5;
            if twinkle < 0.08 {
                continue;
            }

            let morph_period = 7.0 + star_seed_unit(glitter_seed, 12) * 12.0;
            let morph_phase = star_seed_unit(glitter_seed, 48) * morph_period;
            let morph_step = ((time_seconds + morph_phase) / morph_period).floor() as i64;
            let morph_seed = star_seed(wx ^ (morph_step << 1), wy ^ (morph_step * 29));
            let sparkle_kind = ((morph_seed >> 8) % 6) as u8;
            let (glyph, base_color) = if sparkle_kind <= 1 {
                ('✦', Color::Rgb(234, 242, 255))
            } else if sparkle_kind <= 3 {
                ('✧', Color::Rgb(198, 217, 248))
            } else {
                ('•', Color::Rgb(166, 189, 235))
            };

            let brightness = twinkle.powf(1.35) as f32;
            let color = color_mix(Color::Rgb(24, 29, 45), base_color, brightness);

            paint_graph_char(buf, area, x, y, glyph, Style::default().fg(color));
        }
    }
}

fn draw_canvas_bg_crosshatch(
    buf: &mut Buffer,
    area: Rect,
    pan_x: i64,
    pan_y: i64,
    time_seconds: f64,
) {
    for y in 0..area.height {
        for x in 0..area.width {
            let wx = x as i64 + pan_x;
            let wy = y as i64 + pan_y;
            let seed = star_seed(wx, wy);
            let sum = (wx + wy).rem_euclid(18);
            let diff = (wx - wy).rem_euclid(18);

            if sum == 0 || diff == 0 {
                let shimmer =
                    ((time_seconds * 0.65 + star_seed_unit(seed, 16) * 6.0).sin() + 1.0) * 0.5;
                let base = if sum == 0 {
                    Color::Rgb(35, 46, 64)
                } else {
                    Color::Rgb(30, 40, 58)
                };
                let color = color_mix(base, Color::Rgb(95, 127, 172), (shimmer as f32) * 0.25);
                let glyph = if sum == 0 { '╱' } else { '╲' };
                paint_graph_char(buf, area, x, y, glyph, Style::default().fg(color));
                continue;
            }

            if (seed & 0x3FF) < 40 {
                paint_graph_char(
                    buf,
                    area,
                    x,
                    y,
                    '·',
                    Style::default().fg(Color::Rgb(31, 36, 53)),
                );
            }

            if sum == 9 && diff == 9 {
                let twinkle =
                    ((time_seconds * 0.9 + star_seed_unit(seed, 38) * 7.0).sin() + 1.0) * 0.5;
                let glyph = if twinkle > 0.82 { '✦' } else { '•' };
                let color = color_mix(
                    Color::Rgb(86, 113, 154),
                    Color::Rgb(244, 248, 255),
                    twinkle as f32,
                );
                paint_graph_char(buf, area, x, y, glyph, Style::default().fg(color));
            }
        }
    }
}

fn draw_canvas_bg_rainfall(
    buf: &mut Buffer,
    area: Rect,
    pan_x: i64,
    pan_y: i64,
    time_seconds: f64,
) {
    for y in 0..area.height {
        for x in 0..area.width {
            let seed = star_seed(x as i64 + pan_x, y as i64 + pan_y);
            if (seed & 0x3FF) < 14 {
                paint_graph_char(
                    buf,
                    area,
                    x,
                    y,
                    '·',
                    Style::default().fg(Color::Rgb(24, 31, 47)),
                );
            }
        }
    }

    for x in 0..area.width {
        let col_seed = star_seed(x as i64 + pan_x * 3, 97);
        if col_seed % 5 != 0 {
            continue;
        }

        let speed = 0.9 + star_seed_unit(col_seed, 14) * 2.0;
        let phase = star_seed_unit(col_seed, 22) * 120.0;
        let period = 22.0 + star_seed_unit(col_seed, 30) * 30.0;
        let trail = 2.0 + star_seed_unit(col_seed, 38) * 6.0;

        for y in 0..area.height {
            let world_y = y as i64 + pan_y;
            let travel = world_y as f64 + time_seconds * speed * 12.0 + phase;
            let pos = travel.rem_euclid(period);
            if pos > trail {
                continue;
            }

            let intensity = 1.0 - (pos / trail).clamp(0.0, 1.0);
            let glyph = if pos < 0.8 {
                '•'
            } else if intensity > 0.62 {
                '│'
            } else {
                '·'
            };
            let color = color_mix(
                Color::Rgb(48, 69, 102),
                Color::Rgb(182, 216, 255),
                intensity as f32,
            );
            paint_graph_char(buf, area, x, y, glyph, Style::default().fg(color));
        }
    }
}

fn build_canvas_bg_effect() -> tachyonfx::Effect {
    fx::repeating(fx::ping_pong(fx::hsl_shift_fg(
        [12.0, 10.0, 8.0],
        (1800, Interpolation::SineInOut),
    )))
}

fn build_selected_node_border_effect() -> tachyonfx::Effect {
    fx::repeating(fx::ping_pong(fx::hsl_shift_fg(
        [5.0, 12.0, 6.0],
        (2200, Interpolation::SineInOut),
    )))
}

fn draw_spinning_border_shine(buf: &mut Buffer, area: Rect) {
    if area.width < 2 || area.height < 2 {
        return;
    }

    let now_seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis() as f64 / 1000.0)
        .unwrap_or(0.0);

    let border_cells = border_perimeter_cells(area);
    if border_cells.is_empty() {
        return;
    }

    let perimeter = border_cells.len();
    let head = ((now_seconds * 9.0) as usize) % perimeter;
    let tail = (perimeter / 5).max(5);
    let pulse = ((now_seconds * 2.2).sin() * 0.5 + 0.5) as f32;
    let base_color = color_mix(
        Color::Rgb(84, 144, 204),
        Color::Rgb(116, 182, 232),
        pulse * 0.55,
    );

    for (idx, (x, y)) in border_cells.iter().enumerate() {
        let distance = (head + perimeter - idx) % perimeter;
        let shine = if distance < tail {
            1.0 - distance as f32 / tail as f32
        } else {
            0.0
        };

        let color = if shine > 0.0 {
            color_mix(base_color, Color::Rgb(240, 248, 255), 0.25 + shine * 0.75)
        } else {
            base_color
        };

        if let Some(cell) = buf.cell_mut((*x, *y)) {
            cell.set_style(Style::default().fg(color));
        }
    }
}

fn border_perimeter_cells(area: Rect) -> Vec<(u16, u16)> {
    if area.width < 2 || area.height < 2 {
        return Vec::new();
    }

    let mut cells = Vec::with_capacity((area.width as usize + area.height as usize) * 2);
    let left = area.x;
    let right = area.x + area.width - 1;
    let top = area.y;
    let bottom = area.y + area.height - 1;

    for x in left..=right {
        cells.push((x, top));
    }
    for y in (top + 1)..=bottom {
        cells.push((right, y));
    }
    if bottom > top {
        for x in (left..right).rev() {
            cells.push((x, bottom));
        }
    }
    if right > left {
        for y in ((top + 1)..bottom).rev() {
            cells.push((left, y));
        }
    }

    cells
}

fn color_mix(from: Color, to: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    let (fr, fg, fb) = color_rgb(from);
    let (tr, tg, tb) = color_rgb(to);
    Color::Rgb(
        (fr as f32 + (tr as f32 - fr as f32) * t).round() as u8,
        (fg as f32 + (tg as f32 - fg as f32) * t).round() as u8,
        (fb as f32 + (tb as f32 - fb as f32) * t).round() as u8,
    )
}

fn color_rgb(color: Color) -> (u8, u8, u8) {
    match color {
        Color::Rgb(r, g, b) => (r, g, b),
        Color::White => (255, 255, 255),
        Color::Black => (0, 0, 0),
        Color::LightCyan => (224, 255, 255),
        Color::Cyan => (0, 255, 255),
        Color::Gray => (128, 128, 128),
        Color::DarkGray => (64, 64, 64),
        _ => (180, 200, 230),
    }
}

fn build_node_create_effect() -> tachyonfx::Effect {
    fx::hsl_shift_fg([18.0, 34.0, 14.0], (600, Interpolation::SineInOut))
}

fn build_node_delete_effect() -> tachyonfx::Effect {
    fx::fade_to_fg(Color::Rgb(36, 42, 64), (520, Interpolation::SineInOut))
}

fn node_merge_badge(entry: &WorktreeEntry) -> (&'static str, Style) {
    let warm = Style::default().fg(Color::Rgb(245, 163, 72));
    if entry.detached {
        ("detached", Style::default().fg(Color::Red))
    } else if entry.dirty {
        ("dirty", Style::default().fg(Color::Red))
    } else if entry.behind_parent {
        ("behind parent", Style::default().fg(Color::Yellow))
    } else if entry.ahead > 0 {
        ("committed", warm)
    } else if entry.merged_with_parent {
        ("merged with parent", Style::default().fg(Color::Green))
    } else if !entry.has_upstream {
        ("local only", warm)
    } else {
        ("pushed", warm)
    }
}

fn star_seed(x: i64, y: i64) -> u64 {
    let mut h = (x as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    h ^= (y as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    h ^= h >> 33;
    h = h.wrapping_mul(0xFF51_AFD7_ED55_8CCD);
    h ^= h >> 33;
    h = h.wrapping_mul(0xC4CE_B9FE_1A85_EC53);
    h ^ (h >> 33)
}

fn star_seed_unit(seed: u64, shift: u32) -> f64 {
    ((seed >> shift) & 0xFF) as f64 / 255.0
}

fn canvas_node_label(app: &App, entry: &WorktreeEntry, selected: bool) -> String {
    let mut name = if entry.detached {
        "detached".to_string()
    } else {
        entry.branch.clone()
    };
    if name.len() > 16 {
        name = truncate_text(name.as_str(), 16);
    }

    let agent = agent_badge_for_node(app, entry.path.as_str());

    if selected {
        let state = if entry.dirty { "*" } else { "" };
        format!("{}{}{}", name, state, agent)
    } else if entry.dirty {
        format!("{}*{}", name, agent)
    } else if !agent.is_empty() {
        format!("{}{}", name, agent)
    } else {
        name
    }
}

#[derive(Clone, Copy)]
struct CanvasBounds {
    min_x: f64,
    max_x: f64,
    min_y: f64,
    max_y: f64,
}

fn worktree_canvas_bounds(app: &App) -> CanvasBounds {
    let span_x = 1.48 / app.worktree_canvas_zoom.max(0.65);
    let span_y = 1.48 / app.worktree_canvas_zoom.max(0.65);
    CanvasBounds {
        min_x: -span_x + app.worktree_canvas_pan_x,
        max_x: span_x + app.worktree_canvas_pan_x,
        min_y: -span_y + app.worktree_canvas_pan_y,
        max_y: span_y + app.worktree_canvas_pan_y,
    }
}

fn logical_to_canvas_point(point: (f32, f32)) -> (f64, f64) {
    let x = (point.0 as f64 - 0.5) * 2.6;
    let y = 0.9 - point.1 as f64 * 1.8;
    (x, y)
}

fn draw_canvas_label(
    frame: &mut ratatui::Frame<'_>,
    area: Rect,
    bounds: CanvasBounds,
    point: (f64, f64),
    label: &str,
    label_style: Style,
    badge: &str,
    badge_style: Style,
) -> Option<Rect> {
    let Some((sx, sy)) = canvas_point_to_screen(area, bounds, point) else {
        return None;
    };
    let label_width = label.chars().count() as u16;
    let badge_width = badge.chars().count() as u16;
    let content_width = label_width.max(badge_width);
    let horizontal_padding = 1u16;
    let box_width = content_width
        .saturating_add(horizontal_padding.saturating_mul(2))
        .saturating_add(2);
    let box_height = 4u16;
    if content_width == 0 || box_width >= area.width || box_height > area.height {
        return None;
    }

    let mut x = sx.saturating_sub(box_width / 2);
    let min_x = area.x;
    let max_x = area.right().saturating_sub(box_width);
    if x < min_x {
        x = min_x;
    }
    if x > max_x {
        x = max_x;
    }
    let y = sy
        .saturating_add(1)
        .clamp(area.y, area.bottom().saturating_sub(box_height));
    let rect = Rect::new(x, y, box_width, box_height);
    frame.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(label_style)
            .style(Style::default().bg(Color::Black)),
        rect,
    );
    frame.render_widget(
        Paragraph::new(label).style(label_style),
        Rect::new(
            x.saturating_add(1 + horizontal_padding),
            y.saturating_add(1),
            content_width,
            1,
        ),
    );
    frame.render_widget(
        Paragraph::new(badge).style(badge_style.add_modifier(Modifier::DIM)),
        Rect::new(
            x.saturating_add(1 + horizontal_padding),
            y.saturating_add(2),
            content_width,
            1,
        ),
    );
    Some(rect)
}

fn canvas_point_to_screen(
    area: Rect,
    bounds: CanvasBounds,
    point: (f64, f64),
) -> Option<(u16, u16)> {
    if area.width == 0 || area.height == 0 {
        return None;
    }
    let width = (bounds.max_x - bounds.min_x).max(0.001);
    let height = (bounds.max_y - bounds.min_y).max(0.001);
    let nx = (point.0 - bounds.min_x) / width;
    let ny = (bounds.max_y - point.1) / height;
    if !(0.0..=1.0).contains(&nx) || !(0.0..=1.0).contains(&ny) {
        return None;
    }

    let sx = area.x + (nx * area.width.saturating_sub(1) as f64).round() as u16;
    let sy = area.y + (ny * area.height.saturating_sub(1) as f64).round() as u16;
    Some((sx, sy))
}

fn agent_badge_for_node(app: &App, path: &str) -> String {
    let Some(session) = app.agent_sessions.get(path) else {
        return String::new();
    };

    let now = Instant::now();

    let in_foreground = matches!(app.mode, Mode::AgentPopup)
        && app
            .agent_popup_path
            .as_deref()
            .map(|p| p == path)
            .unwrap_or(false);

    if agent_session_is_live(session) {
        if agent_session_is_active(session, now) {
            let spinner = animated_agent_spinner(session, now);
            if in_foreground {
                return format!(" {}*", spinner);
            }
            return format!(" {}", spinner);
        }

        let idle = agent_session_idle_seconds(session, now);
        if in_foreground {
            return format!(" idle({}s)*", idle);
        }
        return format!(" idle({}s)", idle);
    }

    match session.state {
        AgentState::Done => " done".to_string(),
        AgentState::Failed => " fail".to_string(),
        AgentState::Launching | AgentState::Running => {
            let spinner = animated_agent_spinner(session, now);
            if in_foreground {
                format!(" {}*", spinner)
            } else {
                format!(" {}", spinner)
            }
        }
    }
}

fn estimate_tokens(bytes: u64) -> u64 {
    if bytes == 0 {
        return 0;
    }
    bytes.saturating_add(3) / 4
}

fn agent_session_output_text_bytes(session: &AgentSession) -> u64 {
    let screen = session.parser.screen();
    let (rows, cols) = screen.size();
    let mut bytes = 0u64;

    for row in 0..rows {
        let mut line = String::new();
        for col in 0..cols {
            let Some(cell) = screen.cell(row, col) else {
                continue;
            };
            if cell.is_wide_continuation() {
                continue;
            }
            let mut text = cell.contents();
            if text.is_empty() {
                text.push(' ');
            }
            line.push_str(text.as_str());
        }

        let trimmed = line.trim_end_matches(' ');
        if trimmed.is_empty() {
            continue;
        }

        bytes = bytes.saturating_add(trimmed.len() as u64);
        bytes = bytes.saturating_add(1);
    }

    bytes
}

fn agent_session_context_tokens(session: &AgentSession) -> u64 {
    if let Some(usage) = session.opencode_usage {
        return usage.input_tokens;
    }
    estimate_tokens(session.bytes_to_agent)
}

fn agent_session_output_tokens(session: &AgentSession) -> u64 {
    if let Some(usage) = session.opencode_usage {
        return usage.output_tokens;
    }
    estimate_tokens(agent_session_output_text_bytes(session))
}

fn agent_session_total_tokens(session: &AgentSession) -> u64 {
    agent_session_context_tokens(session).saturating_add(agent_session_output_tokens(session))
}

fn agent_session_context_tokens_per_second(session: &AgentSession, now: Instant) -> u64 {
    if let Some(usage) = session.opencode_usage {
        return usage.input_tokens_per_second;
    }
    agent_session_direction_tokens_per_second(session, now, |sample| sample.bytes_to_agent)
}

fn agent_session_output_tokens_per_second(session: &AgentSession, now: Instant) -> u64 {
    if let Some(usage) = session.opencode_usage {
        return usage.output_tokens_per_second;
    }
    agent_session_direction_tokens_per_second(session, now, |sample| sample.bytes_from_agent)
}

fn agent_session_direction_tokens_per_second(
    session: &AgentSession,
    now: Instant,
    byte_selector: fn(&IoSample) -> u64,
) -> u64 {
    if !agent_session_is_live(session) || !agent_session_is_active(session, now) {
        return 0;
    }

    let mut bytes = 0u64;
    let mut has_recent = false;
    let mut oldest_recent = now;

    for sample in session.io_samples.iter().rev() {
        if now.saturating_duration_since(sample.at) > TOKEN_RATE_WINDOW {
            break;
        }
        has_recent = true;
        oldest_recent = sample.at;
        bytes = bytes.saturating_add(byte_selector(sample));
    }

    if !has_recent || bytes == 0 {
        return 0;
    }

    let seconds = now
        .saturating_duration_since(oldest_recent)
        .as_secs_f64()
        .max(1.0);
    (estimate_tokens(bytes) as f64 / seconds).round() as u64
}

fn format_compact_metric(value: u64) -> String {
    const UNITS: [&str; 4] = ["", "k", "m", "b"];
    let mut scaled = value as f64;
    let mut unit_idx = 0usize;
    while scaled >= 1000.0 && unit_idx + 1 < UNITS.len() {
        scaled /= 1000.0;
        unit_idx += 1;
    }

    if unit_idx == 0 {
        value.to_string()
    } else if scaled >= 100.0 {
        format!("{:.0}{}", scaled, UNITS[unit_idx])
    } else if scaled >= 10.0 {
        format!("{:.1}{}", scaled, UNITS[unit_idx])
    } else {
        format!("{:.2}{}", scaled, UNITS[unit_idx])
    }
}

fn animated_agent_spinner(session: &AgentSession, now: Instant) -> char {
    const FRAMES: [char; 10] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
    let tick = now
        .saturating_duration_since(session.launched_at)
        .as_millis()
        / 140;
    FRAMES[(tick % FRAMES.len() as u128) as usize]
}

fn graph_layout(
    parents: &[Option<usize>],
    builder: WorktreeGraphBuilder,
    worktrees: &[WorktreeEntry],
) -> Vec<(f32, f32)> {
    let base = match builder {
        WorktreeGraphBuilder::TopDownBalanced => graph_layout_top_down_balanced(parents),
        WorktreeGraphBuilder::Layered => graph_layout_layered(parents),
        WorktreeGraphBuilder::LeftToRight => graph_layout_left_to_right(parents),
        WorktreeGraphBuilder::Trunk => graph_layout_trunk(parents),
        WorktreeGraphBuilder::Swimlanes => graph_layout_swimlanes(parents, worktrees),
        WorktreeGraphBuilder::Indented => graph_layout_indented(parents),
    };

    spread_graph_layout(base, parents, builder)
}

fn spread_graph_layout(
    base: Vec<(f32, f32)>,
    parents: &[Option<usize>],
    builder: WorktreeGraphBuilder,
) -> Vec<(f32, f32)> {
    let count = base.len();
    if count < 2 {
        return base;
    }

    let (min_dx, min_dy, spring_x, spring_y, edge_pull) = match builder {
        WorktreeGraphBuilder::TopDownBalanced => (0.11, 0.10, 0.028, 0.028, 0.020),
        WorktreeGraphBuilder::Layered => (0.13, 0.12, 0.034, 0.030, 0.022),
        WorktreeGraphBuilder::LeftToRight => (0.12, 0.13, 0.030, 0.034, 0.022),
        WorktreeGraphBuilder::Trunk => (0.13, 0.12, 0.040, 0.030, 0.026),
        WorktreeGraphBuilder::Swimlanes => (0.14, 0.13, 0.028, 0.030, 0.024),
        WorktreeGraphBuilder::Indented => (0.12, 0.13, 0.028, 0.036, 0.020),
    };

    let mut positions = base.clone();
    for step in 0..30 {
        let mut forces = vec![(0.0f32, 0.0f32); count];

        for i in 0..count {
            for j in (i + 1)..count {
                let dx = positions[i].0 - positions[j].0;
                let dy = positions[i].1 - positions[j].1;
                let overlap_x = min_dx - dx.abs();
                let overlap_y = min_dy - dy.abs();
                if overlap_x <= 0.0 || overlap_y <= 0.0 {
                    continue;
                }

                let x_dir = if dx.abs() < 0.001 {
                    if (i + j + step) % 2 == 0 {
                        1.0
                    } else {
                        -1.0
                    }
                } else {
                    dx.signum()
                };
                let y_dir = if dy.abs() < 0.001 {
                    if (i + j + step) % 3 == 0 {
                        1.0
                    } else {
                        -1.0
                    }
                } else {
                    dy.signum()
                };

                let fx = overlap_x * 0.060;
                let fy = overlap_y * 0.060;
                forces[i].0 += x_dir * fx;
                forces[i].1 += y_dir * fy;
                forces[j].0 -= x_dir * fx;
                forces[j].1 -= y_dir * fy;
            }
        }

        for idx in 0..count {
            if let Some(parent_idx) = parents[idx] {
                if parent_idx < count && parent_idx != idx {
                    let pdx = positions[parent_idx].0 - positions[idx].0;
                    let pdy = positions[parent_idx].1 - positions[idx].1;
                    forces[idx].0 += pdx * edge_pull;
                    forces[idx].1 += pdy * (edge_pull * 0.7);
                }
            }

            forces[idx].0 += (base[idx].0 - positions[idx].0) * spring_x;
            forces[idx].1 += (base[idx].1 - positions[idx].1) * spring_y;
        }

        for idx in 0..count {
            positions[idx].0 = (positions[idx].0 + forces[idx].0).clamp(0.05, 0.95);
            positions[idx].1 = (positions[idx].1 + forces[idx].1).clamp(0.08, 0.95);
        }
    }

    positions
}

fn graph_layout_top_down_balanced(parents: &[Option<usize>]) -> Vec<(f32, f32)> {
    let count = parents.len();
    if count == 0 {
        return Vec::new();
    }

    let depths = graph_depths(parents);
    let max_depth = depths.iter().copied().max().unwrap_or(0).max(1);
    let mut by_depth: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for (idx, depth) in depths.iter().enumerate() {
        by_depth.entry(*depth).or_default().push(idx);
    }

    let mut positions = vec![(0.5f32, 0.5f32); count];
    for (depth, nodes) in by_depth {
        let n = nodes.len().max(1);
        for (rank, idx) in nodes.iter().enumerate() {
            let x = (rank as f32 + 1.0) / (n as f32 + 1.0);
            let y = 0.15 + ((depth as f32 + 1.0) / (max_depth as f32 + 1.0)) * 0.78;
            positions[*idx] = (x, y);
        }
    }

    for _ in 0..24 {
        let mut forces = vec![(0.0f32, 0.0f32); count];

        for i in 0..count {
            for j in (i + 1)..count {
                let dx = positions[i].0 - positions[j].0;
                let dy = positions[i].1 - positions[j].1;
                let dist_sq = (dx * dx + dy * dy).max(0.0006);
                let force = 0.0022 / dist_sq;
                let nx = dx / dist_sq.sqrt();
                let ny = dy / dist_sq.sqrt();
                forces[i].0 += nx * force;
                forces[i].1 += ny * force;
                forces[j].0 -= nx * force;
                forces[j].1 -= ny * force;
            }
        }

        for (idx, parent_opt) in parents.iter().enumerate() {
            let (tx, ty) = if let Some(parent_idx) = parent_opt {
                positions[*parent_idx]
            } else {
                (0.5, 0.06)
            };

            let dx = tx - positions[idx].0;
            let dy = ty - positions[idx].1;
            let dist = (dx * dx + dy * dy).sqrt().max(0.001);
            let desired = if parent_opt.is_some() { 0.18 } else { 0.24 };
            let spring = (dist - desired) * 0.024;
            forces[idx].0 += (dx / dist) * spring;
            forces[idx].1 += (dy / dist) * spring;

            let target_y = 0.15 + ((depths[idx] as f32 + 1.0) / (max_depth as f32 + 1.0)) * 0.78;
            forces[idx].1 += (target_y - positions[idx].1) * 0.015;
            forces[idx].0 += (0.5 - positions[idx].0) * 0.002;
        }

        for idx in 0..count {
            positions[idx].0 = (positions[idx].0 + forces[idx].0).clamp(0.06, 0.94);
            positions[idx].1 = (positions[idx].1 + forces[idx].1).clamp(0.12, 0.95);
        }
    }

    positions
}

fn graph_layout_layered(parents: &[Option<usize>]) -> Vec<(f32, f32)> {
    let count = parents.len();
    if count == 0 {
        return Vec::new();
    }

    let depths = graph_depths(parents);
    let (children, roots) = graph_children_and_roots(parents);
    let x_units = assign_layered_x_units(&children, &roots);

    let max_depth = depths.iter().copied().max().unwrap_or(0).max(1);
    let min_x = x_units.iter().copied().reduce(f32::min).unwrap_or(0.0);
    let max_x = x_units.iter().copied().reduce(f32::max).unwrap_or(1.0);
    let x_span = (max_x - min_x).max(1.0);

    let mut positions = vec![(0.5f32, 0.5f32); count];
    for idx in 0..count {
        let x = 0.08 + ((x_units[idx] - min_x) / x_span) * 0.84;
        let y = 0.14 + (depths[idx] as f32 / max_depth as f32) * 0.78;
        positions[idx] = (x.clamp(0.06, 0.94), y.clamp(0.12, 0.95));
    }

    positions
}

fn graph_layout_left_to_right(parents: &[Option<usize>]) -> Vec<(f32, f32)> {
    let count = parents.len();
    if count == 0 {
        return Vec::new();
    }

    let depths = graph_depths(parents);
    let (children, roots) = graph_children_and_roots(parents);
    let x_units = assign_layered_x_units(&children, &roots);
    let min_order = x_units.iter().copied().reduce(f32::min).unwrap_or(0.0);
    let max_order = x_units.iter().copied().reduce(f32::max).unwrap_or(1.0);
    let order_span = (max_order - min_order).max(1.0);
    let max_depth = depths.iter().copied().max().unwrap_or(0).max(1);

    let mut positions = vec![(0.5f32, 0.5f32); count];
    for idx in 0..count {
        let x = 0.10 + (depths[idx] as f32 / max_depth as f32) * 0.82;
        let y = 0.10 + ((x_units[idx] - min_order) / order_span) * 0.82;
        positions[idx] = (x.clamp(0.06, 0.95), y.clamp(0.10, 0.94));
    }

    positions
}

fn graph_layout_trunk(parents: &[Option<usize>]) -> Vec<(f32, f32)> {
    let count = parents.len();
    if count == 0 {
        return Vec::new();
    }

    let depths = graph_depths(parents);
    let max_depth = depths.iter().copied().max().unwrap_or(0).max(1);
    let (children, roots) = graph_children_and_roots(parents);
    let subtree = subtree_sizes(&children);

    let trunk_root = roots
        .iter()
        .copied()
        .max_by_key(|idx| subtree[*idx])
        .unwrap_or(0);

    let mut trunk_nodes: Vec<usize> = Vec::new();
    let mut current = trunk_root;
    loop {
        trunk_nodes.push(current);
        let next = children[current]
            .iter()
            .copied()
            .max_by_key(|idx| subtree[*idx]);
        let Some(next_idx) = next else {
            break;
        };
        current = next_idx;
    }
    let trunk_set: BTreeSet<usize> = trunk_nodes.iter().copied().collect();

    let mut positions = graph_layout_layered(parents);
    for idx in 0..count {
        let y = 0.12 + (depths[idx] as f32 / max_depth as f32) * 0.80;
        positions[idx].1 = y.clamp(0.12, 0.95);
    }
    for idx in &trunk_nodes {
        positions[*idx].0 = 0.5;
    }

    fn place_side_branches(
        node: usize,
        center_x: f32,
        base_side: f32,
        depth: usize,
        children: &[Vec<usize>],
        trunk_set: &BTreeSet<usize>,
        positions: &mut [(f32, f32)],
    ) {
        let mut branch_children: Vec<usize> = children[node]
            .iter()
            .copied()
            .filter(|idx| !trunk_set.contains(idx))
            .collect();
        if branch_children.is_empty() {
            return;
        }
        branch_children.sort_unstable();

        let base_span = (0.14f32 / (depth as f32 + 1.0)).max(0.04);
        for (i, child) in branch_children.iter().enumerate() {
            let side = if i % 2 == 0 { base_side } else { -base_side };
            let lane = (i / 2) as f32 + 1.0;
            let x = (center_x + side * lane * base_span).clamp(0.06, 0.94);
            positions[*child].0 = x;
            place_side_branches(
                *child,
                x,
                side,
                depth.saturating_add(1),
                children,
                trunk_set,
                positions,
            );
        }
    }

    for (depth_idx, node) in trunk_nodes.iter().enumerate() {
        let base_side = if depth_idx % 2 == 0 { -1.0 } else { 1.0 };
        place_side_branches(
            *node,
            0.5,
            base_side,
            depth_idx,
            &children,
            &trunk_set,
            &mut positions,
        );
    }

    positions
}

fn graph_layout_swimlanes(
    parents: &[Option<usize>],
    worktrees: &[WorktreeEntry],
) -> Vec<(f32, f32)> {
    let count = parents.len();
    if count == 0 {
        return Vec::new();
    }

    let depths = graph_depths(parents);
    let max_depth = depths.iter().copied().max().unwrap_or(0).max(1);

    let mut lane_names: Vec<String> = Vec::new();
    let mut lane_by_idx = vec![0usize; count];
    for idx in 0..count {
        let lane = worktrees
            .get(idx)
            .map(|wt| {
                if wt.detached {
                    "detached".to_string()
                } else {
                    wt.branch.split('/').next().unwrap_or("root").to_string()
                }
            })
            .unwrap_or_else(|| "root".to_string());

        let lane_idx = lane_names
            .iter()
            .position(|value| value == &lane)
            .unwrap_or_else(|| {
                lane_names.push(lane);
                lane_names.len() - 1
            });
        lane_by_idx[idx] = lane_idx;
    }

    let lane_count = lane_names.len().max(1);
    let mut grouped: BTreeMap<(usize, usize), Vec<usize>> = BTreeMap::new();
    for idx in 0..count {
        grouped
            .entry((lane_by_idx[idx], depths[idx]))
            .or_default()
            .push(idx);
    }

    let mut positions = vec![(0.5f32, 0.5f32); count];
    for ((lane_idx, depth), nodes) in grouped {
        let lane_center = if lane_count == 1 {
            0.5
        } else {
            0.10 + (lane_idx as f32 / (lane_count - 1) as f32) * 0.80
        };
        let y = 0.12 + (depth as f32 / max_depth as f32) * 0.80;
        let n = nodes.len().max(1);
        for (rank, idx) in nodes.iter().enumerate() {
            let centered = rank as f32 - (n.saturating_sub(1) as f32 * 0.5);
            let x = lane_center + centered * 0.070;
            positions[*idx] = (x.clamp(0.06, 0.94), y.clamp(0.12, 0.95));
        }
    }

    for _ in 0..4 {
        for idx in 0..count {
            if let Some(parent) = parents[idx] {
                if parent >= count || parent == idx {
                    continue;
                }
                let lane_center = if lane_count == 1 {
                    0.5
                } else {
                    0.10 + (lane_by_idx[idx] as f32 / (lane_count - 1) as f32) * 0.80
                };
                let target = (positions[parent].0 * 0.35) + (lane_center * 0.65);
                positions[idx].0 = (positions[idx].0 * 0.70 + target * 0.30).clamp(0.06, 0.94);
            }
        }
    }

    positions
}

fn graph_layout_indented(parents: &[Option<usize>]) -> Vec<(f32, f32)> {
    let count = parents.len();
    if count == 0 {
        return Vec::new();
    }

    let depths = graph_depths(parents);
    let max_depth = depths.iter().copied().max().unwrap_or(0).max(1);
    let (children, roots) = graph_children_and_roots(parents);

    fn dfs(node: usize, children: &[Vec<usize>], out: &mut Vec<usize>) {
        out.push(node);
        for child in &children[node] {
            dfs(*child, children, out);
        }
    }

    let mut order = Vec::with_capacity(count);
    for root in roots {
        dfs(root, &children, &mut order);
    }
    for idx in 0..count {
        if !order.contains(&idx) {
            order.push(idx);
        }
    }

    let mut rank_by_idx = vec![0usize; count];
    for (rank, idx) in order.iter().enumerate() {
        rank_by_idx[*idx] = rank;
    }

    let mut positions = vec![(0.5f32, 0.5f32); count];
    let max_rank = count.saturating_sub(1).max(1);
    for idx in 0..count {
        let x = 0.08 + (depths[idx] as f32 / max_depth as f32) * 0.84;
        let y = 0.10 + (rank_by_idx[idx] as f32 / max_rank as f32) * 0.82;
        positions[idx] = (x.clamp(0.06, 0.95), y.clamp(0.10, 0.95));
    }

    positions
}

fn graph_children_and_roots(parents: &[Option<usize>]) -> (Vec<Vec<usize>>, Vec<usize>) {
    let count = parents.len();
    let mut children: Vec<Vec<usize>> = vec![Vec::new(); count];
    let mut roots: Vec<usize> = Vec::new();

    for idx in 0..count {
        match parents[idx] {
            Some(parent) if parent < count && parent != idx => children[parent].push(idx),
            _ => roots.push(idx),
        }
    }

    for list in &mut children {
        list.sort_unstable();
    }
    roots.sort_unstable();
    (children, roots)
}

fn assign_layered_x_units(children: &[Vec<usize>], roots: &[usize]) -> Vec<f32> {
    fn assign_layered_x(
        idx: usize,
        children: &[Vec<usize>],
        x_units: &mut [f32],
        cursor: &mut f32,
    ) {
        if children[idx].is_empty() {
            x_units[idx] = *cursor;
            *cursor += 1.0;
            return;
        }

        for child in &children[idx] {
            assign_layered_x(*child, children, x_units, cursor);
        }

        let first = children[idx][0];
        let last = *children[idx].last().unwrap_or(&first);
        x_units[idx] = (x_units[first] + x_units[last]) * 0.5;
    }

    let mut x_units = vec![0.0f32; children.len()];
    let mut cursor = 0.0f32;
    for root in roots {
        assign_layered_x(*root, children, &mut x_units, &mut cursor);
        cursor += 0.8;
    }
    x_units
}

fn subtree_sizes(children: &[Vec<usize>]) -> Vec<usize> {
    fn size_of(idx: usize, children: &[Vec<usize>], cache: &mut [Option<usize>]) -> usize {
        if let Some(size) = cache[idx] {
            return size;
        }
        let size = 1usize.saturating_add(
            children[idx]
                .iter()
                .map(|child| size_of(*child, children, cache))
                .sum::<usize>(),
        );
        cache[idx] = Some(size);
        size
    }

    let mut cache = vec![None; children.len()];
    (0..children.len())
        .map(|idx| size_of(idx, children, &mut cache))
        .collect()
}

fn graph_depths(parents: &[Option<usize>]) -> Vec<usize> {
    fn depth_for(i: usize, parents: &[Option<usize>], cache: &mut [Option<usize>]) -> usize {
        if let Some(depth) = cache[i] {
            return depth;
        }

        let depth = match parents[i] {
            Some(parent) if parent != i => depth_for(parent, parents, cache) + 1,
            _ => 0,
        };
        cache[i] = Some(depth);
        depth
    }

    let mut cache = vec![None; parents.len()];
    (0..parents.len())
        .map(|i| depth_for(i, parents, &mut cache))
        .collect()
}

fn worktree_parent_map(worktrees: &[WorktreeEntry], root_branch: &str) -> Vec<Option<usize>> {
    let mut branch_to_idx: BTreeMap<String, usize> = BTreeMap::new();
    for (idx, wt) in worktrees.iter().enumerate() {
        if !wt.detached && !wt.branch.is_empty() {
            branch_to_idx.entry(wt.branch.clone()).or_insert(idx);
        }
    }

    let mut parents = vec![None; worktrees.len()];
    for (idx, wt) in worktrees.iter().enumerate() {
        if wt.detached || is_root_branch(wt.branch.as_str(), root_branch) {
            continue;
        }

        if let Some(hint) = wt.parent_hint.as_deref() {
            if let Some(parent_idx) = branch_to_idx.get(hint) {
                if *parent_idx != idx {
                    parents[idx] = Some(*parent_idx);
                    continue;
                }
            }
        }

        if let Some(parent_idx) = find_branch_parent_idx(idx, wt.branch.as_str(), &branch_to_idx) {
            parents[idx] = Some(parent_idx);
        }
    }

    let root_idx = root_worktree_idx(worktrees, root_branch);

    if let Some(root_idx) = root_idx {
        for (idx, wt) in worktrees.iter().enumerate() {
            if idx == root_idx {
                continue;
            }
            if wt.detached {
                continue;
            }
            if parents[idx].is_none() {
                parents[idx] = Some(root_idx);
            }
        }
    }

    parents
}

fn root_worktree_idx(worktrees: &[WorktreeEntry], root_branch: &str) -> Option<usize> {
    worktrees
        .iter()
        .enumerate()
        .find_map(|(idx, wt)| {
            if !wt.detached && wt.branch == root_branch && wt.is_current {
                Some(idx)
            } else {
                None
            }
        })
        .or_else(|| {
            worktrees.iter().enumerate().find_map(|(idx, wt)| {
                if !wt.detached && wt.branch == root_branch {
                    Some(idx)
                } else {
                    None
                }
            })
        })
}

fn find_branch_parent_idx(
    current_idx: usize,
    branch: &str,
    branch_to_idx: &BTreeMap<String, usize>,
) -> Option<usize> {
    let mut parts: Vec<&str> = branch.split('/').collect();
    while parts.len() > 1 {
        parts.pop();
        let candidate = parts.join("/");
        if let Some(idx) = branch_to_idx.get(candidate.as_str()) {
            if *idx != current_idx {
                return Some(*idx);
            }
        }
    }
    None
}

fn is_root_branch(branch: &str, root_branch: &str) -> bool {
    branch == root_branch
}

fn worktree_parent_label(app: &App, parents: &[Option<usize>]) -> String {
    if app.selected_worktree >= app.worktrees.len() {
        return current_session_branch(app);
    }

    if let Some(parent_idx) = parents.get(app.selected_worktree).and_then(|v| *v) {
        if let Some(parent) = app.worktrees.get(parent_idx) {
            if parent.detached {
                return "detached".to_string();
            }
            return parent.branch.clone();
        }
    }

    current_session_branch(app)
}

fn current_session_branch(app: &App) -> String {
    if let Some(current) = app.worktrees.iter().find(|entry| entry.is_current) {
        if !current.detached && !current.branch.is_empty() {
            return current.branch.clone();
        }
        if !current.head.is_empty() {
            return current.head.clone();
        }
    }

    let raw = app.branch.trim();
    let name = raw
        .strip_prefix("HEAD (detached at ")
        .and_then(|value| value.strip_suffix(')'))
        .unwrap_or(raw);
    if name.is_empty() {
        "current".to_string()
    } else {
        name.to_string()
    }
}

fn draw_worktree_details_panel(frame: &mut ratatui::Frame<'_>, app: &App, area: Rect) {
    let mut lines: Vec<Line<'_>> = Vec::new();
    let root_branch = current_session_branch(app);
    let parents = worktree_parent_map(&app.worktrees, root_branch.as_str());
    let now = Instant::now();
    let live_sessions = app
        .agent_sessions
        .values()
        .filter(|session| agent_session_is_live(session))
        .count();
    let active_sessions = app
        .agent_sessions
        .values()
        .filter(|session| agent_session_is_active(session, now))
        .count();
    let idle_sessions = live_sessions.saturating_sub(active_sessions);

    if let Some(selected) = app.selected_worktree() {
        lines.push(Line::from(vec![
            Span::styled("branch: ", Style::default().fg(Color::Gray)),
            Span::styled(selected.branch.as_str(), Style::default().fg(Color::Cyan)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("path:   ", Style::default().fg(Color::Gray)),
            Span::styled(selected.path.as_str(), Style::default().fg(Color::White)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("head:   ", Style::default().fg(Color::Gray)),
            Span::styled(
                selected.head.as_str(),
                Style::default().fg(Color::LightBlue),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::styled("source: ", Style::default().fg(Color::Gray)),
            Span::styled(
                worktree_parent_label(app, &parents),
                Style::default().fg(Color::LightMagenta),
            ),
        ]));
        lines.push(Line::from(""));

        lines.push(Line::from(vec![
            Span::styled("dirty:  ", Style::default().fg(Color::Gray)),
            Span::styled(
                if selected.dirty { "yes" } else { "no" },
                Style::default().fg(if selected.dirty {
                    Color::Yellow
                } else {
                    Color::Green
                }),
            ),
            Span::raw("   "),
            Span::styled("locked: ", Style::default().fg(Color::Gray)),
            Span::styled(
                if selected.locked { "yes" } else { "no" },
                Style::default().fg(if selected.locked {
                    Color::Yellow
                } else {
                    Color::Green
                }),
            ),
        ]));

        lines.push(Line::from(vec![
            Span::styled("ahead:  ", Style::default().fg(Color::Gray)),
            Span::styled(
                selected.ahead.to_string(),
                Style::default().fg(Color::Green),
            ),
            Span::raw("   "),
            Span::styled("behind: ", Style::default().fg(Color::Gray)),
            Span::styled(
                selected.behind.to_string(),
                Style::default().fg(Color::Yellow),
            ),
        ]));

        lines.push(Line::from(vec![
            Span::styled("parent: ", Style::default().fg(Color::Gray)),
            Span::styled(
                if selected.behind_parent {
                    "behind parent"
                } else {
                    "in sync"
                },
                Style::default().fg(if selected.behind_parent {
                    Color::Yellow
                } else {
                    Color::Green
                }),
            ),
        ]));

        lines.push(Line::from(vec![
            Span::styled("flags:  ", Style::default().fg(Color::Gray)),
            Span::styled(
                worktree_flags(selected),
                Style::default().fg(Color::LightMagenta),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::styled("pty:    ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{} live", live_sessions),
                Style::default().fg(Color::LightCyan),
            ),
            Span::raw("  "),
            Span::styled("active ", Style::default().fg(Color::Gray)),
            Span::styled(
                active_sessions.to_string(),
                Style::default().fg(Color::Green),
            ),
            Span::raw("  "),
            Span::styled("idle ", Style::default().fg(Color::Gray)),
            Span::styled(
                idle_sessions.to_string(),
                Style::default().fg(Color::Yellow),
            ),
        ]));
        if let Some(session) = app.agent_sessions.get(selected.path.as_str()) {
            let context_tokens = agent_session_context_tokens(session);
            let output_tokens = agent_session_output_tokens(session);
            let total_tokens = agent_session_total_tokens(session);
            let context_rate = agent_session_context_tokens_per_second(session, now);
            let output_rate = agent_session_output_tokens_per_second(session, now);
            let token_label = if session.opencode_usage.is_some() {
                "tokens:"
            } else {
                "tokens~:"
            };
            lines.push(Line::from(vec![
                Span::styled(token_label, Style::default().fg(Color::Gray)),
                Span::styled(
                    format!(
                        "in {} ({}/s)",
                        format_compact_metric(context_tokens),
                        format_compact_metric(context_rate)
                    ),
                    Style::default().fg(Color::LightBlue),
                ),
                Span::raw("  "),
                Span::styled(
                    format!(
                        "out {} ({}/s)",
                        format_compact_metric(output_tokens),
                        format_compact_metric(output_rate)
                    ),
                    Style::default().fg(Color::LightGreen),
                ),
                Span::raw("  "),
                Span::styled(
                    format!("total {}", format_compact_metric(total_tokens)),
                    Style::default().fg(Color::LightCyan),
                ),
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::styled("tokens~:", Style::default().fg(Color::Gray)),
                Span::styled("no pty session", Style::default().fg(Color::DarkGray)),
            ]));
        }

        if app.perf_debug.enabled {
            lines.push(Line::from(vec![
                Span::styled("perf:   ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!(
                        "fps {:.1}  avg {:.1}ms  p95 {:.1}ms  worst {:.1}ms",
                        app.perf_debug.fps(),
                        app.perf_debug.avg_frame_ms(),
                        app.perf_debug.p95_frame_ms(),
                        app.perf_debug.worst_frame_ms()
                    ),
                    Style::default().fg(Color::LightCyan),
                ),
            ]));
            if let Some(hitch) = app.perf_debug.last_hitch {
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
                lines.push(Line::from(vec![
                    Span::styled("hitch:  ", Style::default().fg(Color::Gray)),
                    Span::styled(
                        format!(
                            "{:.1}ms total  draw {:.1}ms  opencode {:.1}ms  status {:.1}ms  trees {:.1}ms  unattributed {:.1}ms  ({:.1}s ago)",
                            hitch.phases.total_loop.as_secs_f64() * 1000.0,
                            hitch.phases.draw.as_secs_f64() * 1000.0,
                            hitch.phases.refresh_opencode_usage.as_secs_f64() * 1000.0,
                            hitch.phases.refresh_status.as_secs_f64() * 1000.0,
                            hitch.phases.refresh_worktrees.as_secs_f64() * 1000.0,
                            unattributed.as_secs_f64() * 1000.0,
                            now.saturating_duration_since(hitch.at).as_secs_f64(),
                        ),
                        Style::default().fg(Color::LightYellow),
                    ),
                ]));
            }
            lines.push(Line::from(vec![
                Span::styled("log:    ", Style::default().fg(Color::Gray)),
                Span::styled(
                    app.perf_debug.hitch_log_path.display().to_string(),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
        }

        lines.push(Line::from(""));
        let status_max = area.width.saturating_sub(4) as usize;
        let mut status_text = sanitize_for_tui(app.status_line.as_str());
        let mut status_color = Color::White;
        if let Some(task) = app.git_task.as_ref() {
            let elapsed = Instant::now().saturating_duration_since(task.started_at);
            status_text = format!(
                "[{}] {} ({:.1}s)",
                spinner_glyph(elapsed),
                task.label,
                elapsed.as_secs_f32()
            );
            status_color = Color::LightYellow;
        }
        let inner_height = area.height.saturating_sub(2) as usize;
        let status_max_lines = inner_height.saturating_sub(lines.len() + 1).max(1);
        lines.push(Line::from(vec![Span::styled(
            "status:",
            Style::default().fg(Color::Gray),
        )]));
        for wrapped in wrap_text_lines(status_text.as_str(), status_max.max(12), status_max_lines) {
            lines.push(Line::from(vec![Span::styled(
                wrapped,
                Style::default().fg(status_color),
            )]));
        }
    } else {
        lines.push(Line::from("No worktree selected"));
    }

    let border_color = if app.worktree_focus == WorktreePane::Details {
        Color::Cyan
    } else {
        Color::Gray
    };

    let panel = Paragraph::new(lines)
        .block(
            Block::default()
                .title("details [?]")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
                .style(Style::default().bg(Color::Black)),
        )
        .style(Style::default().bg(Color::Black).fg(Color::White))
        .alignment(Alignment::Left);

    frame.render_widget(panel, area);
}

fn worktree_right_panel_constraints(app: &App, area: Rect) -> [Constraint; 3] {
    let desired_art_lines = trimmed_art_line_count(app);
    let desired_art_height = desired_art_lines.saturating_add(2) as u16;

    let total_height = area.height;
    let spacing = 2u16;
    let available = total_height.saturating_sub(spacing);
    let min_details = 8u16;
    let min_actions = 8u16;
    let max_art = available.saturating_sub(min_details.saturating_add(min_actions));
    let art_height = desired_art_height.clamp(3, max_art.max(3));

    [
        Constraint::Length(art_height),
        Constraint::Min(min_details),
        Constraint::Min(min_actions),
    ]
}

fn trimmed_art_line_count(app: &App) -> usize {
    let mut start = 0usize;
    let mut end = app.config.worktree_graph_art.len();

    while start < end && app.config.worktree_graph_art[start].trim().is_empty() {
        start += 1;
    }
    while end > start && app.config.worktree_graph_art[end - 1].trim().is_empty() {
        end -= 1;
    }

    end.saturating_sub(start)
}

fn trimmed_art_lines<'a>(app: &'a App) -> &'a [String] {
    let mut start = 0usize;
    let mut end = app.config.worktree_graph_art.len();

    while start < end && app.config.worktree_graph_art[start].trim().is_empty() {
        start += 1;
    }
    while end > start && app.config.worktree_graph_art[end - 1].trim().is_empty() {
        end -= 1;
    }

    &app.config.worktree_graph_art[start..end]
}

fn draw_worktree_art_panel(frame: &mut ratatui::Frame<'_>, app: &App, area: Rect) {
    let max_width = area.width.saturating_sub(2) as usize;
    let max_lines = area.height.saturating_sub(2) as usize;
    let mut lines: Vec<Line<'_>> = Vec::new();

    if max_width > 0 && max_lines > 0 {
        for raw in trimmed_art_lines(app).iter().take(max_lines) {
            let clean = sanitize_for_tui(raw.as_str());
            lines.push(Line::from(truncate_text(clean.as_str(), max_width)));
        }
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            "No art configured",
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(Span::styled(
            "Set worktree_graph_art in config.toml",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let panel = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Gray))
                .style(Style::default().bg(Color::Black)),
        )
        .style(Style::default().bg(Color::Black).fg(Color::White))
        .alignment(Alignment::Left);

    frame.render_widget(panel, area);
}

fn draw_worktree_actions_panel(frame: &mut ratatui::Frame<'_>, app: &App, area: Rect) {
    let border_color = if app.worktree_focus == WorktreePane::Actions {
        Color::Cyan
    } else {
        Color::Gray
    };

    let mut lines = Vec::new();

    if let Some(task) = app.git_task.as_ref() {
        let elapsed = Instant::now().saturating_duration_since(task.started_at);
        let spinner = spinner_glyph(elapsed);
        let queued = app.git_task_queue.len();
        let queue_suffix = if queued > 0 {
            format!(" +{} queued", queued)
        } else {
            String::new()
        };
        lines.push(Line::from(vec![
            Span::styled("busy ", Style::default().fg(Color::Yellow)),
            Span::styled(
                format!(
                    "[{}] {} ({:.1}s){}",
                    spinner,
                    task.label,
                    elapsed.as_secs_f32(),
                    queue_suffix,
                ),
                Style::default().fg(Color::LightYellow),
            ),
        ]));
        lines.push(Line::from(""));
    }

    lines.extend(vec![
        Line::from(vec![
            Span::styled("w", Style::default().fg(Color::LightBlue)),
            Span::raw(" file changes view"),
        ]),
        Line::from(vec![
            Span::styled("r", Style::default().fg(Color::Cyan)),
            Span::raw(" refresh worktrees"),
        ]),
        Line::from(vec![
            Span::styled("q", Style::default().fg(Color::Red)),
            Span::raw(" quit"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("a", Style::default().fg(Color::LightGreen)),
            Span::raw(" create worktree"),
        ]),
        Line::from(vec![
            Span::styled("g", Style::default().fg(Color::LightGreen)),
            Span::raw(" orchestrate worktrees from feature requirement"),
        ]),
        Line::from(vec![
            Span::styled("o", Style::default().fg(Color::LightBlue)),
            Span::raw(" open terminal popup"),
        ]),
        Line::from(vec![
            Span::styled("O", Style::default().fg(Color::LightBlue)),
            Span::raw(" open agent picker"),
        ]),
        Line::from(vec![
            Span::styled("f", Style::default().fg(Color::Cyan)),
            Span::raw(" fetch + pull parent"),
        ]),
        Line::from(vec![
            Span::styled("F", Style::default().fg(Color::Cyan)),
            Span::raw(" rebase selected onto parent"),
        ]),
        Line::from(vec![
            Span::styled("c", Style::default().fg(Color::Magenta)),
            Span::raw(" add+commit"),
        ]),
        Line::from(vec![
            Span::styled("p", Style::default().fg(Color::Magenta)),
            Span::raw(" push"),
        ]),
        Line::from(vec![
            Span::styled("d", Style::default().fg(Color::LightRed)),
            Span::raw(" delete selected (prompts if dirty)"),
        ]),
        Line::from(vec![
            Span::styled("m", Style::default().fg(Color::LightGreen)),
            Span::raw(" merge to parent"),
        ]),
        Line::from(vec![
            Span::styled("n", Style::default().fg(Color::LightCyan)),
            Span::raw(" notes (vim-style)"),
        ]),
        Line::from(vec![
            Span::styled("x", Style::default().fg(Color::Yellow)),
            Span::raw(" prune stale"),
        ]),
        Line::from(vec![
            Span::styled("L", Style::default().fg(Color::LightCyan)),
            Span::raw(" git command history popup"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("tab", Style::default().fg(Color::LightBlue)),
            Span::raw(" switch panel"),
        ]),
        Line::from(vec![
            Span::styled("?", Style::default().fg(Color::Yellow)),
            Span::raw(" panel help"),
        ]),
        Line::from(vec![
            Span::styled("arrows", Style::default().fg(Color::LightBlue)),
            Span::raw(" move on canvas"),
        ]),
        Line::from(vec![
            Span::styled("+/-", Style::default().fg(Color::LightBlue)),
            Span::raw(" zoom canvas"),
        ]),
        Line::from(vec![
            Span::styled("0", Style::default().fg(Color::LightBlue)),
            Span::raw(" reset camera"),
        ]),
        Line::from(vec![
            Span::styled("Shift+WASD", Style::default().fg(Color::LightBlue)),
            Span::raw(" pan camera"),
        ]),
        Line::from(vec![
            Span::styled("h/l", Style::default().fg(Color::LightBlue)),
            Span::raw(" left/right in level"),
        ]),
        Line::from(vec![
            Span::styled("j/k", Style::default().fg(Color::LightBlue)),
            Span::raw(" child/parent level"),
        ]),
        Line::from(vec![
            Span::styled("Ctrl+K", Style::default().fg(Color::LightBlue)),
            Span::raw(" next graph builder"),
        ]),
    ]);

    let title = if app.git_task.is_some() {
        let queued = app.git_task_queue.len();
        if queued > 0 {
            format!("actions [?] busy +{} queued", queued)
        } else {
            "actions [?] busy".to_string()
        }
    } else {
        "actions [?]".to_string()
    };

    frame.render_widget(
        Paragraph::new(lines)
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color))
                    .style(Style::default().bg(Color::Black)),
            )
            .style(Style::default().fg(Color::White)),
        area,
    );
}

fn spinner_glyph(elapsed: Duration) -> char {
    const FRAMES: [char; 4] = ['|', '/', '-', '\\'];
    let idx = ((elapsed.as_millis() / 120) as usize) % FRAMES.len();
    FRAMES[idx]
}

fn draw_worktree_help_modal(frame: &mut ratatui::Frame<'_>, app: &App) {
    let popup = centered_rect(64, 42, frame.area());
    frame.render_widget(Clear, popup);

    let lines = worktree_help_lines(app.worktree_focus);
    let panel = Paragraph::new(lines)
        .block(
            Block::default()
                .title("Panel Help")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow))
                .style(Style::default().bg(Color::Black)),
        )
        .style(Style::default().fg(Color::White));

    frame.render_widget(panel, popup);
}

fn worktree_help_lines(pane: WorktreePane) -> Vec<Line<'static>> {
    match pane {
        WorktreePane::Canvas => vec![
            Line::from("Worktree Graph"),
            Line::from(""),
            Line::from("- Each node is an isolated git worktree for parallel agent runs"),
            Line::from("- Edges show parent/child branch lineage for safe merges"),
            Line::from("- Token telemetry appears in details for selected PTY only"),
            Line::from("- Blue node = current branch worktree"),
            Line::from("- Cyan ring = selected worktree (drives details + actions)"),
            Line::from("- Red nodes = dirty (uncommitted changes)"),
            Line::from("- Yellow nodes = behind parent"),
            Line::from("- Spinner suffix means active session; done/fail marks completion"),
            Line::from(""),
            Line::from("Navigation:"),
            Line::from("  arrows  - move by graph direction"),
            Line::from("  h/l     - move between siblings"),
            Line::from("  j/k     - move child/parent levels"),
            Line::from("  Ctrl+K  - cycle graph builder (6 layouts)"),
            Line::from("  Tab     - cycle graph/details/actions panels"),
            Line::from("  L       - open git command history popup"),
            Line::from(""),
            Line::from("Camera:"),
            Line::from("  +/-     - zoom in/out"),
            Line::from("  0       - reset view"),
            Line::from("  Shift+WASD - pan"),
            Line::from("  Ctrl+B  - cycle canvas background"),
            Line::from("  Ctrl+L  - toggle perf debugging + hitch log"),
            Line::from(""),
            Line::from("Flow: o/O launch shells or agents, c/p/m/d run git lifecycle"),
            Line::from(""),
            Line::from("  ?: close this help"),
        ],
        WorktreePane::Details => vec![
            Line::from("Details panel"),
            Line::from("- Reflects the selected graph node"),
            Line::from("- Shows branch/path/HEAD and worktree flags"),
            Line::from("- Shows ahead/behind and dirty/locked state"),
            Line::from("- Includes total PTY counts (live/active/idle)"),
            Line::from("- Shows OpenCode exact token usage when available, else PTY estimates"),
            Line::from("- Status section reports the latest command outcome"),
            Line::from("- Use this panel to validate readiness before push/merge"),
            Line::from("- Tab: move focus to next panel"),
            Line::from("- ?: close this help"),
        ],
        WorktreePane::Actions => vec![
            Line::from("Actions panel"),
            Line::from("- a: create worktree from branch name"),
            Line::from("- g: orchestrate feature into a multi-worktree plan + create"),
            Line::from("- o: open/reopen terminal popup for selected node"),
            Line::from("- O: open agent picker for selected/conflicted parent"),
            Line::from("- c: selected worktree add+commit with message popup"),
            Line::from("- p: push selected worktree branch (sets upstream if needed)"),
            Line::from("- f: fetch + pull connected parent node"),
            Line::from("- F: rebase selected branch onto connected parent node"),
            Line::from("- m: merge selected branch into connected parent node"),
            Line::from("- d: delete selected worktree (asks force-delete if dirty)"),
            Line::from("- x: prune stale worktrees"),
            Line::from("- n: open notes.md (vim-style editor), L: git command history"),
            Line::from("- Terminal popup: ':' control mode, Ctrl+G toggles input/control"),
            Line::from("- Agent defaults/prompts live in ~/.config/openswarm"),
            Line::from("- ?: close this help"),
        ],
    }
}

fn draw_worktree_git_log_modal(frame: &mut ratatui::Frame<'_>, app: &App) {
    let popup = centered_rect(82, 72, frame.area());
    frame.render_widget(Clear, popup);

    let border = Block::default()
        .title("Git Command History")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black))
        .border_style(Style::default().fg(Color::LightCyan));
    frame.render_widget(border, popup);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(8),
            Constraint::Length(1),
        ])
        .split(popup);

    let path = app
        .git_log_popup_path
        .as_deref()
        .unwrap_or("(no worktree selected)");
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("worktree: ", Style::default().fg(Color::Gray)),
            Span::styled(path, Style::default().fg(Color::White)),
        ]))
        .style(Style::default().fg(Color::White)),
        layout[0],
    );

    let lines: Vec<Line<'_>> = if app.git_log_lines.is_empty() {
        vec![Line::from("(no git log entries)")]
    } else {
        app.git_log_lines
            .iter()
            .map(|line| Line::from(line.as_str()))
            .collect()
    };

    frame.render_widget(
        Paragraph::new(lines)
            .scroll((app.git_log_scroll, 0))
            .block(Block::default().title("reflog").borders(Borders::ALL))
            .style(Style::default().fg(Color::White)),
        layout[1],
    );

    frame.render_widget(
        Paragraph::new("j/k or arrows scroll, PgUp/PgDn jump, Home/End, L|q|Esc close")
            .style(Style::default().fg(Color::Gray)),
        layout[2],
    );
}

fn draw_worktree_create_modal(frame: &mut ratatui::Frame<'_>, app: &App) {
    let popup = centered_rect(74, 30, frame.area());
    frame.render_widget(Clear, popup);

    let border = Block::default()
        .title("Create Worktree")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black))
        .border_style(Style::default().fg(Color::LightGreen));
    frame.render_widget(border, popup);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(popup);

    frame.render_widget(
        Paragraph::new(
            "Choose source above, then type worktree branch. Enter creates '.<repo>-workspaces/<branch>' next to the repo.",
        )
            .style(Style::default().fg(Color::Gray)),
        layout[0],
    );

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("Base: ", Style::default().fg(Color::Gray)),
            Span::styled(
                worktree_create_base_label(app.new_worktree_base),
                Style::default().fg(Color::LightGreen),
            ),
            Span::raw("  (use ←/→)"),
        ]))
        .block(Block::default().title("Source").borders(Borders::ALL))
        .style(Style::default().fg(Color::White)),
        layout[1],
    );

    frame.render_widget(
        Paragraph::new(app.new_worktree_branch.as_str())
            .block(Block::default().title("Branch").borders(Borders::ALL))
            .style(Style::default().fg(Color::Cyan)),
        layout[2],
    );

    frame.render_widget(
        Paragraph::new("Esc cancels")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray)),
        layout[3],
    );
}

fn draw_worktree_orchestrate_modal(frame: &mut ratatui::Frame<'_>, app: &App) {
    let popup = centered_rect(84, 36, frame.area());
    frame.render_widget(Clear, popup);

    let border = Block::default()
        .title("Orchestrate Worktrees")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black))
        .border_style(Style::default().fg(Color::LightGreen));
    frame.render_widget(border, popup);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(popup);

    let planner_state = if app.config.worktree_orchestrator_enabled {
        "enabled"
    } else {
        "disabled"
    };
    frame.render_widget(
        Paragraph::new(vec![
            Line::from(
                "Describe a feature; OpenSwarm asks OpenCode for a split plan and creates worktree abstractions only.",
            ),
            Line::from(format!(
                "Planner: {} | max nodes: {}",
                planner_state, app.config.worktree_orchestrator_max_nodes
            )),
        ])
        .style(Style::default().fg(Color::Gray)),
        layout[0],
    );

    frame.render_widget(
        Paragraph::new(format!(
            "Prompt template: {}",
            app.config.worktree_orchestrator_prompt_path
        ))
        .block(Block::default().title("Template").borders(Borders::ALL))
        .style(Style::default().fg(Color::White)),
        layout[1],
    );

    frame.render_widget(
        Paragraph::new(app.orchestrator_requirement_input.as_str())
            .block(
                Block::default()
                    .title("Feature Requirement")
                    .borders(Borders::ALL),
            )
            .style(Style::default().fg(Color::Cyan)),
        layout[2],
    );

    frame.render_widget(
        Paragraph::new("Enter: plan+create, Esc: cancel")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray)),
        layout[3],
    );
}

fn draw_agent_popup(frame: &mut ratatui::Frame<'_>, app: &App) {
    let popup = terminal_popup_rect(frame.area());
    frame.render_widget(Clear, popup);

    let border = Block::default()
        .title("Terminal Session")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black))
        .border_style(Style::default().fg(Color::Cyan));
    frame.render_widget(border, popup);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(1),
        ])
        .split(popup);

    let path = app
        .agent_popup_path
        .as_deref()
        .unwrap_or("(no worktree selected)");
    let state = agent_state_for_path(app, path);
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("worktree: ", Style::default().fg(Color::Gray)),
            Span::styled(path, Style::default().fg(Color::White)),
            Span::raw("   "),
            Span::styled("terminal: ", Style::default().fg(Color::Gray)),
            Span::styled("shell", Style::default().fg(Color::LightCyan)),
            Span::raw("   "),
            Span::styled("mode: ", Style::default().fg(Color::Gray)),
            Span::styled(
                terminal_popup_mode_text(app.terminal_popup_mode),
                terminal_popup_mode_style(app.terminal_popup_mode),
            ),
            Span::raw("   "),
            Span::styled("status: ", Style::default().fg(Color::Gray)),
            Span::styled(agent_state_text(state), agent_state_style(state)),
        ])),
        layout[0],
    );

    let mut lines: Vec<Line<'_>> = Vec::new();
    if let Some(session) = app.agent_sessions.get(path) {
        let visible_rows = layout[2].height.saturating_sub(2) as usize;
        let width = layout[2].width.saturating_sub(2).max(1);
        lines = render_terminal_lines(session, width, visible_rows);
    }
    if lines.is_empty() {
        lines.push(Line::from("(terminal booting...)"));
    }

    frame.render_widget(
        Paragraph::new(lines)
            .block(Block::default().title("Terminal").borders(Borders::ALL))
            .style(Style::default().fg(Color::White)),
        layout[2],
    );
    frame.render_widget(
        Paragraph::new(terminal_footer_text(app.terminal_popup_mode))
            .style(Style::default().fg(Color::Gray)),
        layout[3],
    );
}

fn draw_agent_select_modal(frame: &mut ratatui::Frame<'_>, app: &App) {
    let popup = centered_rect(64, 40, frame.area());
    frame.render_widget(Clear, popup);

    let border = Block::default()
        .title("Resolve With Agent")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black))
        .border_style(Style::default().fg(Color::LightBlue));
    frame.render_widget(border, popup);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Min(4),
            Constraint::Length(2),
        ])
        .split(popup);

    let target = app
        .agent_select_path
        .as_deref()
        .unwrap_or("(no target worktree)");
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("worktree: ", Style::default().fg(Color::Gray)),
            Span::styled(target, Style::default().fg(Color::White)),
        ]))
        .style(Style::default().fg(Color::White)),
        layout[0],
    );

    let hint = if app.pending_conflict_context.is_some() {
        "Conflict context found: Enter launches agent and pastes resolve prompt"
    } else {
        "Select agent: Enter to launch, Esc to cancel"
    };
    frame.render_widget(
        Paragraph::new(hint).style(Style::default().fg(Color::Gray)),
        layout[1],
    );

    let rows: Vec<Line<'_>> = if app.detected_agents.is_empty() {
        vec![Line::from(Span::styled(
            "No agent CLI found (expected opencode or claude)",
            Style::default().fg(Color::Red),
        ))]
    } else {
        app.detected_agents
            .iter()
            .enumerate()
            .map(|(idx, agent)| {
                let selected = idx == app.agent_select_index;
                let marker = if selected { ">" } else { " " };
                let style = if selected {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                Line::from(vec![
                    Span::styled(format!(" {} ", marker), style),
                    Span::styled(
                        format!("{}  ({})", agent.display_name(), agent.command_name()),
                        style,
                    ),
                ])
            })
            .collect()
    };
    frame.render_widget(
        Paragraph::new(rows)
            .block(Block::default().title("agents").borders(Borders::ALL))
            .style(Style::default().fg(Color::White)),
        layout[2],
    );

    frame.render_widget(
        Paragraph::new(format!(
            "Up/Down or j/k select, Enter launch, Esc cancel | default via {}",
            app.config.config_path
        ))
        .style(Style::default().fg(Color::Gray)),
        layout[3],
    );
}

fn terminal_popup_mode_text(mode: TerminalPopupMode) -> &'static str {
    match mode {
        TerminalPopupMode::Input => "INPUT",
        TerminalPopupMode::Control => "CONTROL",
    }
}

fn terminal_popup_mode_style(mode: TerminalPopupMode) -> Style {
    match mode {
        TerminalPopupMode::Input => Style::default().fg(Color::LightGreen),
        TerminalPopupMode::Control => Style::default().fg(Color::LightYellow),
    }
}

fn terminal_footer_text(mode: TerminalPopupMode) -> &'static str {
    match mode {
        TerminalPopupMode::Input => {
            "INPUT mode: typing goes to terminal (Esc is forwarded). Ctrl+G/Cmd+G toggles CONTROL."
        }
        TerminalPopupMode::Control => {
            "CONTROL mode: Esc background, q quit session, r restart, Ctrl+G/Cmd+G returns INPUT."
        }
    }
}

fn agent_state_for_path(app: &App, path: &str) -> AgentState {
    app.agent_sessions
        .get(path)
        .map(|s| s.state)
        .unwrap_or(AgentState::Launching)
}

fn agent_state_text(state: AgentState) -> &'static str {
    match state {
        AgentState::Launching => "loading",
        AgentState::Running => "running",
        AgentState::Done => "done",
        AgentState::Failed => "failed",
    }
}

fn agent_state_style(state: AgentState) -> Style {
    match state {
        AgentState::Launching => Style::default().fg(Color::Yellow),
        AgentState::Running => Style::default().fg(Color::LightCyan),
        AgentState::Done => Style::default().fg(Color::Green),
        AgentState::Failed => Style::default().fg(Color::Red),
    }
}

fn render_terminal_lines(
    session: &AgentSession,
    width: u16,
    visible_rows: usize,
) -> Vec<Line<'static>> {
    if width == 0 || visible_rows == 0 {
        return Vec::new();
    }

    let screen = session.parser.screen();
    let (rows, cols) = screen.size();
    let cols = cols.min(width);
    let start_row = rows.saturating_sub(visible_rows as u16);
    let mut out = Vec::new();

    for row in start_row..rows {
        let mut spans: Vec<Span<'static>> = Vec::new();
        let mut run = String::new();
        let mut run_style: Option<Style> = None;

        for col in 0..cols {
            let Some(cell) = screen.cell(row, col) else {
                continue;
            };
            if cell.is_wide_continuation() {
                continue;
            }
            let style = vt_cell_style(cell);
            let mut text = cell.contents();
            if text.is_empty() {
                text.push(' ');
            }

            match run_style {
                Some(existing) if existing == style => {
                    run.push_str(text.as_str());
                }
                _ => {
                    if !run.is_empty() {
                        let taken = std::mem::take(&mut run);
                        spans.push(Span::styled(taken, run_style.unwrap_or_default()));
                    }
                    run_style = Some(style);
                    run.push_str(text.as_str());
                }
            }
        }

        if !run.is_empty() {
            spans.push(Span::styled(run, run_style.unwrap_or_default()));
        }

        if spans.is_empty() {
            out.push(Line::from(""));
        } else {
            out.push(Line::from(spans));
        }
    }

    out
}

fn vt_cell_style(cell: &vt100::Cell) -> Style {
    let mut style = Style::default();
    style = style.fg(vt_color_to_ratatui(cell.fgcolor(), true));
    style = style.bg(vt_color_to_ratatui(cell.bgcolor(), false));
    if cell.bold() {
        style = style.add_modifier(Modifier::BOLD);
    }
    if cell.italic() {
        style = style.add_modifier(Modifier::ITALIC);
    }
    if cell.underline() {
        style = style.add_modifier(Modifier::UNDERLINED);
    }
    style
}

fn vt_color_to_ratatui(color: vt100::Color, is_fg: bool) -> Color {
    match color {
        vt100::Color::Default => {
            if is_fg {
                Color::White
            } else {
                Color::Black
            }
        }
        vt100::Color::Rgb(r, g, b) => Color::Rgb(r, g, b),
        vt100::Color::Idx(i) => ansi_idx_to_color(i),
    }
}

fn ansi_idx_to_color(i: u8) -> Color {
    match i {
        0 => Color::Black,
        1 => Color::Red,
        2 => Color::Green,
        3 => Color::Yellow,
        4 => Color::Blue,
        5 => Color::Magenta,
        6 => Color::Cyan,
        7 => Color::Gray,
        8 => Color::DarkGray,
        9 => Color::LightRed,
        10 => Color::LightGreen,
        11 => Color::LightYellow,
        12 => Color::LightBlue,
        13 => Color::LightMagenta,
        14 => Color::LightCyan,
        15 => Color::White,
        16..=231 => {
            let idx = i - 16;
            let r = idx / 36;
            let g = (idx % 36) / 6;
            let b = idx % 6;
            let scale = [0, 95, 135, 175, 215, 255];
            Color::Rgb(scale[r as usize], scale[g as usize], scale[b as usize])
        }
        232..=255 => {
            let shade = 8 + (i - 232) * 10;
            Color::Rgb(shade, shade, shade)
        }
    }
}

fn draw_worktree_remove_dirty_confirm_modal(frame: &mut ratatui::Frame<'_>, app: &App) {
    let popup = centered_rect(76, 28, frame.area());
    frame.render_widget(Clear, popup);

    let border = Block::default()
        .title("Dirty Worktree")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black))
        .border_style(Style::default().fg(Color::LightRed));
    frame.render_widget(border, popup);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(popup);

    let target = if app.pending_remove_worktree_path.is_empty() {
        "(selected worktree)"
    } else {
        app.pending_remove_worktree_path.as_str()
    };

    frame.render_widget(
        Paragraph::new(format!(
            "Worktree '{}' has uncommitted changes. Force delete anyway?",
            target
        ))
        .style(Style::default().fg(Color::White)),
        layout[0],
    );

    frame.render_widget(
        Paragraph::new(
            "Yes runs `git worktree remove --force` and discards uncommitted changes in that worktree.",
        )
        .style(Style::default().fg(Color::Gray)),
        layout[1],
    );

    let yes_style = if app.confirm_remove_worktree_yes {
        Style::default().fg(Color::Black).bg(Color::LightRed)
    } else {
        Style::default().fg(Color::White)
    };
    let no_style = if app.confirm_remove_worktree_yes {
        Style::default().fg(Color::White)
    } else {
        Style::default().fg(Color::Black).bg(Color::LightGreen)
    };

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("[ Yes: force delete ]", yes_style),
            Span::raw("   "),
            Span::styled("[ No: keep worktree ]", no_style),
        ]))
        .alignment(Alignment::Center),
        layout[2],
    );

    frame.render_widget(
        Paragraph::new("No is selected by default")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray)),
        layout[3],
    );
}

fn draw_branch_conflict_confirm_modal(frame: &mut ratatui::Frame<'_>, app: &App) {
    let popup = centered_rect(72, 26, frame.area());
    frame.render_widget(Clear, popup);

    let border = Block::default()
        .title("Branch Exists")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black))
        .border_style(Style::default().fg(Color::LightRed));
    frame.render_widget(border, popup);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(popup);

    let branch = if app.pending_create_branch.is_empty() {
        app.new_worktree_branch.as_str()
    } else {
        app.pending_create_branch.as_str()
    };

    frame.render_widget(
        Paragraph::new(format!(
            "Branch '{}' already exists. Delete and create new worktree?",
            branch
        ))
        .style(Style::default().fg(Color::White)),
        layout[0],
    );

    frame.render_widget(
        Paragraph::new(
            "Default selection is No. Use ←/→ (or y/n), Enter to confirm, Esc to cancel.",
        )
        .style(Style::default().fg(Color::Gray)),
        layout[1],
    );

    let yes_style = if app.confirm_delete_branch_yes {
        Style::default().fg(Color::Black).bg(Color::LightRed)
    } else {
        Style::default().fg(Color::White)
    };
    let no_style = if app.confirm_delete_branch_yes {
        Style::default().fg(Color::White)
    } else {
        Style::default().fg(Color::Black).bg(Color::LightGreen)
    };

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("[ Yes: delete + recreate ]", yes_style),
            Span::raw("   "),
            Span::styled("[ No: keep branch ]", no_style),
        ]))
        .alignment(Alignment::Center),
        layout[2],
    );

    frame.render_widget(
        Paragraph::new("No is selected by default")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray)),
        layout[3],
    );
}

fn draw_conflict_resolve_confirm_modal(frame: &mut ratatui::Frame<'_>, app: &App) {
    let popup = centered_rect(84, 62, frame.area());
    frame.render_widget(Clear, popup);

    let border = Block::default()
        .title("Resolve With OpenCode")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black))
        .border_style(Style::default().fg(Color::LightBlue));
    frame.render_widget(border, popup);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Min(8),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(popup);

    let Some(context) = app.pending_conflict_context.as_ref() else {
        frame.render_widget(
            Paragraph::new("No merge-conflict context found")
                .style(Style::default().fg(Color::Red)),
            layout[0],
        );
        return;
    };

    let files = if context.conflicted_files.is_empty() {
        "(none reported)".to_string()
    } else {
        truncate_text(context.conflicted_files.join(", ").as_str(), 180)
    };
    let prompt = build_conflict_resolve_prompt(app, context);
    let prompt_path = truncate_text(
        app.config.conflict_resolve_prompt_path.as_str(),
        layout[4].width.saturating_sub(4) as usize,
    );
    let prompt_lines = wrap_text_lines(
        prompt.as_str(),
        layout[5].width.saturating_sub(4) as usize,
        layout[5].height.saturating_sub(2) as usize,
    )
    .into_iter()
    .map(Line::from)
    .collect::<Vec<Line<'_>>>();

    frame.render_widget(
        Paragraph::new("Merge conflicts were detected. Resolve with OpenCode using this prompt?")
            .style(Style::default().fg(Color::White)),
        layout[0],
    );

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("parent: ", Style::default().fg(Color::Gray)),
            Span::styled(
                context.parent_path.as_str(),
                Style::default().fg(Color::White),
            ),
        ])),
        layout[1],
    );

    frame.render_widget(
        Paragraph::new("conflicted files:").style(Style::default().fg(Color::Gray)),
        layout[2],
    );

    frame.render_widget(
        Paragraph::new(files)
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::White)),
        layout[3],
    );

    frame.render_widget(
        Paragraph::new(format!("prompt file: {}", prompt_path))
            .style(Style::default().fg(Color::Gray)),
        layout[4],
    );

    frame.render_widget(
        Paragraph::new(prompt_lines)
            .block(
                Block::default()
                    .title("prompt preview")
                    .borders(Borders::ALL),
            )
            .style(Style::default().fg(Color::White)),
        layout[5],
    );

    let yes_style = if app.confirm_conflict_resolve_yes {
        Style::default().fg(Color::Black).bg(Color::LightBlue)
    } else {
        Style::default().fg(Color::White)
    };
    let no_style = if app.confirm_conflict_resolve_yes {
        Style::default().fg(Color::White)
    } else {
        Style::default().fg(Color::Black).bg(Color::LightGreen)
    };

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("[ Yes: launch OpenCode ]", yes_style),
            Span::raw("   "),
            Span::styled("[ No: resolve manually ]", no_style),
        ]))
        .alignment(Alignment::Center),
        layout[6],
    );

    frame.render_widget(
        Paragraph::new(
            "No is default | <-/-> toggle | e edit+save prompt | Enter confirm | Esc cancel",
        )
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray)),
        layout[7],
    );
}

fn draw_legacy_workspace_migrate_modal(frame: &mut ratatui::Frame<'_>, app: &App) {
    let popup = centered_rect(82, 36, frame.area());
    frame.render_widget(Clear, popup);

    let border = Block::default()
        .title("Legacy Workspace Layout")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black))
        .border_style(Style::default().fg(Color::Yellow));
    frame.render_widget(border, popup);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(popup);

    frame.render_widget(
        Paragraph::new("Detected legacy in-repo '.gitfetch-worktrees'. Migrate to sibling '.<repo>-workspaces'?")
            .style(Style::default().fg(Color::White)),
        layout[0],
    );

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("from: ", Style::default().fg(Color::Gray)),
            Span::styled(
                app.pending_legacy_workspace_path.as_str(),
                Style::default().fg(Color::White),
            ),
        ]))
        .style(Style::default().fg(Color::White)),
        layout[1],
    );

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("to:   ", Style::default().fg(Color::Gray)),
            Span::styled(
                app.pending_new_workspace_path.as_str(),
                Style::default().fg(Color::White),
            ),
        ]))
        .style(Style::default().fg(Color::White)),
        layout[2],
    );

    let yes_style = if app.confirm_legacy_workspace_migrate_yes {
        Style::default().fg(Color::Black).bg(Color::LightGreen)
    } else {
        Style::default().fg(Color::White)
    };
    let no_style = if app.confirm_legacy_workspace_migrate_yes {
        Style::default().fg(Color::White)
    } else {
        Style::default().fg(Color::Black).bg(Color::LightRed)
    };

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("[ Yes: migrate now ]", yes_style),
            Span::raw("   "),
            Span::styled("[ No: skip ]", no_style),
        ]))
        .alignment(Alignment::Center),
        layout[3],
    );

    frame.render_widget(
        Paragraph::new(
            "Uses 'git worktree move' for tracked worktrees, moves .parent-hints when possible",
        )
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray)),
        layout[4],
    );
}

fn draw_quit_with_sessions_modal(frame: &mut ratatui::Frame<'_>, app: &App) {
    let popup = centered_rect(68, 26, frame.area());
    frame.render_widget(Clear, popup);

    let border = Block::default()
        .title("Active Sessions")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black))
        .border_style(Style::default().fg(Color::LightRed));
    frame.render_widget(border, popup);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(popup);

    let count = live_terminal_session_count(app);
    frame.render_widget(
        Paragraph::new(format!(
            "You have {} active terminal session(s). Quit anyway?",
            count
        ))
        .style(Style::default().fg(Color::White)),
        layout[0],
    );

    frame.render_widget(
        Paragraph::new("Choosing Yes will close the TUI and terminate those PTY sessions.")
            .style(Style::default().fg(Color::Gray)),
        layout[1],
    );

    let yes_style = if app.confirm_quit_with_sessions_yes {
        Style::default().fg(Color::Black).bg(Color::LightRed)
    } else {
        Style::default().fg(Color::White)
    };
    let no_style = if app.confirm_quit_with_sessions_yes {
        Style::default().fg(Color::White)
    } else {
        Style::default().fg(Color::Black).bg(Color::LightGreen)
    };

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("[ Yes: quit ]", yes_style),
            Span::raw("   "),
            Span::styled("[ No: stay ]", no_style),
        ]))
        .alignment(Alignment::Center),
        layout[2],
    );

    frame.render_widget(
        Paragraph::new("No is selected by default")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray)),
        layout[3],
    );
}

fn worktree_create_base_label(base: WorktreeCreateBase) -> &'static str {
    match base {
        WorktreeCreateBase::Main => "main branch",
        WorktreeCreateBase::Selected => "selected branch/worktree",
        WorktreeCreateBase::SelectedWithChanges => "selected branch/worktree + uncommitted changes",
    }
}

fn worktree_flags(entry: &WorktreeEntry) -> String {
    let mut flags: Vec<&str> = Vec::new();
    if entry.is_current {
        flags.push("current");
    }
    if entry.detached {
        flags.push("detached");
    }
    if entry.bare {
        flags.push("bare");
    }
    if entry.locked {
        flags.push("locked");
    }
    if entry.prunable {
        flags.push("prunable");
    }

    if flags.is_empty() {
        "none".to_string()
    } else {
        flags.join(", ")
    }
}

fn build_tree_items(files: &[FileEntry], repo_path: Option<&str>) -> Vec<TreeItem> {
    let mut file_status: BTreeMap<String, PathStatus> = BTreeMap::new();
    let mut folder_status: BTreeMap<String, PathStatus> = BTreeMap::new();
    let mut file_delta = collect_file_deltas(files, repo_path);
    let mut folder_delta: BTreeMap<String, PathDelta> = BTreeMap::new();

    for file in files {
        let status = PathStatus {
            staged: file.staged,
            unstaged: file.unstaged,
            untracked: file.untracked,
        };
        file_status.insert(file.path.clone(), status);

        if file.untracked {
            let file_path = repo_path
                .map(|base| Path::new(base).join(file.path.as_str()))
                .unwrap_or_else(|| PathBuf::from(file.path.as_str()));
            let added = fs::read_to_string(file_path)
                .map(|text| text.lines().count())
                .unwrap_or(0);
            file_delta
                .entry(file.path.clone())
                .and_modify(|d| d.added_lines = d.added_lines.max(added))
                .or_insert(PathDelta {
                    added_lines: added,
                    removed_lines: 0,
                });
        }

        let mut parts: Vec<&str> = file.path.split('/').collect();
        let _ = parts.pop();
        for depth in 0..parts.len() {
            let folder_path = parts[..=depth].join("/");
            merge_status(&mut folder_status, folder_path, status);
        }
    }

    for (path, delta) in &file_delta {
        let mut parts: Vec<&str> = path.split('/').collect();
        let _ = parts.pop();
        for depth in 0..parts.len() {
            let folder_path = parts[..=depth].join("/");
            merge_delta(&mut folder_delta, folder_path, *delta);
        }
    }

    let mut root = TreeNode::default();
    for path in file_status.keys() {
        insert_path_into_tree(&mut root, path);
    }

    let mut items = Vec::new();
    flatten_tree(
        &root,
        "",
        0,
        &folder_status,
        &file_status,
        &folder_delta,
        &file_delta,
        &mut items,
    );
    items
}

fn collect_file_deltas(
    files: &[FileEntry],
    repo_path: Option<&str>,
) -> BTreeMap<String, PathDelta> {
    let mut deltas: BTreeMap<String, PathDelta> = BTreeMap::new();

    if let Some(numstat) = git_output_in(repo_path, &["diff", "--numstat", "HEAD"]) {
        for line in numstat.lines() {
            let mut parts = line.split('\t');
            let added_raw = parts.next().unwrap_or_default();
            let removed_raw = parts.next().unwrap_or_default();
            let path = parts.next().unwrap_or_default().trim();
            if path.is_empty() {
                continue;
            }

            let added = added_raw.parse::<usize>().unwrap_or(0);
            let removed = removed_raw.parse::<usize>().unwrap_or(0);
            deltas.insert(
                path.to_string(),
                PathDelta {
                    added_lines: added,
                    removed_lines: removed,
                },
            );
        }
    }

    for file in files {
        deltas.entry(file.path.clone()).or_default();
    }

    deltas
}

#[derive(Default)]
struct TreeNode {
    children: BTreeMap<String, TreeNode>,
    is_file: bool,
}

fn insert_path_into_tree(root: &mut TreeNode, path: &str) {
    let mut current = root;
    let parts: Vec<&str> = path.split('/').collect();

    for (idx, part) in parts.iter().enumerate() {
        let node = current.children.entry((*part).to_string()).or_default();
        if idx == parts.len() - 1 {
            node.is_file = true;
        }
        current = node;
    }
}

fn flatten_tree(
    node: &TreeNode,
    parent_path: &str,
    depth: usize,
    folder_status: &BTreeMap<String, PathStatus>,
    file_status: &BTreeMap<String, PathStatus>,
    folder_delta: &BTreeMap<String, PathDelta>,
    file_delta: &BTreeMap<String, PathDelta>,
    out: &mut Vec<TreeItem>,
) {
    let mut entries: Vec<(&String, &TreeNode)> = node.children.iter().collect();
    entries.sort_by(
        |(a_name, a_node), (b_name, b_node)| match (a_node.is_file, b_node.is_file) {
            (false, true) => std::cmp::Ordering::Less,
            (true, false) => std::cmp::Ordering::Greater,
            _ => a_name.cmp(b_name),
        },
    );

    for (name, child) in entries.into_iter() {
        let path = if parent_path.is_empty() {
            (*name).to_string()
        } else {
            format!("{}/{}", parent_path, name)
        };

        if child.is_file {
            let status = *file_status.get(&path).unwrap_or(&PathStatus::default());
            let delta = *file_delta.get(&path).unwrap_or(&PathDelta::default());
            out.push(TreeItem {
                path: path.clone(),
                label: format!("{}{}", "  ".repeat(depth), name),
                kind: TreeKind::File,
                staged: status.staged,
                unstaged: status.unstaged,
                untracked: status.untracked,
                added_lines: delta.added_lines,
                removed_lines: delta.removed_lines,
            });
        } else {
            let status = *folder_status.get(&path).unwrap_or(&PathStatus::default());
            let delta = *folder_delta.get(&path).unwrap_or(&PathDelta::default());
            out.push(TreeItem {
                path: path.clone(),
                label: format!("{}{}/", "  ".repeat(depth), name),
                kind: TreeKind::Folder,
                staged: status.staged,
                unstaged: status.unstaged,
                untracked: status.untracked,
                added_lines: delta.added_lines,
                removed_lines: delta.removed_lines,
            });
        }

        flatten_tree(
            child,
            &path,
            depth + 1,
            folder_status,
            file_status,
            folder_delta,
            file_delta,
            out,
        );
    }
}

fn merge_status(store: &mut BTreeMap<String, PathStatus>, key: String, status: PathStatus) {
    let entry = store.entry(key).or_default();
    entry.staged |= status.staged;
    entry.unstaged |= status.unstaged;
    entry.untracked |= status.untracked;
}

fn merge_delta(store: &mut BTreeMap<String, PathDelta>, key: String, delta: PathDelta) {
    let entry = store.entry(key).or_default();
    entry.added_lines += delta.added_lines;
    entry.removed_lines += delta.removed_lines;
}

fn max_overview_scroll(app: &App) -> u16 {
    let lines = overview_line_count(app);
    let visible = 22usize;
    lines.saturating_sub(visible) as u16
}

fn overview_line_count(app: &App) -> usize {
    let Some(info) = app.selected_overview.as_ref() else {
        return 1;
    };

    let mut count = 6usize;
    if info.use_traditional_overview {
        count += 2 + info.traditional_diff.len().min(24);
    } else {
        count += 2 + info.method_changes.len();
        if app.overview_method_expanded {
            if let Some(method) = info.method_changes.get(app.overview_method_index) {
                count += if method.diff_lines.is_empty() {
                    2
                } else {
                    method.diff_lines.len().min(40) + 1
                };
            }
        }
    }
    count
}

fn should_hide_internal_worktree_path(path: &str) -> bool {
    if path.starts_with(".gitfetch-worktrees/") {
        return path != ".gitfetch-worktrees/.parent-hints";
    }

    let repo_name = repo_root()
        .and_then(|root| {
            Path::new(root.as_str())
                .file_name()
                .and_then(|value| value.to_str())
                .map(|value| value.to_string())
        })
        .unwrap_or_else(|| "repo".to_string());
    let prefix = format!(".{}-workspaces/", repo_name);
    if !path.starts_with(prefix.as_str()) {
        return false;
    }

    path != format!("{}{}", prefix, ".parent-hints")
}

fn truncate_text(text: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }

    let total = text.chars().count();
    if total <= max_chars {
        return text.to_string();
    }

    if max_chars <= 3 {
        return ".".repeat(max_chars);
    }

    let mut out = String::new();
    for (idx, ch) in text.chars().enumerate() {
        if idx >= max_chars - 3 {
            out.push_str("...");
            return out;
        }
        out.push(ch);
    }
    out
}

fn single_line(text: &str) -> String {
    text.lines().next().unwrap_or_default().trim().to_string()
}

fn wrap_text_lines(text: &str, max_chars: usize, max_lines: usize) -> Vec<String> {
    if max_chars == 0 || max_lines == 0 {
        return vec![String::new()];
    }

    let mut out: Vec<String> = Vec::new();
    for raw_line in text.lines() {
        let words: Vec<&str> = raw_line.split_whitespace().collect();
        if words.is_empty() {
            out.push(String::new());
            if out.len() >= max_lines {
                return out;
            }
            continue;
        }

        let mut current = String::new();
        for word in words {
            let candidate = if current.is_empty() {
                word.to_string()
            } else {
                format!("{} {}", current, word)
            };

            if candidate.chars().count() <= max_chars {
                current = candidate;
                continue;
            }

            if !current.is_empty() {
                out.push(current);
                if out.len() >= max_lines {
                    return out;
                }
                current = String::new();
            }

            if word.chars().count() <= max_chars {
                current = word.to_string();
            } else {
                for segment in split_text_chunks(word, max_chars) {
                    out.push(segment);
                    if out.len() >= max_lines {
                        return out;
                    }
                }
            }
        }

        if !current.is_empty() {
            out.push(current);
            if out.len() >= max_lines {
                return out;
            }
        }
    }

    if out.is_empty() {
        out.push(String::new());
    }

    out
}

fn split_text_chunks(text: &str, chunk_size: usize) -> Vec<String> {
    if chunk_size == 0 {
        return vec![String::new()];
    }

    let mut out = Vec::new();
    let mut current = String::new();
    let mut count = 0usize;
    for ch in text.chars() {
        if count >= chunk_size {
            out.push(current);
            current = String::new();
            count = 0;
        }
        current.push(ch);
        count += 1;
    }
    if !current.is_empty() {
        out.push(current);
    }
    if out.is_empty() {
        out.push(String::new());
    }
    out
}

fn draw_commit_modal(frame: &mut ratatui::Frame<'_>, app: &App) {
    let popup = centered_rect(70, 22, frame.area());
    frame.render_widget(Clear, popup);

    let border = Block::default()
        .title("Create Commit")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black))
        .border_style(Style::default().fg(Color::Yellow));
    frame.render_widget(border, popup);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(popup);

    frame.render_widget(
        Paragraph::new("Write commit message and press Enter")
            .alignment(Alignment::Center)
            .style(
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        layout[0],
    );

    frame.render_widget(
        Paragraph::new(app.commit_input.as_str())
            .block(Block::default().title("Message").borders(Borders::ALL))
            .style(Style::default().fg(Color::Cyan)),
        layout[1],
    );

    frame.render_widget(
        Paragraph::new("Esc cancels")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray)),
        layout[2],
    );
}

fn draw_worktree_commit_push_modal(frame: &mut ratatui::Frame<'_>, app: &App) {
    let popup = centered_rect(74, 26, frame.area());
    frame.render_widget(Clear, popup);

    let border = Block::default()
        .title("Worktree Commit")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black))
        .border_style(Style::default().fg(Color::LightGreen));
    frame.render_widget(border, popup);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(popup);

    let path = app
        .worktree_commit_path
        .as_deref()
        .map(|p| truncate_text(p, 62))
        .unwrap_or_else(|| "(no selected worktree)".to_string());

    frame.render_widget(
        Paragraph::new(format!("Target: {}", path))
            .alignment(Alignment::Left)
            .style(Style::default().fg(Color::Gray)),
        layout[0],
    );

    frame.render_widget(
        Paragraph::new("Enter message, then Enter runs: git add . -> git commit -m")
            .alignment(Alignment::Left)
            .style(Style::default().fg(Color::White)),
        layout[1],
    );

    frame.render_widget(
        Paragraph::new(app.worktree_commit_input.as_str())
            .block(Block::default().title("Message").borders(Borders::ALL))
            .style(Style::default().fg(Color::Cyan)),
        layout[2],
    );

    frame.render_widget(
        Paragraph::new("Esc cancels")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray)),
        layout[3],
    );
}

fn draw_notes_modal(frame: &mut ratatui::Frame<'_>, app: &App) {
    let popup = centered_rect(86, 82, frame.area());
    frame.render_widget(Clear, popup);

    let mode = match app.notes_edit_mode {
        NotesEditMode::Normal => "NORMAL",
        NotesEditMode::Insert => "INSERT",
    };
    let title = match app.notes_context {
        NotesContext::Notes => format!("Notes (notes.md) [{}]", mode),
        NotesContext::ConflictPrompt => format!("Conflict Prompt (config) [{}]", mode),
    };

    let border = Block::default()
        .title(title.as_str())
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black))
        .border_style(Style::default().fg(Color::LightCyan));
    frame.render_widget(border, popup);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(6),
            Constraint::Length(1),
        ])
        .split(popup);

    let path = truncate_text(
        app.notes_path.as_str(),
        layout[0].width.saturating_sub(2) as usize,
    );
    frame.render_widget(
        Paragraph::new(format!("Path: {}", path)).style(Style::default().fg(Color::Gray)),
        layout[0],
    );

    let editor_area = layout[1];
    let editor_inner = Block::default()
        .title("Editor")
        .borders(Borders::ALL)
        .inner(editor_area);
    frame.render_widget(
        Paragraph::new("").block(Block::default().title("Editor").borders(Borders::ALL)),
        editor_area,
    );

    let visible_rows = editor_inner.height.max(1) as usize;
    let total_rows = app.notes_lines.len();
    let mut scroll = app.notes_scroll as usize;
    if app.notes_cursor_row < scroll {
        scroll = app.notes_cursor_row;
    }
    if app.notes_cursor_row >= scroll + visible_rows {
        scroll = app
            .notes_cursor_row
            .saturating_sub(visible_rows.saturating_sub(1));
    }
    let max_scroll = total_rows.saturating_sub(visible_rows);
    scroll = scroll.min(max_scroll);

    let lines = app
        .notes_lines
        .iter()
        .skip(scroll)
        .take(visible_rows)
        .map(|line| Line::from(line.clone()))
        .collect::<Vec<Line<'_>>>();

    frame.render_widget(
        Paragraph::new(lines)
            .style(Style::default().fg(Color::White))
            .scroll((0, 0)),
        editor_inner,
    );

    if app.notes_cursor_row >= scroll {
        let cursor_y = editor_inner
            .y
            .saturating_add((app.notes_cursor_row - scroll) as u16);
        let cursor_x = editor_inner.x.saturating_add(app.notes_cursor_col as u16);
        let max_x = editor_inner
            .x
            .saturating_add(editor_inner.width.saturating_sub(1));
        let max_y = editor_inner
            .y
            .saturating_add(editor_inner.height.saturating_sub(1));
        frame.set_cursor_position((cursor_x.min(max_x), cursor_y.min(max_y)));
    }

    frame.render_widget(
        Paragraph::new(match app.notes_context {
            NotesContext::Notes => match app.notes_edit_mode {
                NotesEditMode::Normal => {
                    "NORMAL: i/a/o/O insert, hjkl move, dd delete line, q save+close, Ctrl+S save"
                }
                NotesEditMode::Insert => {
                    "INSERT: type/edit text. Esc returns to NORMAL, Ctrl+S saves"
                }
            },
            NotesContext::ConflictPrompt => match app.notes_edit_mode {
                NotesEditMode::Normal => {
                    "NORMAL: i/a/o/O insert, hjkl move, dd delete line, q save+back, Ctrl+S save"
                }
                NotesEditMode::Insert => {
                    "INSERT: edit template text. Esc returns to NORMAL, Ctrl+S saves"
                }
            },
        })
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray)),
        layout[2],
    );
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}

fn terminal_popup_rect(area: Rect) -> Rect {
    let vertical_margin = 1;
    let available_height = area.height.saturating_sub(vertical_margin * 2);

    let width = area.width.max(1);
    let height = available_height.max(1);

    let x = area.x;
    let y = area.y.saturating_add(vertical_margin);

    Rect::new(x, y, width, height)
}
