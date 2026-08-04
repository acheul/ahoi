<div align="center">
    <img src="https://raw.githubusercontent.com/acheul/ahoi/main/documents/ahoi.svg" width="200" />
</div>

# Ahoi: Reactivity _from_ Rust _to_ JS

> Render components with JS, manage state with Rust — the best of both.

[![github.com](https://img.shields.io/badge/github-repo-blue?logo=github)](https://github.com/acheul/ahoi)
[![crates.io](https://img.shields.io/crates/v/ahoi.svg)](https://crates.io/crates/ahoi)
[![npm](https://img.shields.io/npm/v/@acheul/ahoi-js.svg)](https://www.npmjs.com/package/@acheul/ahoi-js)
[![docs.rs](https://img.shields.io/docsrs/ahoi)](https://docs.rs/ahoi)
[![Book](https://img.shields.io/badge/Book-v0.1.1-blue)](https://acheul.github.io/ahoi/)

Ahoi is a fine-grained reactive state engine written in Rust. Rust owns the data
and reactivity; a JS framework owns rendering. The two talk over a thin,
type-safe wasm bridge.

Adapters ship for **Solid, React, Vue, and Svelte** — the core is
framework-agnostic, so plain JS works too.

Look the [Book](https://acheul.github.io/ahoi/) to get started!

---

## Core idea

> _"Rust for Rust, Js for Js"_

- **Rust holds the truth** — all state, derivations, and effects live in Rust.
- **JS just renders** — the frontend subscribes to values and pushes user input back.
- **Bridge is bidirectional** — values flow Rust→JS (push) and JS→Rust (write).

### Why Ahoi?

<div align="center">
    <img src="https://raw.githubusercontent.com/acheul/ahoi/main/documents/PainPoints.png" width="800" />
</div>

- 💥 **Js Framework + Wasm**: maintain communication between JS reactive state & rust-side data all by hand.

- 💥 **Rust Framework**: should handle everything in rust, including ones which JS fits better! (ex. event handling)

- ✔️ **Ahoi** removes these pain points. Use rust for rust, JS for JS. Keep reactivity.

---

## Quickstart

### 1. Rust side

```toml
# Cargo.toml
[dependencies]
ahoi = { version = "0.1", features = ["serde-wasm-bindgen"] }
serde = { version = "1", features = ["derive"] }
serde-wasm-bindgen = "0.6"
wasm-bindgen = "0.2"

[lib]
crate-type = ["cdylib", "rlib"]
```

```rust
use ahoi::js_bridge::SerdeWasmBindgenConverter as Converter;
use ahoi::js_bridge::*;
use ahoi::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Stock, Serialize, Deserialize)]
pub struct State {
    count: i32,
}

/// Scopes: set up state/context for a region of the UI.
#[derive(Serialize, Deserialize)]
pub enum Pier {
    Top,
}

wasm_bindgen_enrol_sphere!(@pier, Pier, run_pier, Converter);

fn run_pier(key: Pier) {
    match key {
        Pier::Top => {
            set_js_hail_dispatcher();
            provide_context(Stock::new(State { count: 0 }));
        }
    }
}

/// Reactive value channels: `#[ret(..)]` declares what JS gets back.
#[derive(Rets, Serialize, Deserialize)]
pub enum Hail {
    #[ret(i32)]
    Count,
    #[ret(i32)]
    Doubled,
}

wasm_bindgen_enrol_sphere!(@hail, Hail, run_hail, Converter);

fn run_hail(key: Hail) -> JsValue {
    let state = use_context::<Stock<State>>().unwrap();
    match key {
        // read-write: JS can write back into the stock
        Hail::Count => state.count().set_hail::<Converter>(),
        // read-only, and recomputed only when `count` actually changes
        Hail::Doubled => state.count().memo(|c| *c * 2).set_read_hail::<Converter>(),
    }
}

/// One-shot JS→Rust commands.
#[derive(Rets, Serialize, Deserialize)]
pub enum Tell {
    #[ret(i32)]
    Increase,
}

wasm_bindgen_tell!(Tell, run_tell, Converter);

fn run_tell(tell: Tell) -> JsValue {
    let state = use_context::<Stock<State>>().unwrap();
    match tell {
        Tell::Increase => {
            let new_count = {
                let mut count = state.count().write();
                *count += 1;
                *count
            };
            serde_wasm_bindgen::to_value(&new_count).unwrap()
        }
    }
}
```

### 2. Export the types

Ahoi is **not** a Rust→TypeScript converter — use whichever you like
([ts-rs], [Tsify], …) for your key and data types. Ahoi only adds the one thing
no general-purpose converter knows: **what each key returns**.

```rust
// run with `cargo test` to (re)generate
#[test]
fn generate() {
    ahoi::js_bridge::TsFile::new()
        .with::<Hail>()
        .with::<Tell>()
        .export("./bindings/Rets.ts");
}
```

```ts
// bindings/Rets.ts — generated
export type HailRets = { Count: number; Doubled: number };
export type TellRets = { Increase: number };
```

Then build the wasm package:

```sh
wasm-pack build --target web
```

### 3. JS side

```sh
npm i @acheul/ahoi-js
```

```ts
// ahoi.ts — the only wiring an app needs
import wasmInit, {
  abi_version,
  clear,
  hail,
  pier,
  tell,
  write,
} from "../pkg/my_app";
import { createAhoi } from "@acheul/ahoi-js/solid"; // or /react, /vue, /svelte
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

```tsx
// Counter.tsx (SolidJS)
import { usePier } from "./ahoi";

function Counter() {
  const sphere = usePier();
  const [count, setCount] = sphere.hail("Count"); // () => number
  const doubled = sphere.readHail("Doubled"); // () => number

  return (
    <>
      <p>
        {count()} · {doubled()}
      </p>
      <button onClick={() => setCount(count() + 1)}>write</button>
      <button onClick={() => sphere.tell("Increase")}>tell</button>
    </>
  );
}

// <PierProvider pier="Top"><Counter /></PierProvider>
```

Keys are plain values (`"Count"`, `{ Item: 3 }`) — no constructors — and their
return types resolve from the generated `Rets` maps, so `count()` is `number`
and `sphere.tell("Increase")` is `number`.

[ts-rs]: https://github.com/Aleph-Alpha/ts-rs
[Tsify]: https://github.com/madonoharu/tsify

---

## Framework adapters

Every adapter is a subpath of the same package; the framework is an optional
peer dependency, so you only install what you use.

| Import                   | Provides                                                           |
| ------------------------ | ------------------------------------------------------------------ |
| `@acheul/ahoi-js/solid`  | `PierProvider`, `usePier()` → `.hail` / `.readHail` / `.tell`      |
| `@acheul/ahoi-js/react`  | `PierProvider`, `useHail`, `useReadHail`, `useTell`                |
| `@acheul/ahoi-js/vue`    | `PierProvider`, `useHail` (writable ref), `useReadHail`, `useTell` |
| `@acheul/ahoi-js/svelte` | `providePier`, `useHail` (store), `useReadHail`, `useTell`         |
| `@acheul/ahoi-js`        | `AhoiStorage` — the framework-agnostic core, for anything else     |

Using **Preact**? Use `@acheul/ahoi-js/react` and alias `react`/`react-dom` to
`preact/compat`, as with any React library.

---

## Reactive primitives

- **`Stock<T>`** — a writable reactive value (the source of truth).
  - `ReadStock<T>` — read-only view (`Stock` derefs to it).
  - `OptStock<T>` / `OptReadStock<T>` — value may be absent (`Vec` index, map key,
    optional/enum-variant field); use the `try_*` accessors.
- **`Memo<T>`** — cached derived value; recomputes only when dependencies change,
  writes only when the result actually differs.
- **`Effect`** — runs a side effect, re-runs when its dependencies change.
- **`Callback<A, R>`** — a sync function that can read/write reactive state.
- **`Action<A, R>`** — an async, cancellable task with `pending`/`ready` state.
- **`Resource<T>`** — an async value that auto-refetches when dependencies change.

Dependencies are tracked automatically: any `read()` inside a runner records a
citation, and mutations propagate to exactly the runners that cited them.

---

## Stock derivation

- `#[derive(Stock)]` generates field/variant accessors so you can drill into
  nested state and get a reactive sub-stock:

  ```rust
  #[derive(Stock)]
  struct State { count: i32, items: Vec<i32> }

  let item = state.items().get(0);   // OptStock<i32>
  let len  = state.items().memo(|v| v.len());
  ```

  Derivation is **path-selective**: writing `items[0]` notifies subscribers of
  that path and of its ancestors, but not of a sibling path.

- `#[stock]` adds custom methods to a `Stock<YourType>` via an extension trait.
- **Getter pipeline** — how a sub-stock maps from its root:
  - **Chained** (default for derives) — stack-built, zero state cost; recreated
    each access. Prefer this.
  - **Pooled** (`.pool()`) — stored as state, always `Copy`, lets you drop the
    `Pipe` generic. Handy for context values.

---

## Sphere — the unit of reactive lifetime

- Every state created inside `make_sphere(parent, || { … })` is **owned by that
  sphere** and lives until `clear_sphere(id)`.
- **`clear_sphere` cascades to children and is order-independent.** Spheres form
  a parent→child tree, so clearing a sphere recursively frees its whole subtree.
  Clearing is idempotent, so a host can wire one `clear_sphere` per component
  (e.g. SolidJS `onCleanup`) and stay correct no matter the teardown order.
- The parent link also drives **context lookup**: `provide_context::<T>(value)` /
  `use_context::<T>()` resolve up the parent chain (React-context style).

---

## JS bridge: Pier, Hail & Tell

The bridge exposes two kinds of sphere to the frontend:

- **Pier** — a _scope_ sphere. Sets up context, effects, and shared state for a
  region of the UI. Created via a `PierProvider` (no value returned).
- **Hail** — a _reactive value channel_. Binds one Rust value to a JS signal.
  - `set_read_hail` — read-only (Rust→JS push).
  - `set_hail` — read-write (JS can also write back into the Rust stock).
- **Tell** — a one-shot JS→Rust command (mutate state, call an action, …).

Both Pier and Hail are the same `Sphere` underneath; they're split only at the
key-type surface so JS can't mix them up.

Macros generate the wasm exports:

- `wasm_bindgen_enrol_sphere!(@pier, PierKey, run_pier, Converter)`
- `wasm_bindgen_enrol_sphere!(@hail, HailKey, run_hail, Converter)`
- `wasm_bindgen_tell!(TellKey, run_tell, Converter)`

A `HailConverter` defines how a Rust value crosses to/from a `JsValue`. The
`serde-wasm-bindgen` feature provides `SerdeWasmBindgenConverter`; implement the
trait yourself to use a different one.

Dispatches are collected and sent to JS **in one batch** per propagation cycle,
so a single write that touches many hails costs one crossing, not many.

The crate and the npm package agree on an `ABI_VERSION`, checked when the bridge
starts — a version mismatch fails immediately with a clear message instead of
surfacing as a confusing runtime error later.

---

## Type generation, without a type converter

`#[derive(Rets)]` collects `#[ret(..)]` annotations into one TS map per key
enum, keyed by variant name:

```rust
#[derive(Rets, Serialize, Deserialize)]
pub enum Hail {
    #[ret(i32)]
    Count,
    #[ret(Vec<(String, Fruit)>)]
    Fruits,
    #[ret(ts = "MyHandWrittenType")] // escape hatch: literal TS
    Custom,
}
```

Ret types are rendered syntactically at macro-expansion time, so referenced
types (`Fruit` above) need **no derive and no trait impl** — they render by
their identifier, matching how every mainstream exporter names TS types. A
generated compile-time assertion still catches typos.

The JS side resolves a key's return type from that map by variant name — or, if
your converter brands keys with a `ret` type directly, straight from the brand.
Either way the bridge stays converter-agnostic.

> Key enums assume serde's **default (externally-tagged)** representation.
> Attributes that rename variants or change the enum representation are
> rejected at compile time rather than silently producing mismatched types.

---

## Examples

Four playgrounds share one wasm crate, each exercising the same bridge features
(writable hails, memos, path-derived writes, async `Resource`/`Action`, enum and
map values on the wire, nested-pier cleanup), plus a benchmark app:

- [`playgrounds/solid`](playgrounds/solid), [`playgrounds/react`](playgrounds/react),
  [`playgrounds/vue`](playgrounds/vue), [`playgrounds/svelte`](playgrounds/svelte)
- [`playgrounds/ahoi-wasm`](playgrounds/ahoi-wasm) — the shared Rust side
- [`playgrounds/bench`](playgrounds/bench) — reactivity micro-benchmarks
  (doubles as the no-framework example)

See [`playgrounds/playgrounds.md`](playgrounds/playgrounds.md) for how to run them.

---

## License

[MIT LICENSE](LICENSE)
