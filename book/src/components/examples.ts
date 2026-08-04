/**
 * Reads example sources at build time.
 *
 * Every snippet in the book comes from a real file under `book/examples/`.
 * Nothing here falls back to a placeholder: a missing file or region must fail
 * the build, because a silently empty code block is worse than a broken one.
 */

const sources = import.meta.glob(
  [
    '../../examples/**/*.{rs,ts,tsx,vue,svelte}',
    // wasm-pack output — generated, never shown.
    '!../../examples/rust/pkg/**',
  ],
  { query: '?raw', import: 'default', eager: true }
) as Record<string, string>;

/** Sources keyed by path relative to `book/examples/`. */
const byPath: Record<string, string> = {};
for (const [key, source] of Object.entries(sources)) {
  byPath[key.replace(/^\.\.\/\.\.\/examples\//, '')] = source;
}

export const FRAMEWORKS = [
  { id: 'solid', label: 'Solid' },
  { id: 'react', label: 'React' },
  { id: 'vue', label: 'Vue' },
  { id: 'svelte', label: 'Svelte' },
] as const;

export type FrameworkId = (typeof FRAMEWORKS)[number]['id'];

export function readFile(path: string): string {
  const source = byPath[path];
  if (source === undefined) {
    throw new Error(
      `Example file not found: book/examples/${path}\n` +
        `Known files:\n  ${Object.keys(byPath).sort().join('\n  ')}`
    );
  }
  return source.replace(/\s+$/, '');
}

/**
 * Extracts every `// #region <name>` block from a file and joins them.
 *
 * Regions repeat because one example's Rust is usually several separate items
 * (a struct, a key enum, its runner) rather than one contiguous run of lines.
 */
export function readRegion(path: string, name: string): string {
  const escaped = name.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  const start = new RegExp(`^\\s*//\\s*#region\\s+${escaped}\\s*$`);
  const end = new RegExp(`^\\s*//\\s*#endregion\\s+${escaped}\\s*$`);

  // Any *other* region's markers must not survive into the shown snippet.
  const anyMarker = /^\s*\/\/\s*#(?:end)?region\b/;

  const blocks: string[] = [];
  let current: string[] | null = null;

  for (const line of readFile(path).split(/\r?\n/)) {
    if (start.test(line)) {
      current = [];
    } else if (end.test(line)) {
      if (current) blocks.push(current.join('\n').replace(/^\n+|\n+$/g, ''));
      current = null;
    } else if (current && !anyMarker.test(line)) {
      current.push(line);
    }
  }

  if (blocks.length === 0) {
    throw new Error(`Region "${name}" not found in book/examples/${path}`);
  }
  return dedent(blocks.join('\n\n'));
}

function dedent(source: string): string {
  const indents = source
    .split('\n')
    .filter((line) => line.trim() !== '')
    .map((line) => line.match(/^[ \t]*/)?.[0].length ?? 0);
  const shortest = indents.length ? Math.min(...indents) : 0;
  return shortest === 0
    ? source
    : source
        .split('\n')
        .map((line) => line.slice(shortest))
        .join('\n');
}

const LANGS: Record<string, string> = {
  rs: 'rust',
  ts: 'ts',
  tsx: 'tsx',
  vue: 'vue',
  svelte: 'svelte',
};

export function langOf(path: string): string {
  return LANGS[path.split('.').pop() ?? ''] ?? 'text';
}

export function basename(path: string): string {
  return path.split('/').pop() ?? path;
}

/**
 * Finds the component file for each framework under `examples/<name>/<fw>/`.
 *
 * Wiring (`ahoi.ts`) and the island plumbing (`island.tsx`, `mount.tsx`) are
 * infrastructure, not teaching material, so they are never shown.
 */
export function frameworkFiles(
  name: string,
  file?: string
): Array<{ id: FrameworkId; label: string; path: string }> {
  const ignored = /\/(?:ahoi\.ts|island\.tsx|mount\.tsx)$/;
  const found = [];

  for (const { id, label } of FRAMEWORKS) {
    const prefix = `${name}/${id}/`;
    // An explicit filename wins, so infrastructure files can still be shown
    // when they are the point of the example (the setup wiring, say).
    const path = file
      ? byPath[prefix + file] !== undefined
        ? prefix + file
        : undefined
      : Object.keys(byPath)
          .sort()
          .find((candidate) => candidate.startsWith(prefix) && !ignored.test(candidate));
    if (path) found.push({ id, label, path });
  }

  if (found.length === 0) {
    throw new Error(`No framework files found for example "${name}"`);
  }
  return found;
}
