use syn::parse::{ParseStream};
use syn::spanned::Spanned;
use syn::{Path, Token, Error};

use proc_macro2::Span;

use quote::ToTokens as _;

use std::ops::Deref;

/// Types that can be parsed from a [`Meta`] list.
///
/// You should be using the derive macro instead.
pub trait FromMeta: Sized {
    fn from_meta(a: Meta) -> Result<Self, Error>;
}

/// Types that can be parsed as values of a [`Meta`] list.
pub trait FromMetaValue: Sized {
    fn from_meta_value(p: ParseStream) -> Result<Self, Error>;
}

/// A list of meta values that can be interpreted as a list or as a name-value
/// paired list.
pub struct Meta<'a>(ParseStream<'a>);

impl<'a> Meta<'a> {
    pub fn new(p: ParseStream<'a>) -> Meta<'a> {
        Meta(p)
    }

    /// Gets the next name of the meta.
    ///
    /// Returns `Ok(None)` if there are no more values.
    pub fn next_name(&self) -> Result<Option<Name>, Error> {
        if self.0.is_empty() {
            Ok(None)
        } else {
            // get the next path
            let path = self.0.parse::<Path>()?;
            // eat the next equals
            self.0.parse::<Token![=]>()?;

            Ok(Some(Name::new(path)))
        }
    }

    /// Gets the next value of the meta.
    ///
    /// This can safely be called successively, as if you were interpreting a
    /// list.
    pub fn next_value<T>(&self) -> Result<Option<T>, Error>
    where
        T: FromMetaValue
    {
        if self.0.is_empty() {
            Ok(None)
        } else {
            // parse the type
            let result = T::from_meta_value(self.0)?;
            // eat the next comma, if it exists
            if !self.0.is_empty() {
                self.0.parse::<Token![,]>()?;
            }

            Ok(Some(result))
        }
    }
}

/// A name of a name-value paired [`Meta`] list.
///
/// This type can be matched with string literals.
/// ```
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
where T: Into<String> {
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

