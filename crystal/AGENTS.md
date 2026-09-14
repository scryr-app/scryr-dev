# Crystal CLI and server

- Read `README.md`; CLI dependencies flow to server/core, server to core.
- Core owns canonical GraphQL types, persistence, and pure artifact generation.
  HTTP/auth/schema wiring and root resolvers belong in server; command parsing
  and runtime orchestration belong in CLI. Reuse shared models rather than
  introducing parallel transport DTOs.
- Dependency versions and lint policy belong in the workspace `Cargo.toml`.
- Coordinate GraphQL changes with map operations and generated client types.
- Run `mise run verify:crystal` from the repository root; it refreshes the embedded
  manifest SDK. Review generated changes with the integration owner.
- Use `mise run release:build` when testing the standalone embedded UI; direct
  Cargo builds do not prepare fresh frontend assets.
