# Repository maintenance

## Free public-repository CI

Scryr is public. Every checked-in workflow uses standard GitHub-hosted
`ubuntu-24.04` runners. Standard public-repository runner compute is free; larger
runners and external deployment services are outside this setup. Do not switch
to paid runner labels. [GitHub billing documentation](https://docs.github.com/en/billing/concepts/product-billing/github-actions).

PRs, main pushes, and manual CI runs check workflows/secrets, Python, Rust, and the
frontend, then build and smoke-test the complete CLI. Workflow jobs have timeouts,
use immutable action pins and read-only tokens, and install only required tools.
CI does not need repository secrets. External-contributor runs require maintainer
approval. No deployment workflow is configured.

Normal CI uploads no artifacts. Repository logs/artifacts default to seven days;
release handoffs expire after one day. Cache storage is capped at the included
10 GB repository limit. Watch artifact usage;
free runner compute is not permission to enable paid storage or larger runners.
Do not increase cache limits. A repository-specific Actions budget is set to $0
with paid usage stopped at the limit. It applies only to `scryr-app/scryr-dev`;
other repositories' budgets are unchanged.

`main` retains pull-request, signed-commit, linear-history, code-owner, deletion,
and force-push protections. The required CI checks are:

- Workflow and secret checks
- Python manifest
- Rust server and CLI
- Frontend map
- Release CLI smoke

CodeQL default setup scans GitHub Actions, Python, and JavaScript/TypeScript using
standard runners. GitHub secret scanning and push protection complement the
Gitleaks history scan in CI. `.gitleaksignore` contains two exact historical
fingerprints for reviewed public test fixtures, not file/directory exclusions.
Review each proposed exception; do not broadly suppress scans.

## Dependabot and security notices

Dependabot checks Actions, npm, Cargo, uv, and Docker weekly. Minor/patch version
updates are grouped; major upgrades remain individually reviewable. Security
updates are enabled independently of that weekly schedule, with no auto-merge.
Review bot PRs using the same CI and ownership requirements as other changes.

The Python dependency-graph workflow reads all third-party packages from the
checked-in `manifest/uv.lock`, including transitive and development dependencies,
and submits them without installing or executing package code. It only writes
snapshots from `main`; fork PRs do not receive a write token. Cargo/npm lockfiles
are also recognized by GitHub. Check update logs after changing workspace layout.

Maintainers with repository access receive GitHub security notifications according
to their personal preferences. Enable email/web Dependabot notifications and a
weekly digest in your own GitHub settings; repository YAML cannot change another
person's notification preferences. Triage critical/high alerts promptly and other
alerts weekly. [Notification settings](https://docs.github.com/en/code-security/how-tos/secure-your-supply-chain/manage-your-dependency-security/configure-dependabot-notifications).

The setup audit found upstream advisories in `rustls-webpki` (through libsql),
`jsonwebtoken` (through clerk-rs), and `libsql-sqlite3-parser`. Keep these alerts
open until a compatible upgrade or reviewed remediation resolves them. One parser
advisory has no published patched version. CI readiness does not certify absence
of runtime vulnerabilities; consult the live Security tab before a release.

## Releases

1. Align package versions and lockfiles, refresh the embedded SDK, and merge a PR
   with all required checks green.
2. A repository administrator creates a `vX.Y.Z` tag on a commit in `main`.
3. The release workflow validates tag/version consistency, runs the complete CI,
   and produces a Linux x86_64 archive containing the tested binary and license
   notices with a SHA-256 checksum file.
4. A separate job verifies the handoff and creates a draft GitHub Release. Review
   the notes, test installation on a clean machine, then publish the draft.
   Checksum validation detects corruption; it is not a provenance attestation.
5. If a run fails after creating a draft, reconcile the existing draft before
   retrying. The workflow does not silently overwrite existing releases.

No PyPI/crates.io publishing or cloud deployment is enabled. Do not create a
release tag merely to test a contributor's PR.

## Routine upkeep

Update pinned tool versions/checksums in `.github/scripts/check-workflows.sh` when
new actionlint or Gitleaks versions are adopted. Dependabot manages action pins,
but does not update these CLI checksums or `mise.lock`. Run the complete CI after
toolchain changes. Maintain notices for redistributed dependencies and branding.
