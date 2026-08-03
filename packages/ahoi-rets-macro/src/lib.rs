//! `#[derive(Rets)]` — ret-map generation for ahoi's JS bridge.
//!
//! ahoi does **not** convert your types to TypeScript; use whatever exporter
//! you like (ts-rs, Tsify, Tsain, hand-written `.d.ts`, ...). What ahoi needs
//! is one extra piece of information no general-purpose converter carries:
//! *which value each key variant returns*. This derive emits exactly that.
//!
//! On a key enum (Hail / Tell), the `#[ret(...)]` annotations are collected
//! into a single `{Enum}Rets` TS map, keyed by variant name:
//!
//! ```rust, ignore
//! #[derive(Rets, Serialize, Deserialize)]
//! pub enum Hail {
//!     #[ret(i32)]
//!     Count,
//!     #[ret(Vec<(String, Fruit)>)]
//!     Fruits,
//!     #[ret(ts = "number | undefined")] // escape hatch: literal TS
//!     Item(usize),
//!     Unannotated, // simply omitted from the map
//! }
//! ```
//!
//! ```ts
//! // generated (collect via `ahoi::ts::TsFile`):
//! export type HailRets = {
//!   Count: number;
//!   Fruits: [string, Fruit][];
//!   Item: number | undefined;
//! };
//! ```
//!
//! The JS bridge resolves a key's return type against this map by variant name
//! (or, for converters like Tsain that brand keys with `ret` directly, from
//! the brand — see `KeyRet` in the npm package).
//!
//! ## Type rendering
//!
//! `#[ret(<Type>)]` types are rendered to TS *syntactically*: primitives,
//! `Option`/`Vec`/sets (`T[]`), maps (`Map<K, V>`), tuples and smart pointers
//! are recognized; any other path type is rendered by its identifier
//! (`Fruit` → `Fruit`, `Foo<i32>` → `Foo<number>`), which matches how
//! mainstream exporters name TS types. No trait impls or extra derives are
//! required on referenced data types — but each referenced type is still
//! asserted to *exist* at compile time, so typos don't slip into the TS
//! output. Type aliases render by their (unresolved) name; if the TS-side
//! name differs, use `#[ret(ts = "...")]`.
//!
//! Variant names must match the wire format, so serde attributes that rename
//! variants or change the enum representation are rejected (`rename`, `tag`,
//! `untagged`, ...).

use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, LitStr, Type, parse_macro_input, spanned::Spanned};

