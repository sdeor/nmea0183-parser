//! # NMEA 0183 Message Parser
//!
//! This module provides the main parsing functionality for NMEA 0183-style messages.
//! It handles the standard NMEA 0183 format: `$HHH,D1,D2,...,Dn*CC\r\n`
//!
//! The parser is configurable to handle variations in:
//! - Checksum requirements (required or optional)
//! - Line ending requirements (CRLF required or forbidden)

use nom::{
    AsBytes, AsChar, Compare, Err, FindSubstring, Input, Parser,
    branch::alt,
    bytes::complete::{tag, take, take_until},
    character::complete::{char, hex_digit0},
    combinator::{opt, rest, rest_len, verify},
    error::{ErrorKind, ParseError},
    number::complete::hex_u32,
    sequence::terminated,
};

use crate::{Error, IResult};

/// Defines how the parser should handle NMEA message checksums.
///
/// NMEA 0183 messages can include an optional checksum in the format `*CC` where
/// CC is a two-digit hexadecimal value representing the XOR of all bytes in the
/// message content (excluding the '$' prefix and '*' delimiter).
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum ChecksumMode {
    #[default]
    /// Checksum is required and must be present.
    ///
    /// The parser will fail if no `*CC` checksum is found at the end of the message.
    /// If a checksum is present, it will be validated against the calculated checksum.
    ///
    /// Use this mode for strict NMEA 0183 compliance or when data integrity is critical.
    Required,

    /// Checksum is optional but will be validated if present.
    ///
    /// The parser will accept messages both with and without checksums:
    /// - If no checksum is present (`*CC` missing), parsing continues normally
    /// - If a checksum is present, it must be valid or parsing will fail
    ///
    /// Use this mode when working with mixed message sources or legacy equipment
    /// that may not always include checksums.
    Optional,
}

/// Defines how the parser should handle CRLF line endings.
///
/// NMEA 0183 messages typically end with a carriage return and line feed (`\r\n`),
/// but some systems or applications may omit these characters.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum LineEndingMode {
    #[default]
    /// CRLF line ending is required and must be present.
    ///
    /// The parser will fail if the message does not end with `\r\n`.
    /// This is the standard NMEA 0183 format for messages transmitted over
    /// serial connections or stored in files.
    ///
    /// Use this mode when parsing standard NMEA log files or serial port data.
    Required,

    /// CRLF line ending is forbidden and must not be present.
    ///
    /// The parser will fail if the message ends with `\r\n`.
    /// This mode is useful when parsing NMEA messages that have been processed
    /// or transmitted through systems that strip line endings.
    ///
    /// Use this mode when parsing messages from APIs, databases, or other
    /// sources where line endings have been removed.
    Forbidden,
}

