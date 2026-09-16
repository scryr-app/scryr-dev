# Working in scryr-dev

Read `CONTRIBUTING.md` before changing code. Run mise commands from the checkout
root. Read the component's `AGENTS.md` before editing that component, even when
your session starts at the root.

## Worktree requirement

- Only create a worktree if I ask you to
- Before editing, run `mise run verify:worktree`. If it fails, create or select a
  linked worktree and continue there. Do not bypass the check or reset, stash,
  move, or overwrite the user's uncommitted work to make it pass.
- Launch CLI sessions with `mise run contribute:agent` in the linked worktree.
  For app sessions, select Worktree mode and run the same preflight.
- Each concurrently writing agent needs its own worktree. Read-only research or
  review agents may share a checkout. A subagent does not imply isolation: give
  it an explicit checkout path and require the preflight before writes. If the
  runtime cannot isolate writers, use one writer and read-only helpers.
- Worktrees share Git refs and configuration. Do not change global/repository
  Git configuration, remove another task's worktree, or run concurrent Git
  mutations in the same checkout. Keep task work until integration is complete.

## Collaboration and architecture

- Use parallel agents for bounded, independent work when useful. The coordinator
  defines shared contracts first, assigns file ownership, and owns integration.
- `manifest/`: Python SDK and samples; `crystal/`: Rust CLI, core, and server;
  `map/`: React/Three.js UI; `docs/`: Astro documentation.
- Assign a single integration owner for `mise.toml`, dependency lockfiles,
  cross-component contracts, and generated/bundled files. Other agents request
  changes to these through the coordinator.
- For contract changes, use `.agents/skills/scryr-contract-change/SKILL.md`.
- Separate worktrees isolate files, not network ports or external services.
  Current development tasks use ports 8000 and 3000; run only one default stack
  at a time. Do not share SQLite files or cloud state between test runs.

## Validation and handoff

- Use existing mise tasks; implementations belong in `mise.toml`.
- Run relevant component checks during implementation. After integrating
  application changes, run `mise run verify`. Automation-only changes use
  `mise run verify:automation`; documentation-only changes use the relevant
  documentation checks.
- `pre-commit` applies broad fixes; run it only in an exclusively owned checkout.
  Builds/checks can regenerate assets. Review the diff and include only intended
  generated changes; never hand-edit generated code.
- Return the worktree/branch, changed files, tests and results, and outstanding
  integration issues. Do not merge, publish, or deploy unless requested.
