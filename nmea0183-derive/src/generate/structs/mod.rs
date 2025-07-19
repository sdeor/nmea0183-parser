use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Attribute, DataStruct, Generics, Ident, Path, Result, parse_quote};

use crate::{
    config::Config,
    generate::{Generator, pre_post_exec, structs::parser::StructParser},
    meta,
};

pub mod parser;

pub struct Struct {
    pub name: Path,
    pub config: Config,
    pub generics: Generics,
    pub pre_exec: Option<TokenStream>,
    pub post_exec: Option<TokenStream>,
    pub struct_parser: StructParser,
}

impl Struct {
    pub fn from_datastruct(
        name: &Ident,
        datastruct: &DataStruct,
        attributes: &[Attribute],
        generics: &Generics,
    ) -> Result<Self> {
        let attributes = meta::parse_top_level_attributes(attributes)?;

        for attribute in &attributes {
            match attribute.r#type {
                meta::MetaAttributeType::Selector => {
                    return Err(syn::Error::new(
                        attribute.span(),
                        "nmea0183-derive: Structs does not support `selector` attributes; only enums support this feature.",
                    ));
                }
                meta::MetaAttributeType::SelectionError => {
                    return Err(syn::Error::new(
                        attribute.span(),
                        "nmea0183-derive: Structs do not support `selection_error` attributes; only enums support this feature.",
                    ));
                }
                meta::MetaAttributeType::Separator => {
                    return Err(syn::Error::new(
                        attribute.span(),
                        "nmea0183-derive: Structs do not support `separator` attributes yet; this will be implemented in the future.",
                    ));
                }
                _ => {}
            }
        }

        let config = Config::from_meta_attributes(&attributes)?;
        let struct_parser = StructParser::from_fields(&datastruct.fields, &config, false)?;
        let (pre_exec, post_exec) = pre_post_exec(&attributes, &config)?;

        Ok(Self {
            name: parse_quote!(#name),
            config,
            generics: generics.clone(),
            pre_exec,
            post_exec,
            struct_parser,
        })
    }
}

impl Generator for Struct {
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
        let name = &self.name;
        let (pre_exec, post_exec) = (&self.pre_exec, &self.post_exec);
        let input = &self.config.input_name;

        let (variable_name, parser): (Vec<_>, Vec<_>) = self
            .struct_parser
            .parsers
            .iter()
            .map(|field_parser| {
                (
                    Ident::new(&field_parser.variable_name, Span::call_site()),
                    &field_parser.parser,
                )
            })
            .unzip();

        let (field_pre_exec, field_post_exec): (Vec<_>, Vec<_>) = self
            .struct_parser
            .parsers
            .iter()
            .map(|field_parser| {
                (
                    field_parser.pre_exec.as_ref(),
                    field_parser.post_exec.as_ref(),
                )
            })
            .unzip();

        let struct_def = match (self.struct_parser.empty, self.struct_parser.unnamed) {
            (true, _) => {
                // If the struct is empty, we just return an empty struct
                quote! { #name }
            }
            (_, true) => {
                // If the struct is unnamed, we create a tuple struct
                quote! { #name(#(#variable_name),*) }
            }
            (_, false) => {
                // If the struct is named, we create a named struct
                quote! { #name { #(#variable_name),* } }
            }
        };

        let use_nom_parser = if use_nom_parser {
            quote! { use nom::Parser; }
        } else {
            quote! {}
        };

        let body = quote! {
            #use_nom_parser
            #pre_exec
            #(#field_pre_exec let (#input, #variable_name) = #parser.parse(#input)?; #field_post_exec)*
            let struct_def = #struct_def;
            #post_exec
            Ok((#input, struct_def))
        };

        Ok(body)

        // todo!("Implement generate_parse_body for Struct");
    }
}
