fn refresh_status(app: &mut App) {
    let output = match run_git_in(
        app.changes_worktree_path.as_deref(),
        &["status", "--porcelain=1", "-b", "-uall"],
    ) {
        Ok(text) => text,
        Err(err) => {
            app.status_line = err.to_string();
            return;
        }
    };

    let mut lines = output.lines();
    if let Some(head) = lines.next() {
        parse_branch_line(app, head);
    }

    let mut files = Vec::new();
    for line in lines {
        if line.len() < 4 {
            continue;
        }

        let x = line.chars().next().unwrap_or(' ');
        let y = line.chars().nth(1).unwrap_or(' ');
        let path = line[3..].trim().to_string();

        if should_hide_internal_worktree_path(path.as_str()) {
            continue;
        }

        files.push(FileEntry {
            path,
            staged: x != ' ' && x != '?',
            unstaged: y != ' ',
            untracked: x == '?' && y == '?',
        });
    }

    app.files = files;
    app.tree_items = build_tree_items(&app.files, app.changes_worktree_path.as_deref());

    if app.tree_items.is_empty() {
        app.selected = 0;
    } else if app.selected >= app.tree_items.len() {
        app.selected = app.tree_items.len() - 1;
    }

    let max_scroll = max_overview_scroll(app);
    if app.overview_scroll > max_scroll {
        app.overview_scroll = max_scroll;
    }

    refresh_selected_overview(app);
    refresh_worktrees(app);
}

fn refresh_worktrees(app: &mut App) {
    let output = match git_output(&["worktree", "list", "--porcelain"]) {
        Some(text) => text,
        None => {
            app.worktrees.clear();
            app.selected_worktree = 0;
            return;
        }
    };

    let current_path = std::env::current_dir()
        .ok()
        .map(|path| normalize_path(path.to_string_lossy().as_ref()));
    let root = create_root_for_app(app);
    let parent_hints = load_parent_hint_map(root.as_str());

    let mut entries: Vec<WorktreeEntry> = Vec::new();
    let mut current = WorktreeEntry::default();
    let mut in_block = false;

    for line in output.lines() {
        if line.trim().is_empty() {
            if in_block {
                hydrate_worktree_runtime_state(
                    &mut current,
                    current_path.as_deref(),
                    &parent_hints,
                );
                entries.push(current.clone());
                current = WorktreeEntry::default();
                in_block = false;
            }
            continue;
        }

        if let Some(path) = line.strip_prefix("worktree ") {
            if in_block {
                hydrate_worktree_runtime_state(
                    &mut current,
                    current_path.as_deref(),
                    &parent_hints,
                );
                entries.push(current.clone());
                current = WorktreeEntry::default();
            }
            current.path = path.trim().to_string();
            in_block = true;
            continue;
        }

        if let Some(head) = line.strip_prefix("HEAD ") {
            current.head = head.trim().to_string();
            continue;
        }

        if let Some(branch) = line.strip_prefix("branch ") {
            current.branch = branch
                .trim()
                .strip_prefix("refs/heads/")
                .unwrap_or(branch.trim())
                .to_string();
            continue;
        }

        if line == "detached" {
            current.detached = true;
            if current.branch.is_empty() {
                current.branch = "detached".to_string();
            }
            continue;
        }

        if line == "bare" {
            current.bare = true;
            continue;
        }

        if line.starts_with("locked") {
            current.locked = true;
            continue;
        }

        if line.starts_with("prunable") {
            current.prunable = true;
            continue;
        }
    }

    if in_block {
        hydrate_worktree_runtime_state(&mut current, current_path.as_deref(), &parent_hints);
        entries.push(current);
    }

    entries.sort_by(|a, b| {
        b.is_current
            .cmp(&a.is_current)
            .then_with(|| a.branch.cmp(&b.branch))
            .then_with(|| a.path.cmp(&b.path))
    });

    let new_paths: BTreeSet<String> = entries.iter().map(|entry| entry.path.clone()).collect();
    app.sync_worktree_animations(&new_paths);

    app.worktrees = entries;
    if let Some(target_path) = app.changes_worktree_path.as_deref() {
        let exists = app.worktrees.iter().any(|entry| entry.path == target_path);
        if !exists {
            app.changes_worktree_path = None;
        }
    }
    if app.worktrees.is_empty() {
        app.selected_worktree = 0;
    } else if app.selected_worktree >= app.worktrees.len() {
        app.selected_worktree = app.worktrees.len() - 1;
    }

    maybe_prompt_legacy_workspace_migration(app, root.as_str());
}

fn maybe_prompt_legacy_workspace_migration(app: &mut App, root: &str) {
    if app.legacy_workspace_prompt_dismissed {
        return;
    }

    if !matches!(app.mode, Mode::Normal) {
        return;
    }

    if app.view_mode != ViewMode::Worktrees {
        return;
    }

    let legacy = Path::new(root).join(".gitfetch-worktrees");
    if !legacy.exists() {
        return;
    }

    let new_container = workspaces_container_for_root(root);
    let has_legacy_entries = fs::read_dir(legacy.as_path())
        .ok()
        .and_then(|mut iter| iter.next().map(|entry| entry.ok()))
        .flatten()
        .is_some();
    if !has_legacy_entries {
        return;
    }

    app.pending_legacy_workspace_root = root.to_string();
    app.pending_legacy_workspace_path = legacy.to_string_lossy().to_string();
    app.pending_new_workspace_path = new_container.to_string_lossy().to_string();
    app.confirm_legacy_workspace_migrate_yes = true;
    app.mode = Mode::LegacyWorkspaceMigrateConfirm;
    app.status_line = "Detected legacy in-repo worktree folder".to_string();
}

