use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Error, Fields, PathArguments, Result, Type, TypePath, parse2, spanned::Spanned};

use crate::{
    config::Config,
    generate::pre_post_exec,
    meta::{self, MetaAttribute, MetaAttributeType},
    parser::Parser,
};

#[derive(Clone)]
pub struct FieldParser {
    pub variable_name: String,
    pub parser: Parser,
    pub pre_exec: Option<TokenStream>,
    pub post_exec: Option<TokenStream>,
}

#[derive(Clone)]
pub struct StructParser {
    pub empty: bool,
    pub unnamed: bool,
    pub parsers: Vec<FieldParser>,
}

impl StructParser {
    pub fn from_fields(fields: &Fields, config: &Config, preceded: bool) -> Result<Self> {
        let mut empty = false;
        let mut unnamed = false;
        match fields {
            Fields::Named(_) => (),
            Fields::Unnamed(_) => {
                unnamed = true;
            }
            Fields::Unit => {
                unnamed = false;
                empty = true;
            }
        }

        let separator = &config.separator;

        let mut first_field = !preceded;
        let mut parsers = vec![];
        for (index, field) in fields.iter().enumerate() {
            let variable_name = field.ident.as_ref().map_or_else(
                || format!("_nmea_unnamed_{index}"),
                |ident| ident.to_string(),
            );
            let attributes = meta::parse_field_level_attributes(&field.attrs)?;

            let mut ignore = false;
            for attribute in &attributes {
                if attribute.r#type == MetaAttributeType::Ignore {
                    ignore = true;
                }
            }

            let separator = Some(separator).filter(|_| !first_field && !ignore);
            let parser = Self::get_parser(&field.ty, &attributes, separator.cloned())?;

            if first_field && !ignore {
                first_field = false;
            }

            let (pre_exec, post_exec) = pre_post_exec(&attributes, config)?;

            parsers.push(FieldParser {
                variable_name,
                parser,
                pre_exec,
                post_exec,
            });
        }

        Ok(Self {
            empty,
            unnamed,
            parsers,
        })
    }

    fn get_parser(
        ty: &Type,
        attributes: &[MetaAttribute],
        separator: Option<TokenStream>,
    ) -> Result<Parser> {
        let mut attributes = attributes;
        while let Some((attribute, rest)) = attributes.split_first() {
            match attribute.r#type {
                MetaAttributeType::Parser => {
                    let parser = attribute.arg().unwrap();
                    let parser = if let Some(separator) = &separator {
                        quote! { nom::sequence::preceded(#separator, #parser) }
                    } else {
                        parser.clone()
                    };
                    return Ok(Parser::Raw(parser));
                }
                MetaAttributeType::ParseAs => {
                    let parse_as = attribute.arg().unwrap();
                    let parse_as_type = parse2::<Type>(parse_as.clone())?;
                    let parser = Self::get_parser(&parse_as_type, rest, separator)?;
                    return Ok(parser);
                }
                MetaAttributeType::Ignore => {
                    let default = quote! { <#ty>::default() };
                    let parser = quote! { nom::combinator::success(#default) };
                    return Ok(Parser::Raw(parser));
                }
                MetaAttributeType::Cond => {
                    let option = Self::get_innermost_type_parser(ty, "Option", "cond")?;
                    let option_type = parse2::<Type>(option)?;
                    let parser = Self::get_parser(&option_type, rest, separator)?;
                    let condition = attribute.arg().unwrap();
                    return Ok(Parser::Cond {
                        parser: Box::new(parser),
                        condition: condition.clone(),
                    });
                }
                MetaAttributeType::Into => {
                    let parser = Self::get_parser(ty, rest, separator)?;
                    return Ok(Parser::Into(Box::new(parser)));
                }
                MetaAttributeType::Map => {
                    let map = attribute.arg().unwrap();
                    let parser = Self::get_parser(ty, rest, separator)?;
                    return Ok(Parser::Map {
                        parser: Box::new(parser),
                        map: map.clone(),
                    });
                }
                _ => {}
            }

            attributes = rest;
        }

        Ok(Parser::Type {
            ty: Box::new(ty.clone()),
            separator,
        })
    }

    fn get_innermost_type_parser(ty: &Type, expected: &str, attr: &str) -> Result<TokenStream> {
        if let Type::Path(TypePath { path, .. }) = ty {
            if let Some(segment) = path.segments.last() {
                let ident = &segment.ident.to_string();
                if ident == expected {
                    if let PathArguments::AngleBracketed(ref args) = segment.arguments {
                        return Ok(args.args.to_token_stream());
                    }
                } else {
                    return Ok(quote! { #ty });
                }
            }
        }

        Err(Error::new(
            ty.span(),
            format!(
                "nmea0183-derive: Unexpected type for attribute `{attr}`. Expected `{expected}`.",
            ),
        ))
    }
}
