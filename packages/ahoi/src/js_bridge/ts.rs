//! Ret-map declarations for the JS bridge.
//!
//! ahoi does not convert your types to TypeScript: pick any exporter (ts-rs,
//! Tsify, Tsain, hand-written `.d.ts`). ahoi only describes what key variants
//! *return*: `#[derive(Rets)]` renders a `{Enum}Rets` map.
//!
//! - [`TsDecl`] is that rendered map. It is available everywhere, because the
//!   derive emits an impl of it wherever it is used, wasm included.
//! - `TsFile` collects the maps and writes the `.ts` file. Host-only: writing
//!   files is what a `#[test] fn generate()` does, and a wasm build has no use
//!   for it, so it is absent there. (Not linked from here on purpose: the link
//!   would dangle in a wasm build.)

#[cfg(not(target_arch = "wasm32"))]
mod generate;
#[cfg(not(target_arch = "wasm32"))]
pub use generate::TsFile;

/// A TS declaration contributed to the generated file.
///
/// Implemented by `#[derive(Rets)]` for key enums; collected by `TsFile`.
pub trait TsDecl {
    fn ts_decl() -> String;
}