fn migrate_legacy_workspace_layout(root: &str) -> Result<String, Box<dyn Error>> {
    let legacy = Path::new(root).join(".gitfetch-worktrees");
    if !legacy.exists() {
        return Ok("No legacy .gitfetch-worktrees folder found".to_string());
    }

    let new_container = workspaces_container_for_root(root);
    fs::create_dir_all(new_container.as_path())?;

    let list = Command::new("git")
        .args(["-C", root, "worktree", "list", "--porcelain"])
        .output()?;
    if !list.status.success() {
        let stderr = sanitize_for_tui(String::from_utf8_lossy(&list.stderr).as_ref())
            .trim()
            .to_string();
        let stdout = sanitize_for_tui(String::from_utf8_lossy(&list.stdout).as_ref())
            .trim()
            .to_string();
        let reason = if !stderr.is_empty() { stderr } else { stdout };
        return Ok(format!("Failed reading worktree list: {}", reason));
    }

    let mut moved = 0usize;
    let mut failed: Vec<String> = Vec::new();

    let listing = sanitize_for_tui(String::from_utf8_lossy(&list.stdout).as_ref());
    for line in listing.lines() {
        let Some(raw_path) = line.strip_prefix("worktree ") else {
            continue;
        };
        let old_path = PathBuf::from(raw_path.trim());
        if !old_path.starts_with(legacy.as_path()) {
            continue;
        }

        let rel = old_path
            .strip_prefix(legacy.as_path())
            .unwrap_or_else(|_| Path::new(""));
        let target_path = new_container.join(rel);
        if let Some(parent) = target_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let move_out = Command::new("git")
            .args([
                "-C",
                root,
                "worktree",
                "move",
                old_path.to_string_lossy().as_ref(),
                target_path.to_string_lossy().as_ref(),
            ])
            .output()?;
        if move_out.status.success() {
            moved += 1;
        } else {
            let stderr = sanitize_for_tui(String::from_utf8_lossy(&move_out.stderr).as_ref())
                .trim()
                .to_string();
            let stdout = sanitize_for_tui(String::from_utf8_lossy(&move_out.stdout).as_ref())
                .trim()
                .to_string();
            let reason = if !stderr.is_empty() { stderr } else { stdout };
            failed.push(format!("{} ({})", old_path.to_string_lossy(), reason));
        }
    }

    let legacy_hints = legacy.join(".parent-hints");
    let new_hints = new_container.join(".parent-hints");
    let mut hints_moved = false;
    if legacy_hints.exists() && !new_hints.exists() {
        if fs::rename(legacy_hints.as_path(), new_hints.as_path()).is_ok() {
            hints_moved = true;
        }
    }

    let old_empty = fs::read_dir(legacy.as_path())
        .ok()
        .map(|mut iter| iter.next().is_none())
        .unwrap_or(false);
    if old_empty {
        let _ = fs::remove_dir(legacy.as_path());
    }

    if failed.is_empty() {
        Ok(format!(
            "Migrated {} worktree(s) to {}{}",
            moved,
            new_container.to_string_lossy(),
            if hints_moved {
                " + moved .parent-hints"
            } else {
                ""
            }
        ))
    } else {
        Ok(format!(
            "Migrated {} worktree(s), {} failed: {}",
            moved,
            failed.len(),
            truncate_text(failed.join("; ").as_str(), 180)
        ))
    }
}

fn hydrate_worktree_runtime_state(
    entry: &mut WorktreeEntry,
    current_path: Option<&str>,
    parent_hints: &BTreeMap<String, String>,
) {
    let normalized = normalize_path(entry.path.as_str());
    entry.is_current = current_path
        .map(|cwd| cwd == normalized.as_str())
        .unwrap_or(false);

    if entry.branch.is_empty() {
        entry.branch = "detached".to_string();
    }

    let (dirty, ahead, behind) = worktree_branch_state(entry.path.as_str());
    entry.dirty = dirty;
    entry.ahead = ahead;
    entry.behind = behind;
    entry.parent_hint = parent_hints.get(entry.branch.as_str()).cloned();
}

fn worktree_branch_state(path: &str) -> (bool, usize, usize) {
    let output = match Command::new("git")
        .args(["-C", path, "status", "--porcelain=1", "-b", "-uall"])
        .output()
    {
        Ok(out) if out.status.success() => {
            sanitize_for_tui(String::from_utf8_lossy(&out.stdout).as_ref())
        }
        _ => return (false, 0, 0),
    };

    let mut lines = output.lines();
    let mut ahead = 0usize;
    let mut behind = 0usize;
    if let Some(head) = lines.next() {
        let (_, parsed_ahead, parsed_behind) = parse_branch_snapshot(head);
        ahead = parsed_ahead;
        behind = parsed_behind;
    }
    let dirty = lines.any(|line| status_line_counts_as_dirty(line));
    (dirty, ahead, behind)
}

fn status_line_counts_as_dirty(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return false;
    }

    if let Some(path) = trimmed.strip_prefix("?? ") {
        return !should_hide_internal_worktree_path(path.trim());
    }

    if let Some(path) = trimmed.strip_prefix("!! ") {
        return !should_hide_internal_worktree_path(path.trim());
    }

    true
}

fn parse_branch_snapshot(line: &str) -> (String, usize, usize) {
    let mut ahead = 0usize;
    let mut behind = 0usize;

    let stripped = line.strip_prefix("## ").unwrap_or(line);
    let branch = if let Some((name, rest)) = stripped.split_once("...") {
        if let Some(start) = rest.find('[') {
            if let Some(end) = rest[start + 1..].find(']') {
                let info = &rest[start + 1..start + 1 + end];
                for token in info.split(',').map(|part| part.trim()) {
                    if let Some(v) = token.strip_prefix("ahead ") {
                        ahead = v.parse::<usize>().unwrap_or(0);
                    }
                    if let Some(v) = token.strip_prefix("behind ") {
                        behind = v.parse::<usize>().unwrap_or(0);
                    }
                }
            }
        }
        name.trim().to_string()
    } else {
        stripped.trim().to_string()
    };

    (branch, ahead, behind)
}

fn normalize_path(path: &str) -> String {
    fs::canonicalize(Path::new(path))
        .unwrap_or_else(|_| Path::new(path).to_path_buf())
        .to_string_lossy()
        .to_string()
}

fn remove_selected_worktree(app: &mut App) -> Result<String, Box<dyn Error>> {
    let Some(selected) = app.selected_worktree() else {
        return Ok("No worktree selected".to_string());
    };
    let selected_path = selected.path.clone();
    remove_worktree_by_path(app, selected_path.as_str(), false)
}

fn remove_worktree_by_path(
    app: &mut App,
    worktree_path: &str,
    force: bool,
) -> Result<String, Box<dyn Error>> {
    let Some(selected) = app
        .worktrees
        .iter()
        .find(|worktree| worktree.path == worktree_path)
        .cloned()
    else {
        return Ok(format!("Worktree not found: {}", worktree_path));
    };

    let selected_path = selected.path;

    if selected.is_current {
        return Ok("Refusing to remove current worktree".to_string());
    }

    if selected.dirty && !force {
        return Ok("Refusing to remove dirty worktree (clean it first)".to_string());
    }

    let had_live_session = has_live_terminal_session(app, selected_path.as_str());
    terminate_terminal_session(app, selected_path.as_str());

    if app.agent_popup_path.as_deref() == Some(selected_path.as_str()) {
        app.agent_popup_path = None;
    }

    let remove_output = if force {
        run_git(&["worktree", "remove", "--force", selected_path.as_str()])?
    } else {
        run_git(&["worktree", "remove", selected_path.as_str()])?
    };
    if had_live_session {
        Ok(format!(
            "{} (closed terminal session for worktree)",
            remove_output
        ))
    } else {
        Ok(remove_output)
    }
}

