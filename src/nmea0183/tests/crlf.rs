use nom::{Err, IResult, Parser, error::ErrorKind};

use crate::nmea0183::{LineEndingMode, crlf};

#[test]
fn test_crlf() {
    let res: IResult<_, _> = crlf(LineEndingMode::Required).parse("12345\r\n");
    assert!(res.is_ok());
    let (data, _) = res.unwrap();
    assert_eq!(data, "12345");

    let res: IResult<_, _> = crlf(LineEndingMode::Required).parse("12345");
    assert!(res.is_err());
    let err = res.unwrap_err();
    if let Err::Error(e) = err {
        assert_eq!(e.code, ErrorKind::CrLf);
    }

    let res: IResult<_, _> = crlf(LineEndingMode::Forbidden).parse("12345");
    assert!(res.is_ok());
    let (data, _) = res.unwrap();
    assert_eq!(data, "12345");

    let res: IResult<_, _> = crlf(LineEndingMode::Forbidden).parse("12345\r\n");
    assert!(res.is_err());
    let err = res.unwrap_err();
    if let Err::Error(e) = err {
        assert_eq!(e.code, ErrorKind::CrLf);
    }
}
