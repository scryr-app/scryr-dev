# Map UI

- Read `README.md` for GraphQL, source editor, and theme contracts.
- Edit operations in `src/graphql/**/*.graphql`; regenerate with
  `mise run contribute:generate:graphql`. Never edit `src/graphql/generated.ts`.
- Coordinate schema generation with the integration owner and ensure the running
  server comes from the intended code revision.
- `src/pyodide/sdkSources.generated.json` comes from `manifest/scryr`. Include its
  regenerated update when changing the SDK, through the integration owner.
- Preserve editor drafts and revision/conflict behavior when changing UI state.
- Run `mise run verify:map` from the repository root. For source editor workflows,
  also run `mise run verify:editor` when relevant; see README for prerequisites.
