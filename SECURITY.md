# Security Policy

## Supported Versions

Scryr is currently pre-1.0. Security fixes are applied to the main branch unless
a release branch is explicitly announced.

## Reporting a Vulnerability

If you find a vulnerability, please open an issue and be sure to include:

- Affected component: `manifest`, `crystal`, `map`, or deployment tooling.
- A clear description of the issue and impact.
- Reproduction steps or proof-of-concept details.
- Whether credentials, tokens, or tenant data may be exposed.

The project will acknowledge credible reports, investigate, and publish fixes
or mitigations before public disclosure when practical.

## Secrets

Do not commit local `.env`, `mise.local.toml`, deployment secrets, Clerk secret
keys, database URLs, OAuth tokens, or CLI auth state.
