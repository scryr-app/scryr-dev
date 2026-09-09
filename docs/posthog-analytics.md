# PostHog diagram analytics

Northwind's `index.scry` declares a `PostHogSource` on `northwind-web`:
24-hour event counts, production filtering, two-minute ingestion delay, 60-second
cache, labels, units, and the `northwind-posthog-read` credential reference.
Payment intents are split into Stripe and stubbed counts; neither means paid orders.

Add this connection alongside Grafana in the private JSON file named by
`SCRYR_METRICS_CONNECTIONS_FILE` (replace placeholders with your real settings):

```json
{
  "local-dev-org": {
    "northwind-posthog-read": {
      "endpoint": "https://us.posthog.com",
      "project_id": 12345,
      "token": "YOUR_PERSONAL_API_KEY"
    }
  }
}
```

Use your actual PostHog application host (US, EU, or self-hosted), not the event
ingestion host. The personal API key needs Query Read access to the project.
Keep this file outside version control. Restart Scryr after changing connections.
For authenticated deployments use the actual Scryr organization ID.

Once known, also record `project_id`, `query_endpoint`, and `dashboard_url` on
`_NORTHWIND_ANALYTICS` in Northwind's `index.scry`. The server rejects project IDs
or endpoints that differ from the private connection. No project ID or dashboard
URL was invented for the current repository; the private connection can resolve
its project ID until these public details are supplied.

Build the deployed Northwind frontend with `VITE_POSTHOG_ENVIRONMENT=production`
and its existing PostHog project ingestion token. Untagged historical events are
excluded. The default environment is `local`, even for a local production build;
only explicitly tagged production traffic enters this diagram's aggregates.
The sanitizer writes the configured environment and strips caller-supplied values.

Opening the diagram calls `diagramMetrics` once. Its `northwind-commerce/web`
entry contains an `analytics` snapshot independent of Grafana. Ordinary `blocks`
polling, focus, and reconnect do not fetch analytics. Reopening within the cache
TTL reuses the snapshot. Open the repository card group to see the Analytics card.
The card displays aggregate counts, environment, requested UTC window, fetch time,
and an optional PostHog link. No person records or event payloads reach the UI.

Queries must return one numeric cell. `{start}` and `{end}` become Unix seconds;
`{environment}` becomes an escaped SQL string literal. Every query must contain
all three placeholders. Null/empty results remain missing; numeric zero is real
zero. HTTP errors, incomplete results, unexpected shapes, and provider-cached
responses never turn into zero. An expired successful snapshot may remain visible
as explicitly stale if refresh fails. The API requests `force_blocking` execution
so the returned result is computed for the requested window, with an eight-second
request timeout and one-megabyte response limit.

Verification:

- Run `python docs/examples/diagram-metrics-demo.py` against an isolated fixture
  server: three block reads produce zero upstream calls, opening the diagram
  produces five Grafana queries and six PostHog queries, and a cached reopen plus
  more block reads produce no additional calls. This never writes fixture values
  into the normal local server database.
- For live validation, configure the real connection, reload the Northwind diagram,
  and inspect `diagramMetrics` in browser DevTools. Compare its event counts with
  PostHog using the displayed UTC window and environment filter.

API reference: https://posthog.com/docs/api/queries