/// Creates a configurable NMEA 0183-style parser factory.
///
/// This struct allows you to configure the NMEA 0183 framing parser with different
/// checksum and line ending modes before building the final parser.
///
/// It uses the builder pattern to allow for flexible configuration of the parser settings.
///
/// # Examples
///
/// ```rust
/// use nmea0183_parser::{IResult, Nmea0183ParserBuilder};
///
/// fn content_parser(input: &str) -> IResult<&str, Vec<&str>> {
///     Ok(("", input.split(',').collect()))
/// }
///
/// // Create a parser with required checksum and CRLF
/// let parser_factory = Nmea0183ParserBuilder::new();
/// let mut parser = parser_factory.build(content_parser);
/// ```
///
/// ## Configuration
///
/// ```rust
/// use nmea0183_parser::{ChecksumMode, IResult, LineEndingMode, Nmea0183ParserBuilder};
/// use nom::Parser;
///
/// fn content_parser(i: &str) -> IResult<&str, bool> {
///     Ok((i, true))
/// }
///
/// // Strict: checksum and CRLF both required
/// let mut strict_parser = Nmea0183ParserBuilder::new()
///     .checksum_mode(ChecksumMode::Required)
///     .line_ending_mode(LineEndingMode::Required)
///     .build(content_parser);
/// assert!(strict_parser.parse("$GPGGA,data*6A\r\n").is_ok());
/// assert!(strict_parser.parse("$GPGGA,data*6A").is_err()); // (missing CRLF)
/// assert!(strict_parser.parse("$GPGGA,data\r\n").is_err()); // (missing checksum)
///
/// // Checksum required, no CRLF allowed
/// let mut no_crlf_parser = Nmea0183ParserBuilder::new()
///     .checksum_mode(ChecksumMode::Required)
///     .line_ending_mode(LineEndingMode::Forbidden)
///     .build(content_parser);
/// assert!(no_crlf_parser.parse("$GPGGA,data*6A").is_ok());
/// assert!(no_crlf_parser.parse("$GPGGA,data*6A\r\n").is_err()); // (CRLF present)
/// assert!(no_crlf_parser.parse("$GPGGA,data").is_err()); // (missing checksum)
///
/// // Checksum optional, CRLF required
/// let mut optional_checksum_parser = Nmea0183ParserBuilder::new()
///     .checksum_mode(ChecksumMode::Optional)
///     .line_ending_mode(LineEndingMode::Required)
///     .build(content_parser);
/// assert!(optional_checksum_parser.parse("$GPGGA,data*6A\r\n").is_ok()); // (with valid checksum)
/// assert!(optional_checksum_parser.parse("$GPGGA,data\r\n").is_ok()); // (without checksum)
/// assert!(optional_checksum_parser.parse("$GPGGA,data*99\r\n").is_err()); // (invalid checksum)
/// assert!(optional_checksum_parser.parse("$GPGGA,data*6A").is_err()); // (missing CRLF)
///
/// // Lenient: checksum optional, CRLF forbidden
/// let mut lenient_parser = Nmea0183ParserBuilder::new()
///     .checksum_mode(ChecksumMode::Optional)
///     .line_ending_mode(LineEndingMode::Forbidden)
///     .build(content_parser);
/// assert!(lenient_parser.parse("$GPGGA,data*6A").is_ok()); // (with valid checksum)
/// assert!(lenient_parser.parse("$GPGGA,data").is_ok()); // (without checksum)
/// assert!(lenient_parser.parse("$GPGGA,data*99").is_err()); // (invalid checksum)
/// assert!(lenient_parser.parse("$GPGGA,data\r\n").is_err()); // (CRLF present)
/// ```
#[must_use]
pub struct Nmea0183ParserBuilder {
    /// Checksum mode for the parser.
    checksum_mode: ChecksumMode,

    /// Line ending mode for the parser.
    line_ending_mode: LineEndingMode,
}

impl Nmea0183ParserBuilder {
    /// Creates a new NMEA 0183 parser builder with default settings.
    ///
    /// The default settings are:
    /// - Checksum mode: [`ChecksumMode::Required`]
    /// - Line ending mode: [`LineEndingMode::Required`]
    pub fn new() -> Self {
        Nmea0183ParserBuilder {
            checksum_mode: ChecksumMode::Required,
            line_ending_mode: LineEndingMode::Required,
        }
    }

    /// Sets the checksum mode for the parser.
    ///
    /// # Arguments
    ///
    /// * `mode` - The desired checksum mode:
    ///   - [`ChecksumMode::Required`]: Checksum must be present and valid
    ///   - [`ChecksumMode::Optional`]: Checksum may be absent or must be valid if present
    pub fn checksum_mode(mut self, mode: ChecksumMode) -> Self {
        self.checksum_mode = mode;
        self
    }

    /// Sets the line ending mode for the parser.
    ///
    /// # Arguments
    ///
    /// * `mode` - The desired line ending mode:
    ///   - [`LineEndingMode::Required`]: Message must end with `\r\n`
    ///   - [`LineEndingMode::Forbidden`]: Message must not end with `\r\n`
    pub fn line_ending_mode(mut self, mode: LineEndingMode) -> Self {
        self.line_ending_mode = mode;
        self
    }

    /// Builds the NMEA 0183-style parser with the configured settings.
    ///
    /// This method takes a user-provided parser function that will handle the
    /// content of the message after the framing has been processed.
    ///
    /// The returned parser will:
    /// * Validate that the input is ASCII-only
    /// * Expect the message to start with `$`
    /// * Extract the message content (everything before `*CC` or `\r\n`)
    /// * Parse and validate the checksum using the provided checksum parser
    /// * Call the user-provided parser on the message content
    ///
    /// # Arguments
    ///
    /// * `content_parser` - User-provided parser for the message content.
    ///
    /// # Returns
    ///
    /// A parser function that takes an input and returns a result containing the parsed content
    /// or an error if the input does not conform to the expected NMEA 0183 format.
    pub fn build<'a, I, O, F, E>(self, mut content_parser: F) -> impl FnMut(I) -> IResult<I, O, E>
    where
        I: Input + AsBytes + Compare<&'a str> + FindSubstring<&'a str>,
        <I as Input>::Item: AsChar,
        F: Parser<I, Output = O, Error = Error<I, E>>,
        E: ParseError<I>,
    {
        move |i: I| {
            if !i.as_bytes().is_ascii() {
                return Err(nom::Err::Error(Error::NonAscii));
            }

            let (i, _) = char('$').parse(i)?;
            let (cc, data) = alt((take_until("*"), take_until("\r\n"), rest)).parse(i)?;
            let (_, cc) = checksum_crlf(self.checksum_mode, self.line_ending_mode).parse(cc)?;
            let (data, calc_cc) = checksum(data);

            if let Some(cc) = cc
                && cc != calc_cc
            {
                return Err(nom::Err::Error(Error::ChecksumMismatch {
                    expected: calc_cc,
                    found: cc,
                }));
            }

            content_parser.parse(data)
        }
    }
}

