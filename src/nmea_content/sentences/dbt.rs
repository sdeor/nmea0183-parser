use nom::{
    AsBytes, AsChar, Compare, Input, Offset, ParseTo, Parser, character::complete::char,
    error::ParseError,
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{self as nmea0183_parser, IResult, NmeaParse, nmea_content::parse::with_unit};

/// DBT - Depth Below Transducer
///
/// <https://gpsd.gitlab.io/gpsd/NMEA.html#_dbt_depth_below_transducer>
///
/// ```text
///         1   2 3   4 5   6
///         |   | |   | |   |
///  $--DBT,x.x,f,x.x,M,x.x,F*hh<CR><LF>
/// ```
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[derive(Debug, NmeaParse)]
pub struct DBT {
    #[nmea(parser(water_depth))]
    /// Water depth in meters
    pub water_depth: Option<f32>,
}

fn water_depth<I, E>(i: I) -> IResult<I, Option<f32>, E>
where
    I: Input + Offset + ParseTo<f32> + AsBytes,
    I: for<'a> Compare<&'a [u8]> + Compare<&'static str>,
    <I as Input>::Item: AsChar,
    <I as Input>::Iter: Clone,
    E: ParseError<I>,
{
    let (i, water_depth_feet) = with_unit('f').parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, water_depth_meters) = with_unit('M').parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, water_depth_fathoms) = with_unit('F').parse(i)?;

    let water_depth = water_depth_meters
        .or(water_depth_feet.map(|feet: f32| feet * 0.3048))
        .or(water_depth_fathoms.map(|fathoms: f32| fathoms * 1.8288));

    Ok((i, water_depth))
}
