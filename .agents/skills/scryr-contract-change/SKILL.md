---
name: scryr-contract-change
description: Coordinate Scryr changes to Python manifest serialization, Rust GraphQL types, or generated frontend contracts across manifest, crystal, and map. Use for producer-consumer contract changes, not isolated UI or implementation edits.
---

# Scryr contract changes

Read root/component AGENTS.md and CONTRIBUTING.md. Run `mise run verify:worktree`
before writes. Resolve the repository root with Git; commands below run there.

Trace the actual producer and consumers before splitting work:

- Python public models and serialization originate in `manifest/scryr`.
- Rust core owns GraphQL models and the generated manifest envelope; server owns
  schema wiring and resolvers; CLI orchestrates manifest execution.
- Map GraphQL operations consume server types. Browser Python executes a generated
  SDK bundle from the same Python source.

State the intended field/type behavior, defaults, and compatibility for existing
samples or stored data. Give each writing agent a dedicated worktree and explicit
file ownership. Assign one integration owner for generation and shared files.
Parallelize consumers only once their shared contract is concrete.

Implement source changes and meaningful producer/consumer coverage. Integrate the
source changes into one worktree before refreshing derived artifacts:

- `mise run contribute:generate:graphql` for GraphQL operations/types.
- `mise run contribute:generate:manifest-types` for manifest field metadata.
- Map development/build refreshes `map/src/pyodide/sdkSources.generated.json`;
  include that file when the SDK changes.
- Crystal checks/release builds refresh the embedded SDK. Follow CONTRIBUTING.md
  on bundled assets and exclude unrelated build churn.

Generation may reuse a running server on port 8000. Verify that it belongs to the
integration worktree and includes the new schema before generating. Do not run
another default stack concurrently or terminate an unowned process.

Run affected `verify:manifest`, `verify:crystal`, and `verify:map` suites during
development, then `mise run verify` after integration. For editor save/load
contracts, also use `mise run verify:editor` and its documented prerequisites.
Report compatibility decisions, generated files, validation results, and any
unresolved consumer mismatch.