fn merge_selected_into_parent(app: &mut App) -> Result<String, Box<dyn Error>> {
    if app.selected_worktree >= app.worktrees.len() {
        return Ok("No worktree selected".to_string());
    }

    app.pending_conflict_context = None;

    let selected = app.worktrees[app.selected_worktree].clone();
    if selected.detached || selected.branch.is_empty() {
        return Ok("Selected worktree is detached; merge requires a branch".to_string());
    }

    let Some(parent_idx) = connected_parent_index(app) else {
        return Ok("No connected parent node found for selected worktree".to_string());
    };
    let parent = app.worktrees[parent_idx].clone();

    if parent.detached || parent.branch.is_empty() {
        return Ok("Parent node is detached; cannot merge into detached HEAD".to_string());
    }
    if parent.branch == selected.branch {
        return Ok("Selected and parent are the same branch; nothing to merge".to_string());
    }

    let before_head = git_output(&["-C", parent.path.as_str(), "rev-parse", "HEAD"])
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    // Use an explicit commit/ref target to avoid ambiguous names like "stash"
    // resolving to refs/stash instead of refs/heads/stash.
    let merge_target = if !selected.head.is_empty() {
        selected.head.clone()
    } else {
        format!("refs/heads/{}", selected.branch)
    };

    let merge = Command::new("git")
        .args([
            "-C",
            parent.path.as_str(),
            "merge",
            "--no-edit",
            merge_target.as_str(),
        ])
        .output()?;

    let stdout = sanitize_for_tui(String::from_utf8_lossy(&merge.stdout).as_ref())
        .trim()
        .to_string();
    let stderr = sanitize_for_tui(String::from_utf8_lossy(&merge.stderr).as_ref())
        .trim()
        .to_string();

    if !merge.status.success() {
        let conflicts = conflicted_files_in_worktree(parent.path.as_str());
        if !conflicts.is_empty() {
            refresh_runtime_settings(app);
            app.pending_conflict_context = Some(ConflictResolveContext {
                parent_path: parent.path.clone(),
                source_branch: selected.branch.clone(),
                target_branch: parent.branch.clone(),
                conflicted_files: conflicts.clone(),
            });
            app.confirm_conflict_resolve_yes = false;
            app.mode = Mode::WorktreeConflictResolveConfirm;
            return Ok(format!(
                "Merge '{}' -> '{}' has conflicts in {} file(s): {}. Resolve with agent now?",
                selected.branch,
                parent.branch,
                conflicts.len(),
                truncate_text(conflicts.join(", ").as_str(), 120)
            ));
        }

        let reason = if !stderr.is_empty() {
            stderr
        } else if !stdout.is_empty() {
            stdout
        } else {
            "merge failed".to_string()
        };
        return Ok(format!(
            "Merge '{}' -> '{}' failed:\n{}",
            selected.branch,
            parent.branch,
            sanitize_for_tui(reason.as_str())
        ));
    }

    let after_head = git_output(&["-C", parent.path.as_str(), "rev-parse", "HEAD"])
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    let details = if !stdout.is_empty() {
        single_line(stdout.as_str())
    } else if !stderr.is_empty() {
        single_line(stderr.as_str())
    } else {
        "ok".to_string()
    };

    if !before_head.is_empty() && before_head == after_head {
        Ok(format!(
            "No new merge for '{}' -> '{}' ({}) - {}",
            selected.branch, parent.branch, parent.path, details
        ))
    } else {
        Ok(format!(
            "Merged '{}' into '{}' ({}) [{} -> {}] - {}",
            selected.branch,
            parent.branch,
            parent.path,
            truncate_text(before_head.as_str(), 8),
            truncate_text(after_head.as_str(), 8),
            details
        ))
    }
}

fn conflicted_files_in_worktree(path: &str) -> Vec<String> {
    let output = match Command::new("git")
        .args(["-C", path, "diff", "--name-only", "--diff-filter=U"])
        .output()
    {
        Ok(out) if out.status.success() => out,
        _ => return Vec::new(),
    };

    sanitize_for_tui(String::from_utf8_lossy(&output.stdout).as_ref())
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| line.to_string())
        .collect()
}

fn update_connected_parent(app: &App) -> Result<String, Box<dyn Error>> {
    let Some(parent_idx) = connected_parent_index(app) else {
        return Ok("No connected parent node found for selected worktree".to_string());
    };
    let parent = app.worktrees[parent_idx].clone();

    if parent.detached || parent.branch.is_empty() {
        return Ok("Parent node is detached; cannot fetch updates".to_string());
    }

    let fetch = run_git(&["-C", parent.path.as_str(), "fetch", "--all", "--prune"])?;
    let pull = run_git(&["-C", parent.path.as_str(), "pull"])?;

    Ok(format!(
        "Fetched + pulled parent '{}' - fetch: {}; pull: {}",
        parent.branch,
        single_line(fetch.as_str()),
        single_line(pull.as_str()),
    ))
}

fn connected_parent_index(app: &App) -> Option<usize> {
    if app.selected_worktree >= app.worktrees.len() {
        return None;
    }

    let root_branch = current_session_branch(app);
    let parents = worktree_parent_map(&app.worktrees, root_branch.as_str());
    if let Some(parent_idx) = parents.get(app.selected_worktree).and_then(|v| *v) {
        return Some(parent_idx);
    }

    app.worktrees.iter().enumerate().find_map(|(idx, wt)| {
        if idx == app.selected_worktree {
            return None;
        }
        if !wt.detached && wt.branch == root_branch {
            Some(idx)
        } else {
            None
        }
    })
}

#[derive(Clone, Copy)]
enum NavDirection {
    Left,
    Right,
    Up,
    Down,
}

