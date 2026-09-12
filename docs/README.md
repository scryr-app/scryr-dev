# Scryr documentation site

The Scryr documentation and marketing site is built with [Astro Starlight](https://starlight.astro.build/). Most site content lives in Markdown or MDX under `src/content/docs/`.

```sh
npm ci
npm run dev
npm run build
```

The static production build is written to `dist/`.

## Cloudflare Workers Builds

The repository includes `wrangler.jsonc`, which deploys the generated `dist/`
directory as static assets. Configure the connected Git repository in the
Cloudflare dashboard with:

- Build command: `npm run build`
- Deploy command: `npx wrangler deploy`
- Version command: `npx wrangler versions upload`
- Root directory: `docs`
- Production branch: `main`
- Node.js: 22.12 or later

There is no build-output field in this version of the Cloudflare UI. Wrangler
reads `assets.directory` from `wrangler.jsonc` instead. The Wrangler `name`
must match the existing Cloudflare Worker name (`scryr-dev`).

After a successful production deployment, open the Worker's **Domains** tab,
choose **Add > Custom Domain**, and enter `scryr.dev`.

No server adapter or runtime bindings are required; this is a fully static site.
