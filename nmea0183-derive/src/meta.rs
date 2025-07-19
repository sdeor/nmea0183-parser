use std::fmt::Display;

use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use syn::{
    Attribute, Error, Expr, Ident, Lit, Pat, Result, Stmt, Token, Type, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::Paren,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MetaAttributeType {
    Cond,
    Exact,
    Ignore,
    Into,
    Map,
    ParseAs,
    Parser,
    PreExec,
    PostExec,
    Selector,
    SelectionError,
    Separator,
    SkipAfter,
    SkipBefore,
}

impl MetaAttributeType {
    pub fn from_ident(ident: &Ident) -> Option<Self> {
        match ident.to_string().as_str() {
            "cond" => Some(Self::Cond),
            "exact" => Some(Self::Exact),
            "ignore" => Some(Self::Ignore),
            "into" => Some(Self::Into),
            "map" => Some(Self::Map),
            "parse_as" => Some(Self::ParseAs),
            "parser" => Some(Self::Parser),
            "pre_exec" => Some(Self::PreExec),
            "post_exec" => Some(Self::PostExec),
            "selector" => Some(Self::Selector),
            "selection_error" => Some(Self::SelectionError),
            "separator" => Some(Self::Separator),
            "skip_after" => Some(Self::SkipAfter),
            "skip_before" => Some(Self::SkipBefore),
            _ => None,
        }
    }

    fn takes_argument(&self) -> bool {
        matches!(
            self,
            Self::Cond
                | Self::Map
                | Self::ParseAs
                | Self::Parser
                | Self::PreExec
                | Self::PostExec
                | Self::Selector
                | Self::SelectionError
                | Self::Separator
                | Self::SkipAfter
                | Self::SkipBefore
        )
    }

    fn allowed_multiple(&self) -> bool {
        matches!(
            self,
            Self::Cond | Self::Map | Self::PreExec | Self::PostExec
        )
    }
}

impl Display for MetaAttributeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Cond => "cond",
            Self::Exact => "exact",
            Self::Ignore => "ignore",
            Self::Into => "into",
            Self::Map => "map",
            Self::ParseAs => "parse_as",
            Self::Parser => "parser",
            Self::PreExec => "pre_exec",
            Self::PostExec => "post_exec",
            Self::Selector => "selector",
            Self::SelectionError => "selection_error",
            Self::Separator => "separator",
            Self::SkipAfter => "skip_after",
            Self::SkipBefore => "skip_before",
        };
        write!(f, "{name}")
    }
}

#[derive(Debug)]
pub struct MetaAttribute {
    pub r#type: MetaAttributeType,
    arg: Option<TokenStream>,
    span: Span,
}

impl MetaAttribute {
    pub fn new(r#type: MetaAttributeType, arg: Option<TokenStream>, span: Span) -> Self {
        Self { r#type, arg, span }
    }

    pub fn is_top_level(&self) -> bool {
        matches!(
            self.r#type,
            MetaAttributeType::Exact
                | MetaAttributeType::PreExec
                | MetaAttributeType::PostExec
                | MetaAttributeType::Selector
                | MetaAttributeType::SelectionError
                | MetaAttributeType::Separator
                | MetaAttributeType::SkipAfter
                | MetaAttributeType::SkipBefore
        )
    }

    pub fn is_field_level(&self) -> bool {
        !matches!(
            self.r#type,
            MetaAttributeType::Exact
                | MetaAttributeType::Separator
                | MetaAttributeType::SelectionError
        )
    }

    pub fn arg(&self) -> Option<&TokenStream> {
        self.arg.as_ref()
    }

    pub fn span(&self) -> Span {
        self.span
    }
}

impl Parse for MetaAttribute {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident: Ident = input.parse()?;
        let attribute_type = MetaAttributeType::from_ident(&ident)
            .ok_or_else(|| Error::new(ident.span(), "nmea0183-derive: Unknown nmea attribute"))?;

        let arg = if attribute_type.takes_argument() {
            // read (value) or ="value"

            let token_stream = match attribute_type {
                MetaAttributeType::PreExec | MetaAttributeType::PostExec => {
                    parse_argument::<Stmt>(input)?
                }
                MetaAttributeType::ParseAs => parse_argument::<Type>(input)?,
                MetaAttributeType::Selector => parse_argument::<PatAndGuard>(input)?,
                _ => parse_argument::<Expr>(input)?,
            };
            Some(token_stream)
        } else {
            None
        };

        Ok(MetaAttribute::new(attribute_type, arg, ident.span()))
    }
}

