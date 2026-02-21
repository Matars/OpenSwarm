---
title: Configuration
nav_order: 4
---

# Configuration

## Config directory

```
~/.config/openswarm/
```

Created automatically on first run.

## config.toml

```toml
# ~/.config/openswarm/config.toml

# Agent to auto-launch with `O`. Leave empty to show the picker.
# Supported: "opencode", "claude"
default_agent = ""

# Path to the conflict resolution prompt template.
# Relative paths resolve from ~/.config/openswarm/
conflict_resolve_prompt = "prompts/conflict-resolve-prompt.md"

# Feature-to-worktree orchestrator toggle and planner prompt.
worktree_orchestrator_enabled = true
worktree_orchestrator_prompt = "prompts/worktree-orchestrator-prompt.md"
worktree_orchestrator_max_nodes = 8

# Optional art box shown above the worktree details panel.
# Supports ASCII or Unicode.
worktree_graph_art = """
  .-""""-.   .-""""-.
 /  .--.  \ /  .--.  \
|  /    \_/\_/    \  |
| |  o    . .    o  | |
| |      ( ^ )      | |
 \ \   .`-.-`.   / /
  `._`--(_____)--`_.`
"""
```

### `default_agent`

Controls what happens when you press `O` on a worktree node:

| Value | Behavior |
|-------|----------|
| `""` (empty) | Shows a picker if multiple agents are detected |
| `"claude"` | Launches Claude directly |
| `"opencode"` | Launches OpenCode directly and tries to resume the latest matching OpenCode session for that worktree |

OpenSwarm auto-detects available agents by scanning your PATH for `claude` and `opencode`. If the configured default isn't found, it falls back to the picker.

OpenCode session resume only applies to this default-launch path (`default_agent = "opencode"`). Manually choosing OpenCode from the picker starts a normal OpenCode launch.

### `conflict_resolve_prompt`

Path to a Markdown template used when launching agent-assisted merge conflict resolution. Can be absolute or relative to `~/.config/openswarm/`.

A default template is created at `~/.config/openswarm/prompts/conflict-resolve-prompt.md` on first run.

### `worktree_graph_art`

Multiline string rendered in a dedicated `art` panel above `details` in worktree view.

If this key is missing from an existing `config.toml`, OpenSwarm appends the default ASCII block automatically on startup.

- Supports ASCII and Unicode glyphs.
- Keep line width short for narrow terminals (the panel truncates overflow).
- You can also set it as a single-line string and use `\n` escapes.

### `worktree_orchestrator_enabled`

Controls whether the `g` action in worktree view can run automated feature planning and batch worktree creation.

| Value | Behavior |
|-------|----------|
| `true` | `g` opens the orchestration flow and can call OpenCode planner |
| `false` | `g` shows disabled status and does not create nodes |

### `worktree_orchestrator_prompt`

Path to a Markdown prompt template used for feature planning with OpenCode (`opencode run --format json`). Can be absolute or relative to `~/.config/openswarm/`.

A default template is created at `~/.config/openswarm/prompts/worktree-orchestrator-prompt.md` on first run.

### `worktree_orchestrator_max_nodes`

Hard cap on how many worktree nodes are created from a single orchestration request.

- Valid range is clamped to `1..=24`.
- Default is `8`.

**Template placeholders:**

| Placeholder | Replaced with |
|-------------|---------------|
| `{parent_path}` | Filesystem path of the parent worktree |
| `{source_branch}` | Branch being merged |
| `{target_branch}` | Branch being merged into |
| `{conflicted_files}` | List of files with merge conflicts |

Orchestrator template placeholders:

| Placeholder | Replaced with |
|-------------|---------------|
| `{requirement}` | Raw feature requirement typed in modal |
| `{root_branch}` | Resolved root branch (`main`/`master`) |
| `{selected_branch}` | Currently selected branch in canvas |
| `{existing_branches}` | Existing branch list at planning time |
| `{max_nodes}` | Current configured max nodes |

You can edit the prompt template inline during conflict resolution by pressing `e` in the confirmation dialog.

---

## Workspace layout

Worktrees are created in a sibling directory:

```
your-repo/                    # Main repository
.<repo-name>-workspaces/      # Created by OpenSwarm
  feature-auth/               # Worktree for feature-auth branch
  fix-pagination/             # Worktree for fix-pagination branch
```

Branch name slashes are replaced with hyphens for filesystem safety (e.g., `feature/auth` becomes `feature-auth`).

## Parent hints

OpenSwarm tracks parent-child worktree relationships in a `.parent-hints` file inside the workspaces directory. This is a TSV file mapping child branches to parent branches. It's used to build the graph hierarchy and determine merge targets.

Parent relationships are also inferred from branch name hierarchy (e.g., `feature/auth` is treated as a child of `feature` if both exist).

## Notes file

The notes editor (`n` key) reads and writes `notes.md` in the repository root. Created automatically if it doesn't exist.

## Agent detection

On startup and when launching agents, OpenSwarm checks your PATH for:

- `opencode`
- `claude`

No additional configuration is needed -- if the binary is on your PATH, it appears in the agent picker.
