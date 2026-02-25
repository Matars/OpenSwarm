fn refresh_status(app: &mut App) {
    if app.status_refresh_in_flight {
        return;
    }

    let snapshot = match load_status_snapshot(app.changes_worktree_path.clone()) {
        Ok(snapshot) => snapshot,
        Err(err) => {
            app.status_line = err;
            return;
        }
    };
    apply_status_snapshot(app, snapshot);
}

fn start_status_refresh_task(app: &mut App) {
    if app.status_refresh_in_flight {
        return;
    }

    app.status_refresh_in_flight = true;
    let path = app.changes_worktree_path.clone();
    let tx = app.status_refresh_tx.clone();
    thread::spawn(move || {
        let event = match load_status_snapshot(path) {
            Ok(snapshot) => StatusRefreshEvent {
                snapshot: Some(snapshot),
                error: None,
            },
            Err(err) => StatusRefreshEvent {
                snapshot: None,
                error: Some(err),
            },
        };
        let _ = tx.send(event);
    });
}

fn drain_status_refresh_events(app: &mut App) {
    let mut last_event: Option<StatusRefreshEvent> = None;
    while let Ok(event) = app.status_refresh_rx.try_recv() {
        last_event = Some(event);
    }

    let Some(event) = last_event else {
        return;
    };

    app.status_refresh_in_flight = false;
    if let Some(snapshot) = event.snapshot {
        apply_status_snapshot(app, snapshot);
    } else if let Some(err) = event.error {
        app.status_line = err;
    }
}

