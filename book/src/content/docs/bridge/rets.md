---
title: Rets
description: How each key declares what it returns, so the JS side gets a real type instead of unknown.
sidebar:
  order: 4
---

A key on its own does not say what it gives back. `"Count"` is just a string.

`#[derive(Rets)]` fixes that. You annotate each variant with what it returns,
and ahoi collects those into one TypeScript map.

## Declaring

```rust
#[derive(Rets, Serialize, Deserialize)]
pub enum Hail {
    #[ret(i32)]
    Count,
    #[ret(Vec<(String, Fruit)>)]
    Fruits,
    #[ret(Option<i32>)]
    Item(usize),
}
```

Run the generator and you get:

```ts title="bindings/Rets.ts"
export type HailRets = {
  Count: number;
  Fruits: [string, Fruit][];
  Item: number | undefined;
};
```

Hand `HailRets` to `createAhoi` and `sphere.hail("Count")` is a `number`.

## Referenced types need nothing

`Fruit` above has no `Rets` derive and no trait implementation.

Ret types are rendered **syntactically**, at macro expansion time. `Fruit`
becomes the identifier `Fruit`, which is what every mainstream exporter names
it. That is the whole trick, and it is why ahoi stays out of the type-conversion
business.

You still get typo protection. The derive generates a compile-time assertion, so
`#[ret(Fruti)]` fails to build.

## When the type does not map cleanly

Sometimes the Rust type and the TypeScript type genuinely differ. Write the
TypeScript directly:

```rust
#[ret(ts = "MyHandWrittenType")]
Custom,
```

Whatever you put in the string is emitted as-is. This is the escape hatch — use
it when you need it, not by default.

## Generating the file

`TsFile` collects the maps and writes them. A test is the usual place, so
`cargo test` keeps the output fresh.

```rust
#[test]
fn generate() {
    ahoi::js_bridge::TsFile::new()
        .import("Fruit", "./Fruit")
        .with::<Hail>()
        .with::<Tell>()
        .export("./bindings/Rets.ts");
}
```

`.import(..)` adds an import line to the generated file. Use it for every type
your rets reference, pointing at wherever your exporter put it.

```ts title="bindings/Rets.ts"
import type { Fruit } from "./Fruit";
```

## Serde rules

Key enums assume serde's **default, externally tagged** representation. That is
what makes `"Count"` and `{ Item: 3 }` line up on both sides.

Attributes that would break that alignment are **compile errors**, not silent
mismatches:

- renaming variants (`#[serde(rename = "...")]`, `rename_all`)
- changing the enum representation (`tag`, `untagged`)

If the build rejects one of these, the fix is to change the variant name in
Rust rather than rename it in serde.

## Next

Rets covers the return types. [Exporting types](../exporting-types/) covers
everything else.