fn move_worktree_selection(app: &mut App, direction: NavDirection) {
    if app.worktrees.len() < 2 || app.selected_worktree >= app.worktrees.len() {
        return;
    }

    let root_branch = current_session_branch(app);
    let parents = worktree_parent_map(&app.worktrees, root_branch.as_str());
    let depths = graph_depths(&parents);
    let points = graph_layout(&parents);
    let (cx, cy) = points[app.selected_worktree];
    let mut best_idx: Option<usize> = None;
    let mut best_score = f32::MAX;

    for (idx, (x, y)) in points.iter().enumerate() {
        if idx == app.selected_worktree {
            continue;
        }

        let dx = *x - cx;
        let dy = *y - cy;
        let in_front = match direction {
            NavDirection::Left => dx < -0.15,
            NavDirection::Right => dx > 0.15,
            NavDirection::Up => dy < -0.15,
            NavDirection::Down => dy > 0.15,
        };
        if !in_front {
            continue;
        }

        let directional_penalty = match direction {
            NavDirection::Left | NavDirection::Right => dy.abs() * 1.7,
            NavDirection::Up | NavDirection::Down => dx.abs() * 1.7,
        };
        let score = dx.abs() + dy.abs() + directional_penalty;
        if score < best_score {
            best_score = score;
            best_idx = Some(idx);
        }
    }

    if best_idx.is_none() {
        let current = app.selected_worktree;
        let current_depth = depths[current];
        let max_depth = depths.iter().copied().max().unwrap_or(0);
        let mut rows: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
        for (idx, depth) in depths.iter().enumerate() {
            rows.entry(*depth).or_default().push(idx);
        }
        for nodes in rows.values_mut() {
            nodes.sort_by(|a, b| points[*a].0.total_cmp(&points[*b].0));
        }

        let current_pos = rows
            .get(&current_depth)
            .and_then(|nodes| nodes.iter().position(|idx| *idx == current))
            .unwrap_or(0);

        let next_nonempty_depth = |order: Vec<usize>, rows: &BTreeMap<usize, Vec<usize>>| {
            order
                .into_iter()
                .find(|depth| rows.get(depth).map(|n| !n.is_empty()).unwrap_or(false))
        };

        best_idx = match direction {
            NavDirection::Right => {
                let next_on_row = rows
                    .get(&current_depth)
                    .and_then(|nodes| nodes.get(current_pos + 1))
                    .copied();
                next_on_row.or_else(|| {
                    let order: Vec<usize> = ((current_depth + 1)..=max_depth)
                        .chain(0..=current_depth)
                        .collect();
                    next_nonempty_depth(order, &rows)
                        .and_then(|depth| rows.get(&depth))
                        .and_then(|nodes| nodes.last())
                        .copied()
                })
            }
            NavDirection::Left => {
                let prev_on_row = rows
                    .get(&current_depth)
                    .and_then(|nodes| current_pos.checked_sub(1).and_then(|pos| nodes.get(pos)))
                    .copied();
                prev_on_row.or_else(|| {
                    let order: Vec<usize> = (0..current_depth)
                        .rev()
                        .chain((current_depth..=max_depth).rev())
                        .collect();
                    next_nonempty_depth(order, &rows)
                        .and_then(|depth| rows.get(&depth))
                        .and_then(|nodes| nodes.first())
                        .copied()
                })
            }
            NavDirection::Down => {
                let order: Vec<usize> = ((current_depth + 1)..=max_depth)
                    .chain(0..=current_depth)
                    .collect();
                next_nonempty_depth(order, &rows)
                    .and_then(|depth| rows.get(&depth))
                    .and_then(|nodes| {
                        nodes.iter().copied().min_by(|a, b| {
                            let ax = (points[*a].0 - cx).abs();
                            let bx = (points[*b].0 - cx).abs();
                            ax.total_cmp(&bx)
                        })
                    })
            }
            NavDirection::Up => {
                let order: Vec<usize> = (0..current_depth)
                    .rev()
                    .chain((current_depth..=max_depth).rev())
                    .collect();
                next_nonempty_depth(order, &rows)
                    .and_then(|depth| rows.get(&depth))
                    .and_then(|nodes| {
                        nodes.iter().copied().min_by(|a, b| {
                            let ax = (points[*a].0 - cx).abs();
                            let bx = (points[*b].0 - cx).abs();
                            ax.total_cmp(&bx)
                        })
                    })
            }
        };
    }

    if let Some(idx) = best_idx {
        app.selected_worktree = idx;
    }
}

fn move_worktree_level_siblings(app: &mut App, move_right: bool) {
    if app.worktrees.len() < 2 || app.selected_worktree >= app.worktrees.len() {
        return;
    }

    let root_branch = current_session_branch(app);
    let parents = worktree_parent_map(&app.worktrees, root_branch.as_str());
    let depths = graph_depths(&parents);
    let points = graph_layout(&parents);
    let current = app.selected_worktree;
    let current_depth = depths[current];
    let (cx, cy) = points[current];

    let mut best_idx: Option<usize> = None;
    let mut best_score = f32::MAX;
    for (idx, (x, y)) in points.iter().enumerate() {
        if idx == current || depths[idx] != current_depth {
            continue;
        }

        let dx = *x - cx;
        let in_direction = if move_right { dx > 0.02 } else { dx < -0.02 };
        if !in_direction {
            continue;
        }

        let score = dx.abs() + ((*y - cy).abs() * 1.4);
        if score < best_score {
            best_score = score;
            best_idx = Some(idx);
        }
    }

    if let Some(idx) = best_idx {
        app.selected_worktree = idx;
    }
}

fn move_worktree_level_vertical(app: &mut App, move_up: bool) {
    if app.worktrees.len() < 2 || app.selected_worktree >= app.worktrees.len() {
        return;
    }

    let root_branch = current_session_branch(app);
    let parents = worktree_parent_map(&app.worktrees, root_branch.as_str());
    let depths = graph_depths(&parents);
    let points = graph_layout(&parents);
    let current = app.selected_worktree;
    let current_depth = depths[current];
    let (cx, cy) = points[current];

    if move_up {
        if let Some(parent_idx) = parents.get(current).and_then(|parent| *parent) {
            app.selected_worktree = parent_idx;
            return;
        }

        if current_depth == 0 {
            return;
        }
    }

    let target_depth = if move_up {
        current_depth.saturating_sub(1)
    } else {
        current_depth + 1
    };

    let mut best_idx: Option<usize> = None;
    let mut best_score = f32::MAX;

    for (idx, depth) in depths.iter().enumerate() {
        if idx == current || *depth != target_depth {
            continue;
        }

        if !move_up && parents.get(idx).copied().flatten() != Some(current) {
            continue;
        }

        let (x, y) = points[idx];
        let score = (x - cx).abs() + (y - cy).abs();
        if score < best_score {
            best_score = score;
            best_idx = Some(idx);
        }
    }

    if best_idx.is_none() && !move_up {
        for (idx, depth) in depths.iter().enumerate() {
            if idx == current || *depth != target_depth {
                continue;
            }

            let (x, y) = points[idx];
            let score = (x - cx).abs() + (y - cy).abs();
            if score < best_score {
                best_score = score;
                best_idx = Some(idx);
            }
        }
    }

    if let Some(idx) = best_idx {
        app.selected_worktree = idx;
    }
}

fn create_root_for_app(app: &App) -> String {
    app.selected_worktree()
        .and_then(|wt| repo_container_from_path(wt.path.as_str()))
        .or_else(|| repo_container_from_path("."))
        .or_else(repo_root)
        .unwrap_or_else(|| ".".to_string())
}

