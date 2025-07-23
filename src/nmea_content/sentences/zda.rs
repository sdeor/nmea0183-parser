#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use nom::{
    AsChar, Compare, Input, Parser,
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, one_of},
    combinator::{opt, value},
    error::ParseError,
};

use crate::{self as nmea0183_parser, IResult, NmeaParse};

/// ZDA - Time & Date - UTC, day, month, year and local time zone
///
/// <https://gpsd.gitlab.io/gpsd/NMEA.html#_zda_time_date_utc_day_month_year_and_local_time_zone>
///
/// ```text
///         1         2  3  4    5  6  
///         |         |  |  |    |  |  
///  $--ZDA,hhmmss.ss,xx,xx,xxxx,xx,xx*hh<CR><LF>
/// ```
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[derive(Debug, Default, Clone, PartialEq, NmeaParse)]
pub struct ZDA {
    /// Fix time in UTC
    pub time: Option<time::Time>,
    #[nmea(parser(date_full_year))]
    /// Fix date in UTC
    pub date: Option<time::Date>,
    #[nmea(parser(utc_offset))]
    /// Local zone description, offset from UTC
    pub utc_offset: Option<time::UtcOffset>,
}

impl From<time::OffsetDateTime> for ZDA {
    fn from(value: time::OffsetDateTime) -> Self {
        ZDA {
            time: Some(value.time()),
            date: Some(value.date()),
            utc_offset: Some(value.offset()),
        }
    }
}

impl From<ZDA> for Option<time::OffsetDateTime> {
    fn from(value: ZDA) -> Self {
        if let (Some(time), Some(date), Some(utc_offset)) =
            (value.time, value.date, value.utc_offset)
        {
            Some(time::OffsetDateTime::new_in_offset(date, time, utc_offset))
        } else {
            None
        }
    }
}

fn date_full_year<I, E>(i: I) -> IResult<I, Option<time::Date>, E>
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

fn utc_offset<I, E>(i: I) -> IResult<I, Option<time::UtcOffset>, E>
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::IResult;

    #[test]
    fn test_zda_parsing() {
        let cases = [
            "123456.78,01,01,2023,,",
            "132502.00,11,07,2025,+03,00",
            ",,,,,",
            "132502.00,11,07,2025,,",
            "132502.00,,,,,",
            "132502.00,,,,-03,30",
            "120000.00,29,02,2024,01,00",
            "101112.13,12,11,2025,+14,00",
        ];

        for &input in &cases {
            let result: IResult<_, _> = ZDA::parse(input);
            assert!(result.is_ok(), "Failed: {input:?}\n\t{result:?}");
        }

        let cases = [
            "132502.00,11,,,,",
            "132502.00,,07,2025,,",
            "123456.78,01,,2023,,",
            "132502.00,00,07,2025,,",
            "132502.00,11,07,,+03,",
        ];

        for &input in &cases {
            let result: IResult<_, _> = ZDA::parse(input);
            println!("{:?}", &result);
            assert!(result.is_err(), "Failed: {input:?}\n\t{result:?}");
        }
    }
}
