// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import solid from '@astrojs/solid-js';

const repo = 'https://github.com/acheul/ahoi';

// Published to GitHub Pages, so every URL is served under `/ahoi/`.
// Starlight prefixes its own links with `base`; hand-written links in MDX must
// use relative paths or `import.meta.env.BASE_URL`.
export default defineConfig({
  site: 'https://acheul.github.io',
  base: '/ahoi/',
  integrations: [
    // Scoped by directory so React can be added alongside it later — both use
    // JSX, and Astro cannot tell them apart without these globs.
    solid({ include: ['**/examples/**/solid/**'] }),
    starlight({
      title: 'Ahoi',
      description:
        'Reactivity from Rust to JS. Rust owns the state, your JS framework renders it.',
      // Emitted as <img>, so the mark cannot inherit `currentColor`.
      // Starlight swaps these two with its own theme toggle.
      logo: {
        light: './src/assets/ahoi-light.svg',
        dark: './src/assets/ahoi-dark.svg',
        alt: 'Ahoi',
        replacesTitle: true,
      },
      favicon: '/favicon.svg',
      social: [{ icon: 'github', label: 'GitHub', href: repo }],
      editLink: { baseUrl: `${repo}/edit/main/book/` },
      lastUpdated: true,
      customCss: ['./src/styles/custom.css'],
      sidebar: [
        {
          label: 'Getting Started',
          items: [
            { slug: 'getting-started/what-is-ahoi' },
            { slug: 'getting-started/installation' },
            { slug: 'getting-started/quick-start' },
          ],
        },
      ],
    }),
  ],
});