fn branch_exists(root: &str, branch: &str) -> bool {
    Command::new("git")
        .args([
            "-C",
            root,
            "show-ref",
            "--verify",
            "--quiet",
            &format!("refs/heads/{}", branch),
        ])
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn delete_branch_and_create_worktree(
    app: &App,
    root: &str,
    branch: &str,
) -> Result<String, Box<dyn Error>> {
    let delete = Command::new("git")
        .args(["-C", root, "branch", "-D", branch])
        .output()?;
    if !delete.status.success() {
        let stderr = sanitize_for_tui(String::from_utf8_lossy(&delete.stderr).as_ref())
            .trim()
            .to_string();
        let stdout = sanitize_for_tui(String::from_utf8_lossy(&delete.stdout).as_ref())
            .trim()
            .to_string();
        let reason = if !stderr.is_empty() { stderr } else { stdout };
        return Ok(format!("Failed deleting branch '{}': {}", branch, reason));
    }

    create_worktree(app, branch)
}

fn create_worktree(app: &App, branch: &str) -> Result<String, Box<dyn Error>> {
    let sanitized = branch.replace('/', "-");
    let root = create_root_for_app(app);
    let container = workspaces_container_for_root(root.as_str());
    let _ = fs::create_dir_all(container.as_path());
    let path = container.join(sanitized);
    let path_str = path.to_string_lossy().to_string();
    if path.exists() {
        return Ok(format!(
            "Target path already exists: {} (pick another branch name)",
            path_str
        ));
    }
    let (start_point, parent_branch, source_path) = worktree_create_source(app);

    let output = Command::new("git")
        .args([
            "-C",
            root.as_str(),
            "worktree",
            "add",
            "-b",
            branch,
            path_str.as_str(),
            start_point.as_str(),
        ])
        .output()?;
    let stdout = sanitize_for_tui(String::from_utf8_lossy(&output.stdout).as_ref())
        .trim()
        .to_string();
    let stderr = sanitize_for_tui(String::from_utf8_lossy(&output.stderr).as_ref())
        .trim()
        .to_string();

    if output.status.success() {
        let verified = Command::new("git")
            .args(["-C", root.as_str(), "worktree", "list", "--porcelain"])
            .output()
            .ok()
            .map(|out| sanitize_for_tui(String::from_utf8_lossy(&out.stdout).as_ref()))
            .map(|list| {
                list.lines()
                    .any(|line| line.trim() == format!("worktree {}", path_str).as_str())
            })
            .unwrap_or(false);

        let mut message = if stdout.is_empty() {
            format!(
                "Created worktree '{}' at {} from {}",
                branch, path_str, start_point
            )
        } else {
            stdout
        };

        if app.new_worktree_base == WorktreeCreateBase::SelectedWithChanges {
            if let Some(diff) = capture_uncommitted_patch(source_path.as_str())? {
                if diff.trim().is_empty() {
                    message.push_str(" (no uncommitted tracked changes to apply)");
                } else {
                    let apply_result = run_git_with_input(
                        &["-C", path_str.as_str(), "apply", "--whitespace=nowarn", "-"],
                        diff.as_bytes(),
                    )?;
                    if apply_result.success {
                        message.push_str(" + carried uncommitted tracked changes");
                    } else {
                        message.push_str(" (created, but failed to apply uncommitted changes)");
                        if !apply_result.stderr.is_empty() {
                            message.push_str(": ");
                            message.push_str(apply_result.stderr.as_str());
                        }
                    }
                }
            }
        }

        let _ = save_parent_hint(root.as_str(), branch, parent_branch.as_str());

        if !verified {
            message.push_str(
                " (warning: creation reported success but worktree was not found in list)",
            );
        }

        Ok(message)
    } else if !stderr.is_empty() {
        let tail = stderr
            .lines()
            .rev()
            .find(|line| !line.trim().is_empty())
            .unwrap_or(stderr.as_str())
            .to_string();
        Ok(format!(
            "Failed creating worktree '{}' from '{}': {}",
            branch, start_point, tail
        ))
    } else if !stdout.is_empty() {
        Ok(stdout)
    } else {
        Ok("git worktree add failed".to_string())
    }
}

fn repo_root() -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let root = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if root.is_empty() {
        None
    } else {
        Some(root)
    }
}

fn repo_container_from_path(path: &str) -> Option<String> {
    let output = Command::new("git")
        .args(["-C", path, "rev-parse", "--git-common-dir"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let raw = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if raw.is_empty() {
        None
    } else {
        let common_dir = if Path::new(raw.as_str()).is_absolute() {
            PathBuf::from(raw)
        } else {
            Path::new(path).join(raw)
        };
        let common_abs = fs::canonicalize(common_dir.as_path()).unwrap_or(common_dir);
        let parent = common_abs.parent()?.to_string_lossy().to_string();
        if parent.is_empty() {
            None
        } else {
            Some(parent)
        }
    }
}

fn parent_hint_map_path(root: &str) -> String {
    workspaces_container_for_root(root)
        .join(".parent-hints")
        .to_string_lossy()
        .to_string()
}

fn workspaces_container_for_root(root: &str) -> PathBuf {
    let repo_root = Path::new(root);
    let repo_name = repo_root
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or("repo");
    let parent = repo_root.parent().unwrap_or_else(|| Path::new("."));
    parent.join(format!(".{}-workspaces", repo_name))
}

fn load_parent_hint_map(root: &str) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    let content = match fs::read_to_string(parent_hint_map_path(root)) {
        Ok(v) => v,
        Err(_) => return map,
    };

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some((child, parent)) = trimmed.split_once('\t') {
            let c = child.trim();
            let p = parent.trim();
            if !c.is_empty() && !p.is_empty() {
                map.insert(c.to_string(), p.to_string());
            }
        }
    }

    map
}

fn save_parent_hint(
    root: &str,
    child_branch: &str,
    parent_branch: &str,
) -> Result<(), Box<dyn Error>> {
    let mut map = load_parent_hint_map(root);
    map.insert(child_branch.to_string(), parent_branch.to_string());

    let mut lines = String::new();
    for (child, parent) in map {
        lines.push_str(child.as_str());
        lines.push('\t');
        lines.push_str(parent.as_str());
        lines.push('\n');
    }

    fs::write(parent_hint_map_path(root), lines)?;
    Ok(())
}

fn selected_branch_name(app: &App) -> String {
    if let Some(selected) = app.selected_worktree() {
        if !selected.detached && !selected.branch.is_empty() {
            return selected.branch.clone();
        }
        if !selected.head.is_empty() {
            return selected.head.clone();
        }
    }

    let raw = app.branch.trim();
    let name = raw
        .strip_prefix("HEAD (detached at ")
        .and_then(|value| value.strip_suffix(')'))
        .unwrap_or(raw);
    if name.is_empty() {
        "HEAD".to_string()
    } else {
        name.to_string()
    }
}

fn worktree_create_source(app: &App) -> (String, String, String) {
    match app.new_worktree_base {
        WorktreeCreateBase::Main => {
            let main = resolve_main_branch();
            (main.clone(), main, ".".to_string())
        }
        WorktreeCreateBase::Selected | WorktreeCreateBase::SelectedWithChanges => {
            let selected = selected_branch_name(app);
            let source_path = app
                .selected_worktree()
                .map(|wt| wt.path.clone())
                .unwrap_or_else(|| ".".to_string());
            (selected.clone(), selected, source_path)
        }
    }
}

fn resolve_main_branch() -> String {
    if let Some(main) = git_output(&["show-ref", "--verify", "--quiet", "refs/heads/main"]) {
        if main.is_empty() {
            return "main".to_string();
        }
    }
    if let Some(master) = git_output(&["show-ref", "--verify", "--quiet", "refs/heads/master"]) {
        if master.is_empty() {
            return "master".to_string();
        }
    }
    "main".to_string()
}

fn capture_uncommitted_patch(source_path: &str) -> Result<Option<String>, Box<dyn Error>> {
    let output = Command::new("git")
        .args(["-C", source_path, "diff", "--binary", "HEAD"])
        .output()?;
    if !output.status.success() {
        return Ok(None);
    }
    Ok(Some(sanitize_for_tui(
        String::from_utf8_lossy(&output.stdout).as_ref(),
    )))
}

struct CommandResult {
    success: bool,
    stderr: String,
}

fn run_git_with_input(args: &[&str], input: &[u8]) -> Result<CommandResult, Box<dyn Error>> {
    let mut child = Command::new("git")
        .args(args)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(input)?;
    }

    let output = child.wait_with_output()?;
    Ok(CommandResult {
        success: output.status.success(),
        stderr: sanitize_for_tui(String::from_utf8_lossy(&output.stderr).as_ref())
            .trim()
            .to_string(),
    })
}

