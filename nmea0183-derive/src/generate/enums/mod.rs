use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, DataEnum, Generics, Ident, Path, Result, parse_quote, spanned::Spanned};

use crate::{
    config::Config,
    generate::{Generator, enums::parser::VariantParser, pre_post_exec, structs::Struct},
    meta,
};

pub mod parser;

pub struct Enum {
    pub name: Path,
    pub config: Config,
    pub generics: Generics,
    pub pre_exec: Option<TokenStream>,
    pub post_exec: Option<TokenStream>,
    pub variant_parsers: Vec<VariantParser>,
}

impl Enum {
    pub fn from_dataenum(
        name: &Ident,
        dataenum: &DataEnum,
        attributes: &[Attribute],
        generics: &Generics,
    ) -> Result<Self> {
        let attributes = meta::parse_top_level_attributes(attributes)?;
        let config = Config::from_meta_attributes(&attributes)?;

        let has_selector = config.selector_parser.is_some();
        if !has_selector {
            return Err(syn::Error::new(
                name.span(),
                "nmea0183-derive: Enums must have a `selector` attribute",
            ));
        }

        let separator = attributes
            .iter()
            .find(|attr| attr.r#type == meta::MetaAttributeType::Separator);

        if let Some(attr) = separator {
            return Err(syn::Error::new(
                attr.span(),
                "nmea0183-derive: Enums do not support `separator` attributes yet; this will be implemented in the future.",
            ));
        }

        let variant_parsers = dataenum
            .variants
            .iter()
            .map(|variant| VariantParser::from_variant(variant, &config))
            .collect::<Result<Vec<_>>>()?;
        let (pre_exec, post_exec) = pre_post_exec(&attributes, &config)?;

        Ok(Self {
            name: parse_quote!(#name),
            config,
            generics: generics.clone(),
            pre_exec,
            post_exec,
            variant_parsers,
        })
    }

    pub fn generate_variants(&self) -> Result<(bool, Vec<TokenStream>)> {
        let enum_name = &self.name;
        let input = &self.config.input_name;
        let mut default_case_handled = false;
        let variant_tokens = self
            .variant_parsers
            .iter()
            .map(|variant_parser| {
                if variant_parser.selector.to_string() == "_" {
                    default_case_handled = true;
                }

                let variant_name = &variant_parser.name;
                let selector = &variant_parser.selector;

                let pre_exec = &variant_parser.pre_exec;
                let post_exec = &variant_parser.post_exec;

                let r#struct = Struct {
                    config: self.config.clone(),
                    name: parse_quote!(#enum_name::#variant_name),
                    generics: self.generics.clone(),
                    pre_exec: None,
                    post_exec: None,
                    struct_parser: variant_parser.struct_parser.clone(),
                };

                let struct_body = r#struct.generate_parse_body(false).unwrap();

                quote! {
                    #selector => {
                        #pre_exec
                        let (#input, struct_def) = { #struct_body }?;
                        #post_exec
                        Ok((#input, struct_def))
                    }
                }
            })
            .collect();

        // If the default case is handled, make sure it is the last entry
        if default_case_handled {
            let default_position = self
                .variant_parsers
                .iter()
                .position(|variant_parser| {
                    // Check if the variant_parser.selector (TokenStream) is the default case (_)
                    variant_parser.selector.to_string() == "_"
                })
                .expect("Default case is handled but not found");
            if default_position != self.variant_parsers.len() - 1 {
                return Err(syn::Error::new(
                    self.variant_parsers[default_position].selector.span(),
                    "nmea0183-derive: Default case must be the last entry in the enum",
                ));
            }
        }

        Ok((default_case_handled, variant_tokens))
    }
}

impl Generator for Enum {
    fn name(&self) -> &Path {
        &self.name
    }

    fn config(&self) -> &Config {
        &self.config
    }

    fn generics(&self) -> &Generics {
        &self.generics
    }

    fn generate_parse_body(&self, use_nom_parser: bool) -> Result<TokenStream> {
        let (pre_exec, post_exec) = (&self.pre_exec, &self.post_exec);
        let input = &self.config.input_name;
        let selector = &self.config.selector_name;
        let selector_parser = self.config.selector_parser.as_ref().unwrap();
        let selection_error = self.config.selection_error.as_ref();
        let (default_case_handled, variant_tokens) = self.generate_variants()?;

        let default_case = if default_case_handled {
            quote! {}
        } else if let Some(error) = selection_error {
            quote! { _ => Err(nom::Err::Error(#error)) }
        } else {
            quote! { _ => Err(nom::Err::Error(nom::error::make_error(#input, nom::error::ErrorKind::Switch))) }
        };

        let use_nom_parser = if use_nom_parser {
            quote! { use nom::Parser; }
        } else {
            quote! {}
        };

        let body = quote! {
            #use_nom_parser
            #pre_exec
            let (#input, #selector) = #selector_parser.parse(#input)?;
            let (#input, enum_def) = match #selector {
                #(#variant_tokens)*
                #default_case
            }?;
            #post_exec
            Ok((#input, enum_def))
        };

        Ok(body)
    }
}
