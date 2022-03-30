use proc_macro2::TokenStream;
use syn::spanned::Spanned;
use syn::Error;
use syn::FnArg;
use syn::ItemFn;
use syn::Result;

use crate::spec::Spec;
use crate::structure::expand_impl;
use crate::structure::expand_struct;

pub(crate) fn route(spec: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let spec = syn::parse2::<Spec>(spec)?;
    let item = syn::parse2::<ItemFn>(item)?;

    if item.sig.inputs.len() < spec.num_param() + 1 {
        return Err(Error::new(
            item.sig.paren_token.span,
            "insufficient number of argument",
        ));
    }

    let skip = item.sig.inputs.len() - spec.num_param();
    item.sig
        .inputs
        .iter()
        .skip(skip)
        .try_for_each(|arg| match arg {
            FnArg::Receiver(arg) => {
                Err(Error::new(arg.span(), "receiver cannot become route param"))
            }
            FnArg::Typed(arg) => match arg.pat.as_ref() {
                syn::Pat::Ident(..) => Ok(()),
                _ => Err(Error::new(arg.pat.span(), "argument must be an identifier")),
            },
        })?;

    for param in spec.iter_param() {
        if !item.sig.inputs.iter().skip(skip).any(|arg| match arg {
            FnArg::Typed(pat) => match pat.pat.as_ref() {
                syn::Pat::Ident(pat) => &pat.ident == param,
                _ => false,
            },
            _ => false,
        }) {
            return Err(Error::new(
                param.span(),
                format!(r#""{}" does not exists in function argument"#, param),
            ));
        }
    }

    let struct_ = expand_struct(&item, skip)?;
    let impl_ = expand_impl(&struct_, &spec)?;
    let fn_ = expand_fn(&item, &spec, skip);

    Ok(quote::quote! {
        #struct_

        #impl_

        #fn_
    })
}

fn expand_fn(item: &ItemFn, spec: &Spec, skip: usize) -> TokenStream {
    let attrs = &item.attrs;
    let vis = &item.vis;
    let constness = &item.sig.constness;
    let asyncness = &item.sig.asyncness;
    let unsafety = &item.sig.unsafety;
    let name = &item.sig.ident;
    let generics = &item.sig.generics;

    let output = &item.sig.output;
    let where_ = &item.sig.generics.where_clause;
    let block = &item.block;

    let reserved_inputs = item.sig.inputs.iter().take(skip);
    let params = spec.iter_param();

    quote::quote! {
        #(#attrs)*
        #vis #constness #asyncness #unsafety fn #name #generics (
            #(#reserved_inputs)*,
            #name { #(#params),* }: #name,
        ) #output
        #where_
        #block
    }
}
