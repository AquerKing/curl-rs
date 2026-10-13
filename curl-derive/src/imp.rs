use std::collections::HashSet;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    Data, DataStruct, DeriveInput, Field, Fields, GenericArgument, LitStr, PathArguments, Type,
    parse_macro_input,
};

pub fn derive_from_regex(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    expand(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn expand(input: DeriveInput) -> syn::Result<TokenStream2> {
    let struct_name = &input.ident;

    let regex_attr = input
        .attrs
        .iter()
        .find(|a| a.path().is_ident("regex"))
        .ok_or_else(|| {
            syn::Error::new_spanned(
                &input.ident,
                "missing `#[regex(\"...\")` attribute on the struct",
            )
        })?;

    let pattern_lit: LitStr = regex_attr.parse_args()?;
    let pattern = pattern_lit.value();

    let re = regex::Regex::new(&pattern).map_err(|e| {
        syn::Error::new_spanned(&pattern_lit, format!("invalid regex pattern: {e}"))
    })?;
    let group_names: HashSet<&str> = re.capture_names().flatten().collect();

    let fields = match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(named),
            ..
        }) => &named.named,
        _ => {
            return Err(syn::Error::new_spanned(
                &input.ident,
                "FromRegex can only be derived for structs with named fields.",
            ));
        }
    };

    let mut inits = Vec::with_capacity(fields.len());
    for field in fields {
        let ident = field.ident.as_ref().expect("named field");
        let name = ident.to_string();

        if !group_names.contains(name.as_str()) {
            let available = group_names.iter().copied().collect::<Vec<_>>().join(", ");
            return Err(syn::Error::new_spanned(
                field,
                format!("no capture group named `{name}` in the regex; available: [{available}]"),
            ));
        }

        let expr = field_expr(field)?;
        inits.push(quote! { #ident: #expr });
    }

    Ok(quote! {
        impl ::std::str::FromStr for #struct_name {
            type Err = ::curl_rs::Error;

            fn from_str(__input: &str) -> ::std::result::Result<Self, Self::Err> {
                use ::std::sync::OnceLock;

                static REGEX: OnceLock<::curl_rs::regex::Regex> = OnceLock::new();
                let re = REGEX.get_or_init(|| {
                    ::curl_rs::regex::Regex::new(#pattern)
                        .expect("regex was validated at compile time")
                });

                let caps = re.captures(__input).ok_or(::curl_rs::Error::NoMatch)?;

                ::std::result::Result::Ok(Self {
                    #(#inits),*
                })
            }
        }
    })
}

fn field_expr(field: &Field) -> syn::Result<TokenStream2> {
    let ident = field.ident.as_ref().unwrap();
    let name = ident.to_string();
    let ty = &field.ty;

    if let Some(inner) = option_inner(ty) {
        Ok(quote! {
            caps.name(#name).map(|__m| __m.as_str().parse::<#inner>()).transpose().map_err(|__e| ::curl_rs::Error::Parse {
                field: #name,
                source: ::std::string::ToString::to_string(&__e),
            })?
        })
    } else {
        Ok(quote! {
            caps.name(#name)
            .ok_or(::curl_rs::Error::Missing { field: #name })?
            .as_str()
            .parse::<#ty>()
            .map_err(|__e| ::curl_rs::Error::Parse {
                field: #name,
                source: ::std::string::ToString::to_string(&__e),
            })?
        })
    }
}

fn option_inner(ty: &Type) -> Option<&Type> {
    let tp = match ty {
        Type::Path(tp) => tp,
        _ => return None,
    };

    let seg = tp.path.segments.last()?;
    if seg.ident != "Option" {
        return None;
    }

    let args = match &seg.arguments {
        PathArguments::AngleBracketed(a) => a,
        _ => return None,
    };

    match args.args.first() {
        Some(GenericArgument::Type(inner)) => Some(inner),
        _ => None,
    }
}
