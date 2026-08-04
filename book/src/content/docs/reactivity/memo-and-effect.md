---
title: Memo and Effect
description: Derived values that cache, and side effects that re-run. Both track their dependencies for you.
sidebar:
  order: 3
---

You never register a dependency by hand. Any `read()` inside a memo or effect
records one, and that is what decides what re-runs later.

## Memo

A `Memo<T>` is a cached derived value.

```rust
let count = Stock::new(1i32);
let doubled = Memo::new(move || *count.read() * 2);

println!("{}", *doubled.read()); // 2
```

Or straight from a stock:

```rust
let doubled = count.memo(|c| *c * 2);
let total = state.items().memo(|v| v.iter().sum::<i32>());
```

A memo recomputes when an input changes. It **propagates only when the result
actually differs**, which is why the result type needs `PartialEq`.

That second part matters more than it sounds:

```rust
let parity = count.memo(|c| *c % 2);
let label = parity.memo(|p| if *p == 0 { "even" } else { "odd" });
```

Changing `count` from 1 to 3 recomputes `parity`, gets `1` again, and stops.
`label` never runs. Chains stay cheap without you checking anything.

A memo is lazy. It computes on first read, then only when something it read has
changed.

## Effect

An `Effect` runs for its side effect, and re-runs when its dependencies change.

```rust
let logger = Effect::new(move || {
    log(&format!("count is {}", *count.read()));
});
```

Use it for things outside the reactive graph — logging, storage, calling out to
a browser API.

Do not use one to compute a value. That is what a memo is for, and a memo will
be both cheaper and easier to follow.

## Reading without subscribing

`read()` subscribes. `peek()` does not.

```rust
let ratio = Memo::new(move || {
    let n = *numerator.read(); // re-runs when this changes
    let d = *denominator.peek(); // does not
    n / d
});
```

Use `peek()` when you need a value but do not want it to be a trigger. A common
case is a helper method that should not drag extra dependencies into whichever
memo happens to call it.

## Grouping writes

Every write propagates when its guard drops. To make several writes settle
together, wrap them in `batch`:

```rust
batch(|| {
    state.count().set(0);
    state.items().write().clear();
});
```

Dependents run once at the end instead of after each write.

You rarely need this from a tell — the bridge already batches everything it
dispatches to JS in one pass. Reach for it when a single logical change touches
several stocks and an effect in between would see a half-updated state.

## Next

Both of these are synchronous. [Async work](../async/) covers the rest.
