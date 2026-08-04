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

| What | Who handles it |
| --- | --- |
| Your key and data types (`Pier`, `Hail`, `Tell`, `Fruit`) | Your exporter: [ts-rs], [Tsify], … |
| What each key **returns** | Ahoi's [`#[derive(Rets)]`](../rets/) |

No general-purpose converter can do the second one. It is not a property of a
type, it is a property of a key.

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
export type Hail = "Count" | { "Item": number };
```

That is your key type. `Rets.ts` is the return map. You need both.

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
>({ /* wasm exports */ });
```

The key types make `sphere.hail("Cont")` a compile error. The ret maps make
`sphere.hail("Count")` a `number`.

## If your exporter brands keys

Some setups attach the return type to the key itself rather than listing it in a
map. Ahoi handles that too.

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
