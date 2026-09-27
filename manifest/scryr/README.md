# Scryr

Scryr is a Python package for defining architecture manifests as typed Pydantic
models. Applications can use it as a library.

## GitHub Actions history

Select workflows on a component to populate its CI/CD card through the installed
OSS CLI and your existing GitHub CLI login:

```python
from scryr import CICD, ActionsReportSource, Diagram, Github, Manifest

api = Manifest(
    manifest_id="services/api",
    name="Public API",
    github=Github(repo_url="https://github.com/acme/api"),
    cicd=CICD(
        platform="github_actions",
        source=ActionsReportSource(workflows=["ci.yml", "integration.yaml"], branch="main"),
    ),
)
diagram = Diagram(name="System", manifests=[api])
```

Run `gh auth login` once, then `scryr serve`. The server collects configured
providers immediately and every 300 seconds. Use `--poll 60` to change the interval;
valid intervals range from 15 through 3600 seconds, and bare `--poll` uses 300.
Use `--no-poll` to disable collection. Local credential polling runs only with
local authentication; `--server-only` does not start collectors.
Collection stops with the server and requires no project script or scheduler.

`workflows` contains filenames from `.github/workflows`, without paths. Use the
existing positive numeric `workflow_id` instead when preferred; it cannot be
combined with a nonempty `workflows` list. An omitted branch selects the repository
default. Polling requires the stable `manifest_id`, repository URL, and explicit
workflow selection; event-report declarations containing only `jobs_file` or
`branch` remain valid and do not enable polling.

The CLI initially imports ten recent runs per workflow, catches up across up to
100 recent runs per workflow on later polls, and refreshes observed active attempts.
It imports complete job snapshots (up to 2,000 jobs per attempt) into
Scryr's durable history. To collect once into a running server, use
`scryr sync github --manifest api --endpoint http://127.0.0.1:8000/graphql`.
Repeated observations are deduplicated; missed intermediate states are not
invented. Authentication stays with `gh` and is never serialized into manifests.

Existing `scryr report actions --manifest api --event-file workflow-run.json`
reporting and the optional `jobs_file` declaration continue to work. Source
declarations and collected history remain separate, so collection does not rewrite
the Python manifest. See the [CLI reference](../../crystal/crystal-cli/README.md)
for source selection and reporting options.

### GitHub dependency inventory and security

Use `Dependencies(source=GithubDependencySource())` on a block with a stable
`manifest_id` and `github.repo_url` (or `repo_url`). Import both classes from
`scryr`. `scryr serve --poll 300` and `scryr sync github` then collect GitHub's
repository-wide dependency inventory and open Dependabot alerts through your
existing `gh` login. Both components default to enabled; set `inventory=False`
or `security=False` to collect only one. The selected repository's default
branch is used, independently of any Actions branch selection.

Inventory and security have independent polling/retry and freshness states.
Unavailable or incomplete results never become zero vulnerabilities. Last
successful snapshots remain available, clearly marked as last known after a
failure; stale observations are labeled after two hours. The diagram shows
package counts, versions and licenses where provided, security severity,
affected packages, and remediation links/patched versions. Counts apply to the
whole repository, including when several blocks subscribe to it. Outdated
versions and license compliance are not inferred. GitHub feature availability
and repository contents/Dependabot-alert read permissions still apply.
