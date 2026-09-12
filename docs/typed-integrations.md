# Typed integrations and cards

Declare a provider once, then explicitly list the views you want in each manifest.
Each authentication, integration, card, and typed manifest gets its ID from its
public Python variable name. Do not supply IDs, provider strings, or card-type
strings. Renaming a variable changes its identity, including the identity used
for stored operational history. Existing manifests with legacy sections retain
explicit `manifest_id` support during migration.

```python
from scryr import (
    CardCategory, Diagram, Grafana, GrafanaAuthentication, GrafanaPerformance,
    GrafanaPerformanceQueries, GrafanaUptime, GrafanaUptimeQueries, Manifest,
    MetricUnit, PostHog, PostHogAuthentication, PostHogPerformance, PrometheusQuery,
)

grafana_authentication = GrafanaAuthentication()
posthog_authentication = PostHogAuthentication()
grafana = Grafana(
    authentication=grafana_authentication,
    endpoint="https://your-prometheus-endpoint",
)
posthog = PostHog(authentication=posthog_authentication, project_id=123)

grafana_performance = GrafanaPerformance(
    integration=grafana,
    queries=GrafanaPerformanceQueries(
        cpu_current=PrometheusQuery(expression="YOUR_PROMQL", unit=MetricUnit.percent),
    ),
)
posthog_performance = PostHogPerformance(integration=posthog)
grafana_uptime = GrafanaUptime(
    integration=grafana,
    queries=GrafanaUptimeQueries(
        uptime=PrometheusQuery(expression="avg(up)", unit=MetricUnit.ratio),
    ),
)
api = Manifest(
    name="API",
    integrations=[grafana, posthog],
    cards=[grafana_performance, posthog_performance, grafana_uptime],
    card_categories=[CardCategory.performance, CardCategory.uptime, CardCategory.cicd],
)
diagram = Diagram(name="System", manifests=[api])
```

`posthog_performance` above deliberately has no queries yet. It displays setup
guidance rather than fabricated values. See the complete configurable example in
[`manifest/examples/typed_integrations/index.scry`](../manifest/examples/typed_integrations/index.scry).
Adapt expressions to the measurements your systems actually collect.

## Provider capabilities

| Integration | Concrete view | Category | Query configuration |
| --- | --- | --- | --- |
| `Grafana` | `GrafanaPerformance` | Performance | `GrafanaPerformanceQueries` (CPU and memory) |
| `Grafana` | `GrafanaMetrics` | Metrics | `GrafanaMetricsQueries` (requests, latency, errors) |
| `Grafana` | `GrafanaUptime` | Uptime | `GrafanaUptimeQueries` (availability) |
| `PostHog` | `PostHogPerformance` | Performance | `PostHogPerformanceQueries` (web vitals and latency) |
| `PostHog` | `PostHogAnalytics` | Analytics | `PostHogAnalyticsQueries` (views, users, sessions, conversions) |
| `GitHubActions` | `GitHubActionsPipeline` | CI/CD | Repository, optional workflow ID and branch |

The integration's category membership is derived from its concrete type. An
integration can be included with no views. Multiple views can share an integration,
and multiple providers can supply views in the same category. Future CI providers
can add concrete integration/view types to the union without redefining CI/CD.

`GrafanaPerformance` only accepts `Grafana`, and its queries only accept CPU and
memory fields. `PostHogPerformance` only accepts `PostHog`, with `PostHogQuery`
aggregates requiring `{start}`, `{end}`, and `{environment}` placeholders.
Use `MetricUnit` for units and `MetricWindow` with `datetime.timedelta` for sampling
windows, steps, caching, and ingestion delay. These settings are bounded by the
same limits used by the server. A view can override its default `title`.

Assign each declaration to a unique public variable before referencing it. Aliases,
anonymous nested declarations, duplicate list entries, and references to integrations
outside `Manifest.integrations` are rejected. Wire-format IDs and derived sources
are validated when deserializing artifacts; the source loader assigns identities.

## Category navigation and setup

The tray selects a category across blocks. Each category supports multiple views;
use Previous/Next on a card to select another view. Selection follows the card's
variable identity when its list is reordered. `card_categories` controls visible
categories and their order on each block. Omit it to show all supported categories.

