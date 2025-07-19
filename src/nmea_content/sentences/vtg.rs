use nom::{
    AsBytes, AsChar, Compare, Input, Offset, ParseTo, Parser, character::complete::char,
    error::ParseError,
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "nmea-v2-3")]
use crate::nmea_content::FaaMode;
use crate::{self as nmea0183_parser, IResult, NmeaParse, nmea_content::parse::with_unit};

/// VTG - Track made good and Ground speed
///
/// <https://gpsd.gitlab.io/gpsd/NMEA.html#_vtg_track_made_good_and_ground_speed>
///
/// ```text
///          1  2  3  4  5  6  7  8
///          |  |  |  |  |  |  |  |
///  $--VTG,x.x,T,x.x,M,x.x,N,x.x,K*hh<CR><LF>
/// ```
///
/// NMEA 2.3:
///
/// ```text
///          1  2  3  4  5  6  7  8 9
///          |  |  |  |  |  |  |  | |
///  $--VTG,x.x,T,x.x,M,x.x,N,x.x,K,m*hh<CR><LF>
/// ```
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[derive(Debug, NmeaParse)]
pub struct VTG {
    #[nmea(parser(with_unit('T')))]
    /// Course over ground in degrees true
    pub course_over_ground_true: Option<f32>,
    #[nmea(parser(with_unit('M')))]
    /// Course over ground in degrees magnetic
    pub course_over_ground_magnetic: Option<f32>,
    #[nmea(parser(speed_over_ground))]
    /// Speed over ground in knots
    pub speed_over_ground: Option<f32>,
    #[cfg(feature = "nmea-v2-3")]
    #[cfg_attr(docsrs, doc(cfg(feature = "nmea-v2-3")))]
    /// FAA Mode Indicator
    pub faa_mode: Option<FaaMode>,
}

fn speed_over_ground<I, E>(i: I) -> IResult<I, Option<f32>, E>
where
    I: Input + Clone + Offset + ParseTo<f32> + AsBytes,
    I: for<'a> Compare<&'a [u8]> + Compare<&'static str>,
    <I as Input>::Item: AsChar,
    <I as Input>::Iter: Clone,
    E: ParseError<I>,
{
    let (i, speed_over_ground_knots) = with_unit('N').parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, speed_over_ground_kph) = with_unit('K').parse(i)?;

    Ok((
        i,
        speed_over_ground_knots.or(speed_over_ground_kph.map(|kph: f32| kph / 1.852)),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vtg_parsing() {
        let cases = [
            ",T,,M,,N,,K,N",
            "360.0,T,348.7,M,000.0,N,000.0,K,N",
            "360.0,T,348.7,M,100.0,N,,,N",
            "360.0,T,348.7,M,,,100.0,K,N",
            "360.0,T,348.7,M,,,,,N",
        ];

        for &input in &cases {
            let result: IResult<_, _> = VTG::parse(input);
            println!("{:?}", &result);
            assert!(result.is_ok(), "Failed: {input:?}\n\t{result:?}");
        }
    }
}