impl Default for Nmea0183ParserBuilder {
    fn default() -> Self {
        Nmea0183ParserBuilder::new()
    }
}

/// Creates a parser for checksum and CRLF based on configuration.
///
/// This function returns a parser that can handle the end portion of NMEA messages,
/// specifically the checksum (if present) and line ending (if present).
///
/// # Arguments
///
/// * `cc` - Checksum requirement:
///   - [`ChecksumMode::Required`]: Parser will fail if no '*CC' is present
///   - [`ChecksumMode::Optional`]: Parser accepts messages with or without '*CC',
///     but validates checksum if present
/// * `crlf` - CRLF requirement:
///   - [`LineEndingMode::Required`]: Parser will fail if message doesn't end with `\r\n`
///   - [`LineEndingMode::Forbidden`]: Parser will fail if message ends with `\r\n`
///
/// # Returns
///
/// A parser that extracts the checksum value ([`None`] if no checksum present).
///
/// # Message Format Expectations
///
/// - cc=[`ChecksumMode::Required`], crlf=[`LineEndingMode::Required`]: Expects `*CC\r\n`
/// - cc=[`ChecksumMode::Required`], crlf=[`LineEndingMode::Forbidden`]: Expects `*CC`
/// - cc=[`ChecksumMode::Optional`], crlf=[`LineEndingMode::Required`]: Expects `\r\n` or `*CC\r\n`
/// - cc=[`ChecksumMode::Optional`], crlf=[`LineEndingMode::Forbidden`]: Expects nothing or `*CC`
///
/// # Examples
///
/// ```rust,ignore
/// use nmea0183_parser::{ChecksumMode, IResult, LineEndingMode, checksum_crlf};
/// use nom::Parser;
///
/// // Required checksum, required CRLF
/// let mut parser = checksum_crlf(ChecksumMode::Required, LineEndingMode::Required);
/// let result: IResult<_, _> = parser.parse("*51\r\n");
/// assert_eq!(result, Ok(("", Some(0x51))));
///
/// // Optional checksum, forbidden CRLF
/// let mut parser = checksum_crlf(ChecksumMode::Optional, LineEndingMode::Forbidden);
/// let result1: IResult<_, _> = parser.parse("*51");     // With checksum
/// let result2: IResult<_, _> = parser.parse("");        // Without checksum
/// assert!(result1.is_ok());
/// assert!(result2.is_ok());
/// ```
fn checksum_crlf<'a, I, E: ParseError<I>>(
    cc: ChecksumMode,
    le: LineEndingMode,
) -> impl FnMut(I) -> nom::IResult<I, Option<u8>, E>
where
    I: Input + AsBytes + Compare<&'a str> + FindSubstring<&'a str>,
    <I as Input>::Item: AsChar,
{
    move |i: I| {
        let (i, _) = crlf(le).parse(i)?;

        let (cc, parse_cc) = match cc {
            ChecksumMode::Required => char('*').map(|_| true).parse(i)?,
            ChecksumMode::Optional => opt(char('*')).map(|asterisk| asterisk.is_some()).parse(i)?,
        };

        if parse_cc {
            let (_, cc) = consumed(take(2u8), ErrorKind::Count).parse(cc)?;
            let (_, cc) = consumed(hex_digit0, ErrorKind::IsA).parse(cc)?;

            hex_u32.map(|cc| Some(cc as u8)).parse(cc)
        } else if cc.input_len() != 0 {
            Err(Err::Error(E::from_error_kind(cc, ErrorKind::Count)))
        } else {
            Ok((cc, None))
        }
    }
}

