mod dbt;
mod dpt;
mod gga;
mod gll;
mod gsa;
mod gsv;
mod rmc;
mod vtg;
mod zda;

pub use dbt::DBT;
pub use dpt::DPT;
pub use gga::GGA;
pub use gll::GLL;
pub use gsa::GSA;
pub use gsv::GSV;
pub use rmc::RMC;
pub use vtg::VTG;
pub use zda::ZDA;

use nom::{bytes::complete::take, character::complete::one_of};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{self as nmea0183_parser, Error, NmeaParse};

/// A unified enum representing all supported NMEA 0183 sentence types.
///
/// This enum acts as a comprehensive abstraction over all built-in NMEA sentence
/// types supported by this parser. Each variant wraps the corresponding strongly-typed
/// struct, providing type-safe access to parsed sentence data.
///
/// ## Design Philosophy
///
/// `NmeaSentence` serves as the built-in content parser that works seamlessly with
/// the [`Nmea0183ParserBuilder`](crate::Nmea0183ParserBuilder) framing parser.
/// While the framing parser handles the outer NMEA structure (`$`, checksum, CRLF validation),
/// [`NmeaSentence::parse`] focuses on parsing and validating the inner sentence content.
///
/// This design allows you to:
/// - Easily parse any supported NMEA sentence type using a single parser
/// - Access strongly-typed data for each sentence variant
/// - Extend with custom parsers for additional sentence types if needed
///
/// The parser performs several validations:
/// - Checks the sentence type and content format.
/// - Validates each individual field to ensure all required fields are present and correctly formatted.
/// - Returns an error if any field is missing or malformed, indicating the specific issue.
///   If a field is optional and not present, it will not trigger an error.
/// - Ensures the sentence is fully consumed, with no remaining unparsed content after the last field.
///   If there is unexpected trailing data, an error is returned.
///
/// ## Example Usage
///
/// ```rust
/// use nmea0183_parser::{IResult, NmeaParse, nmea_content::NmeaSentence};
///
/// let result: IResult<_, _> = NmeaSentence::parse("GPZDA,123456.78,29,02,2024,03,00");
/// assert!(result.is_ok());
///
/// let sentence = result.unwrap().1;
/// match sentence {
///     NmeaSentence::ZDA(zda) => {
///         assert!(zda.time.is_some());
///         assert!(zda.date.is_some());
///         assert!(zda.utc_offset.is_some());
///     }
///     _ => println!("Other NMEA sentence parsed"),
/// }
/// ```
///
/// ## Usage with Framing Parser
///
/// ```rust
/// use nmea0183_parser::{
///     ChecksumMode, IResult, LineEndingMode, Nmea0183ParserBuilder, NmeaParse,
///     nmea_content::NmeaSentence,
/// };
/// use nom::Parser;
///
/// // Create a complete NMEA parser
/// let mut parser = Nmea0183ParserBuilder::new()
///     .checksum_mode(ChecksumMode::Required)
///     .line_ending_mode(LineEndingMode::Required)
///     .build(NmeaParse::parse);
///
/// // Parse a complete NMEA sentence
/// let input = "$GPGSV,3,2,12,01,40,083,45*44\r\n";
/// let result: IResult<_, _> = parser.parse(input);
/// match result {
///     Ok((_remaining, sentence)) => match sentence {
///         NmeaSentence::GGA(gga) => {
///             println!("GPS location: {:?}", gga.location);
///             println!("Fix quality: {:?}", gga.fix_quality);
///             println!("Satellites: {:?}", gga.satellite_count);
///         }
///         NmeaSentence::RMC(rmc) => {
///             println!("Speed: {:?} knots", rmc.speed_over_ground);
///             println!("Course: {:?}°", rmc.course_over_ground);
///         }
///         NmeaSentence::GSV(gsv) => {
///             println!("Satellites in view: {:?}", gsv.satellites);
///         }
///         _ => println!("Other sentence type parsed"),
///     },
///     Err(e) => println!("Parse error: {:?}", e),
/// }
/// ```
///
/// ## Supported Sentence Types
///
/// | Variant | Sentence Type                                           | Description                      |
/// |---------|---------------------------------------------------------|----------------------------------|
/// | DBT     | Depth Below Transducer                                  | Water depth measurements         |
/// | DPT     | Depth of Water                                          | Water depth with offset          |
/// | GGA     | Global Positioning System Fix Data                      | GPS position and fix quality     |
/// | GLL     | Geographic Position - Latitude/Longitude                | Latitude/longitude with time     |
/// | GSA     | GPS DOP and active satellites                           | Satellite constellation info     |
/// | GSV     | Satellites in View                                      | Individual satellite details     |
/// | RMC     | Recommended Minimum Navigation Information              | Essential navigation data        |
/// | VTG     | Track made good and Ground speed                        | Velocity information             |
/// | ZDA     | Time & Date - UTC, day, month, year and local time zone | UTC time and date with time zone |
///
/// ## NMEA Version Support
///
/// Different NMEA versions may include additional fields in certain sentence types. You can choose the version that matches your equipment by enabling the appropriate feature flags.
///
/// | Feature Flag   | NMEA Version | When to Use                |
/// | -------------- | ------------ | -------------------------- |
/// | `nmea-content` | Pre-2.3      | Standard NMEA parsing      |
/// | `nmea-v2-3`    | NMEA 2.3     | Older GPS/marine equipment |
/// | `nmea-v3-0`    | NMEA 3.0     | Mid-range equipment        |
/// | `nmea-v4-11`   | NMEA 4.11    | Modern equipment           |
///
/// ## Error Handling
///
/// The parser will return an error for:
/// - Unrecognized sentence types (not in the supported list above)
/// - Malformed sentence content that doesn't match the expected format
/// - Invalid field values (non-numeric where numbers expected, etc.)
///
/// ```rust
/// use nmea0183_parser::{IResult, NmeaParse, nmea_content::NmeaSentence};
///
/// // This will fail - unrecognized sentence type
/// let result: IResult<_, _> = NmeaSentence::parse("GPUNK,some,data,here");
/// assert!(result.is_err());
///
/// // This will fail - malformed GGA sentence
/// let result: IResult<_, _> = NmeaSentence::parse("GPGGA,invalid,data");
/// assert!(result.is_err());
/// ```
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, NmeaParse)]
#[nmea(pre_exec(let msg = nmea_input;))]
// TODO: Handle talker ID
#[nmea(skip_before(2))]
#[nmea(selector(take(3u8)))]
#[nmea(selection_error(Error::UnrecognizedMessage(msg)))]
#[nmea(exact)]
pub enum NmeaSentence {
    #[nmea(selector("DBT"))]
    /// Depth Below Transducer
    DBT(DBT),
    #[nmea(selector("DPT"))]
    /// Depth of Water
    DPT(DPT),
    #[nmea(selector("GGA"))]
    /// Global Positioning System Fix Data
    GGA(GGA),
    #[nmea(selector("GLL"))]
    /// Geographic Position - Latitude/Longitude
    GLL(GLL),
    #[nmea(selector("GSA"))]
    /// GPS DOP and active satellites
    GSA(GSA),
    #[nmea(selector("GSV"))]
    /// Satellites in View
    GSV(GSV),
    #[nmea(selector("RMC"))]
    /// Recommended Minimum Navigation Information
    RMC(RMC),
    #[nmea(selector("VTG"))]
    /// Track made good and Ground speed
    VTG(VTG),
    #[nmea(selector("ZDA"))]
    /// Time & Date - UTC, day, month, year and local time zone
    ZDA(ZDA),
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq, NmeaParse)]
#[nmea(selector(one_of("AV")))]
/// Status Mode Indicator
pub enum Status {
    #[nmea(selector('A'))]
    /// A - Valid
    Valid,
    #[nmea(selector('V'))]
    /// V - Invalid
    Invalid,
}

