---
title: Svelte
description: Using ahoi with Svelte — stores rather than runes, and no provider component.
sidebar:
  order: 4
---

```bash
npm i @acheul/ahoi-js svelte
```

```ts
import { createAhoi } from "@acheul/ahoi-js/svelte";

export const { providePier, useHail, useReadHail, useTell } = createAhoi<
  Pier, Hail, Tell, HailRets, TellRets
>({ /* wasm exports */ });
```

## Stores, not runes

Hails are Svelte **stores**. `$` auto-subscription works, and writing through
`$` writes back to Rust.

```svelte
<script lang="ts">
  import { useHail, useReadHail, useTell } from "./ahoi";

  const count = useHail("Count"); // Writable<number>
  const doubled = useReadHail("Doubled"); // Readable<number>
  const tell = useTell();
</script>

<p>{$count} · {$doubled}</p>
<button on:click={() => ($count += 1)}>+1</button>
<button on:click={() => tell("Increase")}>tell</button>
```

This is a deliberate choice, and it buys you two things: the adapter works in
**Svelte 4 and Svelte 5 alike**, including runes mode, and it stays plain
TypeScript.

Runes would have forced the library itself into `.svelte.ts` files and a Svelte
compilation step, for no gain at the call site.

## No provider component

Svelte has no `<PierProvider>`. Shipping one would mean shipping a compiled
`.svelte` file, so piers use Svelte's function-style context instead.

Call `providePier` at the top of the component that owns the scope:

```svelte
<script lang="ts">
  import { providePier } from "./ahoi";
  import Counter from "./Counter.svelte";

  providePier("Top");
</script>

<Counter />
```

Child components then use `useHail` and friends as normal.

The same component can both provide and use a pier — `setContext` followed by
`getContext` in one script block works:

```svelte
<script lang="ts">
  providePier("Panel");
  const info = useReadHail("PanelInfo"); // reads from the pier just provided
</script>
```

## Call them during initialisation

:::caution
`providePier`, `useHail`, `useReadHail`, and `useTell` all use `setContext`,
`getContext`, or `onDestroy`.

That means they must be called **during component initialisation** — at the top
level of `<script>`, not inside an event handler, a callback, or after an
`await`.
:::

Capture what you need up front:

```svelte
<script lang="ts">
  const tell = useTell(); // here

  function onClick() {
    tell("Increase"); // not here
  }
</script>
```

## Cleanup

Automatic. Destroying the component that called `providePier` clears its sphere,
and that cascades to child piers. Hails release with the component that read
them.

## Notes

- The storage returns the same accessor for a key that is already enrolled, so
  many components reading one hail share a single store.
- `usePierId()` gives the nearest sphere id and throws when no pier was provided
  above.
- Wasm cannot hot-reload. In Vite, force a full reload when your wiring module
  changes:

  ```ts
  if (import.meta.hot) import.meta.hot.accept(() => import.meta.hot!.invalidate());
  ```
