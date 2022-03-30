use proc_macro2::TokenStream;

use syn::spanned::Spanned;
use syn::DeriveInput;
use syn::Error;
use syn::FnArg;
use syn::Ident;
use syn::ItemFn;
use syn::ItemStruct;
use syn::Path;
use syn::Result;

use crate::spec::Segment;
use crate::spec::Spec;

pub(crate) fn into_struct(input: TokenStream) -> Result<ItemStruct> {
    let input = syn::parse2::<DeriveInput>(input)?;

    match input.data {
        syn::Data::Struct(data) => Ok(ItemStruct {
            attrs: input.attrs,
            vis: input.vis,
            struct_token: data.struct_token,
            ident: input.ident,
            generics: input.generics,
            fields: data.fields,
            semi_token: data.semi_token,
        }),
        _ => Err(Error::new(input.span(), "not a struct")),
    }
}

pub(crate) fn expand_impl(item: &ItemStruct, spec: &Spec) -> Result<TokenStream> {
    let fields: Vec<_> = match item.fields {
        syn::Fields::Named(ref fields) => fields
            .named
            .iter()
            .filter_map(|field| field.ident.as_ref())
            .collect(),
        _ => {
            return Err(Error::new(
                item.fields.span(),
                "not a struct with named fields",
            ))
        }
    };

    for field in fields.iter() {
        if !spec.iter_param().any(|x| x == *field) {
            return Err(Error::new(
                field.span(),
                format!(r#""{}" does not exists in route param"#, field),
            ));
        }
    }

    // error
    let err_typ: Path = syn::parse_quote!(::tackt::Error);
    let err_404: Path = syn::parse_quote!(#err_typ::Path);
    let err_405: Path = syn::parse_quote!(#err_typ::Method);

    // local names
    let method_var: Ident = syn::parse_quote!(__method);
    let path_var: Ident = syn::parse_quote!(__path);
    let next_var: Ident = syn::parse_quote!(__next);
    let req_var: Ident = syn::parse_quote!(__req);

    let segment_matching = spec.segments.iter().map(|segment| match segment {
        Segment::Lit(lit) => quote::quote_spanned! {lit.span()=>
            let (#next_var, #path_var) = #path_var.split_once('/').unwrap_or((#path_var, ""));
            if #next_var != #lit {
                return Err(#err_404);
            }
        },
        Segment::Param(name) => quote::quote_spanned! {name.span()=>
            let (#next_var, #path_var) = #path_var.split_once('/').unwrap_or((#path_var, ""));
            let #name = #next_var.parse().map_err(|_|#err_404)?;
        },
        Segment::Wild(name) => quote::quote_spanned! {name.span()=>
            let #name = #path_var.to_string();
            let #path_var = "";
        },
    });

    let method_matching = match spec.methods.len() {
        0 => None,
        _ => Some({
            let methods = spec.methods.iter().map(|method| {
                quote::quote_spanned! {method.span()=>
                    ::tackt::Method::#method != #method_var
                }
            });

            quote::quote! {
                if #(#methods)&&* {
                    return ::std::result::Result::Err(#err_405);
                };
            }
        }),
    };

    let struct_name = &item.ident;

    Ok(quote::quote! {
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl<T> ::tackt::Param<T> for #struct_name
        where T: ::tackt::PathReq + ::tackt::MethodReq,
        {
            fn from_request(#req_var: &T) -> ::std::result::Result<Self, #err_typ> {
                let #path_var = ::tackt::PathReq::path(#req_var);
                let #method_var = ::tackt::MethodReq::method(#req_var);
                let (#next_var, #path_var) = #path_var.split_once('/').ok_or(#err_404)?;
                if !#next_var.is_empty() {
                    return ::std::result::Result::Err(#err_404);
                };

                #(#segment_matching)*

                if !#path_var.is_empty() {
                    return ::std::result::Result::Err(#err_404);
                };

                #method_matching

                ::std::result::Result::Ok(
                    #struct_name {
                        #(#fields),*
                    }
                )
            }
        }
    })
}

pub(crate) fn expand_struct(item: &ItemFn, skip: usize) -> Result<ItemStruct> {
    let name = &item.sig.ident;
    let vis = &item.vis;

    let fields = item.sig.inputs.iter().skip(skip).map(|arg| match arg {
        FnArg::Typed(arg) => match arg.pat.as_ref() {
            syn::Pat::Ident(pat) => {
                let name = &pat.ident;
                let ty = arg.ty.as_ref();
                quote::quote_spanned! {arg.span()=> #vis #name: #ty }
            }
            _ => unreachable!("BUG: a validation is missed."),
        },
        _ => unreachable!("BUG: a validation is missed."),
    });

    syn::parse2(quote::quote! {
        #[allow(dead_code)]
        #[allow(non_camel_case_types)]
        #[doc(hidden)]
        #vis struct #name {
            #(#fields),*
        }
    })
}
