use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use std::collections::HashSet;
use syn::{
    GenericArgument, GenericParam, Ident, ImplItem, ItemImpl, PathArguments, Type,
    parse_macro_input,
};

pub fn expand_stock_attr(attr: TokenStream, item: TokenStream) -> TokenStream {
    let trait_name_override: Option<Ident> = if attr.is_empty() {
        None
    } else {
        match syn::parse::<Ident>(attr) {
            Ok(id) => Some(id),
            Err(e) => return e.to_compile_error().into(),
        }
    };

    let impl_block = parse_macro_input!(item as ItemImpl);

    let inner_type = match extract_stock_inner_type(&impl_block.self_ty) {
        Some(t) => t,
        None => {
            return syn::Error::new_spanned(
                &impl_block.self_ty,
                "#[stock] must be applied to `impl<...> Stock<Type, ..> { ... }`",
            )
            .to_compile_error()
            .into();
        }
    };

    let trait_name = match trait_name_override {
        Some(n) => n,
        None => {
            let base = get_type_base_name(inner_type);
            format_ident!("{}StockExt2", base)
        }
    };

    let generics = &impl_block.generics;
    let self_ty = &impl_block.self_ty;
    let (impl_generics, _ty_generics, where_clause) = generics.split_for_impl();

    // The generated trait is parameterized only by the generic params that the
    // inner type actually references (e.g. `T` in `Stock<AB<T>, P>`), excluding
    // the pipeline param `P` and any other impl-only generics.
    let (trait_def_generics, trait_use_generics) = referenced_trait_generics(generics, inner_type);

    let mut trait_methods = Vec::new();
    let mut impl_methods = Vec::new();

    for impl_item in &impl_block.items {
        if let ImplItem::Fn(method) = impl_item {
            let sig = &method.sig;
            let block = &method.block;
            let attrs = &method.attrs;

            trait_methods.push(quote! { #sig; });
            impl_methods.push(quote! {
                #(#attrs)*
                #sig #block
            });
        }
    }

    quote! {
        pub trait #trait_name #trait_def_generics {
            #(#trait_methods)*
        }

        impl #impl_generics #trait_name #trait_use_generics for #self_ty #where_clause {
            #(#impl_methods)*
        }
    }
    .into()
}

fn extract_stock_inner_type(ty: &Type) -> Option<&Type> {
    if let Type::Path(tp) = ty {
        let last = tp.path.segments.last()?;
        if last.ident != "Stock" {
            return None;
        }
        if let PathArguments::AngleBracketed(args) = &last.arguments {
            if let Some(GenericArgument::Type(inner)) = args.args.first() {
                return Some(inner);
            }
        }
    }
    None
}

fn get_type_base_name(ty: &Type) -> String {
    match ty {
        Type::Path(tp) => tp
            .path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_else(|| "Unknown".to_string()),
        _ => "Unknown".to_string(),
    }
}

/// Builds `(<decl>, <use>)` generic brackets containing only the impl generic
/// params that the inner type references. `decl` keeps each param's bounds
/// (e.g. `<T: 'static>`); `use` is idents only (e.g. `<T>`). Empty when none.
fn referenced_trait_generics(
    generics: &syn::Generics,
    inner: &Type,
) -> (TokenStream2, TokenStream2) {
    let mut names = HashSet::new();
    collect_idents(inner, &mut names);

    let mut decls = Vec::new();
    let mut uses = Vec::new();

    for param in &generics.params {
        match param {
            GenericParam::Type(tp) if names.contains(&tp.ident.to_string()) => {
                decls.push(quote! { #tp });
                let id = &tp.ident;
                uses.push(quote! { #id });
            }
            GenericParam::Lifetime(lt) if names.contains(&lt.lifetime.ident.to_string()) => {
                decls.push(quote! { #lt });
                let l = &lt.lifetime;
                uses.push(quote! { #l });
            }
            GenericParam::Const(c) if names.contains(&c.ident.to_string()) => {
                decls.push(quote! { #c });
                let id = &c.ident;
                uses.push(quote! { #id });
            }
            _ => {}
        }
    }

    let decl = if decls.is_empty() {
        quote! {}
    } else {
        quote! { <#(#decls),*> }
    };
    let usage = if uses.is_empty() {
        quote! {}
    } else {
        quote! { <#(#uses),*> }
    };
    (decl, usage)
}

/// Collects every identifier/lifetime name appearing in `ty`, so impl generics
/// referenced by the inner Stock type can be detected.
fn collect_idents(ty: &Type, out: &mut HashSet<String>) {
    match ty {
        Type::Path(tp) => {
            if let Some(qself) = &tp.qself {
                collect_idents(&qself.ty, out);
            }
            for seg in &tp.path.segments {
                out.insert(seg.ident.to_string());
                if let PathArguments::AngleBracketed(ab) = &seg.arguments {
                    for arg in &ab.args {
                        match arg {
                            GenericArgument::Type(t) => collect_idents(t, out),
                            GenericArgument::Lifetime(l) => {
                                out.insert(l.ident.to_string());
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        Type::Reference(r) => collect_idents(&r.elem, out),
        Type::Tuple(t) => {
            for e in &t.elems {
                collect_idents(e, out);
            }
        }
        Type::Slice(s) => collect_idents(&s.elem, out),
        Type::Array(a) => collect_idents(&a.elem, out),
        Type::Paren(p) => collect_idents(&p.elem, out),
        Type::Group(g) => collect_idents(&g.elem, out),
        Type::Ptr(p) => collect_idents(&p.elem, out),
        _ => {}
    }
}
