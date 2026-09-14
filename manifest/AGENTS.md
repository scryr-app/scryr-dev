# Manifest SDK

- Read `README.md` and `scryr/README.md` for the public Python API.
- Edit the SDK in `manifest/scryr`, not its embedded Rust or browser copies.
- `.scry` files execute Python. Preserve importer behavior and test with trusted
  samples under `tests/samples` when changing imports or serialization.
- Coordinate serialized envelope/field changes with the Rust consumer and map
  metadata. The integration owner refreshes bundled SDK assets.
- Run `mise run verify:manifest` from the repository root. Use
  `mise run verify:test:integration:manifest` for sample execution feedback.
