use nom::{
    AsBytes, AsChar, Compare, Input, Offset, ParseTo, Parser, ToUsize,
    branch::alt,
    bytes::complete::{tag, take},
    character::complete::{char, one_of},
    combinator::{opt, value},
    error::ParseError,
    sequence::separated_pair,
};

use crate::{Error, IResult, NmeaParse, nmea_content::Location};

pub fn with_unit<I, E, T>(unit: char) -> impl Parser<I, Output = Option<T>, Error = Error<I, E>>
where
    T: NmeaParse<I, E>,
    I: Input,
    <I as Input>::Item: AsChar,
    E: ParseError<I>,
{
    separated_pair(<Option<T>>::parse, char(','), opt(char(unit)))
        .map(|(value, unit)| unit.and(value))
}

pub fn with_take<I, E, T, C>(count: C) -> impl Parser<I, Output = T, Error = Error<I, E>>
where
    T: NmeaParse<I, E>,
    I: Input,
    <I as Input>::Item: AsChar,
    E: ParseError<I>,
    C: ToUsize,
{
    take(count).and_then(T::parse)
}

pub fn date_full_year<I, E>(i: I) -> IResult<I, Option<time::Date>, E>
where
    I: Input,
    I: Compare<&'static str> + for<'a> Compare<&'a [u8]>,
    <I as Input>::Item: AsChar,
    E: ParseError<I>,
{
    alt((value(None, tag(",,")), move |i: I| {
        let (i, (day, month, year)) = (
            u8::parse,
            u8::parse_preceded(char(',')),
            u16::parse_preceded(char(',')),
        )
            .parse(i)?;

        let month = month
            .try_into()
            .or(Err(nom::Err::Error(nom::error::make_error(
                i.clone(),
                nom::error::ErrorKind::Verify,
            ))))?;

        let date =
            time::Date::from_calendar_date(year as i32, month, day).or(Err(nom::Err::Error(
                nom::error::make_error(i.clone(), nom::error::ErrorKind::Verify),
            )))?;

        Ok((i, Some(date)))
    }))
    .parse(i)
}

pub fn utc_offset<I, E>(i: I) -> IResult<I, Option<time::UtcOffset>, E>
where
    I: Input,
    I: Compare<&'static str> + for<'a> Compare<&'a [u8]>,
    <I as Input>::Item: AsChar,
    E: ParseError<I>,
{
    alt((value(None, char(',')), move |i: I| {
        let (i, (sign, hours, minutes)) =
            (opt(one_of("+-")), i8::parse, i8::parse_preceded(char(','))).parse(i)?;
        let (hours, minutes) = match sign {
            Some('-') => (-hours, -minutes),
            _ => (hours, minutes),
        };

        let time = time::UtcOffset::from_hms(hours, minutes, 0).or(Err(nom::Err::Error(
            nom::error::make_error(i.clone(), nom::error::ErrorKind::Verify),
        )))?;

        Ok((i, Some(time)))
    }))
    .parse(i)
}

pub fn location<I, E>(i: I) -> IResult<I, Option<Location>, E>
where
    I: Input + Offset + ParseTo<f64> + AsBytes,
    I: for<'a> Compare<&'a [u8]> + Compare<&'static str>,
    <I as Input>::Item: AsChar,
    <I as Input>::Iter: Clone,
    E: ParseError<I>,
{
    alt((
        value(None, tag(",,,")),
        separated_pair(
            (with_take(2u8), f64::parse, char(','), one_of("NS")).map(
                |(deg, min, _, dir): (u8, _, _, _)| {
                    let mut lat = deg as f64 + (min / 60.0);
                    if dir == 'S' {
                        lat = -lat;
                    }
                    lat
                },
            ),
            char(','),
            (with_take(3u8), f64::parse, char(','), one_of("EW")).map(
                |(deg, min, _, dir): (u8, _, _, _)| {
                    let mut lon = deg as f64 + (min / 60.0);
                    if dir == 'W' {
                        lon = -lon;
                    }
                    lon
                },
            ),
        )
        .map(|(lat, lon)| {
            Some(Location {
                latitude: lat,
                longitude: lon,
            })
        }),
    ))
    .parse(i)
}