#[cfg(feature = "nmea-v2-3")]
#[cfg_attr(docsrs, doc(cfg(feature = "nmea-v2-3")))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq, NmeaParse)]
#[cfg_attr(not(feature = "nmea-v4-11"), nmea(selector(one_of("ACDEFMNRSU"))))]
#[cfg_attr(feature = "nmea-v4-11", nmea(selector(one_of("ACDEFMNPRSU"))))]
/// FAA Mode Indicator
///
/// <https://gpsd.gitlab.io/gpsd/NMEA.html#_sentence_mixes_and_nmea_variations>
pub enum FaaMode {
    #[nmea(selector('A'))]
    /// A - Autonomous mode
    Autonomous,
    #[nmea(selector('C'))]
    /// C - Quectel Querk, "Caution"
    Caution,
    #[nmea(selector('D'))]
    /// D - Differential Mode
    Differential,
    #[nmea(selector('E'))]
    /// E - Estimated (dead-reckoning) mode
    Estimated,
    #[nmea(selector('F'))]
    /// F - RTK Float mode
    FloatRtk,
    #[nmea(selector('M'))]
    /// M - Manual Input Mode
    Manual,
    #[nmea(selector('N'))]
    /// N - Data Not Valid
    DataNotValid,
    #[cfg(feature = "nmea-v4-11")]
    #[cfg_attr(docsrs, doc(cfg(feature = "nmea-v4-11")))]
    #[nmea(selector('P'))]
    /// P - Precise
    Precise,
    #[nmea(selector('R'))]
    /// R - RTK Integer mode
    FixedRtk,
    #[nmea(selector('S'))]
    /// S - Simulated Mode
    Simulator,
    #[nmea(selector('U'))]
    /// U - Quectel Querk, "Unsafe"
    Unsafe,
}

