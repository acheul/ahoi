# Ahoi: Reactivity _from_ Rust _to_ JS

> Render components with JS, manage state with Rust — the best of both.

Ahoi is a fine-grained reactive state engine written in Rust. Rust owns the data
and reactivity; a JS framework (e.g. SolidJS) owns rendering. The two talk over a
thin, type-safe wasm bridge.

---

## Core idea

> _"Rust for Rust, Js for Js"_

- **Rust holds the truth** — all state, derivations, and effects live in Rust.
- **JS just renders** — the frontend subscribes to values and pushes user input back.
- **Bridge is bidirectional** — values flow Rust→JS (push) and JS→Rust (write),
  with echo suppression so a write doesn't bounce back as a redundant update.

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

- `#[stock]` adds custom methods to a `Stock<YourType>` via an extension trait.
- **Getter pipeline** — how a sub-stock maps from its root:
  - **Chained** (default for derives) — stack-built, zero state cost; recreated
    each access. Prefer this.
  - **Pooled** (`.pool()`) — stored as state, always `Copy`, lets you drop the
    `Pipe` generic. Handy for context values.

---

## Sphere — the unit of reactive lifetime

- Every state created inside `sphere(parent, || { … })` is **owned by that
  sphere** and lives until `clear_sphere(id)`.
- **`clear_sphere` cascades to children and is order-independent.** Spheres form
  a parent→child tree, so clearing a sphere recursively frees its whole subtree.
  Clearing is idempotent, so a host can wire one `clear_sphere` per component
  (e.g. SolidJS `onCleanup`) and stay correct no matter the teardown order.
- The parent link also drives **context lookup**: `provide_context::<T>(value)` /
  `use_context::<T>()` resolve up the parent chain (React-context style).

---

## JS bridge: Pier & Hail

The bridge exposes two kinds of sphere to the frontend:

- **Pier** — a _scope_ sphere. Sets up context, effects, and shared state for a
  region of the UI. Created via a `PierProvider` (no value returned).
- **Hail** — a _reactive value channel_. Binds one Rust value to a JS signal.
  - `set_read_hail` — read-only (Rust→JS push).
  - `set_hail` — read-write (JS can also write back into the Rust stock).
- **Tell** — a one-shot JS→Rust command (mutate state, call an action, …).

Both Pier and Hail are the same `Sphere` underneath; they're split only at the
key-type surface so JS can't mix them up.

---

## Macros (wasm exports)

- `wasm_bindgen_enrol_sphere!(@pier, PierKey, run_pier, Converter)`
- `wasm_bindgen_enrol_sphere!(@hail, HailKey, run_hail, Converter)`
- `wasm_bindgen_tell!(TellKey, run_tell, Converter)`

A `HailConverter` defines how a Rust value crosses to/from a `JsValue`.

---

## Example

See [`playgrounds/play-rs`](playgrounds/play-rs) (Rust side) and
[`playgrounds/play-solid`](playgrounds/play-solid) (SolidJS side) for a full
working app exercising every primitive.
