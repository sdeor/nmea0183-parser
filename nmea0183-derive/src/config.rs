use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Ident, Lifetime, Result};

use crate::meta::{MetaAttribute, MetaAttributeType};

#[derive(Clone)]
pub struct Config {
    pub input_name: Ident,
    pub selector_name: Ident,
    pub selector_parser: Option<TokenStream>,
    pub selection_error: Option<TokenStream>,
    pub error_type: Ident,
    pub lifetime: Lifetime,
    pub separator: TokenStream,
}

impl Config {
    pub fn from_meta_attributes(attribute_list: &[MetaAttribute]) -> Result<Self> {
        let mut selector_parser = None;
        let mut separator = quote! { nom::character::complete::char(',') };
        let mut selection_error = None;

        for meta in attribute_list {
            match meta.r#type {
                MetaAttributeType::Selector => selector_parser = Some(meta.arg().unwrap().clone()),
                MetaAttributeType::Separator => separator = meta.arg().unwrap().clone(),
                MetaAttributeType::SelectionError => {
                    selection_error = Some(meta.arg().unwrap().clone())
                }
                _ => {}
            }
        }

        Ok(Self {
            input_name: Ident::new("nmea_input", Span::call_site()),
            selector_name: Ident::new("nmea_selector", Span::call_site()),
            selector_parser,
            selection_error,
            error_type: Ident::new("NmeaError", Span::call_site()),
            lifetime: Lifetime::new("'nmea", Span::call_site()),
            separator,
        })
    }
}