#[cfg(feature = "nmea-v4-11")]
#[cfg_attr(docsrs, doc(cfg(feature = "nmea-v4-11")))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq, NmeaParse)]
#[nmea(selector(one_of("ADEMNSV")))]
/// Navigation Status
pub enum NavStatus {
    #[nmea(selector('A'))]
    /// A - Autonomous mode
    Autonomous,
    #[nmea(selector('D'))]
    /// D - Differential Mode
    Differential,
    #[nmea(selector('E'))]
    /// E - Estimated (dead-reckoning) mode
    Estimated,
    #[nmea(selector('M'))]
    /// M - Manual Input Mode
    Manual,
    #[nmea(selector('N'))]
    /// N - Not Valid
    NotValid,
    #[nmea(selector('S'))]
    /// S - Simulated Mode
    Simulator,
    #[nmea(selector('V'))]
    /// V - Valid
    Valid,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq, NmeaParse)]
#[cfg_attr(not(feature = "nmea-v2-3"), nmea(selector(one_of("012"))))]
#[cfg_attr(feature = "nmea-v2-3", nmea(selector(one_of("012345678"))))]
/// Quality of the GPS fix
pub enum Quality {
    #[nmea(selector('0'))]
    /// 0 - Fix not available
    NoFix,
    #[nmea(selector('1'))]
    /// 1 - GPS fix
    GPSFix,
    #[nmea(selector('2'))]
    /// 2 - Differential GPS fix
    DGPSFix,
    #[cfg(feature = "nmea-v2-3")]
    #[cfg_attr(docsrs, doc(cfg(feature = "nmea-v2-3")))]
    #[nmea(selector('3'))]
    /// 3 - PPS fix
    PPSFix,
    #[cfg(feature = "nmea-v2-3")]
    #[cfg_attr(docsrs, doc(cfg(feature = "nmea-v2-3")))]
    #[nmea(selector('4'))]
    /// 4 - Real Time Kinematic
    RTK,
    #[cfg(feature = "nmea-v2-3")]
    #[cfg_attr(docsrs, doc(cfg(feature = "nmea-v2-3")))]
    #[nmea(selector('5'))]
    /// 5 - Float RTK
    FloatRTK,
    #[cfg(feature = "nmea-v2-3")]
    #[cfg_attr(docsrs, doc(cfg(feature = "nmea-v2-3")))]
    #[nmea(selector('6'))]
    /// 6 - estimated (dead reckoning)
    Estimated,
    #[cfg(feature = "nmea-v2-3")]
    #[cfg_attr(docsrs, doc(cfg(feature = "nmea-v2-3")))]
    #[nmea(selector('7'))]
    /// 7 - Manual input mode
    Manual,
    #[cfg(feature = "nmea-v2-3")]
    #[cfg_attr(docsrs, doc(cfg(feature = "nmea-v2-3")))]
    #[nmea(selector('8'))]
    /// 8 - Simulation mode
    Simulation,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq, NmeaParse)]
#[nmea(selector(one_of("AM")))]
/// Selection Mode
pub enum SelectionMode {
    #[nmea(selector('A'))]
    /// A - Automatic, 2D/3D
    Automatic,
    #[nmea(selector('M'))]
    /// M - Manual, forced to operate in 2D or 3D
    Manual,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq, NmeaParse)]
#[nmea(selector(one_of("123")))]
/// Fix Mode
pub enum FixMode {
    #[nmea(selector('1'))]
    /// 1 - No fix
    NoFix,
    #[nmea(selector('2'))]
    /// 2 - 2D Fix
    Fix2D,
    #[nmea(selector('3'))]
    /// 3 - 3D Fix
    Fix3D,
}

