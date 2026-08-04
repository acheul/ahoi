---
title: Converter
description: How values physically cross between Rust and JS, and how to replace the default.
sidebar:
  order: 6
---

Types describe what crosses the bridge. A **converter** does the crossing.

Every key enum and every hail value passes through one.

## The default

The `serde-wasm-bindgen` feature gives you a ready-made converter.

```toml
ahoi = { version = "0.1", features = ["serde-wasm-bindgen"] }
```

```rust
use ahoi::js_bridge::SerdeWasmBindgenConverter as Converter;
```

It works for anything that is `Serialize + DeserializeOwned`, which is why the
key enums derive both.

You name it once per macro:

```rust
wasm_bindgen_enrol_sphere!(@pier, Pier, run_pier, Converter);
wasm_bindgen_enrol_sphere!(@hail, Hail, run_hail, Converter);
wasm_bindgen_tell!(Tell, run_tell, Converter);
```

And once per hail:

```rust
Hail::Count => state.count().set_hail::<Converter>(),
```

## What crosses as what

`serde-wasm-bindgen` maps Rust values to their natural JS shapes.

| Rust | JavaScript |
| --- | --- |
| `i32`, `f64` | `number` |
| `String` | `string` |
| `Vec<T>` | `Array` |
| `HashMap<K, V>` | `Map` — not a plain object |
| `Option<T>` — absent | `undefined` |
| enum variant | `"Name"` or `{ Name: value }` |

The `HashMap` one catches people out. You get a real `Map`, so read it with
`.get(k)`, not `obj[k]`.

## Writing your own

`HailConverter<T>` is small.

```rust
pub trait HailConverter<T>: Sized {
    type HailValue;

    /// What an absent value looks like.
    const NONE: Self::HailValue;

    fn from_raw_value(raw_value: &T) -> Self::HailValue;
    fn into_raw_value(hail_value: Self::HailValue) -> T;
}
```

`HailValue` is the wire type — `JsValue` for the wasm bridge. `NONE` is what an
`OptStock` sends when the value is not there.

Implement it when you want a different serialisation format, or tighter control
over how a specific type crosses.

```rust
pub struct MyConverter;

impl<T: MyTrait> HailConverter<T> for MyConverter {
    type HailValue = JsValue;
    const NONE: Self::HailValue = JsValue::undefined();

    fn from_raw_value(raw_value: &T) -> JsValue { /* ... */ }
    fn into_raw_value(hail_value: JsValue) -> T { /* ... */ }
}
```

Then use `MyConverter` in place of the default. Nothing else changes.

:::note
Conversion is not allowed to fail. There is no `Result` — a value that cannot
cross is a bug in your converter, not a runtime condition to handle.
:::

## Version skew

The crate and the npm package agree on an `ABI_VERSION`, checked when the
bridge starts.

A mismatch fails immediately with a clear message, instead of surfacing later as
a confusing runtime error. If you see it, your crate and npm package versions
have drifted apart — update both.

## Next

That is the whole bridge. Next come the Rust-side primitives that make the
values reactive in the first place.
