fn draw_ui(frame: &mut ratatui::Frame<'_>, app: &App) {
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
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(columns[1]);

        draw_worktree_canvas_panel(frame, app, columns[0]);
        draw_worktree_details_panel(frame, app, right[0]);
        draw_worktree_actions_panel(frame, app, right[1]);
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

    if matches!(app.mode, Mode::WorktreeBranchConflictConfirm) {
        draw_branch_conflict_confirm_modal(frame, app);
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
        .filter(|(_, item)| item.staged)
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
                .bg(Color::Rgb(42, 58, 86))
                .add_modifier(Modifier::BOLD),
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
            push_method_section(
                &mut lines,
                "methods added",
                Color::LightGreen,
                &info.methods_added,
            );
            push_method_section(
                &mut lines,
                "methods modified",
                Color::Yellow,
                &info.methods_modified,
            );
            push_method_section(
                &mut lines,
                "methods deleted",
                Color::LightRed,
                &info.methods_deleted,
            );
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

fn push_method_section(lines: &mut Vec<Line<'_>>, title: &str, color: Color, names: &[String]) {
    lines.push(Line::from(vec![Span::styled(
        title.to_string(),
        Style::default().fg(color),
    )]));
    if names.is_empty() {
        lines.push(Line::from("- none"));
    } else {
        for name in names.iter().take(8) {
            lines.push(Line::from(vec![
                Span::raw("- "),
                Span::styled(truncate_text(name, 56), Style::default().fg(Color::White)),
            ]));
        }
    }
    lines.push(Line::from(""));
}

fn draw_pulse_panel(frame: &mut ratatui::Frame<'_>, app: &App, area: Rect) {
    let staged_count = app.files.iter().filter(|f| f.staged).count();
    let unstaged_count = app
        .files
        .iter()
        .filter(|f| f.unstaged || f.untracked)
        .count();

    let status_limit = area.width.saturating_sub(12) as usize;
    let status_text = truncate_text(
        single_line(app.status_line.as_str()).as_str(),
        status_limit.max(10),
    );

    let now = Instant::now();
    let mut active_sessions = 0usize;
    let mut idle_sessions = 0usize;
    let mut live_summary: Vec<(bool, u64, u64, String)> = app
        .agent_sessions
        .iter()
        .filter_map(|(path, session)| {
            if !agent_session_is_live(session) {
                return None;
            }
            let is_active = agent_session_is_active(session, now);
            if is_active {
                active_sessions += 1;
            } else {
                idle_sessions += 1;
            }
            Some((
                is_active,
                agent_session_avg_bps(session, now),
                agent_session_idle_seconds(session, now),
                session_label_from_path(path),
            ))
        })
        .collect();
    live_summary.sort_by(|a, b| b.cmp(a));

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
            Span::styled("PTY sessions: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{} live", active_sessions + idle_sessions),
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
        ]),
        Line::from({
            if live_summary.is_empty() {
                vec![
                    Span::styled("Live PTY: ", Style::default().fg(Color::Gray)),
                    Span::raw("none"),
                ]
            } else {
                let mut spans = vec![Span::styled("Live PTY: ", Style::default().fg(Color::Gray))];
                for (idx, (is_active, bps, idle_secs, label)) in
                    live_summary.iter().take(2).enumerate()
                {
                    if idx > 0 {
                        spans.push(Span::raw("  "));
                    }
                    let mode = if *is_active { "A" } else { "I" };
                    let color = if *is_active {
                        Color::LightGreen
                    } else {
                        Color::Yellow
                    };
                    spans.push(Span::styled(
                        format!(
                            "{} {} {}B/s {}s",
                            truncate_text(label, 12),
                            mode,
                            bps,
                            idle_secs
                        ),
                        Style::default().fg(color),
                    ));
                }
                spans
            }
        }),
        Line::from(""),
        Line::from(Span::styled(
            "Live refresh every ~700ms",
            Style::default().fg(Color::Blue),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Status: ", Style::default().fg(Color::Gray)),
            Span::styled(status_text, Style::default().fg(Color::White)),
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
            Span::raw(" move selection/scroll"),
        ]),
        Line::from(vec![
            Span::styled("space|enter", Style::default().fg(Color::LightGreen)),
            Span::raw(" stage or unstage"),
        ]),
        Line::from(vec![
            Span::styled("c", Style::default().fg(Color::Yellow)),
            Span::raw(" commit"),
        ]),
        Line::from(vec![
            Span::styled("n", Style::default().fg(Color::LightCyan)),
            Span::raw(" notes popup"),
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

fn draw_worktree_canvas_panel(frame: &mut ratatui::Frame<'_>, app: &App, area: Rect) {
    let border_color = if app.worktree_focus == WorktreePane::Canvas {
        Color::Cyan
    } else {
        Color::Gray
    };
    let title = if app.worktree_canvas_zoom != 1.0
        || app.worktree_canvas_pan_x != 0.0
        || app.worktree_canvas_pan_y != 0.0
    {
        format!("worktree graph  z:{:.1}x", app.worktree_canvas_zoom)
    } else {
        "worktree graph".to_string()
    };
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(Color::Black));

    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.width < 10 || inner.height < 6 {
        return;
    }

    let root_branch = current_session_branch(app);

    if app.worktrees.is_empty() {
        frame.render_widget(
            Paragraph::new("No worktrees. Press 'a' to create one.")
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::DarkGray)),
            inner,
        );
        return;
    }

    let parents = worktree_parent_map(&app.worktrees, root_branch.as_str());
    let collapsed_root_idx = None;
    let logical = graph_layout(&parents);
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
    frame.render_widget(Clear, inner);
    let mut screen_points: Vec<(u16, u16)> = Vec::with_capacity(node_points.len());
    for point in &node_points {
        if let Some((sx, sy)) = canvas_point_to_screen(inner, bounds, *point) {
            screen_points.push((sx, sy));
        } else {
            screen_points.push((inner.x, inner.y));
        }
    }

    let buf = frame.buffer_mut();
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

    for (idx, entry) in app.worktrees.iter().enumerate() {
        if Some(idx) == collapsed_root_idx {
            continue;
        }
        let selected = idx == selected_idx;
        let label = canvas_node_label(app, entry, selected);
        let style = if selected {
            Style::default()
                .fg(Color::Black)
                .bg(Color::LightCyan)
                .add_modifier(Modifier::BOLD)
        } else if entry.is_current {
            Style::default()
                .fg(Color::Black)
                .bg(Color::LightMagenta)
                .add_modifier(Modifier::BOLD)
        } else if entry.dirty {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };
        draw_canvas_label(
            frame,
            inner,
            bounds,
            node_points[idx],
            label.as_str(),
            style,
        );
    }
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
            '●',
            Style::default().fg(Color::LightMagenta),
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
        } else if entry.is_current {
            Color::LightMagenta
        } else if entry.dirty {
            Color::Yellow
        } else {
            graph_palette_color(idx)
        };
        let glyph = if selected {
            '◉'
        } else if entry.is_current {
            '●'
        } else {
            '○'
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
        DR => '╭',
        DL => '╮',
        UL => '╯',
        UR => '╰',
        UDR => '├',
        UDL => '┤',
        LRD => '┬',
        LRU => '┴',
        ALL => '┼',
        DIR_UP => '│',
        DIR_DOWN => '│',
        DIR_LEFT => '─',
        DIR_RIGHT => '─',
        _ => '·',
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
    style: Style,
) {
    let Some((sx, sy)) = canvas_point_to_screen(area, bounds, point) else {
        return;
    };
    let label_width = label.chars().count() as u16;
    let horizontal_padding = 1u16;
    let box_width = label_width
        .saturating_add(horizontal_padding.saturating_mul(2))
        .saturating_add(2);
    let box_height = 3u16;
    if label_width == 0 || box_width >= area.width || box_height > area.height {
        return;
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
            .border_style(style)
            .style(style),
        rect,
    );
    frame.render_widget(
        Paragraph::new(label.to_string()).style(style),
        Rect::new(
            x.saturating_add(1 + horizontal_padding),
            y.saturating_add(1),
            label_width,
            1,
        ),
    );
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

fn animated_agent_spinner(session: &AgentSession, now: Instant) -> char {
    const FRAMES: [char; 10] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
    let tick = now
        .saturating_duration_since(session.launched_at)
        .as_millis()
        / 140;
    FRAMES[(tick % FRAMES.len() as u128) as usize]
}

fn graph_layout(parents: &[Option<usize>]) -> Vec<(f32, f32)> {
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
            Span::styled("flags:  ", Style::default().fg(Color::Gray)),
            Span::styled(
                worktree_flags(selected),
                Style::default().fg(Color::LightMagenta),
            ),
        ]));
        lines.push(Line::from(""));
        let status_max = area.width.saturating_sub(4) as usize;
        let status_text = sanitize_for_tui(app.status_line.as_str());
        let inner_height = area.height.saturating_sub(2) as usize;
        let status_max_lines = inner_height.saturating_sub(lines.len() + 1).max(1);
        lines.push(Line::from(vec![Span::styled(
            "status:",
            Style::default().fg(Color::Gray),
        )]));
        for wrapped in wrap_text_lines(status_text.as_str(), status_max.max(12), status_max_lines) {
            lines.push(Line::from(vec![Span::styled(
                wrapped,
                Style::default().fg(Color::White),
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

fn draw_worktree_actions_panel(frame: &mut ratatui::Frame<'_>, app: &App, area: Rect) {
    let border_color = if app.worktree_focus == WorktreePane::Actions {
        Color::Cyan
    } else {
        Color::Gray
    };

    let lines = vec![
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
            Span::styled("o", Style::default().fg(Color::LightBlue)),
            Span::raw(" open terminal popup"),
        ]),
        Line::from(vec![
            Span::styled("O", Style::default().fg(Color::LightBlue)),
            Span::raw(" open agent picker"),
        ]),
        Line::from(vec![
            Span::styled("f", Style::default().fg(Color::Cyan)),
            Span::raw(" fetch parent"),
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
            Span::raw(" notes popup"),
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
    ];

    frame.render_widget(
        Paragraph::new(lines)
            .block(
                Block::default()
                    .title("actions [?]")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color))
                    .style(Style::default().bg(Color::Black)),
            )
            .style(Style::default().fg(Color::White)),
        area,
    );
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
            Line::from("- Magenta node = current branch worktree"),
            Line::from("- Cyan ring = selected worktree"),
            Line::from("- Yellow nodes = dirty (uncommitted changes)"),
            Line::from("- Node suffix animates for active: | / - \\"),
            Line::from("- Active state uses a braille spinner"),
            Line::from("- Idle sessions show idle(12s), completed show done/fail"),
            Line::from("- Node labels are boxed with ratatui borders"),
            Line::from("- Lines show parent branch relationships"),
            Line::from(""),
            Line::from("Navigation:"),
            Line::from("  arrows  - move by graph direction"),
            Line::from("  h/l     - left/right among siblings"),
            Line::from("  j/k     - down/up by graph level"),
            Line::from("  L       - open git command history popup"),
            Line::from(""),
            Line::from("Camera:"),
            Line::from("  +/-     - zoom in/out"),
            Line::from("  0       - reset view"),
            Line::from("  Shift+WASD - pan"),
            Line::from(""),
            Line::from("  ?: close this help"),
        ],
        WorktreePane::Details => vec![
            Line::from("Details panel"),
            Line::from("- Shows branch/path/head and repo flags"),
            Line::from("- Shows ahead/behind and dirty/locked state"),
            Line::from("- Reflects current canvas selection"),
            Line::from("- tab: move focus to next panel"),
            Line::from("- ?: close this help"),
        ],
        WorktreePane::Actions => vec![
            Line::from("Actions panel"),
            Line::from("- a: create worktree from branch name"),
            Line::from("- L: open git command history (reflog) popup"),
            Line::from("- o: open/reopen normal terminal popup for selected node"),
            Line::from("- O: open agent picker for selected or conflicted parent"),
            Line::from("- default agent and prompts are editable in ~/.config/openswarm"),
            Line::from("- terminal popup: : enters CONTROL, Ctrl+G toggles INPUT/CONTROL"),
            Line::from("- f: fetch connected parent node"),
            Line::from("- c: selected worktree add+commit with message popup"),
            Line::from("- p: push selected worktree branch"),
            Line::from("- n: open notes popup (notes.md)"),
            Line::from("- d: delete selected worktree (asks force-delete if dirty)"),
            Line::from("- m: merge selected branch into connected parent node"),
            Line::from("- x: prune stale worktrees"),
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
            "INPUT mode: typing goes to terminal. : enters CONTROL (Ctrl+G also works)."
        }
        TerminalPopupMode::Control => {
            "CONTROL mode: Esc background, q quit session, r restart, i return INPUT."
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

fn build_tree_items(files: &[FileEntry]) -> Vec<TreeItem> {
    let mut file_status: BTreeMap<String, PathStatus> = BTreeMap::new();
    let mut folder_status: BTreeMap<String, PathStatus> = BTreeMap::new();
    let mut file_delta = collect_file_deltas(files);
    let mut folder_delta: BTreeMap<String, PathDelta> = BTreeMap::new();

    for file in files {
        let status = PathStatus {
            staged: file.staged,
            unstaged: file.unstaged,
            untracked: file.untracked,
        };
        file_status.insert(file.path.clone(), status);

        if file.untracked {
            let added = fs::read_to_string(&file.path)
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

fn collect_file_deltas(files: &[FileEntry]) -> BTreeMap<String, PathDelta> {
    let mut deltas: BTreeMap<String, PathDelta> = BTreeMap::new();

    if let Some(numstat) = git_output(&["diff", "--numstat", "HEAD"]) {
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
    let lines = overview_line_count(app.selected_overview.as_ref());
    let visible = 22usize;
    lines.saturating_sub(visible) as u16
}

fn overview_line_count(info: Option<&FileOverview>) -> usize {
    let Some(info) = info else {
        return 1;
    };

    let mut count = 6usize;
    if info.use_traditional_overview {
        count += 2 + info.traditional_diff.len().min(24);
    } else {
        count += 1 + info.methods_added.len().min(8);
        count += 1 + info.methods_modified.len().min(8);
        count += 1 + info.methods_deleted.len().min(8);
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

    let border = Block::default()
        .title("Notes (notes.md)")
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
        Paragraph::new("Type to edit. Enter newline, arrows move, Ctrl+S saves, Esc saves+closes")
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
