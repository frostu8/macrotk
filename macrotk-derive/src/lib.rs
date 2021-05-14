use syn::spanned::Spanned as _;
use syn::{parse_macro_input, Data, DeriveInput, Error, Fields, LitStr, Ident, Path, Token};
use syn::punctuated::Punctuated;

use quote::quote;
use quote::ToTokens as _;

struct NamedField {
    use_default: bool,
    ident: Ident,
}

impl NamedField {
    pub fn new(f: &syn::Field) -> Result<NamedField, Error> {
        // figure out if we should use default
        let mut use_default = false;

        for attr in f.attrs.iter() {
            if attr.path
                .get_ident()
                .map(|i| i.to_string() == "macrotk")
                .unwrap_or_default() 
            {
                let args: Punctuated<Path, Token![,]> = 
                    attr.parse_args_with(Punctuated::parse_terminated)?;

                for attr in args.iter() {
                    if let Some(s) = attr.get_ident()
                    {
                        match &s.to_string()[..] {
                            "default" => use_default = true,
                            _ => {
                                return Err(Error::new(
                                    s.span(),
                                    format!("unexpected: {}", s),
                                ))
                            }
                        }
                    } else {
                        return Err(Error::new(
                            attr.span(),
                            format!("unexpected: {}", attr.into_token_stream().to_string()),
                        ))
                    }
                }
            }
        }

        Ok(NamedField {
            use_default,
            ident: f.ident.clone().unwrap(),
        })
    }
}

#[proc_macro_derive(FromMeta, attributes(macrotk))]
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

    // parse all of these fields
    let fields = match item.fields {
        Fields::Named(fields) => {
            match fields.named.iter()
                .map(NamedField::new)
                .collect::<Result<Vec<_>, Error>>()
            {
                Ok(fields) => fields,
                Err(err) => return err.into_compile_error().into(),
            }
        }
        e => {
            return Error::new(e.span(), "struct can only have named fields")
                .into_compile_error()
                .into()
        }
    };

    let unwrapper = fields.iter()
        .map(|field| {
            let name = &field.ident;
            let name_str = LitStr::new(&name.to_string(), name.span());

            if field.use_default {
                quote! {
                    #name: __m.get(#name_str).unwrap_or(Ok(Default::default()))?,
                }
            } else {
                quote! {
                    #name: __m.get(#name_str).ok_or(::macrotk::syn::Error::new(::macrotk::Span::call_site(), ::std::concat!("missing value for ", #name_str)))??,
                }
            }
        });

    let expanded = quote! {
        impl ::macrotk::meta::FromMeta for #type_name {
            fn from_meta(
                __m: &::macrotk::meta::MetaValue,
            ) -> ::std::result::Result<Self, ::macrotk::syn::Error> {
                let __m = __m.as_list();
                let __m = __m.list().unwrap();

                Ok(#type_name {
                    #(#unwrapper)*
                })
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}
