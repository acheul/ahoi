---
title: React
description: Using ahoi with React — plain values, useSyncExternalStore underneath, StrictMode safe.
sidebar:
  order: 2
---

```bash
npm i @acheul/ahoi-js react react-dom
```

```ts
import { createAhoi } from "@acheul/ahoi-js/react";

export const { PierProvider, useHail, useReadHail, useTell } = createAhoi<
  Pier, Hail, Tell, HailRets, TellRets
>({ /* wasm exports */ });
```

## Separate hooks

React has no external signal primitive, so hails come through
`useSyncExternalStore` and arrive as **plain values**.

```tsx
function Counter() {
  const [count, setCount] = useHail("Count"); // number
  const doubled = useReadHail("Doubled"); // number
  const tell = useTell();

  return (
    <>
      <p>{count} · {doubled}</p>
      <button onClick={() => setCount(count + 1)}>+1</button>
      <button onClick={() => tell("Increase")}>tell</button>
    </>
  );
}
```

No `.value`, no accessor call. A hail update re-renders the component that read
it, exactly like `useState`.

## The provider

```tsx
<PierProvider pier="Top">
  <Counter />
</PierProvider>
```

One detail worth knowing: the provider enrols its pier in an **effect**, not
during render, and renders children only once the sphere exists.

So children do not mount on the very first pass. That is deliberate — enrolling
during render would be a side effect in render, which React does not allow you
to do safely.

If you need to know whether the pier is ready, `usePierId()` gives you the
sphere id and throws when there is no provider above.

## StrictMode

It works. You do not need to do anything.

React 18+ in development mounts, unmounts, and remounts every component, and
throws away some renders entirely. The adapter reconciles render-phase
enrolment with effect-phase subscriptions internally, so a hail survives the
double-mount cycle instead of being torn down and re-enrolled.

## Preact

Use this adapter and alias `react`/`react-dom` to `preact/compat`, the same way
you would for any React library. There is no separate Preact adapter and none
is needed.

## Notes

- `useTell()` returns a function bound to the nearest pier. Its return type
  comes from your `TellRets` map, so `tell("PopItem")` is
  `number | undefined`.
- Nest `PierProvider` for child scopes. Unmounting one clears its sphere and
  cascades to children.
- Wasm cannot hot-reload. In Vite, force a full reload when your wiring module
  changes:

  ```ts
  if (import.meta.hot) import.meta.hot.accept(() => import.meta.hot!.invalidate());
  ```
