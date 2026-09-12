# Scryr documentation site

The Scryr documentation and marketing site is built with [Astro Starlight](https://starlight.astro.build/). Most site content lives in Markdown or MDX under `src/content/docs/`.

```sh
npm ci
npm run dev
npm run build
```

The static production build is written to `dist/`.

## Cloudflare Pages

When the Pages project uses this directory as its root:

- Build command: `npm run build`
- Build output directory: `dist`
- Node.js: 22.12 or later

When the Pages project uses the repository root:

- Build command: `npm --prefix docs ci && npm --prefix docs run build`
- Build output directory: `docs/dist`

No server adapter or runtime bindings are required; this is a fully static site.
