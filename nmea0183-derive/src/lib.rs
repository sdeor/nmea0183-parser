//! # A Rust procedural macro for NMEA 0183-style content parsing
//!
//! Based on [`nom-derive`] and with a lot of similarities, `nmea0183-derive` is a custom
//! derive attribute to derive content parsers for your NMEA 0183-style data structures.
//!
//! It is not meant to replace [`nmea0183-parser`], but to work alongside it, providing
//! a convenient and easy way to derive parsers for your data structures without having
//! to write boilerplate code.
//!
//! [`nmea0183-parser`]: https://crates.io/crates/nmea0183-parser
//! [`nom-derive`]: https://crates.io/crates/nom-derive

use generate::generate_nmea_parse_impl;
use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

mod config;
mod generate;
mod meta;
mod parser;

#[doc = include_str!("../README.md")]
#[proc_macro_derive(NmeaParse, attributes(nmea))]
pub fn derive_nmea_parse(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match generate_nmea_parse_impl(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