/// Parses CRLF line endings based on configuration.
///
/// This function handles the parsing of carriage return and line feed characters
/// at the end of NMEA messages, with support for both required and forbidden modes.
///
/// # Arguments
///
/// * `crlf` - CRLF requirement:
///   - [`LineEndingMode::Required`]: Parser will fail if message doesn't end with `\r\n`
///   - [`LineEndingMode::Forbidden`]: Parser will fail if message ends with `\r\n`
///
/// # Returns
///
/// A parser function that validates CRLF presence according to the configuration.
///
/// # Examples
///
/// ```rust,ignore
/// use nmea0183_parser::{IResult, LineEndingMode, crlf};
/// use nom::Parser;
///
/// // CRLF required
/// let mut parser = crlf(LineEndingMode::Required);
/// let result: IResult<_, _> = parser.parse("data\r\n");
/// assert_eq!(result, Ok(("data", ())));
///
/// // CRLF forbidden
/// let mut parser = crlf(LineEndingMode::Forbidden);
/// let result: IResult<_, _> = parser.parse("data");
/// assert_eq!(result, Ok(("data", ())));
/// ```
fn crlf<'a, I, E: ParseError<I>>(crlf: LineEndingMode) -> impl Fn(I) -> nom::IResult<I, (), E>
where
    I: Input + Compare<&'a str> + FindSubstring<&'a str>,
{
    move |i: I| {
        let (i, data) = opt(take_until("\r\n")).parse(i)?;

        let data = if crlf == LineEndingMode::Required {
            match data {
                Some(data) => {
                    let (_, _) = consumed(tag("\r\n"), ErrorKind::CrLf).parse(i)?;
                    data
                }
                None => {
                    return Err(Err::Error(E::from_error_kind(i, ErrorKind::CrLf)));
                }
            }
        } else if data.is_some() {
            return Err(Err::Error(E::from_error_kind(i, ErrorKind::CrLf)));
        } else {
            i
        };

        Ok((data, ()))
    }
}

/// Calculates the NMEA 0183 checksum for the given message content.
///
/// The NMEA 0183 checksum is calculated by performing an XOR (exclusive OR) operation
/// on all bytes in the message content. This includes everything between the '$' prefix
/// and the '*' checksum delimiter, but excludes both the '$' and '*' characters themselves.
///
/// # Algorithm
///
/// 1. Initialize checksum to 0
/// 2. For each byte in the message content:
///    - XOR the current checksum with the byte value
/// 3. The final result is an 8-bit value (0-255)
///
/// # Arguments
///
/// * `input` - The message content to calculate checksum for (without '$' prefix or '*' delimiter)
///
/// # Returns
///
/// A tuple of (input, checksum) where:
/// - `input` is returned unchanged (zero-copy)
/// - `checksum` is the calculated XOR value as a u8
///
/// # NMEA 0183 Standard
///
/// According to the NMEA 0183 standard:
/// - The checksum is represented as a two-digit hexadecimal number
/// - It appears after the '*' character at the end of the sentence
/// - Example: `$GPGGA,123456,data*41` where '41' is the hex representation of the checksum
///
/// # Performance Notes
///
/// This function uses `fold()` with XOR operation, which is:
/// - Efficient for small to medium message sizes (typical NMEA messages are < 100 bytes)
/// - Single-pass algorithm with O(n) time complexity
/// - No memory allocation (zero-copy input handling)
fn checksum<I>(input: I) -> (I, u8)
where
    I: Input + AsBytes,
{
    let calculated_checksum = input
        .as_bytes()
        .iter()
        .fold(0u8, |accumulated_xor, &byte| accumulated_xor ^ byte);

    (input, calculated_checksum)
}

/// Ensures that the parser consumes all input.
///
/// This is a convenience function for the common case of wanting to ensure that
/// a parser consumes the entire input with no remainder.
///
/// # Arguments
///
/// * `f` - The parser to run
/// * `e` - Error kind to return if input is not fully consumed
///
/// # Examples
///
/// ```ignore
/// use nmea0183_parser::nmea0183::consumed;
/// use nom::{IResult, Parser, bytes::complete::take, error::ErrorKind};
///
/// // Parse all 3 bytes
/// let mut parser = consumed(take(3u8), ErrorKind::Count);
/// let result: IResult<_, _> = parser.parse("abc");
/// assert!(result.is_ok());
///
/// // This would fail because not all input is consumed
/// let result = parser.parse("abcd");
/// assert!(result.is_err());
/// ```
fn consumed<I, E: ParseError<I>, F>(
    f: F,
    e: ErrorKind,
) -> impl Parser<I, Output = <F as Parser<I>>::Output, Error = E>
where
    I: Input,
    F: Parser<I, Error = E>,
{
    terminated(
        f,
        verify(rest_len, |len| len == &0)
            .or(move |i| Err(Err::Error(nom::error::make_error(i, e)))),
    )
}

#[cfg(test)]
mod tests {
    mod cc_crlf00;
    mod cc_crlf01;
    mod cc_crlf10;
    mod cc_crlf11;
    mod crlf;
}
