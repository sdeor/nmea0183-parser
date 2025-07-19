//! # A Flexible NMEA Framing Parser for Rust
//!
//! [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE-MIT)
//! [![Apache License 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](./LICENSE-APACHE)
//! [![docs.rs](https://docs.rs/nmea0183-parser/badge.svg)](https://docs.rs/nmea0183-parser)
//! [![Crates.io Version](https://img.shields.io/crates/v/nmea0183-parser.svg)](https://crates.io/crates/nmea0183-parser)
//!
//! **A zero-allocation NMEA 0183 parser that separates message framing from
//! content parsing, giving you full control over data handling.**
//!
//! This Rust crate provides a generic and configurable parser for NMEA 0183-style
//! messages, with the typical format:
//!
//! ```text
//! $HHH,D1,D2,...,Dn*CC\r\n
//! ```
//!
//! It focuses on parsing and validating the framing of NMEA 0183-style sentences
//! (start character, optional checksum, and optional CRLF), allowing you to plug
//! in your own domain-specific content parsers — or use built-in ones for common
//! NMEA sentence types.
//!
//! ---
//!
//! ## ✨ Why Use This Crate?
//!
//! Unlike traditional NMEA crates that tightly couple format and content parsing,
//! `nmea0183_parser` lets you:
//!
//! - ✅ Choose your compliance level (strict vs lenient)
//! - ✅ Plug in your own payload parser (GNSS, marine, custom protocols)
//! - ✅ Support both `&str` and `&[u8]` inputs
//! - ✅ Parse without allocations, built on top of [`nom`](https://github.com/Geal/nom),
//!   a parser combinator library in Rust.
//!
//! Perfect for:
//!
//! - GNSS/GPS receiver integration
//! - Marine electronics parsing
//! - IoT devices consuming NMEA 0183-style protocols
//! - Debugging or testing tools for embedded equipment
//! - Legacy formats that resemble NMEA but don’t strictly comply
//!
//! ## 📦 Key Features
//!
//! - ✅ ASCII-only validation
//! - ✅ Required or optional checksum validation
//! - ✅ Required or forbidden CRLF ending enforcement
//! - ✅ Zero-allocation parsing
//! - ✅ Built on `nom` combinators
//! - ✅ Fully pluggable content parser (you bring the domain logic)
//! - ✅ Optional built-in support for common NMEA sentences
//!
//! ---
//!
//! ## ⚡ Quick Start
//!
//! Here's a minimal example to get you started with parsing NMEA 0183-style sentences:
//!
//! ```rust
//! use nmea0183_parser::{ChecksumMode, IResult, LineEndingMode, Nmea0183ParserBuilder};
//! use nom::Parser;
//!
//! // Simple content parser that splits fields by comma
//! fn parse_fields(input: &str) -> IResult<&str, Vec<&str>> {
//!     Ok(("", input.split(',').collect()))
//! }
//!
//! // Create parser with strict validation (checksum + CRLF required)
//! let parser_factory = Nmea0183ParserBuilder::new()
//!     .checksum_mode(ChecksumMode::Required)
//!     .line_ending_mode(LineEndingMode::Required);
//!
//! let mut parser = parser_factory.build(parse_fields);
//!
//! // Parse a GPS sentence
//! let result =
//!     parser.parse("$GPGGA,123456.00,4916.29,N,12311.76,W,1,08,0.9,545.4,M,46.9,M,,*73\r\n");
//!
//! match result {
//!     Ok((_remaining, fields)) => {
//!         println!("Success! Parsed {} fields", fields.len()); // 15 fields
//!         println!("Sentence type: {}", fields[0]); // "GPGGA"
//!     }
//!     Err(e) => println!("Parse error: {:?}", e),
//! }
//! ```
//!
//! For custom parsing logic, you can define your own content parser. The `Nmea0183ParserBuilder`
//! creates a parser factory that you then call with your content parser:
//!
//! ```rust
//! use nmea0183_parser::{ChecksumMode, IResult, LineEndingMode, Nmea0183ParserBuilder};
//! use nom::Parser;
//!
//! // Your custom logic for the inner data portion (after "$" and before "*CC").
//! // The `parse_content` function should return an IResult<Input, Output> type
//! // so it can be used with the framing parser.
//! // The `Output` type can be any Rust type you define, such as a struct or an enum.
//! fn parse_content(input: &str) -> IResult<&str, Vec<&str>> {
//!     // You can decode fields here. In this example, we split the input by commas.
//!     Ok(("", input.split(',').collect()))
//! }
//!
//! // The `Nmea0183ParserBuilder` creates a parser factory that you then call
//! // with your content parser.
//! let parser_factory = Nmea0183ParserBuilder::new()
//!     .checksum_mode(ChecksumMode::Required)
//!     .line_ending_mode(LineEndingMode::Required);
//!
//! let mut parser = parser_factory.build(parse_content);
//!
//! // Or combine into one line:
//! // let mut parser = Nmea0183ParserBuilder::new()
//! //     .checksum_mode(ChecksumMode::Required)
//! //     .line_ending_mode(LineEndingMode::Required)
//! //     .build(parse_content);
//!
//! // Now you can use the parser to parse your NMEA sentences.
//! match parser.parse("$Header,field1,field2*3C\r\n") {
//!     Ok((remaining, fields)) => {
//!         assert_eq!(remaining, "");
//!         assert_eq!(fields, vec!["Header", "field1", "field2"]);
//!     }
//!     Err(e) => println!("Parse error: {:?}", e),
//! }
//! ```
//!
//! ## 🧐 How It Works
//!
//! 1. **Framing parser** handles the outer structure:
//!
//!    - ASCII-only validation
//!    - Start delimiter (`$`)
//!    - Optional checksum validation (`*CC`)
//!    - Optional CRLF endings (`\r\n`)
//!
//! 2. **Your content parser**, or built-in ones, handle the inner data (`D1,D2,...,Dn`):
//!
//!    - Field parsing and validation
//!    - Type conversion
//!    - Domain-specific logic
//!
//! You have full control over sentence content interpretation.
//!
//! In the above example, `parse_content` is your custom logic that processes the inner
//! data of the sentence. The `Nmea0183ParserBuilder` creates a parser that handles the
//! framing, while you focus on the content.
//!
//! ---
//!
//! ## 🔧 Configuration Options
//!
//! You can configure the parser's behavior using `ChecksumMode` and `LineEndingMode`:
//!
//! ```rust
//! use nmea0183_parser::{ChecksumMode, IResult, LineEndingMode, Nmea0183ParserBuilder};
//! use nom::Parser;
//!
//! fn content_parser(input: &str) -> IResult<&str, bool> {
//!     Ok((input, true))
//! }
//!
//! // Strict: checksum and CRLF both required
//! let mut strict_parser = Nmea0183ParserBuilder::new()
//!     .checksum_mode(ChecksumMode::Required)
//!     .line_ending_mode(LineEndingMode::Required)
//!     .build(content_parser);
//!
//! assert!(strict_parser.parse("$GPGGA,data*6A\r\n").is_ok());
//! assert!(strict_parser.parse("$GPGGA,data*6A").is_err()); // (missing CRLF)
//! assert!(strict_parser.parse("$GPGGA,data\r\n").is_err()); // (missing checksum)
//!
//! // Checksum required, no CRLF allowed
//! let mut no_crlf_parser = Nmea0183ParserBuilder::new()
//!     .checksum_mode(ChecksumMode::Required)
//!     .line_ending_mode(LineEndingMode::Forbidden)
//!     .build(content_parser);
//!
//! assert!(no_crlf_parser.parse("$GPGGA,data*6A").is_ok());
//! assert!(no_crlf_parser.parse("$GPGGA,data*6A\r\n").is_err()); // (CRLF present)
//! assert!(no_crlf_parser.parse("$GPGGA,data").is_err()); // (missing checksum)
//!
//! // Checksum optional, CRLF required
//! let mut optional_checksum_parser = Nmea0183ParserBuilder::new()
//!     .checksum_mode(ChecksumMode::Optional)
//!     .line_ending_mode(LineEndingMode::Required)
//!     .build(content_parser);
//!
//! assert!(optional_checksum_parser.parse("$GPGGA,data*6A\r\n").is_ok()); // (with valid checksum)
//! assert!(optional_checksum_parser.parse("$GPGGA,data\r\n").is_ok()); // (without checksum)
//! assert!(optional_checksum_parser.parse("$GPGGA,data*99\r\n").is_err()); // (invalid checksum)
//! assert!(optional_checksum_parser.parse("$GPGGA,data*6A").is_err()); // (missing CRLF)
//!
//! // Lenient: checksum optional, CRLF forbidden
//! let mut lenient_parser = Nmea0183ParserBuilder::new()
//!     .checksum_mode(ChecksumMode::Optional)
//!     .line_ending_mode(LineEndingMode::Forbidden)
//!     .build(content_parser);
//!
//! assert!(lenient_parser.parse("$GPGGA,data*6A").is_ok()); // (with valid checksum)
//! assert!(lenient_parser.parse("$GPGGA,data").is_ok()); // (without checksum)
//! assert!(lenient_parser.parse("$GPGGA,data*99").is_err()); // (invalid checksum)
//! assert!(lenient_parser.parse("$GPGGA,data\r\n").is_err()); // (CRLF present)
//! ```
//!
//! ---
//!
//! ## 🔍 Parsing Both String and Byte Inputs
//!
//! The parser can handle both `&str` and `&[u8]` inputs. You can define your content
//! parser for either type; the factory will adapt accordingly.
//!
//! ```rust
//! use nmea0183_parser::{ChecksumMode, IResult, LineEndingMode, Nmea0183ParserBuilder};
//! use nom::Parser;
//!
//! fn parse_content_str(input: &str) -> IResult<&str, Vec<&str>> {
//!     Ok(("", input.split(',').collect()))
//! }
//!
//! let mut parser_str = Nmea0183ParserBuilder::new()
//!     .checksum_mode(ChecksumMode::Required)
//!     .line_ending_mode(LineEndingMode::Required)
//!     .build(parse_content_str);
//!
//! // Parse from string
//! let string_input = "$Header,field1,field2*3C\r\n";
//! let result = parser_str.parse(string_input);
//!
//! assert!(result.is_ok());
//! assert_eq!(result.unwrap().1, vec!["Header", "field1", "field2"]);
//!
//! fn parse_content_bytes(input: &[u8]) -> IResult<&[u8], u8> {
//!     let (input, first_byte) = nom::number::complete::u8(input)?;
//!     Ok((input, first_byte))
//! }
//!
//! let mut parser_bytes = Nmea0183ParserBuilder::new()
//!     .checksum_mode(ChecksumMode::Required)
//!     .line_ending_mode(LineEndingMode::Required)
//!     .build(parse_content_bytes);
//!
//! // Parse from bytes
//! let byte_input = b"$Header,field1,field2*3C\r\n";
//! let result_bytes = parser_bytes.parse(byte_input);
//!
//! assert!(result_bytes.is_ok());
//! assert_eq!(result_bytes.unwrap().1, 72); // 'H' is the first byte of the content
//! ```
//!
//! ---
//!
//! ## 🧩 `NmeaParse` trait and `#[derive(NmeaParse)]` Macro
//!
//! The `NmeaParse` trait provides a generic interface for parsing values from NMEA 0183-style
//! content, supporting both primitive and composite types. Implementations are provided for
//! primitive types, `Option<T>`, `Vec<T>`, and more types, and you can implement this trait
//! for your own types to enable custom parsing logic.
//!
//! ### Implementing the `NmeaParse` Trait
//!
//! To implement the `NmeaParse` trait for your type, you need to provide a `parse` method that
//! takes an input and returns an `IResult` with the remaining input and the parsed value.
//!
//! NMEA 0183 fields are typically comma-separated. When parsing composite types (like structs),
//! you usually want to consume the separator before parsing each subsequent field. However, for
//! optional fields (`Option<T>`) or repeated fields (`Vec<T>`), always consuming the separator
//! can cause issues if the field is missing.
//!
//! To address this, the trait provides a `parse_preceded(separator)` method. This method ensures
//! the separator is only consumed if the field is present. By default, `parse_preceded` is
//! implemented as a simple wrapper around `preceded(separator, Self::parse)`, but you can override
//! it for custom behavior—such as the implementations for `Option<T>` and `Vec<T>`.
//!
//! This design gives you fine-grained control over field parsing and separator handling, making
//! it easy to implement robust NMEA content parsers for your own types.
//!
//! ### Deriving the `NmeaParse` Trait
//!
//! Based on [`nom-derive`] and with a lot of similarities, `NmeaParse` is a custom derive
//! attribute to derive content parsers for your NMEA 0183-style data structures.
//!
//! The `NmeaParse` derive macro automatically generates an implementation of the `NmeaParse` trait
//! for your structs and enums using `nom` parsers when possible. This allows you to define your
//! data structures and derive parsing logic without writing boilerplate code.
//!
//! For example, you can define a struct and derive the `NmeaParse` trait like this:
//!
//! ```rust
//! use nmea0183_parser::NmeaParse;
//!
//! #[derive(NmeaParse)]
//! struct Data {
//!     pub id: u8,
//!     pub value: f64,
//!     pub timestamp: u64,
//! }
//! ```
//!
//! This will generate an implementation of the `NmeaParse` trait for the `Data` struct,
//! allowing you to parse NMEA 0183-style input into instances of `Data`.
//! The generated code will look something like this (simplified):
//!
//! ```rust,ignore
//! impl NmeaParse for Data {
//!     fn parse(i: &'a str) -> nmea0183_parser::IResult<&'a str, Self, Error> {
//!         let (i, id) = <u8>::parse(i)?;
//!         let (i, value) = <f64>::parse_preceded(nom::character::complete::char(',')).parse(i)?;
//!         let (i, timestamp) = <u64>::parse_preceded(nom::character::complete::char(',')).parse(i)?;
//!
//!         Ok((i, Data { id, value, timestamp }))
//!     }
//! }
//! ```
//!
//! You can now parse an input containing NMEA 0183-style content into a `Data` struct:
//!
//! ```rust
//! # use nmea0183_parser::{IResult, NmeaParse};
//! # use nom::error::ParseError;
//! #
//! # #[derive(NmeaParse)]
//! # struct Data {
//! #     pub id: u8,
//! #     pub value: f64,
//! #     pub timestamp: u64,
//! # }
//! let input = "123,45.67,1622547800";
//! let result: IResult<_, _> = Data::parse(input);
//! let (remaining, data) = result.unwrap();
//! assert!(remaining.is_empty());
//! assert_eq!(data.id, 123);
//! assert_eq!(data.value, 45.67);
//! assert_eq!(data.timestamp, 1622547800);
//! ```
//!
//! The macro also supports enums, which require a `selector` attribute to determine which
//! variant to parse:
//!
//! ```rust
//! use nmea0183_parser::{Error, IResult, NmeaParse};
//!
//! # #[derive(Debug)]
//! #[derive(NmeaParse)]
//! #[nmea(selector(u8::parse))]
//! enum Data {
//!     #[nmea(selector(0))]
//!     TypeA { id: u8, value: u16 },
//!     #[nmea(selector(1))]
//!     TypeB { values: [u8; 4] },
//! }
//!
//! let input = "0,42,100";
//! let result: IResult<_, _> = Data::parse(input);
//! assert!(matches!(result, Ok((_, Data::TypeA { id: 42, value: 100 }))));
//!
//! let input = "1,2,3,4,5";
//! let result: IResult<_, _> = Data::parse(input);
//! assert!(matches!(result, Ok((_, Data::TypeB { values: [2, 3, 4, 5] }))));
//!
//! let input = "2,42";
//! // Expecting an error because no variant matches selector 2
//! let error = Data::parse(input).unwrap_err();
//! assert!(matches!(error,
//!     nom::Err::Error(Error::ParsingError(nom::error::Error {
//!         code: nom::error::ErrorKind::Switch,
//!         ..
//!     }))
//! ));
//! ```
//!
//! You can use the `#[derive(NmeaParse)]` attribute on your structs and enums to automatically
//! generate parsing logic based on the field types. The macro will try to infer parsers for
//! known types (implementors of the `NmeaParse` trait), but you can also customize the parsing
//! behavior using attributes.
//!
//! For more details on how to use the `NmeaParse` derive macro and customize parsing behavior,
//! refer to the [documentation](https://docs.rs/nmea0183-parser/latest/nmea0183-parser/derive.NmeaParse.html).
//!
//! ---
//!
//! ## 🧱 Built-in NMEA Sentence Content Parser
//!
//! Alongside the flexible framing parser, this crate can provide a built-in `NmeaSentence`
//! content parser for common NMEA 0183 sentence types. To use it, enable the `nmea-content`
//! feature in your `Cargo.toml`.
//!
//! This parser uses the `NmeaParse` trait to provide content-only parsing. It does not handle
//! framing — such as the initial `$`, optional checksum (`*CC`), or optional CRLF (`\r\n`).
//! That responsibility belongs to the framing parser, which wraps around the content parser.
//!
//! To parse a complete NMEA sentence, you can use the `Nmea0183ParserBuilder` with the built-in
//! content parser:
//!
//! ```rust
//! use nmea0183_parser::{
//!     IResult, Nmea0183ParserBuilder, NmeaParse,
//!     nmea_content::{GGA, Location, NmeaSentence, Quality},
//! };
//! use nom::Parser;
//!
//! // Defaults to strict parsing with both checksum and CRLF required
//! let mut nmea_parser = Nmea0183ParserBuilder::new().build(NmeaSentence::parse);
//!
//! let result: IResult<_, _> =
//!     nmea_parser.parse("$GPGGA,123456.00,4916.29,N,12311.76,W,1,08,0.9,545.4,M,46.9,M,,*73\r\n");
//!
//! assert!(
//!     result.is_ok(),
//!     "Failed to parse NMEA sentence: {:?}",
//!     result.unwrap_err()
//! );
//!
//! let (_, sentence) = result.unwrap();
//! assert!(matches!(
//!     sentence,
//!     NmeaSentence::GGA(GGA {
//!         location: Some(Location {
//!             latitude: 49.2715,
//!             longitude: -123.196,
//!         }),
//!         fix_quality: Quality::GPSFix,
//!         satellite_count: Some(8),
//!         hdop: Some(0.9),
//!         ..
//!     })
//! ));
//! ```
//!
//! > **Note:** While the `Nmea0183ParserBuilder` framing parser can accept both `&str` and `&[u8]`
//! > inputs, the built-in content parser only accepts `&str`, as it is designed specifically for
//! > text-based NMEA sentences.
//!
//! ### Supported NMEA Sentences
//!
//! - [`DBT`](https://gpsd.gitlab.io/gpsd/NMEA.html#_dbt_depth_below_transducer) - Depth Below Transducer
//! - [`DPT`](https://gpsd.gitlab.io/gpsd/NMEA.html#_dpt_depth_of_water) - Depth of Water
//! - [`GGA`](https://gpsd.gitlab.io/gpsd/NMEA.html#_gga_global_positioning_system_fix_data) - Global Positioning System Fix Data
//! - [`GLL`](https://gpsd.gitlab.io/gpsd/NMEA.html#_gll_geographic_position_latitudelongitude) - Geographic Position: Latitude/Longitude
//! - [`GSA`](https://gpsd.gitlab.io/gpsd/NMEA.html#_gsa_gps_dop_and_active_satellites) - GPS DOP and Active Satellites
//! - [`GSV`](https://gpsd.gitlab.io/gpsd/NMEA.html#_gsv_satellites_in_view) - Satellites in View
//! - [`RMC`](https://gpsd.gitlab.io/gpsd/NMEA.html#_rmc_recommended_minimum_navigation_information) - Recommended Minimum Navigation Information
//! - [`VTG`](https://gpsd.gitlab.io/gpsd/NMEA.html#_vtg_track_made_good_and_ground_speed) - Track made good and Ground speed
//! - [`ZDA`](https://gpsd.gitlab.io/gpsd/NMEA.html#_zda_time_date_utc_day_month_year_and_local_time_zone) - Time & Date: UTC, day, month, year and local time zone
//!
//! ### NMEA Version Support
//!
//! Different NMEA versions may include additional fields in certain sentence types.
//! You can choose the version that matches your equipment by enabling the appropriate feature flags.
//!
//! | Feature Flag   | NMEA Version | When to Use                |
//! | -------------- | ------------ | -------------------------- |
//! | `nmea-content` | Pre-2.3      | Standard NMEA parsing      |
//! | `nmea-v2-3`    | NMEA 2.3     | Older GPS/marine equipment |
//! | `nmea-v3-0`    | NMEA 3.0     | Mid-range equipment        |
//! | `nmea-v4-11`   | NMEA 4.11    | Modern equipment           |
//!
//! For specific field differences between versions, please refer to the
//! [NMEA 0183 standard documentation](https://gpsd.gitlab.io/gpsd/NMEA.html).

#![cfg_attr(docsrs, feature(doc_cfg))]

mod error;
mod nmea0183;
#[cfg(feature = "nmea-content")]
#[cfg_attr(docsrs, doc(cfg(feature = "nmea-content")))]
pub mod nmea_content;
mod parse;

pub use error::{Error, IResult};
pub use nmea0183::{ChecksumMode, LineEndingMode, Nmea0183ParserBuilder};
#[cfg(feature = "derive")]
#[cfg_attr(docsrs, doc(cfg(feature = "derive")))]
pub use nmea0183_derive::NmeaParse;
pub use parse::NmeaParse;
