# Playground Examples of Ahoi

Common setup (once, at the repo root):

```sh
pnpm install
pnpm -C js build
```

> Note: after changing the `js/` package itself, run `pnpm -C js build` again **and restart the dev server** — Vite caches the linked package's `dist`, so a running server keeps serving the old build.

## Solid Js

```sh
cargo test -p ahoi-wasm
wasm-pack build playgrounds/ahoi-wasm --target web --dev
pnpm -C playgrounds/solid dev
```

- `cargo test -p ahoi-wasm` regenerates `bindings/` (ts-rs types + `Keys.ts` ret maps); only needed after changing the key enums.
- Dev server: http://localhost:5173

## React

Same features as the Solid playground (shares the `ahoi-wasm` crate and its `bindings/`), but through the `@acheul/ahoi-js/react` adapter. Runs under `<StrictMode>` on purpose — the adapter must survive its double-mount / discarded-render behavior.

```sh
cargo test -p ahoi-wasm
wasm-pack build playgrounds/ahoi-wasm --target web --dev
pnpm -C playgrounds/react dev
```

- Dev server: http://localhost:5175

## Vue

Same features again, through the `@acheul/ahoi-js/vue` adapter (`shallowRef` + `onScopeDispose`; `useHail` returns a writable ref, so `count++` and `v-model` just work).

```sh
cargo test -p ahoi-wasm
wasm-pack build playgrounds/ahoi-wasm --target web --dev
pnpm -C playgrounds/vue dev
```

- Dev server: http://localhost:5176

## Svelte

Through the `@acheul/ahoi-js/svelte` adapter. Hails are **stores**, so `$count` / `bind:value={$info}` work as usual (Svelte 4 and 5). There is no provider component — `providePier("Top")` at the top of a component's `<script>` sets the pier for it and its children.

```sh
cargo test -p ahoi-wasm
wasm-pack build playgrounds/ahoi-wasm --target web --dev
pnpm -C playgrounds/svelte dev
```

- Dev server: http://localhost:5177

## Other setups (no playground needed)

- **Preact** — use `@acheul/ahoi-js/react` as-is; alias `react`/`react-dom` to `preact/compat` in your bundler config, as with any React library.
- **No framework** — the [bench](#bench) app drives `AhoiStorage` directly (enrol, cleanup scopes, and a hand-written signal in ~30 lines); it doubles as the vanilla-JS example.

## Bench

Core + bridge reactivity micro-benchmarks (for checking regressions while working on the core — not a framework showcase).

```sh
wasm-pack build playgrounds/bench --target web
pnpm -C playgrounds/bench dev
```

- Build wasm in release (no `--dev`) — debug numbers are not meaningful.
- Dev server: http://localhost:5174
- Flow: `run all` → `save baseline` → change core code → rebuild wasm → rerun; the table shows the delta against the saved baseline. Micro-op medians jitter ±20–30%; judge regressions by the heavier scenarios (fan-out / memo chain / enrol).
