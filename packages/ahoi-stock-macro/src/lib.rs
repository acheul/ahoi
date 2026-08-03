use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Literal, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields, GenericParam, Ident, parse_macro_input};

mod stock_attr;

/// Resolves the path to the macro-support module (`__macro_support`), which
/// re-exports every item the generated code references. Pointing the generated
/// code at this stable, hidden module decouples it from the public layout, so
/// the public API can be reorganized without breaking macros.
///
/// The module is defined in `ahoi-core` and re-exported by the `ahoi` facade,
/// so the generated code can root itself in whichever crate the consumer
/// actually depends on. We probe `ahoi` first (the public facade), then
/// `ahoi-core`. For each, we handle:
/// - the macro is expanded inside that crate itself (`crate`) — e.g. the
///   `ahoi-core` tests resolve to `crate`,
/// - the dependency exists, possibly renamed in the downstream `Cargo.toml`
///   (`::<name>`).
/// If neither is found, fall back to the absolute `::ahoi`.
fn macro_support_path() -> TokenStream2 {
    let root = ["ahoi", "ahoi-core"]
        .into_iter()
        .find_map(|name| match crate_name(name) {
            Ok(FoundCrate::Itself) => Some(quote! { crate }),
            Ok(FoundCrate::Name(found)) => {
                let id = format_ident!("{}", found);
                Some(quote! { ::#id })
            }
            Err(_) => None,
        })
        .unwrap_or_else(|| quote! { ::ahoi });
    quote! { #root::__macro_support }
}

#[proc_macro_derive(Stock, attributes(stock))]
pub fn derive_stock(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand(input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

fn is_skipped(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path().is_ident("stock")
            && attr
                .parse_args::<Ident>()
                .map(|i| i == "skip")
                .unwrap_or(false)
    })
}

fn to_snake_case(s: &str) -> String {
    let mut out = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() {
            if i > 0 {
                out.push('_');
            }
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push(ch);
        }
    }
    out
}

// Returns (trait_params, impl_params_with_static, type_generics)
//  - trait_params:  `T: Bound`        (used in trait declaration generics)
//  - impl_params:   `T: 'static + Bound` (used in impl generics)
//  - type_generics: `T`               (used to reference the type, e.g. `Name<T>`)
fn split_generics(generics: &syn::Generics) -> (TokenStream2, TokenStream2, TokenStream2) {
    let trait_params: Vec<_> = generics
        .params
        .iter()
        .map(|p| match p {
            GenericParam::Type(tp) => {
                let id = &tp.ident;
                let bounds = &tp.bounds;
                if bounds.is_empty() {
                    quote! { #id }
                } else {
                    quote! { #id: #bounds }
                }
            }
            GenericParam::Lifetime(lt) => {
                let lt = &lt.lifetime;
                quote! { #lt }
            }
            GenericParam::Const(c) => {
                let id = &c.ident;
                quote! { #id }
            }
        })
        .collect();

    let impl_params: Vec<_> = generics
        .params
        .iter()
        .map(|p| match p {
            GenericParam::Type(tp) => {
                let id = &tp.ident;
                let bounds = &tp.bounds;
                if bounds.is_empty() {
                    quote! { #id: 'static }
                } else {
                    quote! { #id: 'static + #bounds }
                }
            }
            GenericParam::Lifetime(lt) => quote! { #lt },
            GenericParam::Const(c) => quote! { #c },
        })
        .collect();

    let type_generics: Vec<_> = generics
        .params
        .iter()
        .map(|p| match p {
            GenericParam::Type(tp) => {
                let id = &tp.ident;
                quote! { #id }
            }
            GenericParam::Lifetime(lt) => {
                let lt = &lt.lifetime;
                quote! { #lt }
            }
            GenericParam::Const(c) => {
                let id = &c.ident;
                quote! { #id }
            }
        })
        .collect();

    (
        quote! { #(#trait_params),* },
        quote! { #(#impl_params),* },
        quote! { #(#type_generics),* },
    )
}

/// Shared context threaded through struct/enum expansion.
struct Ctx<'a> {
    ahoi: &'a TokenStream2,
    name: &'a Ident,
    name_upper: String,
    has_generics: bool,
    /// `T: Bound` list (no surrounding brackets).
    trait_params: TokenStream2,
    /// `T: 'static + Bound` list (no surrounding brackets).
    impl_params: TokenStream2,
    /// `T` list (no surrounding brackets).
    type_generics: TokenStream2,
    /// `<T>` or empty.
    type_brackets: TokenStream2,
    /// where-clause predicates carried from the source (no `where`).
    extra_where: TokenStream2,
}

impl Ctx<'_> {
    /// The self type, e.g. `Name` or `Name<T>`.
    fn name_ty(&self) -> TokenStream2 {
        let name = self.name;
        let tb = &self.type_brackets;
        quote! { #name #tb }
    }

    /// `where ...` clause, or empty.
    fn where_clause(&self) -> TokenStream2 {
        if self.extra_where.is_empty() {
            quote! {}
        } else {
            let w = &self.extra_where;
            quote! { where #w }
        }
    }

    /// Impl generics with an unbounded `__Pipe`. Used by the field/variant
    /// accessors, which only call `derive`/`try_derive`.
    fn impl_generics_plain(&self) -> TokenStream2 {
        if self.has_generics {
            let ip = &self.impl_params;
            quote! { <#ip, const __OPT: bool, __Pipe> }
        } else {
            quote! { <const __OPT: bool, __Pipe> }
        }
    }
}

fn expand(input: DeriveInput) -> Result<TokenStream2, syn::Error> {
    // `ahoi` here is the macro-support path (`ahoi::__macro_support`), used as the
    // root for every `#ahoi::Type` the generated code references.
    let ahoi = macro_support_path();
    let name = &input.ident;
    let name_upper = name.to_string().to_ascii_uppercase();
    let has_generics = !input.generics.params.is_empty();
    let (trait_params, impl_params, type_generics) = split_generics(&input.generics);

    let type_brackets = if has_generics {
        quote! { <#type_generics> }
    } else {
        quote! {}
    };

    // Where-clause predicates from the struct/enum, carried to generated traits and impls.
    let extra_where: TokenStream2 = match &input.generics.where_clause {
        Some(wc) => {
            let preds = &wc.predicates;
            quote! { #preds }
        }
        None => quote! {},
    };

    let ctx = Ctx {
        ahoi: &ahoi,
        name,
        name_upper,
        has_generics,
        trait_params,
        impl_params,
        type_generics,
        type_brackets,
        extra_where,
    };

    match &input.data {
        Data::Struct(s) => expand_struct(&ctx, &s.fields),
        Data::Enum(e) => expand_enum(&ctx, &e.variants),
        Data::Union(_) => Err(syn::Error::new_spanned(
            name,
            "#[derive(Stock)] is not supported on unions",
        )),
    }
}

fn expand_struct(ctx: &Ctx, fields: &Fields) -> Result<TokenStream2, syn::Error> {
    let ahoi = ctx.ahoi;
    let name = ctx.name;
    let name_upper = &ctx.name_upper;
    let name_ty = ctx.name_ty();
    let trait_name = format_ident!("{}StockExt", name);

    let mut consts = Vec::new();
    let mut trait_fns = Vec::new();
    let mut impl_fns = Vec::new();

    match fields {
        Fields::Named(named) => {
            for (idx, field) in named.named.iter().enumerate() {
                if is_skipped(&field.attrs) {
                    continue;
                }
                let fname = field.ident.as_ref().unwrap();
                let fty = &field.ty;
                let lit = Literal::u64_suffixed(idx as u64);
                let cname = format_ident!(
                    "{}_{}_KEY",
                    name_upper,
                    fname.to_string().to_ascii_uppercase()
                );

                consts.push(quote! { pub const #cname: u64 = #lit; });
                trait_fns.push(quote! {
                    fn #fname(self) -> #ahoi::Stock<#fty, #ahoi::ChainedPipe<__Pipe, #ahoi::GetNext<#name_ty, #fty>, #name_ty, #fty>, __OPT>;
                });
                impl_fns.push(quote! {
                    fn #fname(self) -> #ahoi::Stock<#fty, #ahoi::ChainedPipe<__Pipe, #ahoi::GetNext<#name_ty, #fty>, #name_ty, #fty>, __OPT> {
                        self.derive(#cname, #ahoi::GetNext::new(|x| &x.#fname, |x| &mut x.#fname))
                    }
                });
            }
        }
        Fields::Unnamed(unnamed) => {
            for (idx, field) in unnamed.unnamed.iter().enumerate() {
                if is_skipped(&field.attrs) {
                    continue;
                }
                let fty = &field.ty;
                let lit = Literal::u64_suffixed(idx as u64);
                let method = format_ident!("f{}", idx);
                let idx_syn = syn::Index::from(idx);
                let cname = format_ident!("{}_F{}_KEY", name_upper, idx);

                consts.push(quote! { pub const #cname: u64 = #lit; });
                trait_fns.push(quote! {
                    fn #method(self) -> #ahoi::Stock<#fty, #ahoi::ChainedPipe<__Pipe, #ahoi::GetNext<#name_ty, #fty>, #name_ty, #fty>, __OPT>;
                });
                impl_fns.push(quote! {
                    fn #method(self) -> #ahoi::Stock<#fty, #ahoi::ChainedPipe<__Pipe, #ahoi::GetNext<#name_ty, #fty>, #name_ty, #fty>, __OPT> {
                        self.derive(#cname, #ahoi::GetNext::new(|x| &x.#idx_syn, |x| &mut x.#idx_syn))
                    }
                });
            }
        }
        Fields::Unit => {}
    }

    let where_clause = ctx.where_clause();
    let impl_generics = ctx.impl_generics_plain();

    let (trait_generics, trait_use) = if ctx.has_generics {
        let tp = &ctx.trait_params;
        let tg = &ctx.type_generics;
        (
            quote! { <#tp, const __OPT: bool, __Pipe> },
            quote! { <#tg, __OPT, __Pipe> },
        )
    } else {
        (
            quote! { <const __OPT: bool, __Pipe> },
            quote! { <__OPT, __Pipe> },
        )
    };

    Ok(quote! {
        #(#consts)*

        pub trait #trait_name #trait_generics #where_clause {
            #(#trait_fns)*
        }

        impl #impl_generics #trait_name #trait_use
            for #ahoi::Stock<#name_ty, __Pipe, __OPT>
        #where_clause
        {
            #(#impl_fns)*
        }
    })
}

fn expand_enum(
    ctx: &Ctx,
    variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>,
) -> Result<TokenStream2, syn::Error> {
    let ahoi = ctx.ahoi;
    let name = ctx.name;
    let name_upper = &ctx.name_upper;
    let name_ty = ctx.name_ty();

    let trait_ext_name = format_ident!("{}StockExt", name);

    let mut consts = Vec::new();
    let mut acc_trait_fns = Vec::new();
    let mut acc_impl_fns = Vec::new();

    for (idx, variant) in variants.iter().enumerate() {
        if is_skipped(&variant.attrs) {
            continue;
        }

        let var_ident = &variant.ident;
        let var_str = var_ident.to_string();
        let snake = to_snake_case(&var_str);
        let lit = Literal::u64_suffixed(idx as u64);

        let cname = format_ident!("{}_{}_KEY", name_upper, var_str.to_ascii_uppercase());
        consts.push(quote! { pub const #cname: u64 = #lit; });

        // {variant}() accessor for single-field variants -> OPTIONAL derived Stock
        match &variant.fields {
            Fields::Unnamed(f) if f.unnamed.len() == 1 => {
                let fty = &f.unnamed[0].ty;
                let method = format_ident!("{}", snake);

                acc_trait_fns.push(quote! {
                    fn #method(self) -> #ahoi::Stock<#fty, #ahoi::ChainedPipe<__Pipe, #ahoi::GetNextOpt<#name_ty, #fty>, #name_ty, #fty>, true>;
                });
                acc_impl_fns.push(quote! {
                    fn #method(self) -> #ahoi::Stock<#fty, #ahoi::ChainedPipe<__Pipe, #ahoi::GetNextOpt<#name_ty, #fty>, #name_ty, #fty>, true> {
                        self.try_derive(#cname, #ahoi::GetNextOpt::new(
                            |x| if let #name::#var_ident(v) = x { Some(v) } else { None },
                            |x| if let #name::#var_ident(v) = x { Some(v) } else { None },
                        ))
                    }
                });
            }
            Fields::Named(f) if f.named.len() == 1 => {
                let field = f.named.first().unwrap();
                let fty = &field.ty;
                let fname = field.ident.as_ref().unwrap();
                let method = format_ident!("{}", snake);

                acc_trait_fns.push(quote! {
                    fn #method(self) -> #ahoi::Stock<#fty, #ahoi::ChainedPipe<__Pipe, #ahoi::GetNextOpt<#name_ty, #fty>, #name_ty, #fty>, true>;
                });
                acc_impl_fns.push(quote! {
                    fn #method(self) -> #ahoi::Stock<#fty, #ahoi::ChainedPipe<__Pipe, #ahoi::GetNextOpt<#name_ty, #fty>, #name_ty, #fty>, true> {
                        self.try_derive(#cname, #ahoi::GetNextOpt::new(
                            |x| if let #name::#var_ident { #fname: v } = x { Some(v) } else { None },
                            |x| if let #name::#var_ident { #fname: v } = x { Some(v) } else { None },
                        ))
                    }
                });
            }
            _ => {}
        }
    }

    let where_clause = ctx.where_clause();
    let acc_impl_generics = ctx.impl_generics_plain();

    // Accessor trait carries only `__Pipe` (result OPTIONAL is always `true`),
    // plus the type's own generics.
    let ext_block = if acc_trait_fns.is_empty() {
        quote! {}
    } else {
        let (ext_trait_generics, ext_trait_use) = if ctx.has_generics {
            let tp = &ctx.trait_params;
            let tg = &ctx.type_generics;
            (quote! { <#tp, __Pipe> }, quote! { <#tg, __Pipe> })
        } else {
            (quote! { <__Pipe> }, quote! { <__Pipe> })
        };

        quote! {
            pub trait #trait_ext_name #ext_trait_generics #where_clause {
                #(#acc_trait_fns)*
            }

            impl #acc_impl_generics #trait_ext_name #ext_trait_use
                for #ahoi::Stock<#name_ty, __Pipe, __OPT>
            #where_clause
            {
                #(#acc_impl_fns)*
            }
        }
    };

    Ok(quote! {
        #(#consts)*

        #ext_block
    })
}

#[proc_macro_attribute]
pub fn stock(attr: TokenStream, item: TokenStream) -> TokenStream {
    stock_attr::expand_stock_attr(attr, item)
}
