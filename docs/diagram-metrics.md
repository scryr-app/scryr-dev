# Metrics on diagram load

Grafana/Prometheus metrics are fetched by Crystal only when the frontend opens a
configured diagram. The `diagramMetrics` GraphQL query is separate from `blocks`;
the existing 30-second block polling does not query Grafana. Browser focus,
reconnection, and timers do not trigger collection. Offline editor previews never
resolve connections or fetch metrics.

## Manifest configuration

Northwind's `index.scry` declares `Metrics(provider=PrometheusSource(...))` for
`northwind-commerce/api`. It includes the production environment, Grafana dashboard
link, five PromQL expressions, units, and these timing settings (all in seconds):

- `window=900`: 15 minutes of samples.
- `step=60`: one-minute resolution.
- `ingestion_delay=120`: query a window ending two minutes ago.
- `cache_ttl=60`: reuse results for closely spaced diagram loads.
- `refresh="on_diagram_load"`: the only supported refresh mode.

Use `CredentialRef(name="northwind-grafana-read")` for the server connection.
`query_endpoint` may be explicitly declared in `index.scry`; when omitted,
Crystal resolves it from that approved connection. If supplied, it must exactly
match the server-approved endpoint. URLs and query expressions are public
manifest data. Tokens are not.

Queries must aggregate to a single series. The adapter rejects ambiguous
multiple-series responses, provider warnings, malformed responses, and oversized
results. Non-finite values and missing recent samples are unavailable, never
fabricated zeros. Latency queries explicitly convert seconds to milliseconds.
CPU and memory remain unavailable until Northwind exports those measurements.
The source query map can include `cpuCurrent`, `cpuAvg`, `cpuPeak`, and
`memoryUsage` later, with units declared in `units`.

## Server connection

Create a private JSON file outside source control, readable only by the server
user. Its outer keys are authenticated Scryr organization IDs, not Grafana IDs:

```json
{
  "local-dev-org": {
    "northwind-grafana-read": {
      "endpoint": "https://YOUR-METRICS-HOST/prometheus",
      "username": "YOUR-METRICS-INSTANCE-ID",
      "token": "YOUR-METRICS-READ-TOKEN"
    }
  }
}
```

Copy the exact Prometheus **query endpoint** from Grafana Cloud's connection
details, not the dashboard URL, OTLP endpoint, or remote-write URL. Use a separate
Cloud access policy token with `metrics:read`. The Northwind ingest token only
has write scopes and should not be reused. See
[Grafana's query API documentation](https://grafana.com/docs/grafana-cloud/observe-and-act/send-data/metrics/metrics-prometheus/query-http-api/).

Start/restart the Scryr server with:

```bash
SCRYR_METRICS_CONNECTIONS_FILE=/absolute/private/path/metrics-connections.json \
  ./crystal/target/debug/scryr serve --auth-mode local --port 8001
```

Keep any existing `SCRYR_SQLITE_PATH` setting when restarting. Connection settings
are read at startup. Do not put the token in `index.scry`, a browser variable, or a
CLI argument. Credentials never go to the browser. Crystal refuses redirects and
uses the approved endpoint from the server file. HTTP is permitted only for an
explicitly configured loopback test backend.

The live Northwind query endpoint and read credential have not yet been supplied.
The manifest is ready, but the production source will show **Unavailable** until
this connection is configured.

## Results and lifecycle

Architecture renders immediately; metric cards load separately. Responses contain
values, units, bounded sample history, evaluation timestamps, a fetch timestamp,
and a Grafana link pinned to the queried window. Prometheus evaluation timestamps
are not raw exporter sample timestamps. The adapter accepts only recent finite
query results, but the freshness of underlying telemetry also depends on the
PromQL expression and the backend's lookback behavior.

Concurrent diagram loads share an in-flight request for the same organization,
manifest identity, and complete source configuration. Successes and failures are
cached for the declared TTL. A failed refresh retains previous values with a
**Stale snapshot** label and the original fetch timestamp; no previous snapshot
means **Unavailable**. A successful empty result means **No data**. Missing some
queries produces **Partial data**. Refreshing a diagram can query again after TTL
expiry; TTL expiry alone never starts a request.

The cache is bounded to 256 source configurations, with idle entries eligible for
removal after two hours. It is process-local: restarting clears cached snapshots,
and separate server replicas have independent caches. This is on-demand display
data, not the durable test/deployment event ledger. The metric definitions remain
in the manifest and survive uploads/regeneration.

## Verification

- Python round-trip tests preserve credential references and source settings.
- Rust tests verify authentication, per-organization connections, endpoint
  approval, concurrent deduplication, TTL behavior, failure caching, stale
  snapshots, missing data, and malformed/ambiguous results.
- Frontend tests verify opening/switching diagrams triggers collection while
  ordinary rerenders, unconfigured diagrams, and offline previews do not.
- The Northwind end-to-end check uses its actual manifest with an explicitly
  configured local mock backend. Repeated `blocks` reads make zero backend calls;
  opening the diagram makes five queries; a cached reopen plus more block reads
  makes no additional queries. This does not claim a live Grafana connection.

Run the end-to-end check after building the CLI:

```bash
python3 docs/examples/diagram-metrics-demo.py
```

It starts a temporary Scryr server on port 8002 and stops both test servers when
it finishes. Its database and observed snapshot are under
`.scryr/runtime-metrics-verification/`.

## Real data and missing observations

The UI renders only explicitly recorded observations and provider query results.
Missing repository statistics, CI status, deployments, latency, and CPU history
remain unavailable; architecture metadata never generates operational numbers.
Zero is displayed only when it is actually reported. Manifest versions are not
shown as GitHub releases, and missing success rates are not inferred from errors.

Local server startup does not seed bundled maps or select MERN by default. The
map picker lists persisted maps only, and the Python editor starts empty until
source is loaded. An empty database returns no maps and an explicit missing
artifact error for block queries. Test fixtures remain confined to test runs;
the synthetic reporting demo requires an explicit disposable server endpoint via
`SCRYR_DEMO_GRAPHQL_URL`.

The existing local Northwind databases were backed up under
`.scryr/real-data-backups/` before removing known demo observations and unchanged
bundled sample maps. Real unit, coverage, and Docker integration reports remain.
