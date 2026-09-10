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
