# Contributing to ahoi

Thanks for taking a look. ahoi is young, so issues, questions, and small pull
requests are all welcome.

## Before a big change

Open an issue first. It saves you from building something that
does not fit the direction, and it is a good place to work out the design
together. Small fixes (docs, tests, obvious bugs) can go straight to a PR.

## Repo layout

| Path                         | What                                                                     |
| ---------------------------- | ----------------------------------------------------------------------- |
| `packages/ahoi`, `packages/ahoi-core` | the Rust crate: reactivity engine and the wasm bridge          |
| `packages/ahoi-*-macro`      | the derive macros (`Stock`, `Rets`)                                     |
| `js/`                        | `@acheul/ahoi-js`, the JS-side bridge and the framework adapters        |
| `book/`                      | the guide and its live examples                                         |
| `playgrounds/`               | one wasm crate per type exporter, plus a per-framework app that exercises the bridge |

## Building and testing

One-time setup:

```sh
pnpm install
pnpm -C js build
```

Rust (this is what CI runs):

```sh
cargo test --workspace --all-targets
cargo test --workspace --doc
cargo check --workspace --all-targets --release
```

Tests must run with debug assertions on: the caller-location tracking ahoi
relies on only exists in that build. The separate `--release` check keeps the
`#[cfg(not(debug_assertions))]` paths from bit-rotting, and
`scripts/check-wasm-release.sh` (also in CI) fails if a release wasm build
still carries source paths.

JS:

```sh
pnpm -C js test
```

Running the framework examples is described in
[`playgrounds/playgrounds.md`](playgrounds/playgrounds.md). The book runs with
`pnpm -C book dev`.

## Pull requests

- Keep them focused: one topic per PR.
- Run `cargo fmt` before committing, and match the style of the code around you.
- Add or update tests for behavior changes. Core reactivity tests live in
  `packages/ahoi-core`; bridge behavior is covered by the playground apps.
- If you change a key enum in a playground, regenerate its bindings
  (`cargo test -p <crate>`), as noted in `playgrounds.md`.
- Update the book when you change public API.

## License

By contributing you agree that your work is licensed under the
[MIT License](LICENSE), the same as the rest of the project.
