//! # Parsing Utilities
//!
//! This module provides utility parsers for validating input length and ensuring
//! complete consumption of input data.

use nom::{
    Err, Input, Mode, OutputMode, PResult, Parser, ToUsize,
    error::{ErrorKind, ParseError},
};

/// Verifies that the remaining input length matches the expected length after parsing.
///
/// This combinator runs the provided parser and then checks if the remaining input
/// has exactly `n` bytes/characters left. This is useful for ensuring complete
/// consumption of input or validating that a specific amount of data remains.
///
/// # Arguments
///
/// * `f` - The parser to run
/// * `n` - Expected remaining input length after parsing
/// * `e` - Error kind to return if length verification fails
///
/// # Examples
///
/// ```rust
/// use nmea0183_parser::parsing::verify_rest_length;
/// use nom::{IResult, Parser, bytes::complete::take, error::ErrorKind};
///
/// // Parse 3 bytes and verify nothing remains
/// let mut parser = verify_rest_length(take(2u8), 1u8, ErrorKind::Count);
/// let result: IResult<_, _> = parser.parse("abc");
/// assert!(result.is_ok());
///
/// // This would fail because 2 byte remains
/// let result = parser.parse("abcd");
/// assert!(result.is_err());
/// ```
pub fn verify_rest_length<I, N, E: ParseError<I>, F>(
    f: F,
    n: N,
    e: ErrorKind,
) -> impl Parser<I, Output = <F as Parser<I>>::Output, Error = E>
where
    I: Input,
    N: ToUsize,
    F: Parser<I, Error = E>,
{
    VerifyRestLength {
        f,
        n: n.to_usize(),
        e,
    }
}

/// Ensures that the parser consumes all input (shorthand for `verify_rest_length(f, 0, e)`).
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
/// ```rust
/// use nmea0183_parser::parsing::consumed;
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
pub fn consumed<I, E: ParseError<I>, F>(
    f: F,
    e: ErrorKind,
) -> impl Parser<I, Output = <F as Parser<I>>::Output, Error = E>
where
    I: Input,
    F: Parser<I, Error = E>,
{
    VerifyRestLength { f, n: 0, e }
}

struct VerifyRestLength<F> {
    f: F,
    n: usize,
    e: ErrorKind,
}

impl<I, F> Parser<I> for VerifyRestLength<F>
where
    I: Input,
    F: Parser<I>,
{
    type Output = <F as Parser<I>>::Output;
    type Error = <F as Parser<I>>::Error;

    fn process<OM: OutputMode>(&mut self, i: I) -> PResult<OM, I, Self::Output, Self::Error> {
        let (i, o) = self.f.process::<OM>(i)?;

        if i.input_len() != self.n {
            return Err(Err::Error(OM::Error::bind(|| {
                <F as Parser<I>>::Error::from_error_kind(i, self.e)
            })));
        }

        Ok((i, o))
    }
}
