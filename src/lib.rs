//! # NMEA 0183 Parser
//!
//! This library provides functionality for parsing NMEA-like messages with the format:
//! `$HHH,D1,D2,...,Dn*CC\r\n`
//!
//! The parser is configurable to handle:
//! - Required or optional checksum validation
//! - Required or forbidden CRLF line endings
//! - Custom parsing logic for message content
//!
//! ## Usage
//!
//! ```rust
//! use nmea0183_parser::{ChecksumMode, IResult, LineEndingMode, nmea0183};
//! use nom::Parser;
//!
//! fn parse_content(i: &str) -> IResult<&str, bool> {
//!     // Your custom parsing logic here
//!     Ok((i, true))
//! }
//!
//! fn main() {
//!     let parser_factory = nmea0183(ChecksumMode::Required, LineEndingMode::Required);
//!     let mut parser = parser_factory(parse_content);
//!     
//!     let result = parser.parse("$GPGGA,123456,data*41\r\n");
//!     // Handle result...
//! }
//! ```

pub mod error;
mod nmea0183;
pub mod parsing;

pub use error::Error;
pub use nmea0183::*;

#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct README;

#[cfg(test)]
mod tests {
    mod cc_crlf00;
    mod cc_crlf01;
    mod cc_crlf10;
    mod cc_crlf11;
    mod crlf;
}
