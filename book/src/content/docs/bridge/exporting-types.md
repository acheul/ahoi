---
title: Exporting types
description: Ahoi is not a Rust-to-TypeScript converter. Pick your own exporter and let ahoi fill the one gap.
sidebar:
  order: 5
---

Ahoi does **not** convert Rust types to TypeScript.

That is a deliberate limit. Good converters already exist, they disagree about
details, and re-implementing one inside ahoi would only make you choose again.

## Split the job

Two kinds of type cross the bridge, and they are handled by different tools.

| What                                                      | Who handles it                            |
| --------------------------------------------------------- | ----------------------------------------- |
| Your key and data types (`Pier`, `Hail`, `Tell`, `Fruit`) | Your exporter: [ts-rs], [Tsify], [Tsain], … |
| What each key **returns**                                 | Ahoi's [`#[derive(Rets)]`](../rets/)      |

No general-purpose converter can do the second one. It is not a property of a
type, it is a property of a key.

[Tsain] is the one exception — it covers both rows at once. See
[With Tsain](#with-tsain) below.

## With ts-rs

Derive `TS` on the types you want exported and add `#[ts(export)]`. Running
`cargo test` writes them.

```rust
#[derive(Rets, TS, Serialize, Deserialize)]
#[ts(export)]
pub enum Hail {
    #[ret(i32)]
    Count,
    #[ret(Option<i32>)]
    Item(usize),
}
```

```ts title="bindings/Hail.ts"
export type Hail = "Count" | { Item: number };
```

That is your key type. `Rets.ts` is the return map. You need both.

## With Tsify

[Tsify] works too, and it needs no export step. The declarations are embedded
in the `.d.ts` that `wasm-pack build` writes.

```toml title="Cargo.toml"
tsify = { version = "0.5", default-features = false, features = ["js"] }
```

```rust
#[derive(Rets, Tsify, Serialize, Deserialize)]
pub enum Hail {
    #[ret(i32)]
    Count,
    #[ret(Option<i32>)]
    Item(usize),
}
```

Import the key types from the wasm pkg itself:

```ts
import type { Hail } from "./pkg/my_wasm";
```

The derive alone is enough. Skip `#[tsify(into_wasm_abi)]` and
`#[tsify(from_wasm_abi)]` — ahoi's bridge converts values itself, and tsify
0.5 deprecates those attributes anyway.

One detail changes: if a ret map references one of your data types, its
import must point at the pkg, since there are no per-type binding files.

```rust
TsFile::new()
    .import("Fruit", "../pkg/my_wasm")
    .with::<Hail>()
    .with::<Tell>()
    .export("./bindings/Rets.ts");
```

## With Tsain

[Tsain] is different in kind: it is a converter **and** an exporter in one.
Values cross the wire as positional arrays, and the export step writes the
matching TypeScript.

The return type moves onto the key itself, as a brand:

```rust
#[derive(Tsain, Serialize, Deserialize)]
pub enum Hail {
    #[tsain(brand(ret = i32))]
    Count,
    #[tsain(brand(ret = Option<i32>))]
    Item(usize),
}
```

No `Rets` derive. No `TsFile`. One generator writes everything:

```rust
#[test]
fn generate() {
    tsain::TsScript::export("./bindings/Tsain.ts");
}
```

The file holds the types plus factory functions and getters. You need them —
a positional array has no field names to read:

```ts
import { HailCount_, HailItem_ } from "./bindings/Tsain";

pier.hail(HailCount_()); // () => number
pier.hail(HailItem_(3)); // () => number | undefined
```

Drop the ret-map generics from `createAhoi`; the brands carry the types:

```ts
createAhoi<Pier, Hail, Tell>({
  /* wasm exports */
});
```

Values must cross in the same array format, so pair this with the `tsain`
crate feature and `TsainConverter` — see [Converter](../converter/).

## Wiring them together

The five generic parameters on `createAhoi` are, in order: the pier key, the
hail key, the tell key, the hail ret map, and the tell ret map.

```ts
import type { Pier } from "./bindings/Pier";
import type { Hail } from "./bindings/Hail";
import type { Tell } from "./bindings/Tell";
import type { HailRets, TellRets } from "./bindings/Rets";

export const { PierProvider, usePier } = createAhoi<
  Pier,
  Hail,
  Tell,
  HailRets,
  TellRets
>({
  /* wasm exports */
});
```

The key types make `pier.hail("Cont")` a compile error. The ret maps make
`pier.hail("Count")` a `number`.

## If your exporter brands keys

Some setups attach the return type to the key itself rather than listing it in a
map — [Tsain] above is one. Ahoi handles that too.

The JS side resolves a key's return type in this order:

1. a `ret` brand on the key, if your converter produced one
2. the variant name, looked up in the `Rets` map
3. a fallback — `unknown` for a hail, `undefined` for a tell

So the bridge stays converter-agnostic. Use whichever style your exporter
produces.

## Keeping it fresh

Generation runs under `cargo test`, which means it is easy to forget in CI.

Two habits help:

- Commit the generated `bindings/` directory, so a stale file shows up in a diff.
- Run `cargo test` before `wasm-pack build` in your build script.

## Next

Values still have to physically cross the boundary. That is the
[converter](../converter/).

[ts-rs]: https://github.com/Aleph-Alpha/ts-rs
[Tsify]: https://github.com/madonoharu/tsify
[Tsain]: https://crates.io/crates/tsain
