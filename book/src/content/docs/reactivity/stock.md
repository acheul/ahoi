---
title: Stock
description: The reactive value that owns your state. Read it, write it, and everything watching it follows.
sidebar:
  order: 1
---

A `Stock<T>` is a reactive value. It holds the data, and anything that read it
gets recomputed when it changes.

```rust
let count = Stock::new(0i32);

*count.write() += 1;

println!("{}", *count.read()); // 1
```

## Reading

`read()` returns a guard that derefs to `&T`.

```rust
let n = *count.read();
let name_len = state.name().read().len();
```

Reading inside a memo, effect, or hail **records a dependency**. That is how
ahoi knows what to recompute later.

Sometimes you want the value without subscribing to it. Use `peek()`:

```rust
let n = *count.peek(); // no dependency recorded
```

Reach for `peek` when you need a current value but do not want changes to it to
trigger a rerun.

## Writing

`write()` returns a mutable guard. Dropping it triggers propagation.

```rust
*count.write() += 1;

state.items().write().push(7);
```

`set` replaces the value outright:

```rust
count.set(10);
```

:::caution
While a write guard is alive, that value cannot be read again. Keep guards in a
small scope:

```rust
let doubled = {
    let mut c = count.write();
    *c += 1;
    *c * 2
}; // guard dropped here
```

Holding one too long panics with a message pointing at your line.
:::

## Read-only views

`ReadStock<T>` is a stock without the write half. `Stock<T>` derefs to it, so
anything that takes a `ReadStock` also takes a `Stock`.

Use it to hand out a value that others should observe but not change.

## Values that might not be there

Some derived values are not guaranteed to exist — an index past the end of a
`Vec`, a missing map key, a field of an enum variant that is not currently
active.

Those are `OptStock<T>` and `OptReadStock<T>`. They have the same API, plus
`try_` variants that return `Option` instead of panicking.

```rust
let third = state.items().get(2); // OptStock<i32>

if let Some(v) = third.try_read() {
    println!("{}", *v);
}

third.try_set(99); // None if index 2 does not exist
```

Writing to something that is not there does nothing. It does not panic.

| Method | On a stock | On an opt stock |
| --- | --- | --- |
| `read()` / `write()` | fine | panics if absent |
| `try_read()` / `try_write()` | — | `Option` |
| `set(v)` | fine | panics if absent |
| `try_set(v)` | — | `Option<()>` |

## Deriving a value

`memo` builds a cached value from a stock. It recomputes only when the input
changes, and only propagates when the **result** actually differs.

```rust
let doubled = count.memo(|c| *c * 2);
```

There is more on this in [Memo and Effect](../memo-and-effect/).

## Where stocks live

A stock is owned by the sphere that created it, and lives until that sphere is
cleared. You never free one by hand.

See [Sphere and lifetime](../sphere/).

## Next

A stock of a struct is not very useful on its own. [Deriving
stocks](../deriving-stocks/) is how you reach into it.
