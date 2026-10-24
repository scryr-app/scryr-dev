# Run Scryr from a worktree

Read the worktree's current `CONTRIBUTING.md` and `mise.toml` before using these
commands. Run all mise tasks from that worktree's checkout root.

Replace the path below with the actual created path in the handoff:

```bash
cd /absolute/path/to/scryr-dev-worktrees/diagram-search--keyboard-navigation
mise install
mise run contribute:setup
mise run contribute:oss
```

`contribute:oss` starts both the Rust API with reload and the Vite frontend with
HMR. Open `http://127.0.0.1:3000`; GraphQL is at
`http://127.0.0.1:8000/graphql`. Ctrl+C stops the stack. Default OSS mode uses
local auth and worktree-local SQLite/runtime state in `.cache/oss`, with no cloud
credentials required. Do not share that directory with another checkout.

Run only one default stack at a time because ports 8000 and 3000 are fixed by
the standard workflow. If occupied, identify the existing stack and report the
conflict; do not terminate unrelated processes automatically.

To populate sample diagrams, run from another terminal at the same worktree root:

```bash
mise run contribute:seed:samples
```

The docs site is separate and optional: `mise run contribute:docs`.
Use `contribute:cloud` only when the task calls for cloud behavior; follow the
current contribution guide for development credentials and isolated cloud state.

Run relevant component checks while implementing. After integrating application
changes, run `mise run verify`. Automation-only changes use
`mise run verify:automation`; documentation-only changes use the relevant
documentation checks. `pre-commit` applies broad fixes and requires an exclusively
owned checkout. Inspect the resulting diff for unintended generated changes.
