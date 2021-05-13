use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{Error, Path, Token};

use proc_macro2::Span;

use quote::ToTokens as _;

use std::ops::Deref;

/// Types that can be parsed from a [`Meta`] list.
///
/// # Deriving
/// ```ignore
/// use syn::LitStr;
///
/// #[derive(macrotk::FromMeta)]
/// pub struct MyMeta {
///     keyword: LitStr,
/// }
/// ```
///
/// # Implementing
/// When the `FromMeta` derive macro breaks, you can implement this yourself.
/// ```
/// # use macrotk_core as macrotk;
/// use syn::LitStr;
/// use syn::spanned::Spanned as _;
///
/// use macrotk::meta::{FromMeta, MetaStream};
///
/// pub struct MyMeta {
///     keyword: Option<LitStr>,
/// }
///
/// impl FromMeta for MyMeta {
///     fn from_meta(meta: MetaStream) -> Result<Self, syn::Error> {
///         let mut keyword: Option<LitStr> = None;
///
///         while let Some(name) = meta.next_name() {
///             let name = name?;
///
///             match name.as_str() {
///                 "keyword" => keyword = meta.next_value().transpose()?,
///                 s => return Err(syn::Error::new(name.span(), format!("unknown key: {}", s))),
///             }
///         }
///
///         Ok(MyMeta { keyword })
///     }
/// }
/// ```
pub trait FromMeta: Sized {
    fn from_meta(a: MetaStream) -> Result<Self, Error>;
}

/// Types that can be parsed as values of a [`Meta`] list.
pub trait FromMetaValue: Sized {
    fn from_meta_value(p: ParseStream) -> Result<Self, Error>;
}

/// A list of meta values that can be interpreted as a list or as a name-value
/// paired list.
pub struct MetaStream<'a>(ParseStream<'a>);

impl<'a> MetaStream<'a> {
    pub fn new(p: ParseStream<'a>) -> MetaStream<'a> {
        MetaStream(p)
    }

    /// Gets the next name of the meta.
    ///
    /// Returns `Ok(None)` if there are no more values.
    pub fn next_name(&self) -> Option<Result<Name, Error>> {
        if self.0.is_empty() {
            None
        } else {
            // get the next path
            let path = match self.0.parse::<Path>() {
                Ok(path) => path,
                Err(err) => return Some(Err(err)),
            };
            // eat the next equals
            match self.0.parse::<Token![=]>() {
                Ok(_) => (),
                Err(err) => return Some(Err(err)),
            }

            Some(Ok(Name::new(path)))
        }
    }

    /// Gets the next value of the meta.
    ///
    /// This can safely be called successively, as if you were interpreting a
    /// list.
    pub fn next_value<T>(&self) -> Option<Result<T, Error>>
    where
        T: FromMetaValue,
    {
        if self.0.is_empty() {
            None
        } else {
            // parse the type
            let result = match T::from_meta_value(self.0) {
                Ok(result) => result,
                Err(err) => return Some(Err(err)),
            };
            // eat the next comma, if it exists
            if !self.0.is_empty() {
                match self.0.parse::<Token![,]>() {
                    Ok(_) => (),
                    Err(err) => return Some(Err(err)),
                }
            }

            Some(Ok(result))
        }
    }
}

/// A name of a name-value paired [`Meta`] list.
///
/// This type can be matched with string literals.
/// ```
/// # use macrotk_core as macrotk;
/// use macrotk::meta::Name;
///
/// let name = Name::from("howdy");
///
/// match name.as_str() {
///     "howdy" => println!("How are you doing?"),
///     _ => panic!("Should match with \"howdy\""),
/// }
/// ```
pub struct Name {
    name: String,
    span: Span,
}

impl Name {
    /// Explicitly converts a `&Name` to a `&str`.
    pub fn as_str(&self) -> &str {
        &self
    }

    fn new(path: Path) -> Name {
        Name {
            span: path.span(),
            name: path.into_token_stream().to_string(),
        }
    }
}

impl<T> From<T> for Name
where
    T: Into<String>,
{
    fn from(s: T) -> Name {
        Name {
            name: s.into(),
            span: Span::call_site(),
        }
    }
}

impl Deref for Name {
    type Target = str;

    fn deref(&self) -> &str {
        &self.name
    }
}

impl Spanned for Name {
    fn span(&self) -> Span {
        self.span
    }
}

impl FromMetaValue for Name {
    fn from_meta_value(p: ParseStream) -> Result<Self, Error> {
        Ok(Name::new(p.parse::<Path>()?))
    }
}

/// A helper type for parsing `FromMeta` values from `TokenStream`s.
pub struct Meta<T>(pub T);

impl<T> Meta<T> {
    /// Extracts the inner `T`.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Deref for Meta<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> Parse for Meta<T>
where
    T: FromMeta,
{
    fn parse(p: ParseStream) -> Result<Self, Error> {
        T::from_meta(MetaStream::new(p)).map(|t| Meta(t))
    }
}

// All `FromMeta` values can also be `FromMetaValue` with the use of `{ }`
impl<T> FromMetaValue for T
where
    T: FromMeta,
{
    fn from_meta_value(p: ParseStream) -> Result<Self, Error> {
        let content;
        syn::braced!(content in p);

        T::from_meta(MetaStream::new(&content))
    }
}

// All `FromMetaValue` values can also be optional
impl<T> FromMetaValue for Option<T>
where
    T: FromMetaValue,
{
    fn from_meta_value(p: ParseStream) -> Result<Self, Error> {
        Ok(Some(T::from_meta_value(p)?))
    }
}

// OTHER MISC IMPLEMENTATIONS
impl FromMetaValue for syn::LitStr {
    fn from_meta_value(p: ParseStream) -> Result<Self, Error> {
        p.parse::<syn::LitStr>()
    }
}
