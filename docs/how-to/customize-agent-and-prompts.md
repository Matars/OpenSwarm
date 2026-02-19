---
title: Customize Agent and Prompts
parent: How-To Guides
nav_order: 4
---

# How to Customize Agent and Prompts

OpenSwarm reads runtime settings from `~/.config/openswarm`.

## Config File

- Path: `~/.config/openswarm/config.toml`
- Key `default_agent`: `""`, `"opencode"`, or `"claude"`
- Key `conflict_resolve_prompt`: path to your prompt template

## Prompt Template

- Default location:
  - `~/.config/openswarm/prompts/conflict-resolve-prompt.md`
- Placeholders:
  - `{parent_path}`
  - `{source_branch}`
  - `{target_branch}`
  - `{conflicted_files}`

## In-App Editing

- During conflict confirm mode, press `e` to edit the prompt.
- Save with `Ctrl+S`, close with `Esc`.

## Agent Detection

OpenSwarm detects available agents from `PATH`:

- `opencode`
- `claude`