#[cfg(feature = "nmea-v4-11")]
#[cfg_attr(docsrs, doc(cfg(feature = "nmea-v4-11")))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq, NmeaParse)]
#[nmea(selector(one_of("123456")))]
/// NMEA 4.11 System ID
///
/// <https://gpsd.gitlab.io/gpsd/NMEA.html#_nmea_4_11_system_id_and_signal_id>
pub enum SystemId {
    #[nmea(selector('1'))]
    /// 1 - GPS (GP)
    Gps,
    #[nmea(selector('2'))]
    /// 2 - GLONASS (GL)
    Glonass,
    #[nmea(selector('3'))]
    /// 3 - Galileo (GA)
    Galileo,
    #[nmea(selector('4'))]
    /// 4 - BeiDou (GB/BD)
    Beidou,
    #[nmea(selector('5'))]
    /// 5 - QZSS (GQ)
    Qzss,
    #[nmea(selector('6'))]
    /// 6 - NavIC (GI)
    Navic,
}

/// NMEA 4.11 Signal ID
///
/// <https://gpsd.gitlab.io/gpsd/NMEA.html#_nmea_4_11_system_id_and_signal_id>
#[cfg(feature = "nmea-v4-11")]
#[cfg_attr(docsrs, doc(cfg(feature = "nmea-v4-11")))]
pub type SignalId = u8;
/*
 * // TODO:
 * pub enum SignalId {
 *     Gps(GpsSignalId),
 *     Glonass(GlonassSignalId),
 *     Galileo(GalileoSignalId),
 *     Beidou(BeidouSignalId),
 *     Qzss(QzssSignalId),
 *     Navic(NavicSignalId),
 *     Unknown(u8),
 * }
 */

/// Satellite information used in [`GSV`] sentences
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, NmeaParse)]
pub struct Satellite {
    /// PRN number of the satellite
    pub prn: u8,
    /// Elevation in degrees (0-90)
    pub elevation: Option<u8>,
    /// Azimuth in degrees (0-359)
    pub azimuth: Option<u16>,
    /// Signal-to-Noise Ratio (SNR) in dBHz
    pub snr: Option<u8>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::IResult;

    #[test]
    fn test_status() {
        assert_eq!(
            (Status::parse("A") as IResult<_, _>).unwrap(),
            ("", Status::Valid)
        );
        assert_eq!(
            (Status::parse("V") as IResult<_, _>).unwrap(),
            ("", Status::Invalid)
        );
        assert!((Status::parse("K") as IResult<_, _>).is_err());
    }

    #[test]
    fn test_faa_mode() {
        #[cfg(feature = "nmea-v2-3")]
        {
            assert_eq!(
                (FaaMode::parse("A") as IResult<_, _>).unwrap(),
                ("", FaaMode::Autonomous)
            );
            assert_eq!(
                (FaaMode::parse("C") as IResult<_, _>).unwrap(),
                ("", FaaMode::Caution)
            );
            assert_eq!(
                (FaaMode::parse("D") as IResult<_, _>).unwrap(),
                ("", FaaMode::Differential)
            );
            assert_eq!(
                (FaaMode::parse("E") as IResult<_, _>).unwrap(),
                ("", FaaMode::Estimated)
            );
            assert_eq!(
                (FaaMode::parse("F") as IResult<_, _>).unwrap(),
                ("", FaaMode::FloatRtk)
            );
            assert_eq!(
                (FaaMode::parse("M") as IResult<_, _>).unwrap(),
                ("", FaaMode::Manual)
            );
            assert_eq!(
                (FaaMode::parse("N") as IResult<_, _>).unwrap(),
                ("", FaaMode::DataNotValid)
            );
            #[cfg(feature = "nmea-v4-11")]
            {
                assert_eq!(
                    (FaaMode::parse("P") as IResult<_, _>).unwrap(),
                    ("", FaaMode::Precise)
                );
            }
            #[cfg(not(feature = "nmea-v4-11"))]
            {
                assert!((FaaMode::parse("P") as IResult<_, _>).is_err());
            }
            assert_eq!(
                (FaaMode::parse("R") as IResult<_, _>).unwrap(),
                ("", FaaMode::FixedRtk)
            );
            assert_eq!(
                (FaaMode::parse("S") as IResult<_, _>).unwrap(),
                ("", FaaMode::Simulator)
            );
            assert_eq!(
                (FaaMode::parse("U") as IResult<_, _>).unwrap(),
                ("", FaaMode::Unsafe)
            );
            assert!((FaaMode::parse("X") as IResult<_, _>).is_err());
        }
    }