pub fn magnetic_variation<I, E>(i: I) -> IResult<I, Option<f32>, E>
where
    I: Input + Offset + ParseTo<f32> + AsBytes,
    I: Compare<&'static str> + for<'a> Compare<&'a [u8]>,
    <I as Input>::Item: AsChar,
    <I as Input>::Iter: Clone,
    E: ParseError<I>,
{
    alt((
        value(None, char(',')),
        separated_pair(f32::parse, char(','), one_of("EW")).map(|(value, dir)| {
            if dir == 'W' {
                Some(-value)
            } else {
                Some(value)
            }
        }),
    ))
    .parse(i)
}

impl<T, I, E, const N: usize> NmeaParse<I, E> for heapless::Vec<T, N>
where
    T: NmeaParse<I, E>,
    I: Input,
    <I as Input>::Item: AsChar,
    E: ParseError<I>,
{
    fn parse(i: I) -> IResult<I, Self, E> {
        let mut elems = Vec::with_capacity(N);
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
                return Ok((i, elems.into_iter().collect()));
            }
            Err(e) => return Err(e),
        }

        loop {
            if elems.len() == N {
                return Ok((i, elems.into_iter().collect()));
            }

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
                Err(nom::Err::Error(_)) => return Ok((i, elems.into_iter().collect())),
                Err(e) => return Err(e),
            };
        }
    }

    fn parse_preceded<S>(separator: S) -> impl Parser<I, Output = Self, Error = Error<I, E>>
    where
        S: Parser<I, Error = Error<I, E>>,
    {
        use nom::multi::many_m_n;

        many_m_n(0, N, <T>::parse_preceded(separator))
            .map(|elems| elems.into_iter().collect::<heapless::Vec<_, N>>())
    }
}

impl<I, E> NmeaParse<I, E> for time::Time
where
    I: Input + Offset + ParseTo<f32> + AsBytes,
    I: Compare<&'static str> + for<'a> Compare<&'a [u8]>,
    <I as Input>::Item: AsChar,
    <I as Input>::Iter: Clone,
    E: ParseError<I>,
{
    fn parse(i: I) -> IResult<I, Self, E> {
        let (i, (hour, minute, second)) = (with_take(2u8), with_take(2u8), f32::parse).parse(i)?;

        if second.is_sign_negative() {
            return Err(nom::Err::Error(nom::error::make_error(
                i.clone(),
                nom::error::ErrorKind::Verify,
            )));
        }

        let milliseconds = second.fract() * 1000.0;
        let second = second.trunc();

        let time = time::Time::from_hms_milli(hour, minute, second as u8, milliseconds as u16).or(
            Err(nom::Err::Error(nom::error::make_error(
                i.clone(),
                nom::error::ErrorKind::Verify,
            ))),
        )?;

        Ok((i, time))
    }
}

impl<I, E> NmeaParse<I, E> for time::Date
where
    I: Input + for<'a> Compare<&'a [u8]>,
    I: Compare<&'static str>,
    <I as Input>::Item: AsChar,
    E: ParseError<I>,
{
    fn parse(i: I) -> IResult<I, Self, E> {
        let (i, (day, month, year)): (_, (_, u8, _)) =
            (with_take(2u8), with_take(2u8), with_take(2u8)).parse(i)?;

        let month = month
            .try_into()
            .or(Err(nom::Err::Error(nom::error::make_error(
                i.clone(),
                nom::error::ErrorKind::Verify,
            ))))?;

        let year = match year {
            83..=99 => year + 1900,
            _ => year + 2000,
        };

        let date = time::Date::from_calendar_date(year, month, day).or(Err(nom::Err::Error(
            nom::error::make_error(i.clone(), nom::error::ErrorKind::Verify),
        )))?;

        Ok((i, date))
    }
}

#[cfg(test)]
mod tests {
    use crate::{IResult, NmeaParse};
    use nom::{Parser, character::complete::char};

    #[test]
    fn test_parse_heapless_vec() {
        let input = "1,2,,4";
        let expected: heapless::Vec<u8, 4> = heapless::Vec::from_slice(&[1, 2, 4]).unwrap();
        let result: IResult<_, _> = heapless::Vec::<Option<u8>, 4>::parse
            .map(|v| v.into_iter().flatten().collect())
            .parse(input);
        assert_eq!(result, Ok(("", expected)));

        let input = ",1,2,,4";
        let expected: heapless::Vec<u8, 4> = heapless::Vec::from_slice(&[1, 2, 4]).unwrap();
        let result: IResult<_, _> = heapless::Vec::<Option<u8>, 4>::parse_preceded(char(','))
            .map(|v| v.into_iter().flatten().collect())
            .parse(input);
        assert_eq!(result, Ok(("", expected)));
    }
}
