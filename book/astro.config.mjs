// @ts-check
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import solid from '@astrojs/solid-js';

const repo = 'https://github.com/acheul/ahoi';

/**
 * The crate version the book documents.
 *
 * Read here rather than in a component: config files are not bundled, so
 * `import.meta.url` is still the real path on disk. Inside a component the
 * bundler rewrites it to somewhere under `dist/` and the lookup fails.
 */
const crateVersion = (() => {
  const manifest = readFileSync(
    fileURLToPath(new URL('../Cargo.toml', import.meta.url)),
    'utf8'
  );
  // Scoped to `[workspace.package]` so an unrelated `version =` cannot match.
  const section = manifest.split(/^\[workspace\.package\]\s*$/m)[1];
  const version = section?.match(/^\s*version\s*=\s*"([^"]+)"/m)?.[1];
  if (!version) {
    throw new Error('Could not read `version` from [workspace.package] in Cargo.toml');
  }
  return version;
})();

// Published to GitHub Pages, so every URL is served under `/ahoi/`.
// Starlight prefixes its own links with `base`; hand-written links in MDX must
// use relative paths or `import.meta.env.BASE_URL`.
export default defineConfig({
  site: 'https://acheul.github.io',
  base: '/ahoi/',
  vite: {
    define: { __CRATE_VERSION__: JSON.stringify(crateVersion) },
  },
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
      // Comfortaa is the logo's typeface; headings reuse it. Self-hosted
      // rather than loaded from Google Fonts — no external request, no flash.
      customCss: ['@fontsource-variable/comfortaa', './src/styles/custom.css'],
      // Adds the documented crate version above the nav.
      components: { Sidebar: './src/components/Sidebar.astro' },
      sidebar: [
        {
          label: 'Getting Started',
          items: [
            { slug: 'getting-started/what-is-ahoi' },
            { slug: 'getting-started/installation' },
            { slug: 'getting-started/quick-start' },
          ],
        },
        {
          label: 'The bridge',
          items: [{ autogenerate: { directory: 'bridge' } }],
        },
        {
          label: 'Rust reactivity',
          items: [{ autogenerate: { directory: 'reactivity' } }],
        },
        {
          label: 'Framework guides',
          items: [{ autogenerate: { directory: 'frameworks' } }],
        },
        {
          label: 'Operations',
          items: [{ autogenerate: { directory: 'operations' } }],
        },
      ],
    }),
  ],
});
