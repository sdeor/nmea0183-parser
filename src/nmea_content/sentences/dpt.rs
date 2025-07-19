#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{self as nmea0183_parser, NmeaParse};

/// DPT - Depth of Water
///
/// <https://gpsd.gitlab.io/gpsd/NMEA.html#_dpt_depth_of_water>
///
/// ```text
///         1   2
///         |   |
///  $--DPT,x.x,x.x*hh<CR><LF>
/// ```
///
/// NMEA 3.0:
/// ```text
///        1   2   3
///        |   |   |
/// $--DPT,x.x,x.x,x.x*hh<CR><LF>
/// ```
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[derive(Debug, NmeaParse)]
pub struct DPT {
    /// Water depth relative to transducer in meters
    pub water_depth: Option<f32>,
    /// Offset from transducer in meters,
    /// positive means distance from transducer to water line,
    /// negative means distance from transducer to keel
    pub offset_from_transducer: Option<f32>,
    #[cfg(feature = "nmea-v3-0")]
    #[cfg_attr(docsrs, doc(cfg(feature = "nmea-v3-0")))]
    /// Maximum range scale in used for the measurement in meters
    pub max_range_scale: Option<f32>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::IResult;

    #[cfg(not(feature = "nmea-v3-0"))]
    #[test]
    fn test_dpt_parsing() {
        let compares = [
            (
                "10.0,2.0",
                DPT {
                    water_depth: Some(10.0),
                    offset_from_transducer: Some(2.0),
                    #[cfg(feature = "nmea-v3-0")]
                    max_range_scale: None,
                },
            ),
            (
                ",2.0",
                DPT {
                    water_depth: None,
                    offset_from_transducer: Some(2.0),
                    #[cfg(feature = "nmea-v3-0")]
                    max_range_scale: None,
                },
            ),
            (
                "10.0,",
                DPT {
                    water_depth: Some(10.0),
                    offset_from_transducer: None,
                    #[cfg(feature = "nmea-v3-0")]
                    max_range_scale: None,
                },
            ),
        ];

        for (input, expected) in compares {
            let result: IResult<_, _> = DPT::parse(input);
            if let Ok((remaining, parsed)) = result {
                assert!(
                    remaining.is_empty(),
                    "Expected no remaining input, got: {:?}",
                    remaining
                );
                assert_eq!(parsed.water_depth, expected.water_depth);
                assert_eq!(
                    parsed.offset_from_transducer,
                    expected.offset_from_transducer
                );
            } else {
                let error = result.unwrap_err();
                eprintln!("Failed to parse {:?} - error {:?}", input, error);
                panic!("Failed to parse");
            }
        }
    }

    #[cfg(feature = "nmea-v3-0")]
    #[test]
    fn test_dpt_parsing_v3_0() {
        let compares = [
            (
                "10.0,2.0,20.0",
                DPT {
                    water_depth: Some(10.0),
                    offset_from_transducer: Some(2.0),
                    max_range_scale: Some(20.0),
                },
            ),
            (
                ",2.0,",
                DPT {
                    water_depth: None,
                    offset_from_transducer: Some(2.0),
                    max_range_scale: None,
                },
            ),
            (
                "10.0,,",
                DPT {
                    water_depth: Some(10.0),
                    offset_from_transducer: None,
                    max_range_scale: None,
                },
            ),
        ];

        for (input, expected) in compares {
            let result: IResult<_, _> = DPT::parse(input);
            if let Ok((remaining, parsed)) = result {
                assert!(remaining.is_empty());
                assert_eq!(parsed.water_depth, expected.water_depth);
                assert_eq!(
                    parsed.offset_from_transducer,
                    expected.offset_from_transducer
                );
                assert_eq!(parsed.max_range_scale, expected.max_range_scale);
            } else {
                panic!("Failed to parse DPT with v3.0");
            }
        }
    }
}