fn parse_branch_line(app: &mut App, line: &str) {
    let (branch, ahead, behind) = parse_branch_snapshot(line);
    app.branch = branch;
    app.ahead = ahead;
    app.behind = behind;
}

fn toggle_stage(app: &mut App) -> Result<(), Box<dyn Error>> {
    let item = match app.selected_item() {
        Some(entry) => entry,
        None => {
            app.status_line = "No item selected".to_string();
            return Ok(());
        }
    };

    if item.staged {
        app.status_line = run_git_in(
            app.changes_worktree_path.as_deref(),
            &["restore", "--staged", "--", &item.path],
        )?;
    } else {
        app.status_line = run_git_in(
            app.changes_worktree_path.as_deref(),
            &["add", "--", &item.path],
        )?;
    }

    Ok(())
}

fn refresh_selected_overview(app: &mut App) {
    let item = match app.selected_item() {
        Some(entry) => entry,
        None => {
            app.selected_overview = None;
            app.overview_scroll = 0;
            return;
        }
    };

    app.selected_overview = match item.kind {
        TreeKind::File => Some(build_file_overview(
            &FileEntry {
                path: item.path.clone(),
                staged: item.staged,
                unstaged: item.unstaged,
                untracked: item.untracked,
            },
            app.changes_worktree_path.as_deref(),
        )),
        TreeKind::Folder => Some(build_folder_overview(
            item,
            &app.files,
            app.changes_worktree_path.as_deref(),
        )),
    };

    let max_scroll = max_overview_scroll(app);
    if app.overview_scroll > max_scroll {
        app.overview_scroll = max_scroll;
    }
}

fn build_folder_overview(
    folder: &TreeItem,
    files: &[FileEntry],
    repo_path: Option<&str>,
) -> FileOverview {
    let prefix = format!("{}/", folder.path);
    let mut total_added = 0usize;
    let mut total_removed = 0usize;
    let mut methods_added: HashSet<String> = HashSet::new();
    let mut methods_modified: HashSet<String> = HashSet::new();
    let mut methods_deleted: HashSet<String> = HashSet::new();
    let mut traditional_diff: Vec<DiffPreviewLine> = Vec::new();

    for file in files {
        if !(file.path == folder.path || file.path.starts_with(prefix.as_str())) {
            continue;
        }

        let overview = build_file_overview(file, repo_path);
        total_added += overview.added_lines;
        total_removed += overview.removed_lines;
        methods_added.extend(overview.methods_added);
        methods_modified.extend(overview.methods_modified);
        methods_deleted.extend(overview.methods_deleted);

        if traditional_diff.len() < 24 {
            traditional_diff.push(DiffPreviewLine {
                kind: DiffPreviewKind::Meta,
                text: format!("file: {}", file.path),
            });
            for row in overview.traditional_diff.into_iter().take(6) {
                if traditional_diff.len() >= 24 {
                    break;
                }
                traditional_diff.push(row);
            }
        }
    }

    let methods_added = sorted_from_set(methods_added);
    let methods_modified = sorted_from_set(methods_modified);
    let methods_deleted = sorted_from_set(methods_deleted);
    let use_traditional_overview =
        methods_added.is_empty() && methods_modified.is_empty() && methods_deleted.is_empty();

    FileOverview {
        file: format!("{}/", folder.path),
        state: build_state_label(&FileEntry {
            path: folder.path.clone(),
            staged: folder.staged,
            unstaged: folder.unstaged,
            untracked: folder.untracked,
        }),
        added_lines: total_added,
        removed_lines: total_removed,
        methods_added,
        methods_modified,
        methods_deleted,
        traditional_diff,
        use_traditional_overview,
    }
}

fn build_file_overview(file: &FileEntry, repo_path: Option<&str>) -> FileOverview {
    let state = build_state_label(file);
    let mut added_lines = 0usize;
    let mut removed_lines = 0usize;
    let mut methods_added: Vec<String> = Vec::new();
    let mut methods_modified: Vec<String> = Vec::new();
    let mut methods_deleted: Vec<String> = Vec::new();
    let mut traditional_diff: Vec<DiffPreviewLine> = Vec::new();

    if file.untracked {
        let file_path = repo_path
            .map(|base| Path::new(base).join(file.path.as_str()))
            .unwrap_or_else(|| PathBuf::from(file.path.as_str()));
        let text = fs::read_to_string(file_path).unwrap_or_default();
        added_lines = text.lines().count();
        methods_added = sorted_from_set(collect_methods_from_content(&text, &file.path));
        traditional_diff = preview_for_untracked(&text);
    } else if let Some(diff) = git_output_in(
        repo_path,
        &[
            "diff",
            "--no-color",
            "--unified=0",
            "HEAD",
            "--",
            &file.path,
        ],
    ) {
        let summary = summarize_diff(&diff, &file.path);
        added_lines = summary.added_lines;
        removed_lines = summary.removed_lines;
        methods_added = summary.methods_added;
        methods_modified = summary.methods_modified;
        methods_deleted = summary.methods_deleted;
        traditional_diff = summary.diff_preview;
    }

    let use_traditional_overview =
        methods_added.is_empty() && methods_modified.is_empty() && methods_deleted.is_empty();

    FileOverview {
        file: file.path.clone(),
        state,
        added_lines,
        removed_lines,
        methods_added,
        methods_modified,
        methods_deleted,
        traditional_diff,
        use_traditional_overview,
    }
}

fn build_state_label(file: &FileEntry) -> String {
    let mut states: Vec<&str> = Vec::new();
    if file.staged {
        states.push("staged");
    }
    if file.unstaged {
        states.push("unstaged");
    }
    if file.untracked {
        states.push("new");
    }
    if states.is_empty() {
        "clean".to_string()
    } else {
        states.join(", ")
    }
}

#[derive(Default)]
struct DiffSummary {
    added_lines: usize,
    removed_lines: usize,
    methods_added: Vec<String>,
    methods_modified: Vec<String>,
    methods_deleted: Vec<String>,
    diff_preview: Vec<DiffPreviewLine>,
}

