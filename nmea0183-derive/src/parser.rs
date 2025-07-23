use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::Type;

#[derive(Clone)]
pub enum Parser {
    Cond {
        parser: Box<Parser>,
        condition: TokenStream,
    },
    Into(Box<Parser>),
    Map {
        parser: Box<Parser>,
        map: TokenStream,
    },
    Raw(TokenStream),
    Type {
        ty: Box<Type>,
        separator: Option<TokenStream>,
    },
}

impl Parser {
    pub fn as_nmeaparse(self, error_type: &syn::Ident, nmea_lifetime: &syn::Lifetime) -> Self {
        match self {
            Self::Type { ty, separator } => {
                let parser = if let Some(separator) = separator {
                    quote! { <#ty as nmea0183_parser::NmeaParse<&#nmea_lifetime str, #error_type>>::parse_preceded(#separator) }
                } else {
                    quote! { <#ty as nmea0183_parser::NmeaParse<&#nmea_lifetime str, #error_type>>::parse }
                };
                Self::Raw(parser)
            }
            parser => parser,
        }
    }
}

impl ToTokens for Parser {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let token_stream = match self {
            Self::Cond { parser, condition } => {
                quote! { nom::combinator::cond(#condition, #parser) }
            }
            Self::Into(parser) => {
                quote! { nom::combinator::into(#parser) }
            }
            Self::Map { parser, map } => {
                quote! { nom::combinator::map(#parser, #map) }
            }
            Self::Raw(parser) => parser.to_token_stream(),
            Self::Type { ty, separator } => {
                if let Some(separator) = separator {
                    quote! { <#ty>::parse_preceded(#separator) }
                } else {
                    quote! { <#ty>::parse }
                }
            }
        };

        tokens.extend(token_stream);
    }
}
