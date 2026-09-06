# Third-Party Notices

This project relies on the amazing work of other logo repositories. Please
ensure you follow their legal notices and consider supporting their efforts.

This file tracks third-party materials that are redistributed in this
repository in addition to the package dependencies outlined below.

## Package Dependencies

Runtime and development dependencies are declared in:

- `manifest/pyproject.toml` and `manifest/uv.lock`
- `crystal/Cargo.toml` and `crystal/Cargo.lock`
- `map/package.json` and `map/package-lock.json`

Local dependency license metadata was reviewed from the checked-in lockfiles and
available package metadata:

- Python dependencies in `manifest/uv.lock`: the Scryr package is MIT licensed.
  Installed metadata for the resolved third-party packages includes MIT,
  Apache-2.0 OR BSD-2-Clause, BSD-2-Clause, and PSF-2.0 license declarations.
- Rust dependencies in `crystal/Cargo.lock`: `cargo metadata` reports the
  workspace crates as MIT licensed. Third-party crates declare licenses and
  license expressions including MIT, Apache-2.0, MIT OR Apache-2.0, BSD
  variants, ISC, Unicode-3.0, Zlib, CDLA-Permissive-2.0, Unlicense OR MIT,
  MPL-2.0, and expressions that include BSL-1.0 or the LLVM exception.
- npm dependencies in `map/package-lock.json`: package metadata is mostly MIT,
  with additional declarations including Apache-2.0, ISC, BSD-2-Clause,
  BSD-3-Clause, MPL-2.0, BlueOak-1.0.0, CC-BY-4.0, CC0-1.0, EPL-2.0,
  Python-2.0, 0BSD, and MIT/Apache-2.0 expressions.

## Icons And Images

The repository includes first-party Scryr branding assets and redistributed
third-party icon/logo SVGs in these locations:

- `map/src/icons/`
- `crystal/crystal-server/static/map/favicon/`
- `crystal/crystal-server/static/map/*.png`
- `crystal/crystal-server/static/map/*.jpg`

### Devicon

Some redistributed icon SVGs are sourced from Devicon:

- Source: https://devicon.dev/
- Repository: https://github.com/devicons/devicon
- License: MIT
- Copyright: Copyright (c) 2015 konpa

Devicon's project materials state that product names, logos, and brands are the
property of their respective owners and that use of those names, logos, and
brands does not imply endorsement. Users are responsible for following the
applicable brand policies for those marks.

### SVG Logos

Some redistributed logo SVGs are sourced from SVG Logos:

- Source: https://svglogos.dev/
- Repository: https://github.com/gilbarbara/logos
- License: CC0-1.0

SVG Logos' project materials state that all logos appearing on the site are the
property of their respective owners. CC0-1.0 does not waive, abandon, surrender,
license, or otherwise affect trademark or patent rights.
