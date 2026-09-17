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
