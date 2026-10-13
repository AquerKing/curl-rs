use proc_macro::TokenStream;

mod imp;

#[proc_macro_derive(FromRegex, attributes(regex))]
pub fn derive_from_regex(input: TokenStream) -> TokenStream {
    imp::derive_from_regex(input)
}