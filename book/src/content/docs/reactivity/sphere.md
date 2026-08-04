---
title: Sphere and lifetime
description: The unit that owns reactive state. Everything created inside a sphere lives and dies with it.
sidebar:
  order: 5
---

A sphere is the **unit of reactive lifetime**. Every stock, memo, effect,
callback, action, and resource belongs to the sphere it was created in.

If you are using the JS bridge, you get spheres through
[piers](../../bridge/pier/) and hails. This page is what sits underneath them.

## Creating one

```rust
let (id, result) = make_sphere(parent_id, || {
    let state = Stock::new(State::default());
    provide_context(state);
    state
});
```

Everything created inside the closure is owned by the new sphere. You get back
its id and whatever the closure returned.

`make_top_sphere()` creates an empty sphere with no parent, for a root.

## Clearing

```rust
clear_sphere(id);
```

That frees every state the sphere owns. You never free individual values.

Two properties make this safe to wire into a UI framework:

- **It cascades.** Spheres form a parent-child tree, and clearing one clears its
  whole subtree.
- **It is idempotent and order-independent.** Clearing an already-cleared sphere
  does nothing.

Together these mean a host can register one `clear_sphere` per component and
stay correct no matter which order things unmount in. Whether a child clears
itself first or the parent's cascade reaches it first, the second call finds
nothing to do.

## Context

The parent link also drives context lookup.

```rust
provide_context(Stock::new(State::default()));

let state = use_context::<Stock<State>>().unwrap();
```

`use_context` walks **up** the parent chain and returns `None` if no ancestor
provided that type. Context is keyed by type, and the nearest one wins.

Because it is keyed by type, two values of the same type collide. Wrap them:

```rust
#[derive(Clone, Copy)]
struct PanelTitle(Stock<String>);

#[derive(Clone, Copy)]
struct PanelSubtitle(Stock<String>);
```

Context values must be `Clone`. Stocks and callbacks are `Copy`, so in practice
you are storing handles, not data.

## Knowing where you are

`current_sphere_id()` returns the sphere currently being built or run, if any.

```rust
if let Some(id) = current_sphere_id() { /* ... */ }
```

You mostly need this when writing your own integration rather than using an
adapter.

## Batching across a sphere

`batch` groups writes into one propagation. `batch_with_sphere` does the same
while entering a specific sphere first — useful when you are writing state from
outside any runner.

```rust
batch_with_sphere(sphere_id, || {
    state.count().set(0);
    state.items().write().clear();
});
```

## Next

One last piece: how a derived stock finds its way back to the root, and the one
knob you might want to turn. See the [getter pipeline](../getter-pipeline/).
