use std::time::Duration;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    self as nmea0183_parser, NmeaParse,
    nmea_content::{
        Location, Quality,
        parse::{location, with_unit},
    },
};

/// GGA - Global Positioning System Fix Data
///
/// <https://gpsd.gitlab.io/gpsd/NMEA.html#_gga_global_positioning_system_fix_data>
///
/// ```text
///                                                      11
///         1         2       3 4        5 6 7  8   9  10 |  12 13  14
///         |         |       | |        | | |  |   |   | |   | |   |
///  $--GGA,hhmmss.ss,ddmm.mm,a,dddmm.mm,a,x,xx,x.x,x.x,M,x.x,M,x.x,xxxx*hh<CR><LF>
/// ```
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[derive(Debug, NmeaParse)]
pub struct GGA {
    /// Fix time in UTC
    pub fix_time: Option<time::Time>,
    #[nmea(parser(location))]
    /// Location (latitude and longitude)
    pub location: Option<Location>,
    /// GPS Quality Indicator
    pub fix_quality: Quality,
    /// Number of satellites in use
    pub satellite_count: Option<u8>,
    /// Horizontal Dilution of Precision
    pub hdop: Option<f32>,
    #[nmea(parser(with_unit('M')))]
    /// Altitude above/below mean sea level (geoid) in meters
    pub altitude: Option<f32>,
    #[nmea(parser(with_unit('M')))]
    /// Geoidal separation in meters, the difference between the WGS-84 earth ellipsoid and mean sea level (geoid),
    /// negative values indicate that the geoid is below the ellipsoid
    pub geoidal_separation: Option<f32>,
    #[nmea(map(|value| value.map(|sec| Duration::from_millis((sec * 1000.0) as u64))), parse_as(Option<f32>))]
    /// Age of Differential GPS data in seconds, time since last SC104 type 1 or 9 update, null field when DGPS is not used
    pub age_of_dgps: Option<Duration>,
    /// Differential reference station ID
    pub ref_station_id: Option<u16>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::IResult;

    #[test]
    fn test_gga_parsing() {
        let cases = [",,", ",42.0,", ",,69", ",42.0,69"];

        for &input in &cases {
            let i = format!(
                "001043.00,4404.14036,N,12118.85961,W,1,12,0.98,1113.0,M,-21.3,M{}",
                input
            );

            let result: IResult<_, _> = GGA::parse(i.as_str());
            assert!(result.is_ok(), "Failed: {input:?}\n\t{result:?}");
        }

        let cases = ["", ",", ",42.0", "42.0"];

        for &input in &cases {
            let i = format!(
                "001043.00,4404.14036,N,12118.85961,W,1,12,0.98,1113.0,M,-21.3,M{}",
                input
            );

            let result: IResult<_, _> = GGA::parse(i.as_str());
            assert!(result.is_err(), "Failed: {input:?}\n\t{result:?}");
        }
    }
}
