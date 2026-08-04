---
title: What is Ahoi?
description: The mental model behind Ahoi — Rust owns the state, your JS framework renders it.
sidebar:
  order: 1
---

Ahoi is a reactive state engine written in Rust.

Rust owns the data and the reactivity. Your JS framework owns rendering. The two
talk over a thin wasm bridge.

## The split

Most wasm frontends make you choose. Either you write the whole UI in Rust, or
you keep the state in JS and use Rust for a few hot functions.

Ahoi takes a third path:

- **Rust holds the truth.** State, derived values, and effects all live there.
- **JS just renders.** Components read values and send user input back.
- **The bridge runs both ways.** Values are pushed Rust to JS. Writes go JS to Rust.

You keep your component model, your router, and your ecosystem. You move the
state out.

## Three things cross the bridge

Everything the frontend touches is one of three kinds of key.

| Key      | What it does                                        |
| -------- | --------------------------------------------------- |
| **Pier** | Sets up a scope. State and context for part of the UI. |
| **Hail** | Binds one Rust value to one JS signal.               |
| **Tell** | Sends a one-shot command to Rust.                    |

Keys are plain values, like `"Count"` or `{ Item: 3 }`. There are no
constructors and no wrapper objects.

## Typed both ways

The JS side knows what each key returns.

You annotate each key in Rust with `#[ret(..)]`. Ahoi collects those into a
TypeScript map. So `sphere.hail("Count")` is a `number`, and a typo is a
compile error.

Ahoi is **not** a Rust-to-TypeScript converter. Use [ts-rs] or [Tsify] for your
own types. Ahoi only adds the part no general converter knows: what each key
gives back.

## Which frameworks

Adapters ship for **Solid**, **React**, **Vue**, and **Svelte**. Each is a
subpath of one npm package, and the framework itself is an optional peer
dependency.

Preact works through the React adapter. For anything else, the core is
framework-agnostic.

## When to use it

Ahoi fits when the state is worth the crossing:

- Logic you already have in Rust, or want to share with a native app.
- Rules that must not drift between clients.
- Derived state with real depth, where recomputation should be precise.

It is a poor fit for a page whose state is a couple of booleans. The bridge is
cheap, but it is not free.

## Next

Install the crate and the npm package, then build a counter.

[ts-rs]: https://github.com/Aleph-Alpha/ts-rs
[Tsify]: https://github.com/madonoharu/tsify
