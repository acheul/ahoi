---
title: Getter pipeline
description: How a derived stock finds its way back to the root, and when to pool it.
sidebar:
  order: 6
---

Most people never touch this page. You come here when you try to **store** a
derived stock — in a struct field or in context — and the type turns out to be
unwieldy.

## The extra type parameter

`state.count()` does not copy the value out. It hands you a handle that knows
how to walk from the root to that field.

That walk is carried in a type parameter:

```rust
Stock<i32, ChainedPipe<...>, false>
//    ^^^  ^^^^^^^^^^^^^^^^  ^^^^^
//    the value              may be absent
//         how to reach it
```

Every derivation step adds a layer. `state.items().get(0)` has a deeper
pipeline than `state.count()`.

For local use this is invisible — inference handles it:

```rust
let count = state.count();
*count.write() += 1;
```

## Chained, the default

Derived accessors give you a **chained** pipeline. It is built on the stack and
costs no stored state, and it is recreated each time you access it.

This is what you want almost always. It is free.

The catch is the type. A chained pipeline spells out every step it took, so the
type is long and changes if you add a derivation step. Fine for a local
variable, awkward for a struct field.

## Pooled, when you need to store it

`.pool()` stores the pipeline as state. The handle becomes `Copy`, and the type
collapses to something you can actually write down:

```rust
let count = state.count().pool(); // Stock<i32, PooledPipe<i32>>
```

Reach for it when a derived stock has to be:

- put in context
- held in a struct field
- captured by many closures that each need it to be `Copy`

```rust
#[derive(Clone, Copy)]
struct Count(Stock<i32, PooledPipe<i32>>);

provide_context(Count(state.count().pool()));
```

:::caution
Do not call `.pool()` inside a reactive closure — a memo, effect, or runner.

Pooling allocates a mapper state that lives until the sphere is cleared, so a
closure that pools on every run leaks one per run. Derive inline there instead:

```rust
// in a memo or effect
let count = state.count(); // chained, free
```
:::

## Choosing

| Situation | Use |
| --- | --- |
| Local variable, inside a runner | chained (the default) |
| Struct field or context value | `.pool()` |
| Anywhere inside a closure that re-runs | chained — never pool here |

If you are not sure, use the default. You will find out when the compiler asks
you to name a type.

## Next

That is the Rust side. From here, the [framework
guides](../../bridge/pier/) cover what each adapter does differently.
