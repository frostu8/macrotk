use syn::spanned::Spanned as _;
use syn::{parse_macro_input, Data, DeriveInput, Error, Fields, LitStr};

use quote::quote;

use proc_macro2::TokenStream;

#[proc_macro_derive(FromMeta)]
pub fn derive_from_meta(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = parse_macro_input!(item as DeriveInput);

    // get name
    let type_name = item.ident;

    let item = match item.data {
        Data::Struct(s) => s,
        Data::Enum(e) => {
            return Error::new(e.enum_token.span(), "only structs are supported")
                .into_compile_error()
                .into()
        }
        Data::Union(e) => {
            return Error::new(e.union_token.span(), "only structs are supported")
                .into_compile_error()
                .into()
        }
    };

    let fields = match item.fields {
        Fields::Named(fields) => fields.named,
        e => {
            return Error::new(e.span(), "struct can only have named fields")
                .into_compile_error()
                .into()
        }
    };

    // these are the pre-definitions of each field.
    let definitions = fields
        .iter()
        .map(|field| {
            let name = field.ident.as_ref().unwrap();
            let ty = &field.ty;

            quote! {
                let mut #name: ::std::option::Option<#ty> = None;
            }
        })
        .collect::<TokenStream>();

    // this is us actually looking for the field
    let searches = fields.iter()
        .map(|field| {
            let name = field.ident.as_ref().unwrap();
            let name_str = LitStr::new(&name.to_string(), name.span());

            quote! {
                #name_str => #name = Some(
                    __m.next_value().unwrap_or(
                        // TODO: intelligent spans
                        Err(::macrotk::syn::Error::new(::macrotk::Span::call_site(), ::std::concat!("expected value for ", #name_str)))
                    )?
                ),
            }
        })
        .collect::<TokenStream>();

    // this is us unwrapping all of the fields
    let unwrapper = fields.iter()
        .map(|field| {
            let name = field.ident.as_ref().unwrap();
            let name_str = LitStr::new(&name.to_string(), name.span());

            quote! {
                #name: #name.ok_or(::macrotk::syn::Error::new(::macrotk::Span::call_site(), ::std::concat!("missing value for ", #name_str)))?,
            }
        })
        .collect::<TokenStream>();

    let expanded = quote! {
        impl ::macrotk::meta::FromMeta for #type_name {
            fn from_meta(
                __m: ::macrotk::meta::MetaStream,
            ) -> ::std::result::Result<Self, ::macrotk::syn::Error> {
                #definitions

                while let Some(__name) = __m.next_name() {
                    match __name?.as_str() {
                        #searches
                        s => return Err(::macrotk::syn::Error::new(::macrotk::Span::call_site(), ::std::format!("unexpected value: \"{}\"", s))),
                    }
                }

                Ok(#type_name {
                    #unwrapper
                })
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}
