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
2. Integrate the matching server schema source into the same checkout.
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

Vercel Production and Preview builds require `VITE_SCRYR_AUTH_MODE=clerk`,
`VITE_CLERK_PUBLISHABLE_KEY`, and an absolute HTTPS `VITE_GRAPHQL_ENDPOINT`.
The prebuild check rejects incomplete hosted configuration instead of silently
shipping local authentication. Rebuild after changing these variables because
Vite embeds them in the frontend bundle. The Clerk publishable key must belong
to the same instance as the server's `CLERK_SECRET_KEY`.

Run the configuration regression checks with
`node --test scripts/check-hosted-config.test.mjs` from `map/`.

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

## Diagram themes

The palette button selects a complete theme:

- **Industrial Forest** includes Light Mode, the original ribbed blocks and matte cards, daylight, and the original camera view.
- **Scholarly Grimoire** includes Dark Mode, gilt leather covers, parchment cards tinted to their blocks with ivory text, warm reading light, brass connections, and walnut floorboards.
- **Luminous Crystal** includes Dark Mode, dark crystal materials, layered glowing edges, violet connections, a subdued grid, and an elevated camera view.
- **Celestial Observatory** includes Dark Mode, midnight-blue precision instruments, brushed-brass orbital engravings, starlight, and a slowly moving celestial highlight.
- **Porcelain Reverie** includes Light Mode, glazed ivory ceramic, cobalt botanical borders, delicate gold details, and kintsugi cracks.
- **Sunken Sanctuary** includes Dark Mode, carved submerged stone, weathered copper, translucent sea-glass cards, and drifting underwater caustics.
- **Velvet Oracle** includes Light Mode, champagne velvet with a soft fabric sheen, ivory cards, ethereal gold light, and Greek-inspired laurel and Aegean accents.

Each preset in `src/theme/theme.ts` owns its mode and palette. Its typed
`ThemeAppearance` in `src/theme/appearance.ts` defines textures, face and card
shapes, materials, lighting, floor, regions, connections, and view. Add a preset
and appearance definition to extend the chooser; mode is never stored separately.

Selection updates immediately and persists in `selectedTheme`. Legacy palette and
`diagramMode` settings resolve to the corresponding complete theme. Changing themes
rebuilds the 3D scene while keeping the editor, its draft, and map selection mounted.

The Pyodide console follows each theme with coordinated syntax colors and surfaces. Console text uses 97% opacity (3% transparency).

Drag the console’s top bar to reposition it. Its controls remain clickable, and dragging and resizing keep the panel within the viewport.

## Local evidence cards

Repository, Checks, Metrics, Tests, Dependencies, and Performance are backed by
`Block.evidence` typed GraphQL observations. A section appears only when its typed
collector list is present in `index.scry`; the list order controls collector pages.
Repository contains Git state, pull requests, and remote workflow runs. Checks
contains local verification tasks. Metrics displays CLI-collected OpenMetrics
samples without a browser-side provider connection.

Each card distinguishes collector lifecycle from findings, preserves the last
result with stale/outdated indicators, and opens a keyboard-accessible details
dialog. Dependencies has inventory, license, and vulnerability panels with filters
and server-paginated rows from immutable observation IDs. Polling reads only
summary counts; details are loaded on demand. Package names are joined only to a matching inventory hash,
workspace, and environment. Unknown licenses and incomplete scans remain explicit.
Details include observation provenance, paginated run history, and the exact local
`scryr collect run` command. Reading a card never executes a command. The map polls
passive results every five seconds while visible; editor preview remains inert.

The editor derives public SDK constructors and their typed parameters from the
same generated Python bundle used to execute previews. Update the SDK source and
regenerate its bundle rather than maintaining a separate completion schema.

Local editor writes require the process capability supplied by the embedded UI.
The transport sends it only to the same loopback origin, in a request header.
Opening the standalone `scryr serve` address enables local applies; a Vite preview
without that capability can read and evaluate declarations but cannot save a
command-bearing source file. Hosted stored-source editing remains independent.