fn load_status_snapshot(changes_worktree_path: Option<String>) -> Result<StatusSnapshot, String> {
    let output = git_output_in(
        changes_worktree_path.as_deref(),
        &["status", "--porcelain=1", "-b", "-uall"],
    )
    .ok_or_else(|| "Failed to load git status".to_string())?;

    let mut lines = output.lines();
    let mut branch = "unknown".to_string();
    let mut ahead = 0usize;
    let mut behind = 0usize;
    if let Some(head) = lines.next() {
        let (parsed_branch, parsed_ahead, parsed_behind, _) = parse_branch_snapshot(head);
        branch = parsed_branch;
        ahead = parsed_ahead;
        behind = parsed_behind;
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

    let tree_items = build_tree_items(&files, changes_worktree_path.as_deref());
    Ok(StatusSnapshot {
        branch,
        ahead,
        behind,
        files,
        tree_items,
    })
}

fn selected_item_fingerprint(
    app: &App,
) -> Option<(String, TreeKind, bool, bool, bool, usize, usize)> {
    app.selected_item().map(|item| {
        (
            item.path.clone(),
            item.kind.clone(),
            item.staged,
            item.unstaged,
            item.untracked,
            item.added_lines,
            item.removed_lines,
        )
    })
}

fn apply_status_snapshot(app: &mut App, snapshot: StatusSnapshot) {
    let old_selected = selected_item_fingerprint(app);

    app.branch = snapshot.branch;
    app.ahead = snapshot.ahead;
    app.behind = snapshot.behind;
    app.files = snapshot.files;
    app.tree_items = snapshot.tree_items;

    if app.tree_items.is_empty() {
        app.selected = 0;
    } else if app.selected >= app.tree_items.len() {
        app.selected = app.tree_items.len() - 1;
    }

    let new_selected = selected_item_fingerprint(app);
    let should_refresh_overview = old_selected != new_selected || app.selected_overview.is_none();
    if should_refresh_overview {
        refresh_selected_overview(app);
    }

    let max_scroll = max_overview_scroll(app);
    if app.overview_scroll > max_scroll {
        app.overview_scroll = max_scroll;
    }
}

fn run_startup_checks(app: &mut App) {
    let git_ready = Command::new("git").arg("--version").output().is_ok();
    if !git_ready {
        app.status_line = "git is not available on PATH".to_string();
        return;
    }

    let inside_worktree = Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| sanitize_for_tui(String::from_utf8_lossy(&output.stdout).as_ref()))
        .map(|text| text.trim().eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    if !inside_worktree {
        app.status_line =
            "Not inside a Git worktree. Launch OpenSwarm from a repo root or worktree path"
                .to_string();
        return;
    }

    let top_level = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| sanitize_for_tui(String::from_utf8_lossy(&output.stdout).as_ref()))
        .map(|text| text.trim().to_string())
        .filter(|value| !value.is_empty());

    let Some(root) = top_level else {
        app.status_line = "Unable to resolve git top-level path".to_string();
        return;
    };

    let git_entry = Path::new(root.as_str()).join(".git");
    if !git_entry.exists() {
        app.status_line = format!(
            "Git top-level resolved to {} but .git entry is missing",
            root
        );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WorktreeRefreshMode {
    Full,
    Lightweight,
}

fn refresh_worktrees(app: &mut App) {
    refresh_worktrees_with_mode(app, WorktreeRefreshMode::Full);
}

fn refresh_worktrees_lightweight(app: &mut App) {
    refresh_worktrees_with_mode(app, WorktreeRefreshMode::Lightweight);
}

fn refresh_worktrees_with_mode(app: &mut App, mode: WorktreeRefreshMode) {
    let output = match git_output_with_error(&["worktree", "list", "--porcelain"]) {
        Ok(text) => {
            app.worktree_load_error = None;
            text
        }
        Err(reason) => {
            app.worktree_load_error = Some(reason.clone());
            app.worktrees.clear();
            app.selected_worktree = 0;
            app.status_line = format!("Unable to load git worktrees: {}", single_line(&reason));
            return;
        }
    };

    let current_path = std::env::current_dir()
        .ok()
        .map(|path| normalize_path(path.to_string_lossy().as_ref()));
    let root = create_root_for_app(app);
    let parent_hints = load_parent_hint_map(root.as_str());
    let previous_by_path: BTreeMap<String, WorktreeEntry> = app
        .worktrees
        .iter()
        .cloned()
        .map(|entry| (entry.path.clone(), entry))
        .collect();

    let mut entries: Vec<WorktreeEntry> = Vec::new();
    let mut current = WorktreeEntry::default();
    let mut in_block = false;

    for line in output.lines() {
        if line.trim().is_empty() {
            if in_block {
                let previous = previous_by_path.get(current.path.as_str()).cloned();
                hydrate_worktree_runtime_state(
                    &mut current,
                    current_path.as_deref(),
                    &parent_hints,
                    mode,
                    previous.as_ref(),
                );
                entries.push(current.clone());
                current = WorktreeEntry::default();
                in_block = false;
            }
            continue;
        }

        if let Some(path) = line.strip_prefix("worktree ") {
            if in_block {
                let previous = previous_by_path.get(current.path.as_str()).cloned();
                hydrate_worktree_runtime_state(
                    &mut current,
                    current_path.as_deref(),
                    &parent_hints,
                    mode,
                    previous.as_ref(),
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
        let previous = previous_by_path.get(current.path.as_str()).cloned();
        hydrate_worktree_runtime_state(
            &mut current,
            current_path.as_deref(),
            &parent_hints,
            mode,
            previous.as_ref(),
        );
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
    if mode == WorktreeRefreshMode::Full {
        let root_branch = current_session_branch(app);
        update_worktree_merged_with_parent(&mut app.worktrees, root_branch.as_str());
    }
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

    if mode == WorktreeRefreshMode::Full {
        maybe_prompt_legacy_workspace_migration(app, root.as_str());
    }
}

fn update_worktree_merged_with_parent(entries: &mut [WorktreeEntry], root_branch: &str) {
    if entries.is_empty() {
        return;
    }

    let parents = worktree_parent_map(entries, root_branch);
    for idx in 0..entries.len() {
        entries[idx].merged_with_parent = false;
        entries[idx].behind_parent = false;
        let Some(parent_idx) = parents.get(idx).and_then(|value| *value) else {
            continue;
        };
        if idx == parent_idx {
            continue;
        }

        let child = &entries[idx];
        let parent = &entries[parent_idx];
        if child.detached || child.branch.is_empty() {
            continue;
        }

        let parent_ref = if !parent.detached && !parent.branch.is_empty() {
            parent.branch.as_str()
        } else if !parent.head.is_empty() {
            parent.head.as_str()
        } else {
            continue;
        };

        let parent_contains_child =
            git_is_ancestor(parent.path.as_str(), child.branch.as_str(), parent_ref);
        let child_contains_parent =
            git_is_ancestor(parent.path.as_str(), parent_ref, child.branch.as_str());

        entries[idx].merged_with_parent = parent_contains_child;
        entries[idx].behind_parent = parent_contains_child && !child_contains_parent;
    }
}

fn git_is_ancestor(repo_path: &str, ancestor_ref: &str, descendant_ref: &str) -> bool {
    Command::new("git")
        .args([
            "-C",
            repo_path,
            "merge-base",
            "--is-ancestor",
            ancestor_ref,
            descendant_ref,
        ])
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
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
    mode: WorktreeRefreshMode,
    previous: Option<&WorktreeEntry>,
) {
    let normalized = normalize_path(entry.path.as_str());
    entry.is_current = current_path
        .map(|cwd| cwd == normalized.as_str())
        .unwrap_or(false);

    if entry.branch.is_empty() {
        entry.branch = "detached".to_string();
    }

    if mode == WorktreeRefreshMode::Full {
        let (dirty, ahead, behind, has_upstream) = worktree_branch_state(entry.path.as_str());
        entry.dirty = dirty;
        entry.ahead = ahead;
        entry.behind = behind;
        entry.has_upstream = has_upstream;
    } else if let Some(previous) = previous {
        entry.dirty = previous.dirty;
        entry.ahead = previous.ahead;
        entry.behind = previous.behind;
        entry.has_upstream = previous.has_upstream;
        entry.merged_with_parent = previous.merged_with_parent;
        entry.behind_parent = previous.behind_parent;
    }

    entry.parent_hint = parent_hints.get(entry.branch.as_str()).cloned();
}

fn worktree_branch_state(path: &str) -> (bool, usize, usize, bool) {
    let output = match Command::new("git")
        .args(["-C", path, "status", "--porcelain=1", "-b", "-uno"])
        .output()
    {
        Ok(out) if out.status.success() => {
            sanitize_for_tui(String::from_utf8_lossy(&out.stdout).as_ref())
        }
        _ => return (false, 0, 0, false),
    };

    let mut lines = output.lines();
    let mut ahead = 0usize;
    let mut behind = 0usize;
    let mut has_upstream = false;
    if let Some(head) = lines.next() {
        let (_, parsed_ahead, parsed_behind, parsed_has_upstream) = parse_branch_snapshot(head);
        ahead = parsed_ahead;
        behind = parsed_behind;
        has_upstream = parsed_has_upstream;
    }
    let dirty = lines.any(|line| status_line_counts_as_dirty(line));
    (dirty, ahead, behind, has_upstream)
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

fn parse_branch_snapshot(line: &str) -> (String, usize, usize, bool) {
    let mut ahead = 0usize;
    let mut behind = 0usize;

    let stripped = line.strip_prefix("## ").unwrap_or(line);
    let mut has_upstream = false;
    let branch = if let Some((name, rest)) = stripped.split_once("...") {
        has_upstream = true;
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

    (branch, ahead, behind, has_upstream)
}

fn normalize_path(path: &str) -> String {
    let resolved =
        fs::canonicalize(Path::new(path)).unwrap_or_else(|_| Path::new(path).to_path_buf());
    path_for_git_arg(resolved.as_path())
}

fn path_for_git_arg(path: &Path) -> String {
    #[cfg(windows)]
    {
        return strip_windows_verbatim_prefix(path)
            .to_string_lossy()
            .to_string();
    }

    #[cfg(not(windows))]
    {
        path.to_string_lossy().to_string()
    }
}

#[cfg(windows)]
fn strip_windows_verbatim_prefix(path: &Path) -> PathBuf {
    let raw = path.to_string_lossy();

    if let Some(rest) = raw.strip_prefix(r"\\?\UNC\") {
        return PathBuf::from(format!(r"\\{}", rest));
    }
    if let Some(rest) = raw.strip_prefix(r"\\?\") {
        return PathBuf::from(rest);
    }
    if let Some(rest) = raw.strip_prefix("//?/UNC/") {
        return PathBuf::from(format!("//{}", rest));
    }
    if let Some(rest) = raw.strip_prefix("//?/") {
        return PathBuf::from(rest);
    }

    path.to_path_buf()
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

fn connected_parent_worktree(app: &App) -> Option<WorktreeEntry> {
    connected_parent_index(app)
        .and_then(|idx| app.worktrees.get(idx))
        .cloned()
}

fn update_parent_at(path: &str, branch: &str) -> Result<String, Box<dyn Error>> {
    if branch.is_empty() {
        return Ok("Parent node is detached; cannot fetch updates".to_string());
    }

    if branch == "detached" {
        return Ok("Parent node is detached; cannot fetch updates".to_string());
    }

    let fetch = run_git(&["-C", path, "fetch", "--all", "--prune"])?;
    let pull = run_git(&["-C", path, "pull"])?;

    Ok(format!(
        "Fetched + pulled parent '{}' - fetch: {}; pull: {}",
        branch,
        single_line(fetch.as_str()),
        single_line(pull.as_str()),
    ))
}

fn update_worktree_head_at(path: &str, branch: &str) -> Result<String, Box<dyn Error>> {
    if branch.is_empty() || branch == "detached" {
        return Ok("Selected node is detached; cannot pull head".to_string());
    }

    let fetch = run_git(&["-C", path, "fetch", "--all", "--prune"])?;
    let pull = run_git(&["-C", path, "pull"])?;

    Ok(format!(
        "Fetched + pulled head '{}' - fetch: {}; pull: {}",
        branch,
        single_line(fetch.as_str()),
        single_line(pull.as_str()),
    ))
}

fn rebase_onto_parent_at(
    child_path: &str,
    child_branch: &str,
    parent_path: &str,
    parent_branch: &str,
) -> Result<String, Box<dyn Error>> {
    if child_branch.is_empty() || child_branch == "detached" {
        return Ok("Selected node is detached; cannot rebase onto parent".to_string());
    }
    if parent_branch.is_empty() || parent_branch == "detached" {
        return Ok("Parent node is detached; cannot rebase selected node".to_string());
    }
    if child_branch == parent_branch {
        return Ok("Selected and parent are the same branch; nothing to rebase".to_string());
    }

    let rebase = run_git(&["-C", child_path, "rebase", parent_branch])?;
    let lowered = rebase.to_ascii_lowercase();
    let has_conflicts = lowered.contains("conflict") || lowered.contains("resolve all conflicts");

    if has_conflicts {
        Ok(format!(
            "Rebase '{}' onto parent '{}' needs conflict resolution in {} - {}",
            child_branch,
            parent_branch,
            child_path,
            single_line(rebase.as_str())
        ))
    } else {
        Ok(format!(
            "Rebased '{}' onto parent '{}' (source: {}) - {}",
            child_branch,
            parent_branch,
            parent_path,
            single_line(rebase.as_str())
        ))
    }
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
    let points = graph_layout(&parents, app.worktree_graph_builder, &app.worktrees);
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
    let points = graph_layout(&parents, app.worktree_graph_builder, &app.worktrees);
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
    let points = graph_layout(&parents, app.worktree_graph_builder, &app.worktrees);
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

fn select_worktree_by_branch(app: &mut App, branch: &str) -> bool {
    if branch.trim().is_empty() {
        return false;
    }

    if let Some(idx) = app
        .worktrees
        .iter()
        .position(|worktree| !worktree.detached && worktree.branch == branch)
    {
        app.selected_worktree = idx;
        true
    } else {
        false
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

fn list_local_branches(path: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let output = Command::new("git")
        .args([
            "-C",
            path,
            "for-each-ref",
            "refs/heads",
            "--format=%(refname:short)",
        ])
        .output()?;

    if !output.status.success() {
        let stderr = sanitize_for_tui(String::from_utf8_lossy(&output.stderr).as_ref())
            .trim()
            .to_string();
        let stdout = sanitize_for_tui(String::from_utf8_lossy(&output.stdout).as_ref())
            .trim()
            .to_string();
        let reason = if !stderr.is_empty() { stderr } else { stdout };
        return Err(std::io::Error::other(format!("Failed to list branches: {}", reason)).into());
    }

    let mut branches = sanitize_for_tui(String::from_utf8_lossy(&output.stdout).as_ref())
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| line.to_string())
        .collect::<Vec<_>>();
    branches.sort();
    branches.dedup();
    Ok(branches)
}

fn switch_worktree_branch_at(
    worktree_path: &str,
    target_branch: &str,
    create_if_missing: bool,
) -> Result<String, Box<dyn Error>> {
    let target = target_branch.trim();
    if target.is_empty() {
        return Ok("Branch name is required".to_string());
    }

    let args = if create_if_missing {
        vec!["-C", worktree_path, "checkout", "-b", target]
    } else {
        vec!["-C", worktree_path, "checkout", target]
    };

    let output = Command::new("git").args(args).output()?;
    let stdout = sanitize_for_tui(String::from_utf8_lossy(&output.stdout).as_ref())
        .trim()
        .to_string();
    let stderr = sanitize_for_tui(String::from_utf8_lossy(&output.stderr).as_ref())
        .trim()
        .to_string();

    if output.status.success() {
        let line = if !stdout.is_empty() {
            single_line(stdout.as_str())
        } else if !stderr.is_empty() {
            single_line(stderr.as_str())
        } else if create_if_missing {
            format!("Switched to new branch '{}'", target)
        } else {
            format!("Switched to branch '{}'", target)
        };
        Ok(line)
    } else {
        let reason = if !stderr.is_empty() { stderr } else { stdout };
        if create_if_missing {
            Ok(format!(
                "Failed creating + switching to branch '{}': {}",
                target,
                single_line(reason.as_str())
            ))
        } else {
            Ok(format!(
                "Failed switching to branch '{}': {}",
                target,
                single_line(reason.as_str())
            ))
        }
    }
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
    let root_path = PathBuf::from(root.as_str());
    let root_git_arg = path_for_git_arg(root_path.as_path());
    let container = workspaces_container_for_root(root.as_str());
    if let Err(err) = fs::create_dir_all(container.as_path()) {
        return Ok(format!(
            "Failed preparing worktree container '{}': {}",
            container.to_string_lossy(),
            sanitize_for_tui(err.to_string().as_str())
        ));
    }
    let path = container.join(sanitized);
    let path_str = path_for_git_arg(path.as_path());
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
            root_git_arg.as_str(),
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
            .args([
                "-C",
                root_git_arg.as_str(),
                "worktree",
                "list",
                "--porcelain",
            ])
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

#[derive(Clone, Debug)]
struct ProposedWorktreeNode {
    branch: String,
    parent: String,
    goal: String,
}

#[derive(Clone, Debug)]
struct OrchestratedWorktreePlan {
    planner_source: &'static str,
    nodes: Vec<ProposedWorktreeNode>,
}

fn plan_orchestrated_worktrees_from_requirement(
    root: &str,
    requirement: &str,
    root_branch: &str,
    selected_branch: &str,
    existing_branches: Vec<String>,
    prompt_path: &str,
    max_nodes: usize,
) -> OrchestratedWorktreePlan {
    let opencode_enabled = command_exists_on_path("opencode");
    let mut planner_source = if opencode_enabled {
        "opencode"
    } else {
        "heuristic"
    };
    let prompt_template = load_worktree_orchestrator_prompt_template(prompt_path)
        .unwrap_or_else(default_worktree_orchestrator_prompt_template);

    let mut nodes = if opencode_enabled {
        match propose_worktree_nodes_with_opencode(
            root,
            requirement,
            root_branch,
            selected_branch,
            existing_branches.as_slice(),
            max_nodes,
            prompt_template.as_str(),
        ) {
            Ok(plan) => plan,
            Err(_) => {
                planner_source = "heuristic";
                heuristic_worktree_plan(requirement, root_branch)
            }
        }
    } else {
        heuristic_worktree_plan(requirement, root_branch)
    };

    if nodes.is_empty() {
        nodes = heuristic_worktree_plan(requirement, root_branch);
        planner_source = "heuristic";
    }

    let normalized = normalize_orchestrated_nodes(
        nodes,
        requirement,
        root_branch,
        selected_branch,
        existing_branches.as_slice(),
        max_nodes,
    );
    OrchestratedWorktreePlan {
        planner_source,
        nodes: normalized,
    }
}

fn create_worktrees_from_orchestrated_nodes(
    root: &str,
    requirement: &str,
    planner_source: &str,
    nodes: Vec<ProposedWorktreeNode>,
) -> String {
    if nodes.is_empty() {
        return "Orchestrator produced no valid worktree nodes".to_string();
    }

    let order = orchestrated_node_order(&nodes);
    let root_git_arg = path_for_git_arg(Path::new(root));
    let container = workspaces_container_for_root(root);
    let _ = fs::create_dir_all(container.as_path());

    let mut created: Vec<String> = Vec::new();
    let mut skipped: Vec<String> = Vec::new();
    let mut failed: Vec<String> = Vec::new();

    for idx in order {
        let node = nodes[idx].clone();
        if branch_exists(root, node.branch.as_str()) {
            skipped.push(format!("{} (branch exists)", node.branch));
            continue;
        }

        let path = container.join(node.branch.replace('/', "-"));
        let path_str = path_for_git_arg(path.as_path());
        if path.exists() {
            failed.push(format!("{} (path exists: {})", node.branch, path_str));
            continue;
        }

        let output = match Command::new("git")
            .args([
                "-C",
                root_git_arg.as_str(),
                "worktree",
                "add",
                "-b",
                node.branch.as_str(),
                path_str.as_str(),
                node.parent.as_str(),
            ])
            .output()
        {
            Ok(out) => out,
            Err(err) => {
                failed.push(format!(
                    "{} ({})",
                    node.branch,
                    sanitize_for_tui(err.to_string().as_str())
                ));
                continue;
            }
        };

        if output.status.success() {
            let _ = save_parent_hint(root, node.branch.as_str(), node.parent.as_str());
            let goal_suffix = if node.goal.is_empty() {
                String::new()
            } else {
                format!(": {}", node.goal)
            };
            created.push(format!("{} <- {}{}", node.branch, node.parent, goal_suffix));
        } else {
            let stderr = sanitize_for_tui(String::from_utf8_lossy(&output.stderr).as_ref())
                .trim()
                .to_string();
            let stdout = sanitize_for_tui(String::from_utf8_lossy(&output.stdout).as_ref())
                .trim()
                .to_string();
            let reason = if !stderr.is_empty() { stderr } else { stdout };
            failed.push(format!("{} ({})", node.branch, reason));
        }
    }

    let mut message = format!(
        "Orchestrated '{}' via {}: {} created, {} skipped, {} failed",
        single_line(requirement),
        planner_source,
        created.len(),
        skipped.len(),
        failed.len()
    );
    if !created.is_empty() {
        let preview = created
            .iter()
            .take(3)
            .cloned()
            .collect::<Vec<_>>()
            .join(" | ");
        message.push_str(format!(". Created: {}", preview).as_str());
    }
    if !failed.is_empty() {
        let preview = failed
            .iter()
            .take(2)
            .cloned()
            .collect::<Vec<_>>()
            .join(" | ");
        message.push_str(format!(". Failed: {}", preview).as_str());
    }
    message
}

fn build_leaf_execution_prompt(requirement: &str, node: &ProposedWorktreeNode) -> String {
    format!(
        "Implement only this leaf in branch '{}' (parent '{}'). Goal: {}. Requirement context: {}. Keep scope to this leaf, avoid cross-branch work, and finish with a concise progress note.",
        node.branch,
        node.parent,
        single_line(node.goal.as_str()),
        single_line(requirement),
    )
}

fn load_worktree_orchestrator_prompt_template(path: &str) -> Option<String> {
    if !Path::new(path).exists() {
        if let Some(parent) = Path::new(path).parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(path, default_worktree_orchestrator_prompt_template());
    }
    fs::read_to_string(path).ok()
}

fn propose_worktree_nodes_with_opencode(
    root: &str,
    requirement: &str,
    root_branch: &str,
    selected_branch: &str,
    existing_branches: &[String],
    max_nodes: usize,
    prompt_template: &str,
) -> Result<Vec<ProposedWorktreeNode>, String> {
    let existing_text = if existing_branches.is_empty() {
        "(none discovered)".to_string()
    } else {
        existing_branches
            .iter()
            .map(|branch| format!("- {}", branch))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let prompt = prompt_template
        .replace("{requirement}", requirement)
        .replace("{root_branch}", root_branch)
        .replace("{selected_branch}", selected_branch)
        .replace("{existing_branches}", existing_text.as_str())
        .replace("{max_nodes}", max_nodes.to_string().as_str());

    let output = Command::new("opencode")
        .args(["run", "--format", "json", "--dir", root, prompt.as_str()])
        .output()
        .map_err(|err| sanitize_for_tui(err.to_string().as_str()))?;

    if !output.status.success() {
        let stderr = sanitize_for_tui(String::from_utf8_lossy(&output.stderr).as_ref())
            .trim()
            .to_string();
        return Err(if stderr.is_empty() {
            "opencode run failed".to_string()
        } else {
            stderr
        });
    }

    let stdout = sanitize_for_tui(String::from_utf8_lossy(&output.stdout).as_ref());
    let mut last_text: Option<String> = None;
    for line in stdout.lines() {
        if parse_json_string_field(line, "type").as_deref() != Some("text") {
            continue;
        }
        if let Some(text) = parse_json_string_field(line, "text") {
            last_text = Some(text);
        }
    }

    let text = last_text.ok_or_else(|| "No text response from opencode planner".to_string())?;
    let nodes = parse_nodes_from_planner_text(text.as_str());
    if nodes.is_empty() {
        Err("Opencode planner returned no branch nodes".to_string())
    } else {
        Ok(nodes)
    }
}

fn parse_nodes_from_planner_text(text: &str) -> Vec<ProposedWorktreeNode> {
    let stripped = strip_markdown_fences(text);
    let node_blob = extract_json_array_field(stripped.as_str(), "nodes").unwrap_or(stripped);
    parse_top_level_json_objects(node_blob.as_str())
        .into_iter()
        .filter_map(|object| {
            let branch = parse_json_string_field(object.as_str(), "branch")?;
            let parent = parse_json_string_field(object.as_str(), "parent").unwrap_or_default();
            let goal = parse_json_string_field(object.as_str(), "goal").unwrap_or_default();
            Some(ProposedWorktreeNode {
                branch,
                parent,
                goal,
            })
        })
        .collect()
}

fn extract_json_array_field(text: &str, field_name: &str) -> Option<String> {
    let key = format!("\"{}\"", field_name);
    let key_idx = text.find(key.as_str())?;
    let mut idx = key_idx + key.len();
    skip_json_ws(text, &mut idx);
    if text.as_bytes().get(idx).copied() != Some(b':') {
        return None;
    }
    idx += 1;
    skip_json_ws(text, &mut idx);
    if text.as_bytes().get(idx).copied() != Some(b'[') {
        return None;
    }

    let bytes = text.as_bytes();
    let mut i = idx;
    let mut in_string = false;
    let mut escaping = false;
    let mut depth = 0usize;
    while i < bytes.len() {
        let ch = bytes[i];
        if in_string {
            if escaping {
                escaping = false;
            } else if ch == b'\\' {
                escaping = true;
            } else if ch == b'"' {
                in_string = false;
            }
            i += 1;
            continue;
        }

        match ch {
            b'"' => in_string = true,
            b'[' => depth = depth.saturating_add(1),
            b']' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(text[idx..=i].to_string());
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

fn strip_markdown_fences(text: &str) -> String {
    let trimmed = text.trim();
    if !trimmed.starts_with("```") {
        return trimmed.to_string();
    }
    let lines = trimmed.lines().collect::<Vec<_>>();
    if lines.len() < 3 {
        return trimmed.to_string();
    }
    lines[1..lines.len().saturating_sub(1)].join("\n")
}

fn heuristic_worktree_plan(requirement: &str, root_branch: &str) -> Vec<ProposedWorktreeNode> {
    let slug = slug_from_requirement(requirement);
    let feature_branch = format!("feature/{}", slug);
    let lower = requirement.to_ascii_lowercase();
    let auth_like = lower.contains("auth")
        || lower.contains("login")
        || lower.contains("oauth")
        || lower.contains("session");
    let api_like = lower.contains("api") || lower.contains("backend") || lower.contains("server");
    let ui_like = lower.contains("ui") || lower.contains("frontend") || lower.contains("web");

    let mut nodes = vec![ProposedWorktreeNode {
        branch: feature_branch.clone(),
        parent: root_branch.to_string(),
        goal: "feature integration lane".to_string(),
    }];

    if auth_like {
        nodes.push(ProposedWorktreeNode {
            branch: format!("{}/backend", feature_branch),
            parent: feature_branch.clone(),
            goal: "auth services and persistence".to_string(),
        });
        nodes.push(ProposedWorktreeNode {
            branch: format!("{}/frontend", feature_branch),
            parent: feature_branch.clone(),
            goal: "auth screens and client state".to_string(),
        });
        nodes.push(ProposedWorktreeNode {
            branch: format!("{}/router", feature_branch),
            parent: feature_branch,
            goal: "route guards and middleware wiring".to_string(),
        });
        return nodes;
    }

    if api_like && !ui_like {
        nodes.push(ProposedWorktreeNode {
            branch: format!("{}/api", feature_branch),
            parent: feature_branch.clone(),
            goal: "backend API surface".to_string(),
        });
        nodes.push(ProposedWorktreeNode {
            branch: format!("{}/data", feature_branch),
            parent: feature_branch,
            goal: "schema, migrations, data contracts".to_string(),
        });
        return nodes;
    }

    nodes.push(ProposedWorktreeNode {
        branch: format!("{}/frontend", feature_branch),
        parent: feature_branch.clone(),
        goal: "ui and interaction layer".to_string(),
    });
    nodes.push(ProposedWorktreeNode {
        branch: format!("{}/backend", feature_branch),
        parent: feature_branch.clone(),
        goal: "business logic and APIs".to_string(),
    });
    nodes.push(ProposedWorktreeNode {
        branch: format!("{}/router", feature_branch),
        parent: feature_branch,
        goal: "cross-cutting integration path".to_string(),
    });
    nodes
}

fn normalize_orchestrated_nodes(
    raw_nodes: Vec<ProposedWorktreeNode>,
    requirement: &str,
    root_branch: &str,
    selected_branch: &str,
    existing_branches: &[String],
    max_nodes: usize,
) -> Vec<ProposedWorktreeNode> {
    let fallback_slug = slug_from_requirement(requirement);
    let mut nodes: Vec<ProposedWorktreeNode> = Vec::new();
    let mut seen = BTreeSet::new();
    for (idx, node) in raw_nodes.into_iter().enumerate() {
        if nodes.len() >= max_nodes {
            break;
        }
        let mut branch = normalize_orchestrated_branch_name(node.branch.as_str());
        if branch.is_empty() {
            branch = format!("feature/{}/part-{}", fallback_slug, idx + 1);
        }
        while seen.contains(branch.as_str()) {
            branch.push_str("-x");
        }
        seen.insert(branch.clone());

        let parent_raw = node.parent.trim();
        let parent = if parent_raw.is_empty() {
            if selected_branch.trim().is_empty() {
                root_branch.to_string()
            } else {
                selected_branch.to_string()
            }
        } else {
            normalize_orchestrated_branch_name(parent_raw)
        };

        let goal = single_line(node.goal.as_str());
        nodes.push(ProposedWorktreeNode {
            branch,
            parent,
            goal,
        });
    }

    let mut branch_set: BTreeSet<String> = nodes.iter().map(|node| node.branch.clone()).collect();
    for branch in existing_branches {
        branch_set.insert(branch.clone());
    }
    branch_set.insert(root_branch.to_string());
    if !selected_branch.trim().is_empty() {
        branch_set.insert(selected_branch.to_string());
    }

    for node in &mut nodes {
        if node.parent.is_empty() || node.parent == node.branch {
            node.parent = root_branch.to_string();
        }
        if !branch_set.contains(node.parent.as_str()) {
            node.parent = root_branch.to_string();
        }
    }

    nodes
}

fn normalize_orchestrated_branch_name(raw: &str) -> String {
    let mut out = String::new();
    let mut prev_dash = false;
    for ch in raw.trim().chars() {
        let lower = ch.to_ascii_lowercase();
        let keep = lower.is_ascii_alphanumeric() || matches!(lower, '/' | '-' | '_' | '.');
        if keep {
            out.push(lower);
            prev_dash = false;
            continue;
        }
        if !prev_dash {
            out.push('-');
            prev_dash = true;
        }
    }
    out.trim_matches(|c| c == '-' || c == '/').to_string()
}

fn slug_from_requirement(requirement: &str) -> String {
    let normalized = normalize_orchestrated_branch_name(requirement);
    let mut parts = normalized
        .split(|c| c == '/' || c == '-')
        .filter(|part| !part.trim().is_empty())
        .take(4)
        .map(|part| part.to_string())
        .collect::<Vec<_>>();
    if parts.is_empty() {
        return "feature".to_string();
    }
    if parts.len() == 1 {
        return parts.remove(0);
    }
    parts.join("-")
}

fn orchestrated_node_order(nodes: &[ProposedWorktreeNode]) -> Vec<usize> {
    let mut branch_to_idx = BTreeMap::new();
    for (idx, node) in nodes.iter().enumerate() {
        branch_to_idx.insert(node.branch.clone(), idx);
    }

    let mut indegree = vec![0usize; nodes.len()];
    let mut children: Vec<Vec<usize>> = vec![Vec::new(); nodes.len()];
    for (idx, node) in nodes.iter().enumerate() {
        if let Some(parent_idx) = branch_to_idx.get(node.parent.as_str()).copied() {
            indegree[idx] = indegree[idx].saturating_add(1);
            children[parent_idx].push(idx);
        }
    }

    let mut ready: Vec<usize> = indegree
        .iter()
        .enumerate()
        .filter_map(|(idx, degree)| if *degree == 0 { Some(idx) } else { None })
        .collect();
    let mut out: Vec<usize> = Vec::new();

    while let Some(idx) = ready.pop() {
        out.push(idx);
        for child in &children[idx] {
            indegree[*child] = indegree[*child].saturating_sub(1);
            if indegree[*child] == 0 {
                ready.push(*child);
            }
        }
    }

    if out.len() == nodes.len() {
        out
    } else {
        (0..nodes.len()).collect()
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
        let parent = path_for_git_arg(common_abs.parent()?);
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

fn toggle_stage(app: &mut App) -> Result<(), Box<dyn Error>> {
    let item = match app.selected_item() {
        Some(entry) => entry,
        None => {
            app.status_line = "No item selected".to_string();
            return Ok(());
        }
    };

    if item.unstaged || item.untracked {
        app.status_line = run_git_in(
            app.changes_worktree_path.as_deref(),
            &["add", "--", &item.path],
        )?;
    } else if item.staged {
        app.status_line = run_git_in(
            app.changes_worktree_path.as_deref(),
            &["restore", "--staged", "--", &item.path],
        )?;
    } else {
        app.status_line = "Selected item is already clean".to_string();
    }

    Ok(())
}

fn unstage_selected(app: &mut App) -> Result<(), Box<dyn Error>> {
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
        app.status_line = "Selected item has no staged changes".to_string();
    }

    Ok(())
}

fn stage_all_changes(app: &mut App) -> Result<(), Box<dyn Error>> {
    app.status_line = run_git_in(
        app.changes_worktree_path.as_deref(),
        &["add", "--all", "--", "."],
    )?;
    Ok(())
}

fn unstage_all_changes(app: &mut App) -> Result<(), Box<dyn Error>> {
    app.status_line = run_git_in(
        app.changes_worktree_path.as_deref(),
        &["restore", "--staged", "--", "."],
    )?;
    Ok(())
}

fn stash_push_changes(app: &mut App) -> Result<(), Box<dyn Error>> {
    app.status_line = run_git_in(
        app.changes_worktree_path.as_deref(),
        &["stash", "push", "--include-untracked"],
    )?;
    Ok(())
}

fn stash_pop_changes(app: &mut App) -> Result<(), Box<dyn Error>> {
    let has_stash = git_output_in(
        app.changes_worktree_path.as_deref(),
        &["stash", "list", "--max-count=1"],
    )
    .map(|text| text.lines().any(|line| !line.trim().is_empty()))
    .unwrap_or(false);

    if !has_stash {
        app.status_line = "No stash entries to pop".to_string();
        return Ok(());
    }

    app.status_line = run_git_in(app.changes_worktree_path.as_deref(), &["stash", "pop"])?;
    Ok(())
}

fn refresh_selected_overview(app: &mut App) {
    let item = match app.selected_item() {
        Some(entry) => entry,
        None => {
            app.selected_overview = None;
            app.overview_scroll = 0;
            app.overview_method_index = 0;
            app.overview_method_expanded = false;
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

    sync_overview_method_state(app);

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
    let folder_state = build_state_label(&FileEntry {
        path: folder.path.clone(),
        staged: folder.staged,
        unstaged: folder.unstaged,
        untracked: folder.untracked,
    });
    let mut total_files = 0usize;
    let mut total_untracked = 0usize;
    for file in files {
        if !(file.path == folder.path || file.path.starts_with(prefix.as_str())) {
            continue;
        }
        total_files += 1;
        if file.untracked {
            total_untracked += 1;
        }
    }

    let should_skip_preview = should_skip_untracked_preview(folder.path.as_str())
        || total_files > 240
        || total_untracked > 180;
    if should_skip_preview {
        let mut traditional_diff = Vec::new();
        if total_files > 0 {
            traditional_diff.push(DiffPreviewLine {
                kind: DiffPreviewKind::Meta,
                text: format!("preview suppressed for {} files", total_files),
            });
        }
        return FileOverview {
            file: format!("{}/", folder.path),
            state: folder_state,
            added_lines: 0,
            removed_lines: 0,
            methods_added: Vec::new(),
            methods_modified: Vec::new(),
            methods_deleted: Vec::new(),
            method_changes: Vec::new(),
            traditional_diff,
            use_traditional_overview: true,
        };
    }
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
        state: folder_state,
        added_lines: total_added,
        removed_lines: total_removed,
        methods_added,
        methods_modified,
        methods_deleted,
        method_changes: Vec::new(),
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
    let mut method_changes: Vec<MethodChange> = Vec::new();
    let mut traditional_diff: Vec<DiffPreviewLine> = Vec::new();

    if file.untracked {
        let file_path = repo_path
            .map(|base| Path::new(base).join(file.path.as_str()))
            .unwrap_or_else(|| PathBuf::from(file.path.as_str()));
        if should_skip_untracked_preview(&file.path) || is_probably_binary_path(&file.path) {
            added_lines = 0;
            methods_added = Vec::new();
            let note = if is_probably_binary_path(&file.path) {
                "(binary file; preview suppressed)"
            } else {
                "(preview suppressed)"
            };
            traditional_diff = preview_for_untracked(note);
        } else {
            let text = read_untracked_preview_text(file_path.as_path());
            let line_count = text.lines().count();
            added_lines = line_count;
            methods_added = sorted_from_set(collect_methods_from_content(&text, &file.path));
            traditional_diff = preview_for_untracked(&text);
        }
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
        method_changes = summary.method_changes;
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
        method_changes,
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
    method_changes: Vec<MethodChange>,
    diff_preview: Vec<DiffPreviewLine>,
}

fn summarize_diff(diff: &str, file_path: &str) -> DiffSummary {
    let mut added_methods: HashSet<String> = HashSet::new();
    let mut removed_methods: HashSet<String> = HashSet::new();
    let mut modified_hunks: HashSet<String> = HashSet::new();
    let mut diff_preview: Vec<DiffPreviewLine> = Vec::new();
    let mut method_hunks: BTreeMap<String, Vec<DiffPreviewLine>> = BTreeMap::new();
    let mut current_hunk_lines: Vec<DiffPreviewLine> = Vec::new();
    let mut current_hunk_methods: HashSet<String> = HashSet::new();
    let mut current_hunk_method: Option<String> = None;
    let mut added_lines = 0usize;
    let mut removed_lines = 0usize;

    for line in diff.lines() {
        if line.starts_with("@@") {
            flush_method_hunk(
                &mut method_hunks,
                current_hunk_method.as_deref(),
                &current_hunk_methods,
                &mut current_hunk_lines,
            );
            current_hunk_methods.clear();
            current_hunk_method = parse_hunk_header(line)
                .and_then(|header| extract_method_name(header.as_str(), file_path));
            if let Some(name) = current_hunk_method.as_ref() {
                current_hunk_methods.insert(name.clone());
            }
            current_hunk_lines.push(DiffPreviewLine {
                kind: DiffPreviewKind::Meta,
                text: truncate_text(line, 96),
            });
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
                current_hunk_methods.insert(name.clone());
                added_methods.insert(name);
            }
            if !current_hunk_lines.is_empty() {
                current_hunk_lines.push(DiffPreviewLine {
                    kind: DiffPreviewKind::Added,
                    text: truncate_text(line, 96),
                });
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
                current_hunk_methods.insert(name.clone());
                removed_methods.insert(name);
            }
            if !current_hunk_lines.is_empty() {
                current_hunk_lines.push(DiffPreviewLine {
                    kind: DiffPreviewKind::Removed,
                    text: truncate_text(line, 96),
                });
            }
            push_preview_line(&mut diff_preview, DiffPreviewKind::Removed, line);
            continue;
        }

        if !current_hunk_lines.is_empty() {
            current_hunk_lines.push(DiffPreviewLine {
                kind: DiffPreviewKind::Context,
                text: truncate_text(line, 96),
            });
        }
        push_preview_line(&mut diff_preview, DiffPreviewKind::Context, line);
    }

    flush_method_hunk(
        &mut method_hunks,
        current_hunk_method.as_deref(),
        &current_hunk_methods,
        &mut current_hunk_lines,
    );

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

    let methods_added = sorted_from_set(methods_added_set);
    let methods_modified = sorted_from_set(methods_modified_set);
    let methods_deleted = sorted_from_set(methods_deleted_set);
    let method_changes = build_method_changes(
        &methods_added,
        &methods_modified,
        &methods_deleted,
        &method_hunks,
    );

    DiffSummary {
        added_lines,
        removed_lines,
        methods_added,
        methods_modified,
        methods_deleted,
        method_changes,
        diff_preview,
    }
}

fn flush_method_hunk(
    store: &mut BTreeMap<String, Vec<DiffPreviewLine>>,
    method: Option<&str>,
    fallback_methods: &HashSet<String>,
    hunk_lines: &mut Vec<DiffPreviewLine>,
) {
    if hunk_lines.is_empty() {
        return;
    }
    if let Some(name) = method {
        let entry = store.entry(name.to_string()).or_default();
        entry.extend(hunk_lines.iter().cloned());
    } else {
        for name in fallback_methods {
            let entry = store.entry(name.clone()).or_default();
            entry.extend(hunk_lines.iter().cloned());
        }
    }
    hunk_lines.clear();
}

fn build_method_changes(
    added: &[String],
    modified: &[String],
    deleted: &[String],
    hunk_map: &BTreeMap<String, Vec<DiffPreviewLine>>,
) -> Vec<MethodChange> {
    let mut out = Vec::new();
    append_method_changes(&mut out, MethodChangeKind::Added, added, hunk_map);
    append_method_changes(&mut out, MethodChangeKind::Modified, modified, hunk_map);
    append_method_changes(&mut out, MethodChangeKind::Deleted, deleted, hunk_map);
    out
}

fn append_method_changes(
    out: &mut Vec<MethodChange>,
    kind: MethodChangeKind,
    names: &[String],
    hunk_map: &BTreeMap<String, Vec<DiffPreviewLine>>,
) {
    for name in names {
        out.push(MethodChange {
            kind: kind.clone(),
            name: name.clone(),
            diff_lines: hunk_map.get(name).cloned().unwrap_or_default(),
        });
    }
}

fn sync_overview_method_state(app: &mut App) {
    let method_count = app
        .selected_overview
        .as_ref()
        .map(|overview| overview.method_changes.len())
        .unwrap_or(0);
    if method_count == 0 {
        app.overview_method_index = 0;
        app.overview_method_expanded = false;
        return;
    }
    if app.overview_method_index >= method_count {
        app.overview_method_index = method_count - 1;
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

fn read_untracked_preview_text(path: &Path) -> String {
    let metadata = match fs::metadata(path) {
        Ok(meta) => meta,
        Err(_) => return String::new(),
    };

    if metadata.len() > 256 * 1024 {
        return "(file preview capped at 256KB)".to_string();
    }

    let mut file = match fs::File::open(path) {
        Ok(file) => file,
        Err(_) => return String::new(),
    };

    let mut buffer = Vec::new();
    if file.read_to_end(&mut buffer).is_err() {
        return String::new();
    }

    let text = String::from_utf8_lossy(&buffer).to_string();
    text
}

fn should_skip_untracked_preview(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    let has_segment =
        |needle: &str| lower.starts_with(needle) || lower.contains(&format!("/{needle}"));

    has_segment("node_modules/")
        || has_segment(".pnpm/")
        || has_segment(".yarn/")
        || has_segment("target/")
        || has_segment("dist/")
        || has_segment("build/")
        || has_segment(".next/")
        || has_segment(".git/")
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

fn git_result_text(result: Result<String, Box<dyn Error>>) -> String {
    match result {
        Ok(text) => text,
        Err(err) => sanitize_for_tui(err.to_string().as_str()),
    }
}

fn start_git_task<F>(
    app: &mut App,
    label: &str,
    refresh_worktrees: bool,
    refresh_status: bool,
    task: F,
) where
    F: FnOnce() -> String + Send + 'static,
{
    if app.git_task.is_some() {
        let queued = QueuedGitTask {
            label: label.to_string(),
            refresh_worktrees,
            refresh_status,
            task: Box::new(task),
        };
        app.git_task_queue.push_back(queued);
        let n = app.git_task_queue.len();
        app.status_line = format!("Queued '{}' ({} pending)", label, n);
        return;
    }

    spawn_git_task(app, label, refresh_worktrees, refresh_status, task);
}

fn spawn_git_task<F>(
    app: &mut App,
    label: &str,
    refresh_worktrees: bool,
    refresh_status: bool,
    task: F,
) where
    F: FnOnce() -> String + Send + 'static,
{
    let label_text = label.to_string();
    app.git_task = Some(GitTaskState {
        label: label_text.clone(),
        started_at: Instant::now(),
    });
    app.status_line = format!("{}...", label_text);

    let tx = app.git_task_tx.clone();
    thread::spawn(move || {
        let outcome = task();
        let _ = tx.send(GitTaskEvent {
            label: label_text,
            outcome,
            refresh_worktrees,
            refresh_status,
        });
    });
}

fn pop_next_git_task(app: &mut App) {
    if let Some(queued) = app.git_task_queue.pop_front() {
        spawn_git_task(
            app,
            &queued.label,
            queued.refresh_worktrees,
            queued.refresh_status,
            queued.task,
        );
    }
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

fn git_output_with_error_in(path: Option<&str>, args: &[&str]) -> Result<String, String> {
    let mut cmd = Command::new("git");
    if let Some(path) = path {
        cmd.args(["-C", path]);
    }

    match cmd.args(args).output() {
        Ok(output) if output.status.success() => Ok(sanitize_for_tui(
            String::from_utf8_lossy(&output.stdout).as_ref(),
        )),
        Ok(output) => {
            let stderr = sanitize_for_tui(String::from_utf8_lossy(&output.stderr).as_ref())
                .trim()
                .to_string();
            let stdout = sanitize_for_tui(String::from_utf8_lossy(&output.stdout).as_ref())
                .trim()
                .to_string();
            if !stderr.is_empty() {
                Err(stderr)
            } else if !stdout.is_empty() {
                Err(stdout)
            } else {
                Err(format!("git {} failed", args.join(" ")))
            }
        }
        Err(err) => Err(sanitize_for_tui(err.to_string().as_str())),
    }
}

fn git_output_with_error(args: &[&str]) -> Result<String, String> {
    git_output_with_error_in(None, args)
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
