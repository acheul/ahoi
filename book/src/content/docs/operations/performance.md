---
title: Performance
description: What a boundary crossing costs, what ahoi already does for you, and the few choices that actually matter.
sidebar:
  order: 2
---

The thing to understand is that **cost scales with crossings, not with the
amount of state**. A large store that changes rarely is cheap. A small one that
chatters is not.

## What ahoi already does

Three of these are automatic, and together they mean most apps never need to
tune anything.

**One dispatch per propagation.** Every hail recomputed by a single change is
collected and sent to JS in one batch. A write that touches twenty hails costs
one crossing, not twenty.

**Path-selective propagation.** Writing `items[0]` notifies whatever watches
`items[0]` and its ancestors. It does not touch `items[1]`. A thousand-row list
does not wake up because one row changed.

**Memos stop early.** A memo propagates only when its result actually differs.
Change `count` from 1 to 3 and a `count % 2` memo recomputes, gets the same
answer, and stops — nothing downstream runs.

## Rough scale

A round trip across the boundary — write from JS, propagate in Rust, dispatch
back — is on the order of **a microsecond**.

That is the number to reason with. Thousands per frame are fine. Hundreds of
thousands are not, and neither would they be in pure JS.

Treat this as an order of magnitude, not a specification. It varies with your
converter, your payload size, and the machine.

## What you control

### Hail the value you render, not its container

```rust
// good — one hail per rendered value
#[ret(i32)] Count,
#[ret(String)] Title,

// costly — the whole struct crosses whenever any field changes
#[ret(State)] Everything,
```

A hail on a container recomputes and re-serialises the whole thing on any change
inside it. Derive down to what the component actually displays.

### Prefer a tell for multi-step changes

Writing three hails from JS is three crossings. A tell that makes all three
changes in Rust is one.

```ts
// three crossings
setFirst(a); setSecond(b); setThird(c);

// one
tell({ ApplyPreset: "compact" });
```

This is the same advice as "keep rules in Rust", arriving from the other
direction.

### Batch when writing several stocks in Rust

Inside Rust, wrap a multi-stock change so dependents settle once:

```rust
batch(|| {
    state.count().set(0);
    state.items().write().clear();
});
```

You rarely need this inside a tell — the dispatch to JS is batched regardless.
It matters when an effect in between would otherwise observe a half-updated
state.

### Watch serialisation, not the bridge

For anything beyond trivial payloads, the time goes into converting values, not
into the crossing itself.

If a hail is hot and its value is large, the fix is usually a narrower ret type
rather than anything structural.

## When to measure

Reach for a profiler when a specific interaction feels slow, and look at two
things first: how many dispatches a single user action produces, and how large
the values in them are.

A silently-broken propagation often looks *fast* — fewer dispatches than there
should be. If an interaction got quicker and also stopped updating something,
that is the bug, not a win.
