//! `#[axiom_harness]` — procedural macro for automatic harness registration.
//!
//! Decorating a function with this attribute leaves the original function
//! intact and appends an `inventory::submit!` block that registers a
//! `laplace_harness::registry::HarnessConfig` at link time.

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::{Expr, ItemFn, Lit, Meta, Token};

/// Parsed arguments from `#[axiom_harness(...)]`.
pub(crate) struct HarnessArgs {
    pub(crate) name: String,
    pub(crate) threads: usize,
    pub(crate) resources: usize,
    pub(crate) desc: String,
    /// Expected verdict: `"clean"` or `"bug"`.  Defaults to `"clean"`.
    pub(crate) expected: String,
}

impl Parse for HarnessArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut name = None;
        let mut threads = None;
        let mut resources = None;
        let mut desc = None;
        let mut expected = None;

        let metas = Punctuated::<Meta, Token![,]>::parse_terminated(input)?;

        for meta in metas {
            if let Meta::NameValue(nv) = meta {
                let key = nv.path.get_ident().map(|i| i.to_string());
                match key.as_deref() {
                    Some("name") => {
                        if let Expr::Lit(expr_lit) = &nv.value {
                            if let Lit::Str(s) = &expr_lit.lit {
                                name = Some(s.value());
                            }
                        }
                    }
                    Some("threads") => {
                        if let Expr::Lit(expr_lit) = &nv.value {
                            if let Lit::Int(i) = &expr_lit.lit {
                                threads = Some(i.base10_parse::<usize>()?);
                            }
                        }
                    }
                    Some("resources") => {
                        if let Expr::Lit(expr_lit) = &nv.value {
                            if let Lit::Int(i) = &expr_lit.lit {
                                resources = Some(i.base10_parse::<usize>()?);
                            }
                        }
                    }
                    Some("desc") => {
                        if let Expr::Lit(expr_lit) = &nv.value {
                            if let Lit::Str(s) = &expr_lit.lit {
                                desc = Some(s.value());
                            }
                        }
                    }
                    Some("expected") => {
                        if let Expr::Lit(expr_lit) = &nv.value {
                            if let Lit::Str(s) = &expr_lit.lit {
                                expected = Some(s.value());
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(HarnessArgs {
            name: name.ok_or_else(|| input.error("axiom_harness: `name` attribute is required"))?,
            threads: threads
                .ok_or_else(|| input.error("axiom_harness: `threads` attribute is required"))?,
            resources: resources
                .ok_or_else(|| input.error("axiom_harness: `resources` attribute is required"))?,
            desc: desc.unwrap_or_default(),
            expected: expected.unwrap_or_else(|| "clean".to_string()),
        })
    }
}

/// Register a function as a verification harness via `inventory`.
///
/// The decorated function must have the signature:
/// `fn(ThreadId, usize) -> Option<(Operation, ResourceId)>`
///
/// The macro emits the original function unchanged, followed by an
/// `inventory::submit!` block that statically registers a
/// `laplace_harness::registry::HarnessConfig`.
pub(crate) fn axiom_harness_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    use syn::parse_macro_input;

    let args = parse_macro_input!(attr as HarnessArgs);
    let func = parse_macro_input!(item as ItemFn);

    let func_ident = &func.sig.ident;
    let name = &args.name;
    let threads = args.threads;
    let resources = args.resources;
    let desc = &args.desc;
    let expected = &args.expected;

    let expanded = quote! {
        #func

        ::inventory::submit! {
            crate::registry::HarnessConfig {
                name: #name,
                display_name: #name,
                description: #desc,
                num_threads: #threads,
                num_resources: #resources,
                op_provider: #func_ident,
                expected: #expected,
            }
        }
    };

    TokenStream::from(expanded)
}
