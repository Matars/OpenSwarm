Resolve the current Git merge conflict in this worktree.

Context:
- Parent worktree path: {parent_path}
- Merge source branch: {source_branch}
- Merge target branch: {target_branch}
- Conflicted files:
{conflicted_files}

Instructions:
1) Inspect conflict markers and resolve carefully; prefer minimal safe edits.
2) Keep intended behavior from both branches when possible.
3) Run `git diff --name-only --diff-filter=U` and ensure it is empty.
4) Stage resolved files with `git add`.
5) Summarize what was resolved and any risks.
6) Do not push. Stop after conflicts are resolved and staged.
