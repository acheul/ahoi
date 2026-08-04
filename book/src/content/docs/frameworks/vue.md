---
title: Vue
description: Using ahoi with Vue — writable refs that work with v-model, and one gotcha about switching piers.
sidebar:
  order: 3
---

```bash
npm i @acheul/ahoi-js vue
```

```ts
import { createAhoi } from "@acheul/ahoi-js/vue";

export const { PierProvider, useHail, useReadHail, useTell } = createAhoi<
  Pier, Hail, Tell, HailRets, TellRets
>({ /* wasm exports */ });
```

## Refs, so v-model works

`useHail` returns a **writable computed ref**. That means the usual Vue idioms
apply without any wrapper.

```vue
<script setup lang="ts">
import { useHail, useReadHail, useTell } from "./ahoi";

const count = useHail("Count"); // WritableComputedRef<number>
const doubled = useReadHail("Doubled"); // ComputedRef<number>
const tell = useTell();
</script>

<template>
  <p>{{ count }} · {{ doubled }}</p>
  <button @click="count++">+1</button>
  <button @click="tell('Increase')">tell</button>
</template>
```

Writing to the ref writes back to Rust. In a template that is `count++`; in
script it is `count.value++`.

`v-model="count"` works too — it is a normal writable ref:

```vue
<input v-model.number="count" />
```

## The provider

```vue
<PierProvider pier="Top">
  <Counter />
</PierProvider>
```

The provider enrols during `setup`, synchronously, so children see the sphere on
their first render. No loading pass.

:::caution
**The `pier` prop is read once, at setup.** Changing it later does nothing.

To switch piers, re-mount the provider with a `:key`:

```vue
<PierProvider :key="currentPier" :pier="currentPier">
  <Panel />
</PierProvider>
```
:::

## Works outside components

Cleanup uses `onScopeDispose`, which fires for components **and** for plain
`effectScope`s.

So you can wrap hails in your own composables and use them outside a component
tree, and they will still be released correctly.

```ts
export function useCounter() {
  const count = useHail("Count");
  const tell = useTell();
  return { count, increase: () => tell("Increase") };
}
```

## Notes

- Values arrive from Rust already whole, so the adapter uses `shallowRef`. There
  is no deep tracking to pay for, and no reason to add any.
- `usePierId()` gives the nearest sphere id and throws when no provider is
  above.
- Wasm cannot hot-reload. In Vite, force a full reload when your wiring module
  changes:

  ```ts
  if (import.meta.hot) import.meta.hot.accept(() => import.meta.hot!.invalidate());
  ```
