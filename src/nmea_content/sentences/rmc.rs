#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "nmea-v2-3")]
use crate::nmea_content::FaaMode;
#[cfg(feature = "nmea-v4-11")]
use crate::nmea_content::NavStatus;
use crate::{
    self as nmea0183_parser, NmeaParse,
    nmea_content::{
        Location, Status,
        parse::{location, magnetic_variation},
    },
};

/// RMC - Recommended Minimum Navigation Information
///
/// <https://gpsd.gitlab.io/gpsd/NMEA.html#_rmc_recommended_minimum_navigation_information>
///
/// ```text
///         1         2 3       4 5        6  7   8   9    10 11
///         |         | |       | |        |  |   |   |    |  |
///  $--RMC,hhmmss.ss,A,ddmm.mm,a,dddmm.mm,a,x.x,x.x,xxxx,x.x,a*hh<CR><LF>
/// ```
///
/// NMEA 2.3:
/// ```text
///         1         2 3       4 5        6  7   8   9    10 1112
///         |         | |       | |        |  |   |   |    |  | |
///  $--RMC,hhmmss.ss,A,ddmm.mm,a,dddmm.mm,a,x.x,x.x,xxxx,x.x,a,m*hh<CR><LF>
/// ```
///
/// NMEA 4.1:
/// ```text
///         1         2 3       4 5        6  7   8   9    10 111213
///         |         | |       | |        |  |   |   |    |  | | |
///  $--RMC,hhmmss.ss,A,ddmm.mm,a,dddmm.mm,a,x.x,x.x,xxxx,x.x,a,m,s*hh<CR><LF>
/// ```
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[derive(Debug, NmeaParse)]
pub struct RMC {
    /// Fix time in UTC
    pub fix_time: Option<time::Time>,
    /// Status Mode Indicator
    pub status: Status,
    #[nmea(parser(location))]
    /// Location (latitude and longitude)
    pub location: Option<Location>,
    /// Speed over ground in knots
    pub speed_over_ground: Option<f32>,
    /// Course over ground in degrees
    pub course_over_ground: Option<f32>,
    /// Fix date in UTC
    pub fix_date: Option<time::Date>,
    #[nmea(parser(magnetic_variation))]
    /// Magnetic variation in degrees
    pub magnetic_variation: Option<f32>,
    #[cfg(feature = "nmea-v2-3")]
    #[cfg_attr(docsrs, doc(cfg(feature = "nmea-v2-3")))]
    /// FAA Mode Indicator
    pub faa_mode: Option<FaaMode>,
    #[cfg(feature = "nmea-v4-11")]
    #[cfg_attr(docsrs, doc(cfg(feature = "nmea-v4-11")))]
    /// Navigation status
    pub nav_status: Option<NavStatus>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::IResult;

    #[test]
    fn test_rmc_parsing() {
        let cases = ["001031.00,A,4404.13993,N,12118.86023,W,0.146,,100117,,,A,V"];

        for &input in &cases {
            let result: IResult<_, _> = RMC::parse(input);
            assert!(result.is_ok(), "Failed: {input:?}\n\t{result:?}");
        }
    }
}
