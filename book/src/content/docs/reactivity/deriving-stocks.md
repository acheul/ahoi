---
title: Deriving stocks
description: Reach into nested state with generated accessors, and notify only the paths that actually changed.
sidebar:
  order: 2
---

`#[derive(Stock)]` generates one accessor per field, so you can drill into
nested state and get a reactive handle on just that part.

```rust
#[derive(Stock)]
struct State {
    count: i32,
    items: Vec<i32>,
}

let state = Stock::new(State { count: 0, items: vec![] });

let count = state.count(); // Stock<i32>
let items = state.items(); // Stock<Vec<i32>>
```

Each accessor is itself a stock. You can read it, write it, memo it, or turn it
into a hail.

## Collections

`Vec` and `HashMap` stocks get `get`, which returns an [opt
stock](../stock/#values-that-might-not-be-there) — the entry may not exist.

```rust
let first = state.items().get(0); // OptStock<i32>
let apples = state.fruits().get("apple"); // OptStock<u32>
```

## Enums

On an enum, accessors are generated per variant. A variant is only active some
of the time, so these are opt stocks too.

```rust
#[derive(Stock)]
enum Shape {
    Circle(f32),
    Rect { w: f32, h: f32 },
}

let radius = shape.circle(); // OptStock<f32> — None unless it is a Circle
```

Accessor names are the variant name in snake case, so `Rect` becomes `rect()`.

## Writes notify only the path that changed

This is the part worth understanding.

Derivation is **path-selective**. Writing `items[0]` notifies whatever is
watching `items[0]`, and its ancestors — `items`, and the root. It does not
notify `items[1]`.

```rust
*state.items().get(0).write() = 5;
```

- a hail on `Item(0)` → recomputed
- a hail on `Items` → recomputed (an ancestor)
- a hail on `Item(1)` → untouched
- a hail on `Count` → untouched

So a list of a thousand rows does not rerender because one row changed. You get
that without writing any comparison logic.

## Adding your own methods

`#[stock]` attaches methods to `Stock<YourType>` through an extension trait.

```rust
#[derive(Stock)]
struct Pair {
    x: u32,
    y: u32,
}

#[stock]
impl Stock<Pair> {
    fn sum(&self) -> u32 {
        *self.x().peek() + *self.y().peek()
    }

    fn swap(&self) {
        let x = *self.x().peek();
        let y = *self.y().peek();
        self.x().set(y);
        self.y().set(x);
    }
}

pair.sum();
pair.swap();
```

Use it to keep state logic next to the state instead of scattered across
runners.

Pass a name if you want to control the generated trait:

```rust
#[stock(PointOps)]
impl Stock<Point> { /* ... */ }
```

:::note
Note the `peek()` calls above. A helper that reads with `read()` inside a memo
would subscribe that memo to every field it touched. `peek()` reads the value
without recording a dependency — usually what you want in a plain helper.
:::

## Skipping a field

Mark a field or variant to leave it out of the generated accessors:

```rust
#[derive(Stock)]
struct State {
    count: i32,
    #[stock(skip)]
    scratch: Vec<u8>,
}
```

## Next

Accessors give you the values. [Memo and Effect](../memo-and-effect/) is how you
react to them.