    #[cfg(feature = "nmea-v4-11")]
    #[test]
    fn test_gsa_quality() {
        assert_eq!(
            (Quality::parse("0") as IResult<_, _>).unwrap(),
            ("", Quality::NoFix)
        );
        assert_eq!(
            (Quality::parse("1") as IResult<_, _>).unwrap(),
            ("", Quality::GPSFix)
        );
        assert_eq!(
            (Quality::parse("2") as IResult<_, _>).unwrap(),
            ("", Quality::DGPSFix)
        );
        assert_eq!(
            (Quality::parse("3") as IResult<_, _>).unwrap(),
            ("", Quality::PPSFix)
        );
        assert_eq!(
            (Quality::parse("4") as IResult<_, _>).unwrap(),
            ("", Quality::RTK)
        );
        assert_eq!(
            (Quality::parse("5") as IResult<_, _>).unwrap(),
            ("", Quality::FloatRTK)
        );
        assert_eq!(
            (Quality::parse("6") as IResult<_, _>).unwrap(),
            ("", Quality::Estimated)
        );
        assert_eq!(
            (Quality::parse("7") as IResult<_, _>).unwrap(),
            ("", Quality::Manual)
        );
        assert_eq!(
            (Quality::parse("8") as IResult<_, _>).unwrap(),
            ("", Quality::Simulation)
        );
        assert!((Quality::parse("9") as IResult<_, _>).is_err());
    }

    #[test]
    fn test_selection_mode() {
        assert_eq!(
            (SelectionMode::parse("A") as IResult<_, _>).unwrap(),
            ("", SelectionMode::Automatic)
        );
        assert_eq!(
            (SelectionMode::parse("M") as IResult<_, _>).unwrap(),
            ("", SelectionMode::Manual)
        );
        assert!((SelectionMode::parse("X") as IResult<_, _>).is_err());
    }

    #[test]
    fn test_fix_mode() {
        assert_eq!(
            (FixMode::parse("1") as IResult<_, _>).unwrap(),
            ("", FixMode::NoFix)
        );
        assert_eq!(
            (FixMode::parse("2") as IResult<_, _>).unwrap(),
            ("", FixMode::Fix2D)
        );
        assert_eq!(
            (FixMode::parse("3") as IResult<_, _>).unwrap(),
            ("", FixMode::Fix3D)
        );
        assert!((FixMode::parse("4") as IResult<_, _>).is_err());
    }

    #[cfg(feature = "nmea-v4-11")]
    #[test]
    fn test_system_id() {
        assert_eq!(
            (SystemId::parse("1") as IResult<_, _>).unwrap(),
            ("", SystemId::Gps)
        );
        assert_eq!(
            (SystemId::parse("2") as IResult<_, _>).unwrap(),
            ("", SystemId::Glonass)
        );
        assert_eq!(
            (SystemId::parse("3") as IResult<_, _>).unwrap(),
            ("", SystemId::Galileo)
        );
        assert_eq!(
            (SystemId::parse("4") as IResult<_, _>).unwrap(),
            ("", SystemId::Beidou)
        );
        assert_eq!(
            (SystemId::parse("5") as IResult<_, _>).unwrap(),
            ("", SystemId::Qzss)
        );
        assert_eq!(
            (SystemId::parse("6") as IResult<_, _>).unwrap(),
            ("", SystemId::Navic)
        );
        assert!((SystemId::parse("7") as IResult<_, _>).is_err());
    }

