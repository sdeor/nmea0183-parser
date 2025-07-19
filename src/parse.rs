use nom::{
    AsBytes, AsChar, Compare, Input, Offset, ParseTo, Parser,
    character::complete::{anychar, char},
    combinator::opt,
    error::ParseError,
    multi::many0,
    sequence::preceded,
};

use crate::{Error, IResult};

/// Trait for parsing types from NMEA 0183 sentence fields.
///
/// The `NmeaParse` trait provides a generic interface for parsing values from NMEA 0183-style
/// content, supporting both primitive and composite types. Implementations are provided for
/// primitive types, `Option<T>`, `Vec<T>`, and more types, and you can implement this trait
/// for your own types to enable custom parsing logic.
///
/// # Examples
///
/// ```rust
/// use nmea0183_parser::{NmeaParse, IResult};
///
/// // Parsing a single integer field
/// let input = "42";
/// let result: IResult<_, _> = u8::parse(input);
/// assert_eq!(result, Ok(("", 42)));
///
/// // Parsing an optional field (empty string yields None)
/// let input = "";
/// let result: IResult<_, _> = Option::<u8>::parse(input);
/// assert_eq!(result, Ok(("", None)));
///
/// // Parsing a comma-separated list of integers
/// let input = "1,2,3";
/// let result: IResult<_, _> = Vec::<u8>::parse(input);
/// assert_eq!(result, Ok(("", vec![1, 2, 3])));
/// ```
///
/// # Implementing for Custom Types
///
/// To support custom types, implement `NmeaParse` and define how your type should be parsed from
/// the input. You can compose existing parsers for fields within your type.
///
/// ```rust
/// use nmea0183_parser::{IResult, NmeaParse};
/// use nom::{AsChar, Input, Parser, character::complete::char, error::ParseError};
///
/// struct MyData {
///     a: u8,
///     b: Option<u32>,
/// }
///
/// impl<I, E> NmeaParse<I, E> for MyData
/// where
///     I: Input,
///     <I as Input>::Item: AsChar,
///     E: ParseError<I>,
/// {
///     fn parse(i: I) -> IResult<I, Self, E> {
///         let (i, a) = u8::parse(i)?;
///         let (i, _) = char(',').parse(i)?;
///         let (i, b) = Option::<u32>::parse(i)?;
///
///         Ok((i, MyData { a, b }))
///     }
/// }
/// ```
///
/// This trait provides another method `parse_preceded` that allows you to define how to
/// parse a value that is preceded by a separator, such as a comma in NMEA sentences.
/// This is useful for parsing fields that are separated by specific characters.
///
/// When possible, prefer using `parse_preceded` to handle separators, as it usually provides
/// a cleaner and more efficient way (especially for lists or optional fields) to parse values
/// that are separated by a specific separator.
///
/// The above example could also be implemented using `parse_preceded` to handle the separator:
///
/// ```rust
/// use nmea0183_parser::{IResult, NmeaParse};
/// use nom::{AsChar, Input, Parser, character::complete::char, error::ParseError};
///
/// struct MyData {
///     a: u8,
///     b: Option<u32>,
/// }
///
/// impl<I, E> NmeaParse<I, E> for MyData
/// where
///     I: Input,
///     <I as Input>::Item: AsChar,
///     E: ParseError<I>,
/// {
///     fn parse(i: I) -> IResult<I, Self, E> {
///         let (i, a) = u8::parse(i)?;
///         let (i, b) = Option::<u32>::parse_preceded(char(',')).parse(i)?;
///
///         Ok((i, MyData { a, b }))
///     }
/// }
/// ```
pub trait NmeaParse<I, E = nom::error::Error<I>>
where
    I: Input,
    E: ParseError<I>,
    Self: Sized,
{
    /// Parses the input and returns a result.
    ///
    /// # Arguments
    ///
    /// * `input` - The input to parse into `Self`.
    ///
    /// # Returns
    ///
    /// Returns an [`IResult`] containing:
    /// - On success: A tuple of `(remaining_input, parsed_value)`, where `remaining_input`
    ///   is the unparsed portion of the input and `parsed_value` is the successfully parsed
    ///   instance of `Self`.
    /// - On failure: An [`Error`] indicating the parsing error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[cfg(feature = "nmea-content")] {
    /// use nmea0183_parser::{IResult, NmeaParse, nmea_content::NmeaSentence};
    ///
    /// // Parse complete sentence content (including talker ID and sentence type)
    /// let content = "GPGGA,123456.00,4916.29,N,12311.76,W,1,08,0.9,545.4,M,46.9,M,,";
    /// let result: IResult<_, _> = NmeaSentence::parse(content);
    /// assert!(result.is_ok());
    /// # }
    /// ```
    fn parse(i: I) -> IResult<I, Self, E>;

    /// Returns a parser that first consumes a separator, then parses the value.
    /// This is useful for parsing fields that are separated by a specific character,
    /// such as a comma in NMEA sentences.
    ///
    /// # Arguments
    ///
    /// * `separator` - A parser that matches the separator character(s) before the value.
    ///
    /// # Returns
    ///
    /// Returns a parser that consumes the separator and then parses the value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nmea0183_parser::{NmeaParse, IResult};
    /// use nom::{Parser, character::complete::char};
    ///
    /// let result: IResult<_, _> = u8::parse_preceded(char(',')).parse(",42");
    /// assert_eq!(result, Ok(("", 42)));
    /// ```
    fn parse_preceded<S>(separator: S) -> impl Parser<I, Output = Self, Error = Error<I, E>>
    where
        S: Parser<I, Error = Error<I, E>>,
    {
        preceded(separator, Self::parse)
    }
}

