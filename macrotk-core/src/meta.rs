use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::punctuated::Punctuated;
use syn::{Error, Path, Token, Lit, LitStr};

use proc_macro2::Span;

/// Types that can be parsed from a [`Meta`] list.
pub trait FromMeta: Sized {
    fn from_meta(a: &MetaValue) -> Result<Self, Error>;
}

/// A meta item.
#[derive(Clone)]
pub enum MetaValue {
    Path(Path),
    NameValue(MetaNameValue),
    List(MetaList),
    Lit(Lit),
}

impl MetaValue {
    /// Tries to take the value as a [`Path`], returning an error if it isn't
    /// a path.
    pub fn path(&self) -> Result<&Path, Error> {
        match self {
            Self::Path(p) => Ok(p),
            Self::NameValue(nv) => Err(
                Error::new(
                    nv.eq.span(),
                    "unexpected =; expected a naked path",
                )
            ),
            Self::List(list) => Err(
                Error::new(
                    list.paren.span,
                    "unexpected (...); expected a naked path",
                )
            ),
            Self::Lit(lit) => Err(
                Error::new(
                    lit.span(),
                    "expected a naked path",
                )
            ),
        }
    }

    /// Tries to take the value as a [`MetaNameValue`], returning an error if
    /// it isn't one.
    pub fn name_value(&self) -> Result<&MetaNameValue, Error> {
        match self {
            Self::NameValue(nv) => Ok(nv),
            Self::Path(p) => Err(
                Error::new(
                    p.span(),
                    "expected =",
                )
            ),
            Self::List(list) => Err(
                Error::new(
                    list.paren.span,
                    "expected =",
                )
            ),
            Self::Lit(lit) => Err(
                Error::new(
                    lit.span(),
                    "unexpected literal; expected =",
                )
            ),
        }
    }

    /// Tries to take the value as a [`Lit`], returning an error if it isn't
    /// a literal.
    pub fn literal(&self) -> Result<&Lit, Error> {
        match self {
            Self::Lit(lit) => Ok(lit),
            Self::Path(p) => Err(
                Error::new(
                    p.span(),
                    "expected a literal",
                )
            ),
            Self::List(list) => Err(
                Error::new(
                    list.name.span(),
                    "expected a literal",
                )
            ),
            Self::NameValue(nv) => Err(
                Error::new(
                    nv.name.span(),
                    "expected a literal",
                )
            ),
        }
    }

    /// Tries to take the value as a [`List`], returning an error if it isn't
    /// a list.
    pub fn list(&self) -> Result<&MetaList, Error> {
        match self {
            Self::List(list) => Ok(list),
            Self::Path(p) => Err(
                Error::new(
                    p.span(),
                    "expected a list",
                )
            ),
            Self::NameValue(nv) => Err(
                Error::new(
                    nv.eq.span(),
                    "unexpected =; expected a list",
                )
            ),
            Self::Lit(lit) => Err(
                Error::new(
                    lit.span(),
                    "expected a list",
                )
            ),
        }
    }

    // TODO: make this a bit more cooperative
    /// Forces the MetaValue to behave as a list.
    ///
    /// Does nothing if the meta is already a list.
    pub fn as_list(&self) -> MetaValue {
        if matches!(self, MetaValue::List(_)) {
            self.clone()
        } else {
            let mut list = Punctuated::default();

            list.push(self.clone());

            MetaValue::List(
                MetaList {
                    name: syn::parse_quote!(empty),
                    paren: syn::token::Paren {
                        span: Span::call_site(),
                    },
                    list,
                }
            )
        }
    }

    // TODO: make this a bit more cooperative
    /// Gets an empty list.
    pub fn empty() -> MetaValue {
        MetaValue::List(
            MetaList {
                name: syn::parse_quote!(empty),
                paren: syn::token::Paren {
                    span: Span::call_site(),
                },
                list: Punctuated::default(),
            }
        )
    }

    pub fn name(&self) -> Option<&syn::Ident> {
        let path = match self {
            Self::Path(p) => p,
            Self::List(list) => &list.name,
            Self::NameValue(nv) => &nv.name,
            _ => return None,
        };

        path.segments.last().map(|l| &l.ident)
    }
}

impl From<Lit> for MetaValue {
    fn from(l: Lit) -> MetaValue {
        MetaValue::Lit(l)
    }
}

impl Parse for MetaValue {
    fn parse(p: ParseStream) -> Result<MetaValue, Error> {
        if let Ok(name) = p.parse::<Path>() {
            // check if we have an eq or a paren in front
            if p.peek(syn::token::Paren) {
                // this is a list
                let list;

                Ok(MetaValue::List(
                    MetaList {
                        name,
                        paren: syn::parenthesized!(list in p),
                        list: list.parse_terminated(Self::parse)?,
                    }
                ))
            } else if p.peek(Token![=]) {
                // this is a name-value pair
                Ok(MetaValue::NameValue(
                    MetaNameValue {
                        name,
                        eq: p.parse()?,
                        value: p.parse()?,
                    }
                ))
            } else {
                // this is a path
                Ok(MetaValue::Path(name))
            }
        } else {
            // parse literal
            p.parse::<Lit>().map(Into::into)
        }
    }
}

/// A meta name-value pair.
#[derive(Clone)]
pub struct MetaNameValue {
    pub name: Path,
    pub eq: Token![=],
    pub value: Lit,
}

/// A meta list.
#[derive(Clone)]
pub struct MetaList {
    pub name: Path,
    pub paren: syn::token::Paren,
    pub list: Punctuated<MetaValue, Token![,]>,
}

impl MetaList {
    /// Gets a type by name.
    ///
    /// This considers both list types ([`MetaList`]) and name-value pairs
    /// ([`MetaNameValue`]).
    pub fn get<T>(&self, name: &str) -> Option<Result<T, Error>> 
    where T:
        FromMeta,
    {
        let item = self.list.iter()
            .filter(|meta| meta.name().map(|n| n == name).unwrap_or(false))
            .next()?;

        let item = match item {
            MetaValue::NameValue(nv) => MetaValue::Lit(nv.value.clone()),
            item => item.clone(),
        };

        // try to convert the type
        Some(T::from_meta(&item))
    }
}

/// Helper type for parsing attribute token streams in an attribute proc
/// macro.
pub struct Meta<T>(pub T);

impl<T> Meta<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> std::ops::Deref for Meta<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> Parse for Meta<T> 
where T:
    FromMeta,
{
    fn parse(p: ParseStream) -> Result<Meta<T>, Error> {
        if p.is_empty() {
            // use empty list
            T::from_meta(&MetaValue::empty())
                .map(|t| Meta(t))
        } else {
            p.parse::<MetaValue>()
                .and_then(|meta| T::from_meta(&meta))
                .map(|t| Meta(t))
        }
    }
}

// some basic impls
impl FromMeta for LitStr {
    fn from_meta(meta: &MetaValue) -> Result<LitStr, Error> {
        match meta.literal()? {
            Lit::Str(lit) => Ok(lit.clone()),
            _ => Err(
                Error::new(
                    Span::call_site(),
                    "expected str literal",
                )
            )
        }
    }
}

// other impls
impl<T> FromMeta for Option<T>
where T:
    FromMeta,
{
    fn from_meta(p: &MetaValue) -> Result<Option<T>, Error> {
        Ok(Some(T::from_meta(p)?))
    }
}