An empty visible category displays **Please Set Up** and the relevant available
integrations. The link opens category-specific configuration guidance. A configured
view with no observations also offers setup guidance. Loading, partial data, stale
data, errors, and numeric zero remain distinct.

Existing `github`, `metrics`, `analytics`, `cicd`, `tests`, `dependencies`, and
`performance` sections are still rendered by the compatibility path. Explicit
provider views replace the corresponding generic metric/CI views on typed manifests.
Repository metadata and declared test/dependency reports remain available.

## TOML credentials

Save credentials in `scryr.secrets.toml` beside `index.scry`. Add this filename
to your project's `.gitignore`; this repository already ignores it. All public endpoints, project IDs, queries, and display settings
remain in `index.scry`; authentication declarations contain no secret values.

```toml
[authentication.grafana_authentication]
username = "YOUR_GRAFANA_USER"
token = "YOUR_GRAFANA_TOKEN"

[authentication.posthog_authentication]
api_key = "YOUR_POSTHOG_PERSONAL_API_KEY"

[authentication.github_authentication]
token = "YOUR_GITHUB_TOKEN"
```

Table names must match authentication variable names. Each provider has an exact
schema: Grafana requires username/token, PostHog requires api_key, and GitHub
requires token. Unknown or empty fields fail validation. Use `SCRYR_SECRETS_FILE`
to select another file explicitly.

`scryr check` checks TOML syntax, credential schemas, and references in addition to
formatting, lint, types, execution, and diagram rules. It makes no provider requests
and never includes credential values or TOML source excerpts in errors. Missing
credentials fail for declared authenticated integrations; empty category placeholders
need no credentials. The same validation runs before push, export, and local editor
saves. Secrets are never included in generated artifacts or browser editor files.

Local serving approves destinations from the natively checked `index.scry` source.
`scryr serve --watch` rechecks source and credential file changes. Editor saves also
refresh the approved declarations. Runtime requests keep the existing bounded cache.
A rejected edit leaves the previous diagram and its destination approvals intact.

Hosted servers require organization-specific credentials and explicit destination
approvals. These operator restrictions protect credential routing; they do not
replace the integration configuration in `index.scry`:

```toml
[organizations.org_example.authentication.grafana_authentication]
username = "YOUR_GRAFANA_USER"
token = "YOUR_GRAFANA_TOKEN"

[organizations.org_example.connections.grafana_authentication]
endpoint = "https://your-prometheus-endpoint"

[organizations.org_example.authentication.posthog_authentication]
api_key = "YOUR_POSTHOG_PERSONAL_API_KEY"

[organizations.org_example.connections.posthog_authentication]
endpoint = "https://us.posthog.com"
project_id = 123
```

Use `--clerk-org-id org_example` when checking for that organization. Hosted runtime
requests resolve only that organization's credentials. For local `--server-only`
operation without a source workspace, equivalent `[connections.<authentication>]`
tables provide explicit destination approvals. Restart hosted/server-only servers
after changing their credential file. Use distinct authentication declarations for
different destinations.

## Querying and reporting

```sh
scryr query --list
scryr query cpuCurrent --manifest api --card grafana_performance
scryr report actions --manifest api --card github_pipeline --event-file event.json
```

Query listings include the card ID. `--card` disambiguates repeated query names
within a block. Runtime results are keyed by block and card IDs; sharing a query
can share its cached fetch without sharing a card's UI state.

A pipeline references a `GitHubActions(repository="owner/repository")` integration.
Its optional `GitHubAuthentication` resolves a TOML token for automatic branch
selection. Specify `branch` to report without a GitHub API lookup. The reporter
validates the event repository against the selected integration. Each displayed
pipeline filters durable history by repository, workflow, and branch. Existing
explicit-ID reporting and `GITHUB_TOKEN` fallback remain supported for compatibility.

## Migrate existing connection JSON

```sh
scryr migrate-secrets old-connections.json --output scryr.secrets.toml
```

The migration writes a new file with private permissions and refuses to overwrite
an existing output. It preserves credentials and endpoint/project approvals and
leaves the JSON input intact. PostHog connections are identified by their existing
project ID. Set `SCRYR_SECRETS_FILE`, remove `SCRYR_METRICS_CONNECTIONS_FILE`, match
authentication variable names to the generated TOML tables, and run `scryr check`.
Normal operation only reads TOML.
