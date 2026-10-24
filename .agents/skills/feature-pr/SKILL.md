---
name: feature-pr
description: Implement a feature or fix in an isolated Git worktree, provide full-application startup instructions, and create a concise pull request using consistent feature-first naming. Use when taking a code change through the worktree-to-PR workflow.
---

# Feature PR

Default to a dedicated worktree for each task. Follow the repository's contribution,
commit, and validation conventions. An explicit request to use this workflow for
implementation includes committing the task changes, pushing its branch, and
opening a PR; creating or editing this skill alone does not authorize those actions.
Honor narrower requests such as local-only work or PR drafting.

## Worktree and branch

- Read the repository's `AGENTS.md`, `CONTRIBUTING.md`, and applicable component
  instructions. Inspect status, remotes, existing worktrees, and branch names.
  Preserve existing uncommitted work; do not stash, reset, move, or overwrite it.
- Determine the intended PR base from repository conventions and task context.
  Fetch the base when available; do not assume the current feature branch is the
  right starting point. If fetching fails, disclose the stale-base risk.
- Choose a short lowercase kebab-case feature name, preferably 1–3 words, and
  a brief change slug. Use branch `<feature>/<change>` and worktree directory
  `<feature>--<change>`, for example `diagram-search/keyboard-navigation` and
  `diagram-search--keyboard-navigation`. Start both with the feature name;
  do not prepend author, tool, `agent`, `codex`, or `chatgpt` identifiers.
- Prefer a sibling directory outside the source checkout, under
  `<checkout-parent>/<repo>-worktrees/<feature>--<change>`. Respect a user-selected
  worktree root. If the name exists for unrelated work, append `-2`, `-3`, etc.
  to the change slug in both names. Reuse a matching task worktree on resumption.
- Create it with `git worktree add -b <branch> <worktree-path> <base-ref>`.
  Perform subsequent edits, setup, and checks from that worktree. Serialize Git
  mutations across linked worktrees. Do not change Git identity or configuration.
- If necessary starting changes exist only in a dirty checkout, establish how
  they belong to this task before copying them; never silently omit or relocate
  the user's work. Respect an explicit request to work in the existing checkout.

## Implement and run the whole application

Use the repository's existing setup, build, and validation entry points. Discover
the full stack from its maintained documentation and task definitions rather than
guessing commands. Install dependencies in the worktree. Keep its database,
runtime state, and generated outputs isolated; do not symlink another checkout's
database or copy secrets into tracked files.

For Scryr, read [references/scryr.md](references/scryr.md) and verify its commands
against the current checkout. For other projects, derive equivalent instructions.

Always provide copyable commands with the actual worktree path for setup and
starting the entire application, the UI/API URLs, required environment settings,
and how to stop it. Cover every required service, not only the frontend. Check
port occupancy before starting services; do not kill another checkout's processes.
Use supported port overrides when available, otherwise report the conflict.
Only claim the stack was tested if it was actually started and checked.

Run relevant checks and the repository's required integrated verification.
Review generated changes and retain only intentional outputs. If checks fail,
fix task-related failures; disclose remaining failures and blockers accurately.

## Commit and open the PR

- Review the diff against the selected base and stage only task-owned changes.
  Use the repository's commit convention; if none exists, use a short imperative
  subject about the change. Do not manufacture a human author or alter identity.
- Never add statements that a PR or commit was made by an agent, AI, Codex, or
  ChatGPT. Do not add generation notices, assistant signatures, or AI co-author
  trailers to titles, bodies, commit messages, or authored PR comments. Keep the
  text about actual feature behavior and code. Preserve required existing legal
  sign-offs and do not rewrite unrelated history or metadata.
- Push the task branch without force. Check for an existing PR for that head/base
  before creating one; update it on resumption. Prefer an available GitHub
  connector; use `gh` when its capabilities are needed. If authentication or a
  push fails, retain the local work and prepared PR text and report the blocker.
- Use a concise feature-focused title. Begin the body with the user-visible
  result, then briefly explain the code changes. For internal changes, state the
  developer or operational effect without inventing a user-facing benefit.
  Follow the repository template while preserving this order where possible.
  Usually one sentence per section is enough:

  ```markdown
  ## Summary
  Users can navigate diagram search results with the keyboard.

  ## Code changes
  Add active-result state and keyboard handlers to the search panel.

  ## Validation
  <Checks actually run and their results.>
  ```

- Include material limitations briefly. Use a draft PR when unfinished or blocked
  by failed required checks, and explain the status. Keep descriptions current
  with the final implementation; omit conversational history and abandoned work.
  When using `gh`, supply multiline text through a body file.
- Do not merge, deploy, publish releases, delete the branch, or remove the
  worktree unless requested. Retain task work through integration.

## Handoff

Return the PR link (or blocker), branch, worktree path, changed files, validation
results, outstanding integration issues, and exact full-application startup
commands. Distinguish tested behavior from instructions that were only inspected.
