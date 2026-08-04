---
title: Other frameworks
description: Preact works through the React adapter. For anything else, the core is framework-agnostic.
sidebar:
  order: 5
---

Adapters ship for Solid, React, Vue, and Svelte. That list is settled — those
four cover the two integration shapes a reactive bridge can have, and each extra
adapter costs a peer dependency and a playground to keep honest.

Everything else goes through one of the two routes below.

## Preact

Use the [React adapter](../react/) and alias `react` and `react-dom` to
`preact/compat`, the same as for any React library.

```js
// vite.config.js
export default {
  resolve: {
    alias: {
      react: "preact/compat",
      "react-dom": "preact/compat",
    },
  },
};
```

```ts
import { createAhoi } from "@acheul/ahoi-js/react";
```

Nothing else changes. There is no Preact adapter because there is nothing for
one to do.

## No framework

The core is framework-agnostic. `AhoiStorage` is what the four adapters are
built on, and you can use it directly.

```ts
import { AhoiStorage } from "@acheul/ahoi-js";
```

Two things make an adapter, and you supply both:

- **A signal factory.** Given an initial value, return a getter and a setter.
  The storage writes incoming values through the setter.
- **A cleanup registrar.** Given a function, arrange for it to run when the
  surrounding scope ends.

That is the whole seam. Solid passes `createSignal` and `onCleanup`; Vue passes
`shallowRef` and `onScopeDispose`; Svelte passes a store and `onDestroy`.

For a plain script with no scope to speak of, the setter can write straight to
the DOM and cleanup can be a no-op.

:::note
This is the integration API, and its members are underscore-prefixed to say so.
If you are not writing an adapter, one of the four above will be less work and
better tested.
:::

## Writing a new adapter

If you do build one, the pattern to follow is in the four existing adapters —
they are small, and each is a single file:

- `js/src/solid.ts` — signal injection, the simplest shape
- `js/src/react.ts` — external store, for frameworks without a signal primitive
- `js/src/vue.ts` — ref-based, with scope-aware cleanup
- `js/src/svelte.ts` — stores, and context without a provider component

Start from whichever matches how your framework handles external state.
