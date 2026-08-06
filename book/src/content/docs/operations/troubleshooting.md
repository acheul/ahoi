---
title: Troubleshooting
description: The errors you are most likely to hit, what they mean, and what to change.
sidebar:
  order: 3
---

## Bridge ABI mismatch

```
[ahoi] bridge ABI mismatch: the wasm module speaks ABI v2,
but this JS bridge expects v1.
Align the `ahoi` crate and npm package versions.
```

The `ahoi` crate and `@acheul/ahoi-js` are from different releases.

Upgrade both together. If you pin one version, pin both.

## RefCell already mutably borrowed

```
panicked at src/lib.rs:265:39:
RefCell already mutably borrowed
```

A write guard was still alive when the same value was read again.

```rust
// wrong — the guard lives to the end of the statement
state.count().set(*state.count().read() + 1);

// right — the guard is dropped before anything else runs
let next = {
    let mut c = state.count().write();
    *c += 1;
    *c
};
```

In a debug build the line number is **yours**, not one inside ahoi. Release
builds compile that tracking out, so diagnose this with `--dev`.

Remember a wasm panic aborts the module — reload the page after fixing.

## Hails get an initial value but never update

`set_js_hail_dispatcher()` is missing from your root pier.

```rust
fn run_pier(key: Pier) {
    match key {
        Pier::Top => {
            set_js_hail_dispatcher(); // this
            provide_context(Stock::new(State::default()));
        }
    }
}
```

Without it, Rust has nowhere to push values, so the first read works and nothing
after it does.

## Nothing changed after editing Rust

Wasm cannot hot-reload. Rebuild, then make sure the page fully reloads:

```bash
wasm-pack build --target web --dev
```

```ts
if (import.meta.hot) import.meta.hot.accept(() => import.meta.hot!.invalidate());
```

## Types are stale after changing a key enum

Run `cargo test`. That is what regenerates `bindings/` — both your exporter's
types and ahoi's ret maps.

Nothing else triggers it, and `wasm-pack build` will happily build against the
old bindings.

## A serde attribute is a compile error

Key enums must use serde's default, externally tagged representation. Renaming
variants or changing the representation is rejected at compile time.

Change the variant name in Rust instead of renaming it in serde.

## A map value is not an object

`HashMap<K, V>` arrives as a JavaScript **`Map`**, not a plain object.

```ts
const counts = pier.readHail("FruitCounts"); // Map<string, number>

counts.get("apple"); // right
counts["apple"]; // undefined
```

## usePier / usePierId throws

There is no `PierProvider` above the component, or in Svelte no `providePier`
was called in an ancestor.

In React, remember the provider renders children only after its sphere exists —
a component rendered outside the provider will not find one.

## Vue: changing the pier prop does nothing

The `pier` prop is read once during `setup`. Re-mount the provider to switch:

```vue
<PierProvider :key="currentPier" :pier="currentPier">
  <Panel />
</PierProvider>
```

## Svelte: context errors inside a handler

`providePier`, `useHail`, `useReadHail`, and `useTell` use `setContext`,
`getContext`, and `onDestroy`, so they only work during component
initialisation.

Capture them at the top of `<script>`:

```svelte
<script lang="ts">
  const tell = useTell(); // here

  function onClick() {
    tell("Increase"); // not here
  }
</script>
```

## A key's type is `unknown`

The ret map is not reaching `createAhoi`. Check that you passed all five type
parameters:

```ts
createAhoi<Pier, Hail, Tell, HailRets, TellRets>({ /* ... */ });
```

A missing `HailRets` falls back to `unknown` for hails, and a missing
`TellRets` to `undefined` for tells.

## Writing to a derived value does nothing

The path does not exist — an index past the end, a missing map key, a field of
an inactive enum variant.

That is by design: writes to an absent path are ignored rather than panicking.
The returned `Option` tells you when you need to know.

```rust
if state.items().get(10).set(1).is_none() {
    // there is no item 10
}
```
