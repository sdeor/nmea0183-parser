//! # Error Types
//!
//! This module defines the error types used throughout the NMEA parsing library.

use nom::error::{ErrorKind, ParseError};
use std::fmt::Debug;

/// Represents all possible errors that can occur during NMEA message parsing.
///
/// This enum covers various failure modes including input validation,
/// checksum verification, and parsing errors.
#[derive(Debug, PartialEq)]
pub enum Error<I, E> {
    /// The provided input contains non-ASCII characters.
    ///
    /// NMEA messages must be ASCII-only for proper parsing and checksum calculation.
    NonAscii,

    /// The checksum of the sentence was corrupt or incorrect.
    ///
    /// Contains both the expected checksum (calculated from the message content)
    /// and the actual checksum found in the message.
    ChecksumMismatch {
        /// The checksum calculated from the message content
        expected: u8,
        /// The checksum found in the message
        found: u8,
    },

    /// The sentence could not be parsed because its format was invalid.
    ///
    /// This wraps nom's standard parsing errors and provides context about
    /// what went wrong during parsing.
    ParsingError(E),

    /// A parameter was invalid due to length inconsistency.
    ///
    /// This occurs when a field has an unexpected length, such as a checksum
    /// that should be exactly 2 hex digits but has a different length.
    ParameterLength {
        /// The expected length
        expected: usize,
        /// The actual length found
        found: usize,
    },

    /// The message type is not recognized by the parser.
    ///
    /// This variant is used when a valid NMEA sentence is encountered, but the
    /// parser does not implement handling for this specific message type.
    /// The message type that caused the error is provided for reference.
    UnrecognizedMessage(I),

    /// An unknown error occurred.
    ///
    /// This is a catch-all for unexpected error conditions.
    Unknown,
}

impl<I, E> ParseError<I> for Error<I, E>
where
    E: ParseError<I>,
{
    fn from_error_kind(input: I, kind: ErrorKind) -> Self {
        Error::ParsingError(E::from_error_kind(input, kind))
    }

    fn append(_: I, _: ErrorKind, other: Self) -> Self {
        other
    }
}
