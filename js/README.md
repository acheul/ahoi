# @acheul/ahoi-js

JS bridge for [**ahoi**](https://github.com/acheul/ahoi) — a fine-grained
reactive state engine written in Rust. Rust owns the state and reactivity;
your JS framework owns rendering.

This package is the JS half of the bridge. You also need the
[`ahoi`](https://crates.io/crates/ahoi) crate on the Rust side.

```sh
npm i @acheul/ahoi-js
```

## Adapters

Each adapter is a subpath of this package, and its framework is an **optional
peer dependency** — you only install what you use.

| Import                   | Provides                                                           |
| ------------------------ | ------------------------------------------------------------------ |
| `@acheul/ahoi-js/solid`  | `PierProvider`, `usePier()` → `.hail` / `.readHail` / `.tell`      |
| `@acheul/ahoi-js/react`  | `PierProvider`, `useHail`, `useReadHail`, `useTell`                |
| `@acheul/ahoi-js/vue`    | `PierProvider`, `useHail` (writable ref), `useReadHail`, `useTell` |
| `@acheul/ahoi-js/svelte` | `providePier`, `useHail` (store), `useReadHail`, `useTell`         |
| `@acheul/ahoi-js`        | `AhoiStorage` — the framework-agnostic core, for anything else     |

Using **Preact**? Use `@acheul/ahoi-js/react` and alias `react`/`react-dom` to
`preact/compat`, as with any React library.

## Usage

Wire the wasm exports once:

```ts
// bridge.ts
import wasmInit, {
  abi_version,
  clear,
  hail,
  pier,
  tell,
  write,
} from "../pkg/my_app";
import { createAhoi } from "@acheul/ahoi-js/solid";
import type { Pier } from "./bindings/Pier";
import type { Hail } from "./bindings/Hail";
import type { Tell } from "./bindings/Tell";
import type { HailRets, TellRets } from "./bindings/Rets";

await wasmInit();

export const { PierProvider, usePier } = createAhoi<
  Pier,
  Hail,
  Tell,
  HailRets,
  TellRets
>({
  _enrol_pier: pier,
  _enrol_hail: (p, k) => hail(p, k) as [number, any],
  _clear_sphere: clear,
  _write_hail: write,
  _tell: tell,
  _abi_version: abi_version,
});
```

Then use it (SolidJS shown):

```tsx
const pier = usePier();
const [count, setCount] = pier.hail("Count"); // () => number
const doubled = pier.readHail("Doubled"); // () => number
pier.tell("Increase"); // number
```

Keys are plain wire values (`"Count"`, `{ Item: 3 }`) — no constructors. Their
return types come from the `Rets` maps that `#[derive(Rets)]` generates on the
Rust side, so results are typed without ahoi having to be a TypeScript
converter: export your key and data types with whatever you prefer (ts-rs,
Tsify, …).

## Versioning

The crate and this package agree on an `ABI_VERSION`, checked when the bridge
starts. Keep the `ahoi` crate and `@acheul/ahoi-js` versions aligned; a mismatch fails
immediately with a clear message.

## Docs

Full documentation, the Rust-side quickstart, and runnable playgrounds for all
four frameworks live in the [main repository](https://github.com/acheul/ahoi).

## License

MIT
