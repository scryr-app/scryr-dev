import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

export default defineConfig({
  site: 'https://scryr.dev',
  integrations: [
    starlight({
      title: 'Scryr',
      description: 'Actionable architecture for humans and coding agents.',
      favicon: '/favicon.svg',
      logo: { src: './src/assets/scryr-logo.png' },
      customCss: ['./src/styles/custom.css'],
      social: [{ icon: 'github', label: 'GitHub', href: 'https://github.com/scryr-app/scryr-dev' }],
      editLink: { baseUrl: 'https://github.com/scryr-app/scryr-dev/edit/main/docs/' },
      sidebar: [
        { label: 'Start Here', items: [
          { label: 'Overview', slug: 'index' },
          { label: 'Getting started', slug: 'getting-started' },
          { label: 'Generate with an LLM', slug: 'llm-prompt' },
        ] },
        { label: 'Use Scryr', items: [
          { label: 'Manifest language', slug: 'manifests' },
          { label: 'CLI reference', slug: 'cli' },
          { label: 'Integrations', slug: 'integrations' },
          { label: 'Editor highlighting', slug: 'editors' },
        ] },
      ],
      head: [
        { tag: 'meta', attrs: { name: 'theme-color', content: '#090611' } },
        { tag: 'link', attrs: { rel: 'preconnect', href: 'https://fonts.googleapis.com' } },
        { tag: 'link', attrs: { rel: 'preconnect', href: 'https://fonts.gstatic.com', crossorigin: 'anonymous' } },
        { tag: 'link', attrs: { rel: 'stylesheet', href: 'https://fonts.googleapis.com/css2?family=Cinzel+Decorative:wght@400;700;900&family=Source+Code+Pro:wght@400;500;600;700&display=swap' } },
      ],
    }),
  ],
});
