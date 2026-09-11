# 🗺️ Map

Frontend for the Scryr architecture map.

## What It Does

- renders the 3D map UI
- fetches block data from `crystal` over GraphQL
- displays regions, lines, labels, and detail cards


## GraphQL Workflow

`Map` uses:
- `graphql-request` for the GraphQL client
- `@tanstack/react-query` for fetching and caching
- `graphql-codegen` to generate TypeScript types and query hooks

The generated client code lives in `src/graphql/generated.ts`. Do not edit that
file directly; it is overwritten on each codegen run.

When you add or change GraphQL operations:

1. Update `src/graphql/**/*.graphql`.
2. Make sure the GraphQL server is available at the endpoint configured by `mise`.
3. Run `mise run contribute:generate:graphql`.
4. Import the generated hooks in your components.

Example:

```tsx
import { useGetBlocksQuery } from "@/graphql/generated";

export function Example() {
  const { data, isLoading, error } = useGetBlocksQuery();

  if (isLoading) return <div>Loading...</div>;
  if (error) return <div>Failed to load.</div>;

  return <pre>{JSON.stringify(data?.blocks, null, 2)}</pre>;
}
```

Relevant files:
- `src/graphql/queries.graphql`: GraphQL operations
- `src/graphql/client.ts`: GraphQL client setup
- `src/graphql/generated.ts`: generated types and hooks
- `codegen.yml`: codegen configuration

## Local Defaults

- dev server: `http://localhost:3000`
- GraphQL endpoint: `http://localhost:8000/graphql`
- auth mode: `local`

These defaults come from the root `mise.toml`.

Personal overrides should go in untracked `mise.local.toml` at the repository
root.
Set `VITE_SCRYR_AUTH_MODE=clerk` only when building or testing a hosted
Clerk-backed frontend.

## Source editor

The Python console loads the selected diagram's stored source using
`manifestDocument`, runs the actual `manifest/scryr` SDK in a fresh Pyodide worker,
and commits through `saveManifestDocument`. Revisions protect against lost updates;
all diagrams from one entrypoint are saved together. Successful saves invalidate
the diagram list and blocks cache immediately. Organization changes discard the
previous editor session and cancel its Python worker.

`scryr serve` injects a same-origin `/graphql` runtime configuration into its
embedded UI, so custom ports work without rebuilding. Only the registered local
entrypoint can be written. Cloud mode edits stored snapshots, not Git or local disk.

Regression checks:

```bash
mise run verify:editor
```

This builds the standalone binary, copies it outside the checkout, and uses
Chrome to test local disk saves, reload/restart, invalid/conflicting drafts,
stored-source saves, and `--watch`. Chrome and access to jsDelivr are required.
Development and builds refresh `src/pyodide/sdkSources.generated.json` from
`../manifest/scryr` when the full checkout is available. Commit this generated
file when changing the SDK. Standalone builds (including Vercel) use the checked-in
bundle; the SDK is never read from disk at runtime.
Frontend state tests also run in the normal `npm test` suite; Rust source-storage
tests exercise both SQLite and libSQL transactions and organization isolation.
