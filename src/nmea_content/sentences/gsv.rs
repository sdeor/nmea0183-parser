#[cfg(feature = "nmea-v4-11")]
use nom::{Input, combinator::opt, number::complete::hex_u32};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "nmea-v4-11")]
use crate::nmea_content::SignalId;
use crate::{self as nmea0183_parser, NmeaParse, nmea_content::Satellite};

/// GSV - Satellites in View
///
/// <https://gpsd.gitlab.io/gpsd/NMEA.html#_gsv_satellites_in_view>
///
/// ```text
///         1 2 3 4 5 6 7     n
///         | | | | | | |     |
///  $--GSV,x,x,x,x,x,x,x,...,x*hh<CR><LF>
/// ```
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[derive(Debug, NmeaParse)]
pub struct GSV {
    /// Total number of GSV sentences to be transmitted in this group
    pub total_messages: u8,
    /// Sentence number of this GSV message within current group
    pub message_number: u8,
    /// Total number of satellites in view
    pub satellites_in_view: u8,
    /// Satellite information
    pub satellites: heapless::Vec<Satellite, 4>,
    #[cfg(feature = "nmea-v4-11")]
    #[cfg_attr(docsrs, doc(cfg(feature = "nmea-v4-11")))]
    #[nmea(map(Option::flatten))]
    #[nmea(cond(!satellites.is_empty() || nmea_input.input_len() > 0))]
    #[nmea(map(|id| id.map(|hex| hex as u8)))]
    #[nmea(parser(opt(hex_u32)))]
    /// Signal ID of the GNSS system used for the fix
    pub signal_id: Option<SignalId>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::IResult;

    #[test]
    fn test_gsv_parsing() {
        let cases = [
            "1,1,00",
            "1,1,00,",
            "1,1,00,F",
            "1,1,01,05,45,120,38,",
            "1,1,04,01,60,150,45,02,30,090,30,03,70,270,50,04,10,010,20,",
            "1,1,01,05,45,120,,",
            "1,1,01,06,30,,40,",
            "1,1,01,07,,070,35,",
            "1,1,01,08,,,30,",
            "1,1,01,09,,180,,",
            "1,1,01,10,50,,,",
            "1,1,01,11,,,,",
            "1,1,03,01,60,150,45,02,30,,30,03,,270,,",
            "1,1,01,12,,,,",
            "1,1,01,05,45,,,",
        ];

        for &input in &cases {
            let result: IResult<_, _> = GSV::parse(input);
            assert!(result.is_ok(), "Failed: {input:?}\n\t{result:?}");
            println!("Parsed: {input:?} -> {result:?}");
        }

        #[cfg(feature = "nmea-v4-11")]
        {
            let cases = [
                "1,1,01,05,45,120,38",
                "1,1,04,01,60,150,45,02,30,090,30,03,70,270,50,04,10,010,20",
                "1,1,01,05,45,120,",
                "1,1,01,06,30,,40",
                "1,1,01,07,,070,35",
                "1,1,01,08,,,30",
                "1,1,01,09,,180,",
                "1,1,01,10,50,,",
                "1,1,01,11,,,",
                "1,1,03,01,60,150,45,02,30,,30,03,,270,",
                "1,1,01,12,,,",
                "1,1,01,05,45,,",
            ];

            for &input in &cases {
                let result: IResult<_, _> = GSV::parse(input);
                assert!(result.is_err(), "Failed: {input:?}\n\t{result:?}");
            }
        }
    }
}
