---
title: Production builds
description: Dev and release wasm differ in more than speed. What changes, and how to keep versions aligned.
sidebar:
  order: 1
---

## The build order

Three steps, and they have to happen in this order:

```bash
cargo test           # regenerate bindings/
wasm-pack build --target web --release
npm run build        # your bundler
```

`cargo test` is what runs your `TsFile` export and your exporter's `#[ts(export)]`
tests. Skip it after changing a key enum and your TypeScript will describe the
previous shape.

Only the key enums need it. Changing a runner body does not.

## Dev and release differ

Use `--dev` while developing:

```bash
wasm-pack build --target web --dev
```

The difference is not only optimisation. **Debug builds record where each
reactive value was created**, so a panic blames your line:

```
panicked at src/lib.rs:265:39:
RefCell already mutably borrowed
```

Release builds compile that tracking out. You get a smaller binary, and **no
source paths ship in your wasm**.

That is worth knowing in both directions: debug builds are much easier to
diagnose, and release builds do not leak your file layout to anyone who reads
the binary.

## Panics abort the module

A wasm panic tears down the module. The page has to be **reloaded** — there is
no recovering the state.

So a panic in development is not a message you can dismiss. Fix it, reload,
carry on.

## Keeping the two halves in step

The crate and the npm package share an `ABI_VERSION`, checked when the bridge
starts.

```
[ahoi] bridge ABI mismatch: the wasm module speaks ABI v2,
but this JS bridge expects v1.
Align the `ahoi` crate and npm package versions.
```

It fails immediately and says what to do, rather than surfacing later as a
confusing runtime error.

Practically: **upgrade `ahoi` and `@acheul/ahoi-js` together.** If you pin one,
pin both.

## Wasm does not hot-reload

A wasm module carries live state, so it cannot be swapped into a running page.

In Vite, force a full reload when your wiring module changes:

```ts
if (import.meta.hot) import.meta.hot.accept(() => import.meta.hot!.invalidate());
```

Without that you get a stale module and confusing behaviour after edits.

Note this is about the wasm side only. Your components hot-reload normally.

## Size

`wasm-pack --release` runs `wasm-opt` for you.

The bulk of a small ahoi module is the reactivity runtime plus whatever your own
code pulls in. Serialisation is usually the biggest lever you control — a
converter that moves less data across the boundary makes a smaller and faster
module than one that moves whole structs.
