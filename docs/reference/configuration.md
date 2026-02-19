# Reference: Configuration

## Config Directory

- `~/.config/openswarm`

## Files

- `~/.config/openswarm/config.toml`
- `~/.config/openswarm/prompts/conflict-resolve-prompt.md`

## `config.toml`

Default shape:

```toml
# OpenSwarm config
# default_agent accepts: "", "opencode", "claude"
default_agent = ""

# relative paths are resolved from ~/.config/openswarm
conflict_resolve_prompt = "prompts/conflict-resolve-prompt.md"
```

## Keys

- `default_agent`
  - Empty string means do not auto-launch on picker flow.
  - Supported values: `opencode`, `claude`.
- `conflict_resolve_prompt`
  - Absolute path, or relative path resolved from `~/.config/openswarm`.

## Prompt Placeholders

The conflict prompt template supports:

- `{parent_path}`
- `{source_branch}`
- `{target_branch}`
- `{conflicted_files}`
