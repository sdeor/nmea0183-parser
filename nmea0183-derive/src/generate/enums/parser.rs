use proc_macro2::TokenStream;
use syn::{Error, Ident, Result, Variant, spanned::Spanned};

use crate::{
    config::Config,
    generate::{pre_post_exec, structs::parser::StructParser},
    meta,
};

pub struct VariantParser {
    pub name: Ident,
    pub selector: TokenStream,
    pub pre_exec: Option<TokenStream>,
    pub post_exec: Option<TokenStream>,
    pub struct_parser: StructParser,
}

impl VariantParser {
    pub fn from_variant(variant: &Variant, config: &Config) -> Result<Self> {
        let attributes = meta::parse_field_level_attributes(&variant.attrs)?;

        let selector = attributes
            .iter()
            .find(|attr| attr.r#type == meta::MetaAttributeType::Selector)
            .map(|attr| attr.arg().unwrap().clone())
            .ok_or(Error::new(
                variant.span(),
                "nmea0183-derive: Variants must have a `selector` attribute",
            ))?;

        let struct_parser = StructParser::from_fields(&variant.fields, config, true)?;
        let (pre_exec, post_exec) = pre_post_exec(&attributes, config)?;

        Ok(Self {
            name: variant.ident.clone(),
            selector,
            pre_exec,
            post_exec,
            struct_parser,
        })
    }
}
