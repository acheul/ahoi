---
title: Solid
description: Using ahoi with SolidJS — one sphere object, signal accessors, automatic cleanup.
sidebar:
  order: 1
---

```bash
npm i @acheul/ahoi-js solid-js
```

```ts
import { createAhoi } from "@acheul/ahoi-js/solid";

export const { PierProvider, usePier } = createAhoi<
  Pier, Hail, Tell, HailRets, TellRets
>({ /* wasm exports */ });
```

## One object per pier

Solid is the only adapter that hands you a **sphere object** rather than
separate hooks. Ahoi's hails map straight onto Solid signals, so there is
nothing to reconcile.

```tsx
function Counter() {
  const sphere = usePier();

  const [count, setCount] = sphere.hail("Count"); // () => number
  const doubled = sphere.readHail("Doubled"); // () => number

  return (
    <>
      <p>{count()} · {doubled()}</p>
      <button onClick={() => setCount(count() + 1)}>+1</button>
      <button onClick={() => sphere.tell("Increase")}>tell</button>
    </>
  );
}
```

Values are **accessors** — call them. That is ordinary Solid, and it means a
hail can be passed around without losing reactivity.

## The provider

```tsx
<PierProvider pier="Top">
  <Counter />
</PierProvider>
```

Nest them for child scopes:

```tsx
<PierProvider pier="Top">
  <Counter />
  <Show when={open()}>
    <PierProvider pier="Panel">
      <Panel />
    </PierProvider>
  </Show>
</PierProvider>
```

## Cleanup

Nothing to do. The adapter registers `onCleanup` for you, so unmounting a
provider clears its sphere, and that cascades to any child piers.

Hails release themselves the same way when the component that read them goes
away.

## Fine-grained by default

Because a hail is a signal, only the expressions that actually read it update.

```tsx
<p>{count()}</p>   {/* updates */}
<p>{other()}</p>   {/* does not */}
```

No component re-runs. This is the closest fit of the four adapters — Solid's
model and ahoi's are the same shape.

## Notes

- `usePier()` throws if there is no provider above it.
- The sphere object is stable. You can destructure it once and keep it.
- Wasm cannot hot-reload. In Vite, force a full reload when your wiring module
  changes:

  ```ts
  if (import.meta.hot) import.meta.hot.accept(() => import.meta.hot!.invalidate());
  ```
