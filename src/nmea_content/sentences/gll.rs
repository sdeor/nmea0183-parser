#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "nmea-v2-3")]
use crate::nmea_content::FaaMode;
use crate::{
    self as nmea0183_parser, NmeaParse,
    nmea_content::{Location, Status, parse::location},
};

/// GLL - Geographic Position - Latitude/Longitude
///
/// <https://gpsd.gitlab.io/gpsd/NMEA.html#_gll_geographic_position_latitudelongitude>
///
/// ```text
///         1       2 3        4 5         6
///         |       | |        | |         |
///  $--GLL,ddmm.mm,a,dddmm.mm,a,hhmmss.ss,a*hh<CR><LF>
/// ```
///
/// NMEA 2.3:
/// ```text
///         1       2 3        4 5         6 7
///         |       | |        | |         | |
///  $--GLL,ddmm.mm,a,dddmm.mm,a,hhmmss.ss,a,m*hh<CR><LF>
/// ```
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[derive(Debug, Default, Clone, PartialEq, NmeaParse)]
pub struct GLL {
    #[nmea(parser(location))]
    /// Location (latitude and longitude)
    pub location: Option<Location>,
    /// Fix time in UTC
    pub fix_time: Option<time::Time>,
    /// Status Mode Indicator
    pub status: Status,
    #[cfg(feature = "nmea-v2-3")]
    #[cfg_attr(docsrs, doc(cfg(feature = "nmea-v2-3")))]
    /// FAA Mode Indicator
    pub faa_mode: Option<FaaMode>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::IResult;

    #[test]
    fn test_gll_parsing() {
        let cases = [",", ",A"];

        for &input in &cases {
            let i = format!("4404.14012,N,12118.85993,W,001037.00,A{}", input);

            let result: IResult<_, _> = GLL::parse(i.as_str());
            assert!(result.is_ok(), "Failed: {input:?}\n\t{result:?}");

            println!("Parsed GLL: {:?}", result.unwrap());
        }
    }

    #[cfg(feature = "nmea-v2-3")]
    #[test]
    fn test_gll_parsing_v2_3() {
        let cases = ["", "Z", ",Z"];

        for &input in &cases {
            let i = format!("4404.14012,N,12118.85993,W,001037.00,A{}", input);

            let result: IResult<_, _> = GLL::parse(i.as_str());
            assert!(result.is_err(), "Failed: {input:?}\n\t{result:?}");
        }
    }
}