    #[cfg(feature = "nmea-v2-3")]
    #[cfg(not(feature = "nmea-v3-0"))]
    #[test]
    fn test_nmea_parser() {
        let valid = [
            "GPDBT,12.34,f,3.76,M,2.05,F",
            "GPDBT,0.00,f,0.00,M,0.00,F",
            "GPDBT,50.00,f,15.24,M,8.20,F",
            "GPDBT,1.50,f,0.46,M,0.25,F",
            "GPDBT,100.00,f,30.48,M,16.40,F",
            "GPDPT,10.5,0.2",
            "GPDPT,0.0,",
            "GPDPT,50.0,1.0",
            "GPDPT,1.2,",
            "GPDPT,100.0,0.5",
            "GPGGA,092725.00,4717.113,N,00833.915,E,1,08,1.0,499.7,M,48.0,M,,",
            "GPGGA,235959,0000.000,N,00000.000,W,1,00,99.9,0.0,M,0.0,M,,",
            "GPGGA,000000,9000.000,S,18000.000,W,1,12,0.5,100.0,M,10.0,M,,",
            "GPGGA,010203,1234.567,N,01234.567,E,2,05,2.0,20.0,M,5.0,M,,",
            "GPGLL,4916.45,N,12311.12,W,225444,A,A",
            "GPGLL,0000.00,N,00000.00,E,000000,V,N",
            "GPGLL,9000.00,S,18000.00,W,235959,A,D",
            "GPGLL,3456.78,N,07890.12,E,123456,A,A",
            "GPGLL,1234.56,S,01234.56,W,010203,V,N",
            "GPGSA,A,3,01,02,03,04,05,06,07,08,09,10,11,12,1.5,1.0,2.0",
            "GPGSA,M,1,,,,,,,,,,,,,99.9,99.9,99.9",
            "GPGSA,A,2,10,20,30,,,,,,,,,,2.0,1.5,2.5",
            "GPGSA,A,3,01,03,05,07,09,11,13,15,17,19,21,23,0.5,0.3,0.7",
            "GPGSA,M,2,02,04,06,,,,,,,,,,3.0,2.5,3.5",
            "GPGSV,3,1,11,01,65,123,45,02,40,210,30,03,70,300,35,04,20,090,20",
            "GPGSV,3,2,11,05,50,045,25,06,30,180,15,07,80,270,40,08,10,315,10",
            "GPGSV,3,3,11,09,40,060,22,10,60,150,33,11,75,240,38",
            "GPGSV,1,1,01,01,90,100,50",
            "GPGSV,2,1,04,01,45,120,25,02,30,200,18,03,60,090,30,04,70,310,35",
            "GPGSV,2,2,04,05,20,150,10,06,50,070,28,07,85,240,42",
            "GPRMC,123519,A,4807.038,N,01131.000,E,0.20,0.83,230394,004.2,W,A",
            "GPRMC,092725.00,A,4717.113,N,00833.915,E,0.0,0.0,010190,,,A",
            "GPRMC,235959,V,0000.000,N,00000.000,W,10.5,180.0,311299,,,N",
            "GPRMC,000000,A,9000.000,S,18000.000,W,100.0,0.0,010100,,,A",
            "GPRMC,010203,A,1234.567,N,01234.567,E,5.0,270.0,050607,,,A",
            "GPVTG,054.7,T,034.4,M,005.5,N,010.2,K,A",
            "GPVTG,000.0,T,000.0,M,000.0,N,000.0,K,N",
            "GPVTG,359.9,T,330.0,M,010.0,N,018.5,K,A",
            "GPVTG,090.0,T,060.0,M,001.0,N,001.8,K,A",
            "GPVTG,180.0,T,150.0,M,020.0,N,037.0,K,A",
            "GPZDA,123519,04,07,2025,,",
            "GPZDA,092725.00,01,01,1990,,",
            "GPZDA,235959,31,12,1999,,",
            "GPZDA,000000,01,01,2000,,",
            "GPZDA,010203,05,06,2007,,",
            "GPZDA,100000,15,03,2024,+01,30",
            "GPZDA,153045,20,11,2023,-08,00",
            "GPZDA,204510,02,09,2022,+03,00",
            "GPZDA,051520,10,04,2021,+07,00",
            "GPZDA,220000,25,12,2020,-11,00",
        ];

        for sentence in valid {
            let result: IResult<_, _> = NmeaSentence::parse(sentence);
            assert!(
                result.is_ok(),
                "Failed to parse valid sentence: {}, error: {:?}",
                sentence,
                result.unwrap_err()
            );
        }

        let invalid = [
            "GPDBT,12.34,x,3.76,M,2.05,F",   // Invalid unit 'x'
            "GPDBT,1.0,f,a,M,2.0,F",         // Non-numeric depth
            "GPDBT,10.0,f,5.0,M",            // Missing last field
            "GPDBT,TooDeep,f,1.0,M,2.0,F",   // Non-numeric depth
            "GPDBT,1.0,f,2.0,M,3.0,F,extra", // Extra field
            "GPDPT,10.5,0.2,x",              // Invalid character
            "GPDPT,10.5,0.2,1,2",            // Too many fields
            "GPDPT,abc,,",                   // Non-numeric depth
            "GPDPT,,0.5,",                   // Missing depth
            "GPDPT,10.0",                    // Too few fields
            "GPGGA,123519,4807.038,N,01131.000,X,1,08,0.9,545.4,M,46.9,M,,", // Invalid East/West indicator
            "GPGGA,123519,4807.038,N,01131.000,E,9,08,0.9,545.4,M,46.9,M,,", // Invalid Fix Quality
            "GPGGA,123519,4807.038,N,01131.000,E,1,A8,0.9,545.4,M,46.9,M,,", // Invalid satellites (non-numeric)
            "GPGLL,4916.45,N,12311.12,W,225444,A,X", // Invalid mode indicator
            "GPGLL,4916.45,N,12311.12,W,225444,A",   // Missing mode indicator
            "GPGLL,abc,N,12311.12,W,225444,A,A",     // Non-numeric latitude
            "GPGLL,4916.45,N,def,W,225444,A,A",      // Non-numeric longitude
            "GPGLL,4916.45,N,12311.12,W,25444,A,A",  // Invalid time format (too short)
            "GPGSA,A,3,01,02,03,04,05,06,07,08,09,10,11,12,A,1.0,2.0", // Non-numeric PDOP
            "GPGSA,A,3,01,02,03,04,05,06,07,08,09,10,11,12,1.5,B,2.0", // Non-numeric HDOP
            "GPGSA,A,3,01,02,03,04,05,06,07,08,09,10,11,12,1.5,1.0,C", // Non-numeric VDOP
            "GPGSA,A,4,01,02,03,04,05,06,07,08,09,10,11,12,1.5,1.0,2.0", // Invalid fix mode (4 is not 1, 2, or 3)
            "GPGSA,A,3,01,02,03,04,05,06,07,08,09,10,11,12,1.5,1.0",     // Missing VDOP
            "GPGSV,3,1,11,01,65,123,45,02,40,210,30,03,70,300,35,04,20,090,XX", // Non-numeric SNR
            "GPGSV,3,1,11,01,65,123,45,02,40,210,30,03,70,300,35,04,20,090", // Missing SNR
            "GPRMC,123519,A,4807.038,N,01131.000,E,0.20,0.83,230394,004.2,W,X", // Invalid mode (X not one of ACDEFMNRSU)
            "GPRMC,123519,A,4807.038,N,01131.000,E,0.20,0.83,230394,004.2,W",   // Missing mode
            "GPRMC,123519,A,4807.038,N,01131.000,E,abc,0.83,230394,004.2,W,A",  // Non-numeric speed
            "GPVTG,054.7,T,034.4,M,005.5,N,010.2,K,X", // Invalid mode indicator
            "GPVTG,054.7,T,034.4,M,005.5,N,010.2,K",   // Missing mode indicator
            "GPVTG,abc,T,034.4,M,005.5,N,010.2,K,A",   // Non-numeric true track
            "GPVTG,054.7,T,def,M,005.5,N,010.2,K,A",   // Non-numeric magnetic track
            "GPVTG,054.7,T,034.4,M,ghi,N,010.2,K,A",   // Non-numeric speed over ground (knots)
            "GPZDA,123519,04,07,2025,XX,",             // Non-numeric local time zone hours
            "GPZDA,123519,04,07,2025,,XX",             // Non-numeric local time zone minutes
            "GPZDA,123519,32,07,2025,,",               // Invalid day (32)
            "GPZDA,123519,04,13,2025,,",               // Invalid month (13)
            "GPZDA,123519,04,07,2025",                 // Missing local time zone fields
            "GPZDA,abc,04,07,2025,,",                  // Non-numeric time
            "GPZDA,123519,0,07,2025,,",                // Day 0
            "GPZDA,123519,04,0,2025,,",                // Month 0
            "GPZDA,123519,04,07,2025,01,ab",           // Non-numeric local time zone minutes
            "GPZDA,123519,04,07,2025,ab,00",           // Non-numeric local time zone hours
        ];

        for sentence in invalid {
            let result: IResult<_, _> = NmeaSentence::parse(sentence);
            assert!(
                result.is_err(),
                "Parsed invalid sentence as valid: {}, sentence: {:?}",
                sentence,
                result.unwrap(),
            );
        }
    }
}
