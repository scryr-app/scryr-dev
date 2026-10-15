---
title: Integrations
description: Connect Scryr to source documentation, GraphQL, and developer tooling.
---

Scryr keeps the architecture declaration in `index.scry`, with links to supporting documentation and exports for developer tooling.

## Source and documentation links

Add the canonical repository and useful documentation or operational links to a manifest:

```python
from scryr import Info, Link, Manifest

api = Manifest(
    name="Public API",
    info=Info(
        docs=["https://docs.acme.test/api"],
        links=[
            Link(site_name="Source", http_url="https://github.com/acme/api"),
            Link(site_name="Runbook", http_url="https://ops.acme.test/api"),
        ],
    ),
)
```

These links appear with the block, keeping navigation from architecture to implementation and operations direct.

## GraphQL API

The diagram reads canonical blocks, nested diagrams, source documents through Scryr's GraphQL API. Local defaults are:

- UI: `http://127.0.0.1:8000/`
- GraphQL and WebSocket: `http://127.0.0.1:8000/graphql`
- Playground: `http://127.0.0.1:8000/playground`
- Liveness: `http://127.0.0.1:8000/health`
- Readiness: `http://127.0.0.1:8000/ready`

## mise, Docker Compose, and devcontainers

Model a developer environment once as a `Forge`, then export:

```sh
scryr export mise --forge "Developer environment" > mise.toml
scryr export compose --forge "Developer environment" > compose.yaml
scryr export devcontainer --forge "Developer environment" > devcontainer.json
```

The outputs are generated from the same typed declaration that components reference on the diagram.