fn summarize_diff(diff: &str, file_path: &str) -> DiffSummary {
    let mut added_methods: HashSet<String> = HashSet::new();
    let mut removed_methods: HashSet<String> = HashSet::new();
    let mut modified_hunks: HashSet<String> = HashSet::new();
    let mut diff_preview: Vec<DiffPreviewLine> = Vec::new();
    let mut current_hunk_method: Option<String> = None;
    let mut added_lines = 0usize;
    let mut removed_lines = 0usize;

    for line in diff.lines() {
        if line.starts_with("@@") {
            current_hunk_method = parse_hunk_header(line)
                .and_then(|header| extract_method_name(header.as_str(), file_path));
            push_preview_line(&mut diff_preview, DiffPreviewKind::Meta, line);
            continue;
        }

        if line.starts_with("+++") || line.starts_with("---") {
            push_preview_line(&mut diff_preview, DiffPreviewKind::Meta, line);
            continue;
        }

        if line.starts_with("diff --git") || line.starts_with("index ") {
            push_preview_line(&mut diff_preview, DiffPreviewKind::Meta, line);
            continue;
        }

        if let Some(rest) = line.strip_prefix('+') {
            added_lines += 1;
            if let Some(name) = current_hunk_method.as_ref() {
                modified_hunks.insert(name.clone());
            }
            if let Some(name) = extract_method_name(rest, file_path) {
                added_methods.insert(name);
            }
            push_preview_line(&mut diff_preview, DiffPreviewKind::Added, line);
            continue;
        }

        if let Some(rest) = line.strip_prefix('-') {
            removed_lines += 1;
            if let Some(name) = current_hunk_method.as_ref() {
                modified_hunks.insert(name.clone());
            }
            if let Some(name) = extract_method_name(rest, file_path) {
                removed_methods.insert(name);
            }
            push_preview_line(&mut diff_preview, DiffPreviewKind::Removed, line);
            continue;
        }

        push_preview_line(&mut diff_preview, DiffPreviewKind::Context, line);
    }

    let methods_added_set: HashSet<String> = added_methods
        .difference(&removed_methods)
        .cloned()
        .collect();
    let methods_deleted_set: HashSet<String> = removed_methods
        .difference(&added_methods)
        .cloned()
        .collect();
    let overlap_set: HashSet<String> = added_methods
        .intersection(&removed_methods)
        .cloned()
        .collect();
    let methods_modified_set: HashSet<String> = modified_hunks
        .union(&overlap_set)
        .cloned()
        .filter(|name| !methods_added_set.contains(name) && !methods_deleted_set.contains(name))
        .collect();

    DiffSummary {
        added_lines,
        removed_lines,
        methods_added: sorted_from_set(methods_added_set),
        methods_modified: sorted_from_set(methods_modified_set),
        methods_deleted: sorted_from_set(methods_deleted_set),
        diff_preview,
    }
}

fn push_preview_line(lines: &mut Vec<DiffPreviewLine>, kind: DiffPreviewKind, raw: &str) {
    if lines.len() >= 28 {
        return;
    }
    lines.push(DiffPreviewLine {
        kind,
        text: truncate_text(raw, 96),
    });
}

fn preview_for_untracked(content: &str) -> Vec<DiffPreviewLine> {
    let mut lines = Vec::new();
    for (idx, line) in content.lines().enumerate() {
        if idx >= 24 {
            break;
        }
        lines.push(DiffPreviewLine {
            kind: DiffPreviewKind::Added,
            text: format!("+{}", truncate_text(line, 95)),
        });
    }
    lines
}

fn sorted_from_set(set: HashSet<String>) -> Vec<String> {
    let mut v: Vec<String> = set.into_iter().collect();
    v.sort();
    v
}

fn parse_hunk_header(line: &str) -> Option<String> {
    let mut parts = line.split("@@");
    let _ = parts.next();
    let tail = parts.nth(1).unwrap_or_default().trim();
    if tail.is_empty() {
        None
    } else {
        Some(tail.to_string())
    }
}

fn collect_methods_from_content(content: &str, file_path: &str) -> HashSet<String> {
    let mut methods = HashSet::new();
    for line in content.lines() {
        if let Some(name) = extract_method_name(line, file_path) {
            methods.insert(name);
        }
    }
    methods
}

fn extract_method_name(line: &str, file_path: &str) -> Option<String> {
    let s = line.trim_start();
    let ext = file_extension(file_path);

    match ext {
        "py" => extract_python_method(s),
        "rs" => extract_rust_method(s),
        "js" | "jsx" | "ts" | "tsx" | "mjs" | "cjs" => extract_js_method(s),
        "go" => extract_go_method(s),
        _ => extract_general_method(s),
    }
}

fn file_extension(path: &str) -> &str {
    path.rsplit('.').next().unwrap_or_default()
}

fn extract_python_method(s: &str) -> Option<String> {
    if let Some(rest) = s.strip_prefix("def ") {
        return extract_identifier_until_paren(rest);
    }
    if let Some(rest) = s.strip_prefix("async def ") {
        return extract_identifier_until_paren(rest);
    }
    None
}

fn extract_rust_method(s: &str) -> Option<String> {
    if let Some(idx) = s.find(" fn ") {
        return extract_identifier_until_paren(&s[idx + 4..]);
    }
    if let Some(rest) = s.strip_prefix("fn ") {
        return extract_identifier_until_paren(rest);
    }
    None
}

fn extract_js_method(s: &str) -> Option<String> {
    if let Some(rest) = s.strip_prefix("function ") {
        return extract_identifier_until_paren(rest);
    }
    if let Some(rest) = s.strip_prefix("async function ") {
        return extract_identifier_until_paren(rest);
    }
    if let Some(rest) = s.strip_prefix("const ") {
        if rest.contains("=>") {
            let (left, _) = rest.split_once('=').unwrap_or(("", ""));
            let ident = left.trim();
            if is_identifier_like(ident) {
                return Some(ident.to_string());
            }
        }
    }
    None
}

fn extract_go_method(s: &str) -> Option<String> {
    if let Some(rest) = s.strip_prefix("func ") {
        if rest.starts_with('(') {
            let after_receiver = rest.split(')').nth(1).unwrap_or_default().trim_start();
            return extract_identifier_until_paren(after_receiver);
        }
        return extract_identifier_until_paren(rest);
    }
    None
}

fn extract_general_method(s: &str) -> Option<String> {
    if let Some(rest) = s.strip_prefix("function ") {
        return extract_identifier_until_paren(rest);
    }
    if let Some(rest) = s.strip_prefix("def ") {
        return extract_identifier_until_paren(rest);
    }
    None
}

fn extract_identifier_until_paren(text: &str) -> Option<String> {
    let name = text
        .split('(')
        .next()
        .unwrap_or_default()
        .trim()
        .trim_end_matches('{')
        .trim();

    if !is_identifier_like(name) {
        None
    } else {
        Some(name.to_string())
    }
}

fn is_identifier_like(value: &str) -> bool {
    if value.is_empty() {
        return false;
    }
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first == '_' || first.is_ascii_alphabetic()) {
        return false;
    }
    chars.all(|c| c == '_' || c.is_ascii_alphanumeric())
}

fn sanitize_for_tui(input: &str) -> String {
    let mut out = String::new();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' {
            match chars.peek().copied() {
                Some('[') => {
                    let _ = chars.next();
                    while let Some(c) = chars.next() {
                        if ('@'..='~').contains(&c) {
                            break;
                        }
                    }
                    continue;
                }
                Some(']') => {
                    let _ = chars.next();
                    while let Some(c) = chars.next() {
                        if c == '\u{7}' {
                            break;
                        }
                        if c == '\u{1b}' {
                            if let Some('\\') = chars.peek().copied() {
                                let _ = chars.next();
                                break;
                            }
                        }
                    }
                    continue;
                }
                _ => continue,
            }
        }

        if ch == '\n' || (ch >= ' ' && ch != '\u{7f}') {
            out.push(ch);
        } else if ch == '\t' {
            out.push_str("    ");
        }
    }

    out
}

