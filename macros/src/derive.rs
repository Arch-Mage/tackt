use proc_macro2::TokenStream;
use syn::Result;

use crate::spec::Spec;
use crate::structure::expand_impl;
use crate::structure::into_struct;

pub(crate) fn derive(input: TokenStream) -> Result<TokenStream> {
    let input = into_struct(input)?;
    let spec = Spec::from_attrs("route", input.attrs.as_slice())?;

    expand_impl(&input, &spec)
}