impl Display for MetaAttribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.r#type)?;
        if let Some(arg) = &self.arg {
            write!(f, "({arg})")?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct List<T: Parse>(pub Vec<T>);

impl<T: Parse> Parse for List<T> {
    fn parse(input: ParseStream) -> Result<Self> {
        // let content;
        // parenthesized!(content in input);
        Ok(List(
            Punctuated::<T, Token![,]>::parse_terminated(input)?
                .into_iter()
                .collect(),
        ))
    }
}

pub struct PatAndGuard {
    pub pat: Pat,
    pub guard: Option<(Token![if], Box<Expr>)>,
}

impl Parse for PatAndGuard {
    fn parse(input: ParseStream) -> Result<Self> {
        let pat = Pat::parse_multi_with_leading_vert(input)?;
        let guard = if input.peek(Token![if]) {
            let if_token = input.parse()?;
            let guard = input.parse()?;
            Some((if_token, Box::new(guard)))
        } else {
            None
        };
        Ok(Self { pat, guard })
    }
}

impl quote::ToTokens for PatAndGuard {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.pat.to_tokens(tokens);
        if let Some((if_token, guard)) = &self.guard {
            if_token.to_tokens(tokens);
            guard.to_tokens(tokens);
        }
    }
}

fn parse_argument<P>(input: ParseStream) -> Result<TokenStream>
where
    P: Parse + ToTokens,
{
    if input.peek(Token![=]) {
        let _: Token![=] = input.parse()?;
        let value = Lit::parse(input)?;

        match value {
            Lit::Str(string) => {
                let parsed: P = string.parse()?;
                Ok(quote! { #parsed })
            }
            _ => Err(Error::new(
                value.span(),
                "nmea0183-derive: Unexpected type for nmea attribute content",
            )),
        }
    } else if input.peek(Paren) {
        let content;
        parenthesized!(content in input);
        let parsed: P = content.parse()?;
        Ok(quote! { #parsed })
    } else {
        Err(Error::new(
            input.span(),
            "nmea0183-derive: Expected '= <value>' or '(<value>)' for nmea attribute",
        ))
    }
}

pub fn parse_top_level_attributes(attrs: &[Attribute]) -> Result<Vec<MetaAttribute>> {
    let mut attributes_set = std::collections::HashSet::new();

    attrs
        .iter()
        .filter_map(|attr| {
            if attr.path().is_ident("nmea") {
                Some(attr.parse_args::<List<MetaAttribute>>())
            } else {
                None
            }
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flat_map(|list| list.0.into_iter())
        .map(|meta_attr| {
            if !meta_attr.is_top_level() {
                return Err(Error::new(
                    meta_attr.span(),
                    format!(
                        "nmea0183-derive: Attribute `{}` is not allowed at the top level",
                        meta_attr.r#type
                    ),
                ));
            }

            if !attributes_set.insert(meta_attr.r#type.to_string()) {
                return Err(Error::new(
                    meta_attr.span(),
                    format!(
                        "nmea0183-derive: Duplicate nmea attribute `{}`",
                        meta_attr.r#type
                    ),
                ));
            }

            Ok(meta_attr)
        })
        .collect()
}

pub fn parse_field_level_attributes(attrs: &[Attribute]) -> Result<Vec<MetaAttribute>> {
    let mut attributes_set = std::collections::HashSet::new();

    attrs
        .iter()
        .filter_map(|attr| {
            if attr.path().is_ident("nmea") {
                Some(attr.parse_args::<List<MetaAttribute>>())
            } else {
                None
            }
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flat_map(|list| list.0.into_iter())
        .map(|meta_attr| {
            if !meta_attr.is_field_level() {
                return Err(Error::new(
                    meta_attr.span(),
                    format!(
                        "nmea0183-derive: Attribute `{}` is not allowed at the field level",
                        meta_attr.r#type
                    ),
                ));
            }

            if !meta_attr.r#type.allowed_multiple()
                && !attributes_set.insert(meta_attr.r#type.to_string())
            {
                return Err(Error::new(
                    meta_attr.span(),
                    format!(
                        "nmea0183-derive: Duplicate nmea attribute `{}`",
                        meta_attr.r#type
                    ),
                ));
            }

            // Only one of `parse_as` or `parser` can be used.
            if meta_attr.r#type == MetaAttributeType::ParseAs && attributes_set.contains("parser") {
                return Err(Error::new(
                    meta_attr.span(),
                    "nmea0183-derive: Attribute `parse_as` cannot be used with `parser` attribute.",
                ));
            }
            if meta_attr.r#type == MetaAttributeType::Parser && attributes_set.contains("parse_as")
            {
                return Err(Error::new(
                    meta_attr.span(),
                    "nmea0183-derive: Attribute `parser` cannot be used with `parse_as` attribute.",
                ));
            }

            Ok(meta_attr)
        })
        .collect()
}
