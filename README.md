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

Read the **[Book](https://acheul.github.io/ahoi/)** to get started!

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

## A taste

On the Rust side, state lives in a reactive `Stock`, and a key enum declares
what JS can subscribe to — each `#[ret(..)]` is the type JS gets back:

```rust
#[derive(Stock, Serialize, Deserialize)]
pub struct State {
    count: i32,
}

#[derive(Rets, Serialize, Deserialize)]
pub enum Hail {
    #[ret(i32)]
    Count,
    #[ret(i32)]
    Doubled,
}

fn run_hail(key: Hail) -> JsValue {
    let state = use_context::<Stock<State>>().unwrap();
    match key {
        // read-write: JS can write back into the stock
        Hail::Count => state.count().set_hail::<Converter>(),
        // read-only, recomputed only when `count` actually changes
        Hail::Doubled => state.count().memo(|c| *c * 2).set_read_hail::<Converter>(),
    }
}
```

On the JS side, those keys resolve to fully-typed signals of the host
framework (SolidJS here):

```tsx
function Counter() {
  const pier = usePier();
  const [count, setCount] = pier.hail("Count"); // () => number
  const doubled = pier.readHail("Doubled"); // () => number

  return (
    <>
      <p>
        {count()} · {doubled()}
      </p>
      <button onClick={() => setCount(count() + 1)}>+1</button>
    </>
  );
}
```

Keys are plain values (`"Count"`, `{ Item: 3 }`) — no constructors — and their
return types resolve from the generated `Rets` maps, so `count()` is `number`.
Ahoi is **not** a Rust→TypeScript converter: bring your own
([ts-rs], [Tsify], [Tsain], …); Ahoi only adds what no general-purpose
converter knows — **what each key returns**.

For the full setup — Cargo/npm install, the bridge wiring, type export, and
one-shot `Tell` commands — follow the
**[Quick Start](https://acheul.github.io/ahoi/getting-started/quick-start/)**
in the Book.

[ts-rs]: https://github.com/Aleph-Alpha/ts-rs
[Tsify]: https://github.com/madonoharu/tsify
[Tsain]: https://github.com/acheul/tsain

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

## Learn more

The **[Book](https://acheul.github.io/ahoi/)** covers everything in depth:

- [Reactivity](https://acheul.github.io/ahoi/reactivity/stock/) — `Stock`,
  `Memo`, `Effect`, async `Action`/`Resource`, stock derivation, and `Sphere`
  (the unit of reactive lifetime).
- [The JS bridge](https://acheul.github.io/ahoi/bridge/pier/) — Pier, Hail &
  Tell, converters, and exporting types.
- [Framework guides](https://acheul.github.io/ahoi/frameworks/solid/) — per-
  framework usage with live demos.

---

## Examples

Four playgrounds share one wasm crate, each exercising the same bridge features
(writable hails, memos, path-derived writes, async `Resource`/`Action`, enum and
map values on the wire, nested-pier cleanup), plus a benchmark app:

- [`playgrounds/solid`](playgrounds/solid), [`playgrounds/react`](playgrounds/react),
  [`playgrounds/vue`](playgrounds/vue), [`playgrounds/svelte`](playgrounds/svelte)
- [`playgrounds/ahoi-wasm-tsrs`](playgrounds/ahoi-wasm-tsrs) — the shared Rust side
- [`playgrounds/solid-tsify`](playgrounds/solid-tsify) — the same bridge with
  the [Tsify] exporter
- [`playgrounds/solid-tsain-todo`](playgrounds/solid-tsain-todo) — a small todo
  app on the [Tsain] converter + exporter
- [`playgrounds/bench`](playgrounds/bench) — reactivity micro-benchmarks
  (doubles as the no-framework example)

See [`playgrounds/playgrounds.md`](playgrounds/playgrounds.md) for how to run them.

---

## License

[MIT LICENSE](LICENSE)
