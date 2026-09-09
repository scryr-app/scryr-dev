# Security Policy

## Supported Versions

Scryr is currently pre-1.0. Security fixes are applied to the main branch unless
a release branch is explicitly announced.

## Reporting a Vulnerability

Report vulnerabilities through [private vulnerability reporting](https://github.com/scryr-app/scryr-dev/security/advisories/new).
Do not post exploit details or credentials in public issues. Include:

- Affected component: `manifest`, `crystal`, `map`, or deployment tooling.
- A clear description of the issue and impact.
- Reproduction steps or proof-of-concept details.
- Whether credentials, tokens, or tenant data may be exposed.

The project will acknowledge credible reports, investigate, and publish fixes
or mitigations before public disclosure when practical.

## Secrets

Do not commit local `.env`, `mise.local.toml`, deployment secrets, Clerk secret
keys, database URLs, OAuth tokens, or CLI auth state.

## Local development mode

`AUTH_MODE=local` grants all requests a shared writable development identity.
Bind local development servers to loopback. Use authenticated mode for shared
instances. `.scry` manifests are executable Python: run only trusted manifests.

## Dependency alerts

Dependabot alerts and security updates are enabled. Maintainers review new
critical/high alerts promptly and other alerts monthly. Upstream advisories that
have no compatible fix remain open and visible until resolved; do not dismiss
them merely to make the dashboard green.