macro_rules! impl_uints_type {
    ($($t:tt),*) => ($(
        impl<I, E> NmeaParse<I, E> for $t
        where
            I: Input,
            <I as Input>::Item: AsChar,
            E: ParseError<I>,
        {
            fn parse(i: I) -> IResult<I, Self, E> {
                nom::character::complete::$t.parse(i)
            }
        }
    )*)
}

macro_rules! impl_ints_type {
    ($($t:tt),*) => ($(
        impl<I, E> NmeaParse<I, E> for $t
        where
            I: Input + for<'a> Compare<&'a [u8]>,
            <I as Input>::Item: AsChar,
            E: ParseError<I>,
        {
            fn parse(i: I) -> IResult<I, Self, E> {
                nom::character::complete::$t.parse(i)
            }
        }

    )*)
}

impl_uints_type!(u8, u16, u32, u64, u128, usize);
impl_ints_type!(i8, i16, i32, i64, i128, isize);

macro_rules! impl_float_type {
    ($($t:ty, $p:ident),*) => ($(
        impl<I, E> NmeaParse<I, E> for $t
        where
            I: Input + Offset + ParseTo<$t> + AsBytes,
            I: Compare<&'static str> + for<'a> Compare<&'a [u8]>,
            <I as Input>::Item: AsChar,
            <I as Input>::Iter: Clone,
            E: ParseError<I>,
        {
            fn parse(i: I) -> IResult<I, Self, E> {
                nom::number::complete::$p.parse(i)
            }
        }
    )*)
}

impl_float_type!(f32, float, f64, double);

impl<I, E> NmeaParse<I, E> for char
where
    I: Input,
    <I as Input>::Item: AsChar,
    E: ParseError<I>,
{
    fn parse(i: I) -> IResult<I, Self, E> {
        anychar.parse(i)
    }
}

impl<T, I, E> NmeaParse<I, E> for Option<T>
where
    T: NmeaParse<I, E>,
    I: Input,
    E: ParseError<I>,
{
    fn parse(i: I) -> IResult<I, Self, E> {
        opt(T::parse).parse(i)
    }

    fn parse_preceded<S>(separator: S) -> impl Parser<I, Output = Self, Error = Error<I, E>>
    where
        S: Parser<I, Error = Error<I, E>>,
    {
        let mut separator = separator;
        move |i: I| {
            let input = i.clone();
            let (i, _) = separator.parse(i)?;
            match T::parse.parse(i.clone()) {
                Ok((i, value)) => Ok((i, Some(value))),
                Err(_) => {
                    if let Ok((_, _)) = separator.parse(i.clone()) {
                        // Input was ",," → return (",", None)
                        Ok((i, None))
                    } else if i.input_len() == 0 {
                        // Input was "," → return ("", None)
                        Ok((i, None))
                    } else {
                        Err(nom::Err::Error(nom::error::make_error(
                            input,
                            nom::error::ErrorKind::Verify,
                        )))
                    }
                }
            }
        }
    }
}

