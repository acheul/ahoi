---
title: Stock
description: The reactive value that owns your state. Read it, write it, and everything watching it follows.
sidebar:
  order: 1
---

A `Stock<T>` is a reactive value. It holds the data, and anything that read it
gets recomputed when it changes.

Most JS frameworks split that job in two: a signal for a single value, and a
store for a nested object you want to update field by field. **A stock is
both.** `Stock<i32>` behaves like a signal. A stock of a struct lets you reach
into one field and write just that, and only what read *that field* recomputes
— which is what [deriving stocks](../deriving-stocks/) is about.

This page is the single-value half. Everything below works on any stock.

:::note[Coming from Solid?]
A stock is `createSignal` and `createStore` in one type. You do not pick up
front, and you do not convert between them later.
:::

```rust
let count = Stock::new(0i32);

*count.write() += 1;

println!("{}", *count.read()); // 1
```

## Reading

`read()` returns a guard that derefs to `&T`.

```rust
let name = Stock::new(String::from("ahoi"));

let n = *count.read();
let len = name.read().len(); // the guard derefs, so `&str` methods work
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
let items = Stock::new(vec![10, 20]);

*count.write() += 1;
items.write().push(30);
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

Those are `OptStock<T>` and `OptReadStock<T>`. They do not have the plain
`read()` / `write()` API at all — only the `try_` forms, which return an
`Option`.

That is a compile error rather than a runtime surprise: if a value might be
absent, the type stops you from reading it as though it were not.

A `Vec` stock hands you one of these from `get`, without any derive:

```rust
let items = Stock::new(vec![10, 20]);

let third = items.get(2); // OptStock<i32> — there is no index 2

if let Some(v) = third.try_read() {
    println!("{}", *v);
}

third.try_set(99); // None: nothing to write to
```

Writing to something that is not there does nothing. It does not panic.

| Method | On a stock | On an opt stock |
| --- | --- | --- |
| `read()` / `peek()` | the value | **does not exist** |
| `write()` / `set(v)` | writes | **does not exist** |
| `try_read()` / `try_peek()` | `Option<Ref<T>>` | `Option<Ref<T>>` |
| `try_write()` | `Option<RefMut<T>>` | `Option<RefMut<T>>` |
| `try_set(v)` | `Option<()>` | `Option<()>` |

The `try_` forms exist on both, so code that does not care which kind it has
can just use them throughout.

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