/// Resolves the path to the macro-support module (`__macro_support`) of the
/// `ahoi` crate (which hosts `TsDecl`), handling in-crate expansion and a
/// renamed dependency; same strategy as `ahoi-stock-macro`.
fn macro_support_path() -> TokenStream2 {
    let root = match crate_name("ahoi") {
        Ok(FoundCrate::Itself) => quote! { crate },
        Ok(FoundCrate::Name(found)) => {
            let id = format_ident!("{}", found);
            quote! { ::#id }
        }
        Err(_) => quote! { ::ahoi },
    };
    quote! { #root::__macro_support }
}

#[proc_macro_derive(Rets, attributes(ret))]
pub fn derive_rets(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_rets(input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

fn expand_rets(input: DeriveInput) -> syn::Result<TokenStream2> {
    let Data::Enum(data) = &input.data else {
        return Err(syn::Error::new(
            input.span(),
            "Rets: only enums (key types) are supported",
        ));
    };
    if !input.generics.params.is_empty() {
        return Err(syn::Error::new(
            input.generics.span(),
            "Rets: generic enums are not supported",
        ));
    }
    guard_serde_attrs(&input.attrs)?;

    let root = macro_support_path();
    let ident = &input.ident;
    let name = ident.to_string();

    // the whole map is rendered at expansion time into one static string
    let mut decl = format!("// # Rets of {name}\nexport type {name}Rets = {{\n");
    // ret types, kept for the compile-time existence assertion
    let mut assert_tys: Vec<Type> = Vec::new();

    for variant in &data.variants {
        guard_serde_attrs(&variant.attrs)?;
        let vname = variant.ident.to_string();
        match parse_variant_ret(&variant.attrs)? {
            None => {} // un-annotated variants are simply omitted from the map
            Some(RetSpec::Ty(ty)) => {
                decl.push_str(&format!("  {vname}: {};\n", ts_of(&ty)?));
                assert_tys.push(ty);
            }
            Some(RetSpec::Lit(lit)) => {
                decl.push_str(&format!("  {vname}: {};\n", lit.value()));
            }
        }
    }
    decl.push_str("};\n");

    Ok(quote! {
        #[automatically_derived]
        impl #root::TsDecl for #ident {
            fn ts_decl() -> ::std::string::String {
                #decl.into()
            }
        }

        // Compile-time existence check for every `#[ret(<Type>)]`: the types
        // are rendered syntactically, so without this a typo (`Vec<Frut>`)
        // would silently reach the TS output.
        const _: () = {
            #[allow(dead_code)]
            fn __ahoi_rets_assert() {
                #( let _ = ::core::marker::PhantomData::<#assert_tys>; )*
            }
        };
    })
}

// ────────────────────────────────────────────────────────────────────────────
// syntactic Rust → TS type rendering
// ────────────────────────────────────────────────────────────────────────────

/// Parenthesizes union types for compound positions (array items).
fn grouped(name: &str) -> String {
    if name.contains('|') {
        format!("({name})")
    } else {
        name.to_string()
    }
}

fn ts_of(ty: &Type) -> syn::Result<String> {
    match ty {
        Type::Reference(r) => ts_of(&r.elem),
        Type::Paren(p) => ts_of(&p.elem),
        Type::Group(g) => ts_of(&g.elem),
        Type::Tuple(t) => {
            if t.elems.is_empty() {
                // serde serializes unit as null/undefined; swb → undefined
                Ok("undefined".into())
            } else {
                let parts = t.elems.iter().map(ts_of).collect::<syn::Result<Vec<_>>>()?;
                Ok(format!("[{}]", parts.join(", ")))
            }
        }
        Type::Array(a) => Ok(format!("{}[]", grouped(&ts_of(&a.elem)?))),
        Type::Slice(s) => Ok(format!("{}[]", grouped(&ts_of(&s.elem)?))),
        Type::Path(p) => ts_of_path(p),
        _ => Err(syn::Error::new(
            ty.span(),
            "Rets: cannot render this type to TypeScript; use `#[ret(ts = \"...\")]` instead",
        )),
    }
}

fn ts_of_path(p: &syn::TypePath) -> syn::Result<String> {
    let seg = p
        .path
        .segments
        .last()
        .ok_or_else(|| syn::Error::new(p.span(), "Rets: empty type path"))?;
    let ident = seg.ident.to_string();

    // collect type arguments (lifetimes skipped; const generics unsupported)
    let mut args: Vec<&Type> = Vec::new();
    if let syn::PathArguments::AngleBracketed(ab) = &seg.arguments {
        for arg in &ab.args {
            match arg {
                syn::GenericArgument::Type(t) => args.push(t),
                syn::GenericArgument::Lifetime(_) => {}
                other => {
                    return Err(syn::Error::new(
                        other.span(),
                        "Rets: unsupported generic argument; use `#[ret(ts = \"...\")]` instead",
                    ));
                }
            }
        }
    }

    let arg = |i: usize| -> syn::Result<&Type> {
        args.get(i).copied().ok_or_else(|| {
            syn::Error::new(seg.span(), format!("Rets: `{ident}` needs a type argument"))
        })
    };

    Ok(match ident.as_str() {
        "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32" | "u64"
        | "u128" | "usize" | "f32" | "f64" => "number".into(),
        "bool" => "boolean".into(),
        "String" | "str" | "char" => "string".into(),
        "Option" => format!("{} | undefined", ts_of(arg(0)?)?),
        "Vec" | "VecDeque" | "LinkedList" | "BinaryHeap" | "HashSet" | "BTreeSet"
        | "IndexSet" => {
            format!("{}[]", grouped(&ts_of(arg(0)?)?))
        }
        // NOTE: `serde-wasm-bindgen` serializes maps to JS `Map` by default.
        // With its `json_compatible()` serializer, override via `ret_ts`.
        "HashMap" | "BTreeMap" | "IndexMap" => {
            format!("Map<{}, {}>", ts_of(arg(0)?)?, ts_of(arg(1)?)?)
        }
        "Box" | "Rc" | "Arc" | "Cell" | "RefCell" | "Cow" => ts_of(arg(0)?)?,
        // any other type renders by its identifier — the name every
        // mainstream exporter (ts-rs, Tsify, Tsain) gives the TS type
        _ => {
            if args.is_empty() {
                ident
            } else {
                let parts = args
                    .iter()
                    .map(|t| ts_of(t))
                    .collect::<syn::Result<Vec<_>>>()?;
                format!("{}<{}>", ident, parts.join(", "))
            }
        }
    })
}

// ────────────────────────────────────────────────────────────────────────────
// attribute parsing
// ────────────────────────────────────────────────────────────────────────────

/// `#[ret(<Type>)]` or `#[ret(ts = "...")]` on an enum variant.
enum RetSpec {
    Ty(Type),
    Lit(LitStr),
}

fn parse_variant_ret(attrs: &[syn::Attribute]) -> syn::Result<Option<RetSpec>> {
    let mut ret = None;
    for attr in attrs {
        if !attr.path().is_ident("ret") {
            continue;
        }
        // `ts = "..."` escape form first, otherwise a type
        if let Ok(nv) = attr.parse_args::<syn::MetaNameValue>() {
            if !nv.path.is_ident("ts") {
                return Err(syn::Error::new(
                    nv.path.span(),
                    "Rets: expected `#[ret(<Type>)]` or `#[ret(ts = \"...\")]`",
                ));
            }
            let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(s),
                ..
            }) = &nv.value
            else {
                return Err(syn::Error::new(
                    nv.value.span(),
                    "Rets: `ts` expects a string literal",
                ));
            };
            ret = Some(RetSpec::Lit(s.clone()));
        } else {
            ret = Some(RetSpec::Ty(attr.parse_args::<Type>().map_err(|_| {
                syn::Error::new(
                    attr.span(),
                    "Rets: expected `#[ret(<Type>)]` or `#[ret(ts = \"...\")]`",
                )
            })?));
        }
    }
    Ok(ret)
}

/// serde attributes that would break the "map key == variant ident" assumption
/// (renames) or change the enum representation the JS-side variant-name
/// extraction relies on.
const BANNED_SERDE: &[&str] = &[
    "tag",
    "content",
    "untagged",
    "rename",
    "rename_all",
    "rename_all_fields",
    "transparent",
    "skip",
    "skip_serializing",
    "skip_deserializing",
    "with",
    "serialize_with",
    "deserialize_with",
    "from",
    "into",
    "try_from",
    "remote",
    "other",
];

fn guard_serde_attrs(attrs: &[syn::Attribute]) -> syn::Result<()> {
    for attr in attrs {
        if !attr.path().is_ident("serde") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            if let Some(ident) = meta.path.get_ident() {
                let ident = ident.to_string();
                if BANNED_SERDE.contains(&ident.as_str()) {
                    return Err(meta.error(format!(
                        "Rets: `serde({ident})` renames variants or changes the enum \
                         representation, so the generated ret map would no longer match the \
                         wire format. Remove the attribute (rename support may come later)."
                    )));
                }
            }
            // consume `= value` or `(...)` payloads of benign serde attrs
            if meta.input.peek(syn::Token![=]) {
                let value = meta.value()?;
                let _: syn::Expr = value.parse()?;
            } else if meta.input.peek(syn::token::Paren) {
                let content;
                syn::parenthesized!(content in meta.input);
                let _: TokenStream2 = content.parse()?;
            }
            Ok(())
        })?;
    }
    Ok(())
}
