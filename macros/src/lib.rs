use proc_macro::TokenStream;

mod derive;
mod route;
mod spec;
mod structure;

#[proc_macro_derive(Param, attributes(route))]
pub fn derive(input: TokenStream) -> TokenStream {
    derive::derive(input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// The attribute to describe route's spec.
///
/// See [`tackt`][1]'s documentation.
///
/// [1]: https://docs.rs/tackt/0.1
#[proc_macro_attribute]
pub fn route(args: TokenStream, item: TokenStream) -> TokenStream {
    route::route(args.into(), item.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