impl<T, I, E, const N: usize> NmeaParse<I, E> for [T; N]
where
    T: NmeaParse<I, E> + Default + Copy,
    I: Input,
    <I as Input>::Item: AsChar,
    E: ParseError<I>,
{
    fn parse(i: I) -> IResult<I, Self, E> {
        let mut elems = [T::default(); N];
        let mut i = i;

        match T::parse(i.clone()) {
            Ok((i1, first)) => {
                elems[0] = first;
                i = i1;
            }
            Err(nom::Err::Error(_)) => {
                return Err(nom::Err::Error(nom::error::make_error(
                    i,
                    nom::error::ErrorKind::Count,
                )));
            }
            Err(nom::Err::Failure(e)) => return Err(nom::Err::Failure(e)),
            Err(nom::Err::Incomplete(e)) => return Err(nom::Err::Incomplete(e)),
        }

        for elem in &mut elems[1..] {
            match preceded(char(','), T::parse).parse(i.clone()) {
                Ok((i1, next)) => {
                    *elem = next;
                    i = i1;
                }
                Err(nom::Err::Error(_)) => {
                    return Err(nom::Err::Error(nom::error::make_error(
                        i,
                        nom::error::ErrorKind::Count,
                    )));
                }
                Err(e) => return Err(e),
            };
        }

        Ok((i, elems))
    }

    fn parse_preceded<S>(separator: S) -> impl Parser<I, Output = Self, Error = Error<I, E>>
    where
        S: Parser<I, Error = Error<I, E>>,
    {
        let mut parser = T::parse_preceded(separator);
        move |i: I| {
            let mut i = i;
            let mut elems = [T::default(); N];

            for elem in &mut elems {
                match parser.parse(i.clone()) {
                    Ok((i1, next)) => {
                        *elem = next;
                        i = i1;
                    }
                    Err(nom::Err::Error(_)) => {
                        return Err(nom::Err::Error(nom::error::make_error(
                            i,
                            nom::error::ErrorKind::Count,
                        )));
                    }
                    Err(e) => return Err(e),
                };
            }

            Ok((i, elems))
        }
    }
}

impl<T, I, E> NmeaParse<I, E> for Vec<T>
where
    T: NmeaParse<I, E>,
    I: Clone + Input,
    <I as Input>::Item: AsChar,
    E: ParseError<I>,
{
    fn parse(i: I) -> IResult<I, Self, E> {
        let mut elems = Vec::with_capacity(4);
        let mut i = i;

        match T::parse(i.clone()) {
            Ok((i1, first)) => {
                // infinite loop check: the parser must always consume
                if i1.input_len() == i.input_len() {
                    return Err(nom::Err::Error(nom::error::make_error(
                        i,
                        nom::error::ErrorKind::Many0,
                    )));
                }

                elems.push(first);
                i = i1;
            }
            Err(nom::Err::Error(_)) => {
                return Ok((i, elems));
            }
            Err(e) => return Err(e),
        }

        loop {
            let len = i.input_len();
            match T::parse_preceded(char(',')).parse(i.clone()) {
                Ok((i1, next)) => {
                    // infinite loop check: the parser must always consume
                    if i1.input_len() == len {
                        return Err(nom::Err::Error(nom::error::make_error(
                            i,
                            nom::error::ErrorKind::Many0,
                        )));
                    }

                    elems.push(next);
                    i = i1;
                }
                Err(nom::Err::Error(_)) => return Ok((i, elems)),
                Err(e) => return Err(e),
            };
        }
    }

    fn parse_preceded<S>(separator: S) -> impl Parser<I, Output = Self, Error = Error<I, E>>
    where
        S: Parser<I, Error = Error<I, E>>,
    {
        many0(<T>::parse_preceded(separator))
    }
}

#[cfg(test)]
mod tests {
    use crate::{IResult, NmeaParse};
    use nom::{Parser, character::complete::char};

    #[test]
    fn test_parse_vec() {
        let input = "1,2,,4";
        let expected: Vec<Option<u8>> = vec![Some(1), Some(2), None, Some(4)];
        let result: IResult<_, _> = Vec::<Option<u8>>::parse(input);
        assert_eq!(result, Ok(("", expected)));

        let input = ",1,2,,4";
        let expected: Vec<Option<u8>> = vec![Some(1), Some(2), None, Some(4)];
        let result: IResult<_, _> = Vec::<Option<u8>>::parse_preceded(char(',')).parse(input);
        assert_eq!(result, Ok(("", expected)));
    }
}
