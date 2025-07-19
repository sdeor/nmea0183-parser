#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "nmea-v4-11")]
use crate::nmea_content::SystemId;
use crate::{
    self as nmea0183_parser, NmeaParse,
    nmea_content::{FixMode, SelectionMode},
};

/// GSA - GPS DOP and active satellites
///
/// <https://gpsd.gitlab.io/gpsd/NMEA.html#_gsa_gps_dop_and_active_satellites>
///
/// ```text
///         1 2 3                      15 16  17
///         | | |                       | |   |
///  $--GSA,a,a,x,x,x,x,x,x,x,x,x,x,x,x,x,x.x,x.x,*hh<CR><LF>
/// ```
///
/// NMEA 4.11:
/// ```text
///         1 2 3                      15 16  17  18
///         | | |                       | |   |   |
///  $--GSA,a,a,x,x,x,x,x,x,x,x,x,x,x,x,x,x.x,x.x,x.x*hh<CR><LF>
/// ```
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[derive(Debug, PartialEq, NmeaParse)]
pub struct GSA {
    /// Selection mode
    pub selection_mode: SelectionMode,
    /// Fix mode
    pub fix_mode: FixMode,
    #[nmea(map(|sats| sats.into_iter().flatten().collect()), parse_as([Option<u8>; 12]))]
    /// PRN numbers of the satellites used in the fix, up to 12
    pub fix_sats_prn: heapless::Vec<u8, 12>,
    /// Position Dilution of Precision
    pub pdop: Option<f32>,
    /// Horizontal Dilution of Precision
    pub hdop: Option<f32>,
    /// Vertical Dilution of Precision
    pub vdop: Option<f32>,
    #[cfg(feature = "nmea-v4-11")]
    #[cfg_attr(docsrs, doc(cfg(feature = "nmea-v4-11")))]
    /// System ID of the GNSS system used for the fix
    pub system_id: Option<SystemId>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::IResult;

    #[test]
    fn test_gsa_parsing() {
        let input = "A,3,1,2,3,,5,6,,8,9,,11,12,1.0,,3.0,";
        let expected = GSA {
            selection_mode: SelectionMode::Automatic,
            fix_mode: FixMode::Fix3D,
            fix_sats_prn: heapless::Vec::from_slice(&[1, 2, 3, 5, 6, 8, 9, 11, 12]).unwrap(),
            pdop: Some(1.0),
            hdop: None,
            vdop: Some(3.0),
            #[cfg(feature = "nmea-v4-11")]
            system_id: None,
        };
        let result: IResult<_, _> = GSA::parse(input);
        if cfg!(feature = "nmea-v4-11") {
            assert_eq!(result, Ok(("", expected)));
        } else {
            assert_eq!(result, Ok((",", expected)));
        }
    }
}
