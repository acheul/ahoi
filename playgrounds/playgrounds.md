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

Same features as the Solid playground (shares the `ahoi-wasm` crate and its `bindings/`), but through the `ahoi-js/react` adapter. Runs under `<StrictMode>` on purpose — the adapter must survive its double-mount / discarded-render behavior.

```sh
cargo test -p ahoi-wasm
wasm-pack build playgrounds/ahoi-wasm --target web --dev
pnpm -C playgrounds/react dev
```

- Dev server: http://localhost:5175

## Bench

Core + bridge reactivity micro-benchmarks (for checking regressions while working on the core — not a framework showcase).

```sh
wasm-pack build playgrounds/bench --target web
pnpm -C playgrounds/bench dev
```

- Build wasm in release (no `--dev`) — debug numbers are not meaningful.
- Dev server: http://localhost:5174
- Flow: `run all` → `save baseline` → change core code → rebuild wasm → rerun; the table shows the delta against the saved baseline. Micro-op medians jitter ±20–30%; judge regressions by the heavier scenarios (fan-out / memo chain / enrol).
