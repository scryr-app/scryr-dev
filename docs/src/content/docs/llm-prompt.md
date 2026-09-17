---
title: Generate index.scry with an LLM
description: Give humans a reviewable diagram of agent-written software with a repository-grounded Scryr manifest.
---

AI agents can create software faster than a reviewer can reconstruct its runtime shape from a diff. A generated `index.scry` gives that reviewer a typed, explorable account of the system: its components, connections, owners, deployment targets, source links, and operational evidence.

Use a coding agent that can read your repository. Paste the prompt below; it asks the agent to inspect real runtime boundaries before authoring and to avoid fabricated architecture. The resulting `.scry` file belongs in the same review as the code it describes.

```text title="Prompt of manifest conjuration"
You are working inside my software repository. Examine the repository thoroughly and create an accurate Scryr architecture manifest at index.scry.

Goal
Represent the architecture that actually exists so humans and coding agents can explore it with Scryr. Do not invent services, dependencies, owners, integrations, versions, or links that the repository does not support.

Scryr model
- A .scry file is executable Python.
- Import public models from scryr: Manifest, Diagram, Info, Link, Forge, and ManifestQuery as needed.
- Import enums and value types from scryr.types when useful.
- Each real deployable, application, datastore, queue, worker, external integration, or important infrastructure boundary may become one public Manifest variable.
- Set a stable manifest_id for each important component, using repository-relative identities such as "apps/web" or "services/catalog-api".
- Use the exact Manifest.name in connection references: connections=[Manifest(name="Exact Name")].
- Add at least one public Diagram. A Diagram may use an explicit manifests list or a ManifestQuery, never both.
- Prefer a small number of useful diagrams over one enormous undifferentiated view.

Investigation
1. Read the main README and contribution/build instructions.
2. Inspect workspace/package manifests, lockfiles, containers, infrastructure, CI workflows, deployment config, API schemas, and executable entrypoints.
3. Identify runtime boundaries and data/control flow, not just folders.
4. Derive languages, frameworks, deployment targets, ownership, auth, observability, documentation links, and source repositories only from evidence.
5. Notice background workers, scheduled jobs, message brokers, databases, third-party APIs, and CI/CD paths.

Authoring rules
- Keep the file readable and typed. Reuse component variables in Diagram.manifests.
- Add Info(description=...) that explains responsibility and interactions in one or two precise sentences.
- Add tags for meaningful filtering, not decorative labels.
- Evidence sections are typed lists named repository, checks, metrics, tests, dependencies, and performance.
- Import concrete integrations from scryr.collectors and place them directly in the matching list. For example: repository=[GitStatusCollector()] or tests=[PytestCollector()].
- Use GitHubPullRequestCollector/GitHubActionsCollector in repository only when the canonical owner/repository is known. Remote CI never belongs in checks.
- Dependencies displays inventory, licenses, and vulnerabilities equally; use SyftInventoryCollector, GrantLicenseCollector, and GrypeScanCollector with typed SbomRef/LicensePolicy.
- Metrics uses OpenMetricsCollector(endpoint=...) to scrape instrumented application/exporter metrics, without provider-query configuration.
- Never invent test counts, security findings, performance scores, or timestamps in source. Collector construction is inert; test/check/benchmark execution defaults to manual.
- Preserve stable manifest_id and section-scoped collector IDs; do not add old section wrappers, static summary fields, or a manifest-wide collectors list.
- Use supported Scryr enums when you can verify them; omit uncertain optional fields rather than guessing.
- Model directed connections according to the actual dependency or call direction used by the repository.
- If the repository is a monorepo, group components logically and consider multiple diagrams for system context, runtime flow, and operations.
- Do not include secrets, credentials, internal tokens, or sensitive environment values.

Validation
- If the scryr command is available, run: scryr format; scryr check
- Fix every formatting, lint, typing, execution, missing-reference, duplicate-identifier, and diagram-rule error.
- If scryr is unavailable, still check that the file is valid Python and that every connection name matches a declared Manifest.

Deliverable
Create or update only index.scry unless a supporting local module is clearly necessary for readability. Then summarize:
- the components and diagrams created,
- the strongest repository evidence used,
- any uncertain areas intentionally omitted,
- whether scryr check passed.

Begin by inspecting the repository; do not ask me to describe architecture that can be derived from the code.
```

:::caution[Review before execution]
A `.scry` manifest is Python. Inspect agent-written code before running it, especially in an unfamiliar repository.
:::

That review boundary is intentional: Scryr lets an agent do the high-volume repository investigation while leaving a plain-text, typed artifact for a human to verify.

After the agent finishes:

```sh
scryr check
scryr serve --watch
```

## Next Steps

You've reached the end of the introductory guides. The pages that follow are **Developer Docs**: technical guides and references for writing manifests, using the CLI, generating code, and connecting Scryr to your development tools.

Start with the [Manifest language](/manifests/), or jump to the [CLI reference](/cli/), [code generation](/code-generation/), [integrations](/integrations/), or [editor highlighting](/editors/).
