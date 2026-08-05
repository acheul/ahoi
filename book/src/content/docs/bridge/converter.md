---
title: Converter
description: How values physically cross between Rust and JS, and how to replace the default.
sidebar:
  order: 6
---

Types describe what crosses the bridge. A **converter** does the crossing.

Every key enum and every hail value passes through one.

## The ready-made ones

Three ship with the crate, behind features. All work for anything that is
`Serialize + DeserializeOwned`.

| Feature | Converter | Wire type |
| --- | --- | --- |
| `serde-wasm-bindgen` | `SerdeWasmBindgenConverter` | `JsValue` |
| `tsain` | `TsainConverter` | `JsValue` |
| `serde_json` | `SerdeJsonConverter` | `serde_json::Value` |

**`serde-wasm-bindgen` is the one to reach for.** It builds native JS values
directly, with no intermediate JSON.

Pick `tsain` when you export types with
[Tsain](../exporting-types/#with-tsain). Values cross as positional arrays —
no field or variant names on the wire. That converts faster and keeps your
names out of the emitted JS. The two halves must match: this converter expects
the array shapes that Tsain's export describes.

Pick `serde_json` when you would rather work in `serde_json::Value` on the Rust
side — a type that already goes through JSON, or code you share with a non-wasm
target. Note that an absent value arrives as `null` there, not `undefined`.

## Using one

Turn on the feature, then alias the converter. The rest of the book assumes
`serde-wasm-bindgen`.

```toml
ahoi = { version = "0.1", features = ["serde-wasm-bindgen"] }
```

```rust
use ahoi::js_bridge::SerdeWasmBindgenConverter as Converter;
```

Every converter needs `Serialize + DeserializeOwned`, which is why the key
enums derive both. Aliasing to `Converter` is also what keeps a switch to a
one-line change.

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

`TsainConverter` differs on structs and enums: they cross as positional
arrays, with no names. Its generated file ships the factories and getters
that read them — see [Exporting types](../exporting-types/#with-tsain).

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
