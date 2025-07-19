use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Data, DeriveInput, Error, GenericParam, Generics, LifetimeParam, Path, Result, TypeParam,
    WhereClause, parse_quote,
};

use crate::{
    config::Config,
    generate::{enums::Enum, structs::Struct},
    meta::{MetaAttribute, MetaAttributeType},
};

mod enums;
mod structs;

// Usage:
// #[derive(Nmea0183)]
// pub struct MySentence {
//     #[nmea(parser = "custom_parser")]
//     pub field: Option<MyType>,
//     #[nmea(ignore)]
//     pub computed_field: u32,
//     pub another_field: f64,
// }
//
// #[derive(Nmea0183)]
// #[nmea(selector = "my_enum")]
// pub enum MyEnum {
//     #[nmea(selector=1)]
//     Variant1(Option<MyType>),
//     #[nmea(selector=2)]
//     Variant2(u32),
//     #[nmea(selector=3)]
//     Variant3,
// }

trait Generator {
    fn name(&self) -> &Path;
    fn config(&self) -> &Config;
    fn generics(&self) -> &Generics;
    fn generate_parse_body(&self, use_nom_parser: bool) -> Result<TokenStream>;

    fn generate_parse_decl(&self) -> TokenStream {
        let input = &self.config().input_name;
        let error_type = &self.config().error_type;
        let nmea_lifetime = &self.config().lifetime;

        quote! {
            fn parse(#input: &#nmea_lifetime str) -> nmea0183_parser::IResult<&#nmea_lifetime str, Self, #error_type>
        }
    }

    fn generate_parse(&self) -> Result<TokenStream> {
        let decl = self.generate_parse_decl();
        let body = self.generate_parse_body(true)?;

        let func = quote! {
            #decl
            {
                #body
            }
        };

        Ok(func)
    }

    fn generate_impl(&self) -> Result<TokenStream> {
        let name = self.name();
        let error_type = &self.config().error_type;
        let nmea_lifetime = &self.config().lifetime;
        let parse_tokens = self.generate_parse()?;
        let generics = self.generics();
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        let mut impl_generics: Generics = parse_quote!(#impl_generics);

        // Push nmea lifetime to the generics
        impl_generics
            .params
            .push(GenericParam::Lifetime(LifetimeParam::new(
                nmea_lifetime.clone(),
            )));

        // Push nmea error type to the generics
        impl_generics
            .params
            .push(GenericParam::Type(TypeParam::from(error_type.clone())));

        // If there is no where clause, create a new one
        let mut impl_where: WhereClause = if where_clause.is_some() {
            parse_quote!(#where_clause)
        } else {
            parse_quote!(where)
        };

        // // Make sure nmea lifetime is present in the where clause and outlives all other lifetimes
        // let lifetimes: Vec<_> = generics.lifetimes().collect();
        // if !lifetimes.is_empty() {
        //     impl_where
        //         .predicates
        //         .push(parse_quote!(#nmea_lifetime: #(#lifetimes)+*));
        // }

        // Make sure generic parameters implement NmeaParse
        for param in generics.type_params() {
            let param = &param.ident;
            impl_where.predicates.push(
                parse_quote!(#param: nmea0183_parser::NmeaParse<&#nmea_lifetime str, #error_type>),
            );
        }

        // // Push nmea input type to the where clause
        // impl_where
        //     .predicates
        //     .push(parse_quote!(#input_type: nom::Input));

        // Push nmea error type to the where clause
        impl_where
            .predicates
            .push(parse_quote!(#error_type: nom::error::ParseError<&#nmea_lifetime str>));

        // Generate the implementation
        let impl_tokens = quote! {
            impl #impl_generics nmea0183_parser::NmeaParse<&#nmea_lifetime str, #error_type> for #name #ty_generics #impl_where {
                #parse_tokens
            }
        };

        Ok(impl_tokens)
    }
}

pub fn get_error_if(cond: &TokenStream, config: &Config) -> TokenStream {
    let input = &config.input_name;
    quote! {
        if #cond {
            return Err(nom::Err::Error(nom::error::make_error(#input, nom::error::ErrorKind::Verify)));
        }
    }
}

pub fn pre_post_exec(
    attributes: &[MetaAttribute],
    config: &Config,
) -> Result<(Option<TokenStream>, Option<TokenStream>)> {
    let mut pre_exec = TokenStream::new();
    let mut post_exec = TokenStream::new();

    for attribute in attributes {
        match attribute.r#type {
            MetaAttributeType::Exact => {
                let input = &config.input_name;
                let cond = quote! { !#input.is_empty() };
                post_exec.extend(get_error_if(&cond, config));
            }
            MetaAttributeType::PreExec => {
                pre_exec.extend(attribute.arg().unwrap().clone());
            }
            MetaAttributeType::PostExec => {
                post_exec.extend(attribute.arg().unwrap().clone());
            }
            MetaAttributeType::SkipBefore => {
                let skip = attribute.arg().unwrap();
                let input = &config.input_name;

                let skip = quote! {
                    let (#input, _) = nom::bytes::streaming::take(#skip as usize).parse(#input)?;
                };

                pre_exec.extend(skip);
            }
            MetaAttributeType::SkipAfter => {
                let skip = attribute.arg().unwrap();
                let input = &config.input_name;

                let skip = quote! {
                    let (#input, _) = nom::bytes::streaming::take(#skip as usize).parse(#input)?;
                };

                post_exec.extend(skip);
            }
            _ => {}
        }
    }

    let pre_exec = (!pre_exec.is_empty()).then_some(pre_exec);
    let post_exec = (!post_exec.is_empty()).then_some(post_exec);

    Ok((pre_exec, post_exec))
}

pub fn generate_nmea_parse_impl(input: &DeriveInput) -> Result<TokenStream> {
    let generator: Box<dyn Generator> = match &input.data {
        Data::Struct(datastruct) => {
            let name = &input.ident;
            let attributes = &input.attrs;
            let generics = &input.generics;

            Box::new(Struct::from_datastruct(
                name, datastruct, attributes, generics,
            )?)
        }
        Data::Enum(dataenum) => {
            let name = &input.ident;
            let attributes = &input.attrs;
            let generics = &input.generics;

            Box::new(Enum::from_dataenum(name, dataenum, attributes, generics)?)
        }
        Data::Union(_) => {
            return Err(Error::new(
                input.ident.span(),
                "nmea0183-derive: Unions not supported",
            ));
        }
    };

    generator.generate_impl()
}
