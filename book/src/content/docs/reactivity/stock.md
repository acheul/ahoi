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
into one field and write just that, and only what read _that field_ recomputes
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

Holding one too long panics with a message pointing at your line. The `try_`
forms return an error instead — see [When an access can
fail](#when-an-access-can-fail).
:::

## Read-only views

`ReadStock<T>` is a stock without the write half. `Stock<T>` derefs to it, so
anything that takes a `ReadStock` also takes a `Stock`.

Use it to hand out a value that others should observe but not change.

## Values that might not be there

Some derived values are not guaranteed to exist — an index past the end of a
`Vec`, a missing map key, a field of an enum variant that is not currently
active.

Those are `OptStock<T>` and `OptReadStock<T>`. They have the same methods as
their non-opt counterparts, but every result comes wrapped in an `Option`.
`None` means the value is absent right now — a fact about the data, not an
error.

That is a compile error rather than a runtime surprise: if a value might be
absent, the type makes you say what happens when it is.

A `Vec` stock hands you one of these from `get`, without any derive:

```rust
let items = Stock::new(vec![10, 20]);

let third = items.get(2); // OptStock<i32> — there is no index 2

if let Some(v) = third.read() {
    println!("{}", *v);
}

third.set(99); // None: nothing to write to
```

Writing to something that is not there does nothing. It does not panic.

| Method              | On a stock  | On an opt stock     |
| ------------------- | ----------- | ------------------- |
| `read()` / `peek()` | `Ref<T>`    | `Option<Ref<T>>`    |
| `write()`           | `RefMut<T>` | `Option<RefMut<T>>` |
| `set(v)`            | `()`        | `Option<()>`        |

## When an access can fail

Every method above also has a `try_` twin that returns a `Result`:

| Method                      | On a stock             | On an opt stock                |
| --------------------------- | ---------------------- | ------------------------------ |
| `try_read()` / `try_peek()` | `Result<Ref<T>, _>`    | `Result<Option<Ref<T>>, _>`    |
| `try_write()`               | `Result<RefMut<T>, _>` | `Result<Option<RefMut<T>>, _>` |
| `try_set(v)`                | `Result<(), _>`        | `Result<Option<()>, _>`        |

The error is `BorrowError`, and there are exactly two:

- **`Disposed`** — the sphere that owned the stock was cleared. The usual
  source is async work finishing after its component unmounted.
- **`BorrowConflict`** — a guard on the same value is still alive somewhere up
  the call stack.

The plain methods are the `try_` forms with the error unwrapped — they panic
instead. That is usually what you want: a `BorrowConflict` is a bug in the
code, not a condition to handle.

Reach for `try_` where `Disposed` is a real possibility and you want to bail
out quietly:

```rust
let save: Action<Data, ()> = Action::new(move |data| async move {
    let result = api_save(data).await;
    let Ok(mut status) = state.status().try_write() else {
        return; // the sphere was cleared while we were waiting
    };
    *status = result;
});
```

On an opt stock the two layers stay separate: the `Result` says whether the
access could happen at all, and the `Option` inside says whether the value was
there.

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