fn run_git_in(path: Option<&str>, args: &[&str]) -> Result<String, Box<dyn Error>> {
    let mut cmd = Command::new("git");
    if let Some(path) = path {
        cmd.args(["-C", path]);
    }
    let output = cmd.args(args).output()?;
    let stdout = sanitize_for_tui(String::from_utf8_lossy(&output.stdout).as_ref())
        .trim()
        .to_string();
    let stderr = sanitize_for_tui(String::from_utf8_lossy(&output.stderr).as_ref())
        .trim()
        .to_string();

    if output.status.success() {
        if stdout.is_empty() {
            Ok(format!("✓ git {}", args.join(" ")))
        } else {
            Ok(stdout)
        }
    } else if !stderr.is_empty() {
        Ok(stderr)
    } else {
        Ok(format!("git {} failed", args.join(" ")))
    }
}

fn run_git(args: &[&str]) -> Result<String, Box<dyn Error>> {
    run_git_in(None, args)
}

fn git_output_in(path: Option<&str>, args: &[&str]) -> Option<String> {
    let mut cmd = Command::new("git");
    if let Some(path) = path {
        cmd.args(["-C", path]);
    }
    let output = cmd.args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    Some(sanitize_for_tui(
        String::from_utf8_lossy(&output.stdout).as_ref(),
    ))
}

fn git_output(args: &[&str]) -> Option<String> {
    git_output_in(None, args)
}

fn push_with_upstream() -> Result<String, Box<dyn Error>> {
    let first = Command::new("git").args(["push"]).output()?;
    let first_stdout = sanitize_for_tui(String::from_utf8_lossy(&first.stdout).as_ref())
        .trim()
        .to_string();
    let first_stderr = sanitize_for_tui(String::from_utf8_lossy(&first.stderr).as_ref())
        .trim()
        .to_string();

    if first.status.success() {
        if first_stdout.is_empty() {
            return Ok("✓ git push".to_string());
        }
        return Ok(first_stdout);
    }

    let error_text = if !first_stderr.is_empty() {
        first_stderr.clone()
    } else {
        first_stdout.clone()
    };

    let needs_upstream = error_text.contains("has no upstream branch")
        || error_text.contains("--set-upstream")
        || error_text.contains("set upstream");

    if !needs_upstream {
        if error_text.is_empty() {
            return Ok("git push failed".to_string());
        }
        return Ok(error_text);
    }

    let remote = preferred_remote()?;
    let second = Command::new("git")
        .args(["push", "-u", remote.as_str(), "HEAD"])
        .output()?;
    let second_stdout = sanitize_for_tui(String::from_utf8_lossy(&second.stdout).as_ref())
        .trim()
        .to_string();
    let second_stderr = sanitize_for_tui(String::from_utf8_lossy(&second.stderr).as_ref())
        .trim()
        .to_string();

    if second.status.success() {
        if second_stdout.is_empty() {
            Ok(format!("✓ git push -u {} HEAD", remote))
        } else {
            Ok(format!(
                "Set upstream to {} and pushed\n{}",
                remote, second_stdout
            ))
        }
    } else if !second_stderr.is_empty() {
        Ok(second_stderr)
    } else if !second_stdout.is_empty() {
        Ok(second_stdout)
    } else {
        Ok(format!("git push -u {} HEAD failed", remote))
    }
}

fn commit_worktree(path: &str, message: &str) -> Result<String, Box<dyn Error>> {
    let add = Command::new("git")
        .args(["-C", path, "add", "."])
        .output()?;
    if !add.status.success() {
        let stderr = sanitize_for_tui(String::from_utf8_lossy(&add.stderr).as_ref())
            .trim()
            .to_string();
        let stdout = sanitize_for_tui(String::from_utf8_lossy(&add.stdout).as_ref())
            .trim()
            .to_string();
        let reason = if !stderr.is_empty() { stderr } else { stdout };
        return Ok(format!(
            "git add failed in {}: {}",
            path,
            single_line(reason.as_str())
        ));
    }

    let commit = Command::new("git")
        .args(["-C", path, "commit", "-m", message])
        .output()?;
    let commit_stdout = sanitize_for_tui(String::from_utf8_lossy(&commit.stdout).as_ref())
        .trim()
        .to_string();
    let commit_stderr = sanitize_for_tui(String::from_utf8_lossy(&commit.stderr).as_ref())
        .trim()
        .to_string();

    let nothing_to_commit = !commit.status.success()
        && (commit_stdout.contains("nothing to commit")
            || commit_stderr.contains("nothing to commit")
            || commit_stderr.contains("no changes added to commit"));

    if !commit.status.success() && !nothing_to_commit {
        let reason = if !commit_stderr.is_empty() {
            commit_stderr
        } else {
            commit_stdout
        };
        return Ok(format!(
            "git commit failed in {}: {}",
            path,
            single_line(reason.as_str())
        ));
    }

    if nothing_to_commit {
        Ok(format!("No new commit in {} (nothing to commit)", path))
    } else {
        let commit_line = if !commit_stdout.is_empty() {
            single_line(commit_stdout.as_str())
        } else if !commit_stderr.is_empty() {
            single_line(commit_stderr.as_str())
        } else {
            "commit ok".to_string()
        };
        Ok(format!("Committed in {} - {}", path, commit_line))
    }
}

fn push_with_upstream_at(path: &str) -> Result<String, Box<dyn Error>> {
    let remote = preferred_remote_at(path)?;
    let push = Command::new("git")
        .args(["-C", path, "push", "-u", remote.as_str(), "HEAD"])
        .output()?;
    let push_stdout = sanitize_for_tui(String::from_utf8_lossy(&push.stdout).as_ref())
        .trim()
        .to_string();
    let push_stderr = sanitize_for_tui(String::from_utf8_lossy(&push.stderr).as_ref())
        .trim()
        .to_string();

    if push.status.success() {
        Ok(if push_stdout.is_empty() {
            format!("✓ git push -u {} HEAD", remote)
        } else {
            format!("Set upstream to {} and pushed\n{}", remote, push_stdout)
        })
    } else if !push_stderr.is_empty() {
        Ok(push_stderr)
    } else if !push_stdout.is_empty() {
        Ok(push_stdout)
    } else {
        Ok(format!("git push -u {} HEAD failed", remote))
    }
}

fn preferred_remote_at(path: &str) -> Result<String, Box<dyn Error>> {
    let output = Command::new("git").args(["-C", path, "remote"]).output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let remotes: Vec<&str> = stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();

    if remotes.iter().any(|name| *name == "origin") {
        Ok("origin".to_string())
    } else if let Some(first) = remotes.first() {
        Ok((*first).to_string())
    } else {
        Ok("origin".to_string())
    }
}

fn preferred_remote() -> Result<String, Box<dyn Error>> {
    let output = Command::new("git").args(["remote"]).output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let remotes: Vec<&str> = stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();

    if remotes.iter().any(|name| *name == "origin") {
        Ok("origin".to_string())
    } else if let Some(first) = remotes.first() {
        Ok((*first).to_string())
    } else {
        Ok("origin".to_string())
    }
}
