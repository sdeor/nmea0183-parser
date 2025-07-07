use nom::{Err, IResult, Parser, error::ErrorKind};

use crate::{ChecksumMode, LineEndingMode, checksum_crlf};

#[test]
fn test_checksum_crlf_ok() {
    let i = "*1F\r\n";
    let res: IResult<_, _> =
        checksum_crlf(ChecksumMode::Required, LineEndingMode::Required).parse(i);

    assert!(res.is_ok());
    assert_eq!(res.unwrap().1, Some(0x1F));
}

#[test]
fn test_checksum_crlf_large_hex() {
    let i = "*1F43\r\n";
    let res: IResult<_, _> =
        checksum_crlf(ChecksumMode::Required, LineEndingMode::Required).parse(i);
    assert!(res.is_err());

    let error = res.unwrap_err();
    if let Err::Error(error) = error {
        assert_eq!(error.code, ErrorKind::Count)
    } else {
        panic!("Unexpected error")
    }
}

#[test]
fn test_checksum_crlf_large_text() {
    let i = "*1Fzz\r\n";
    let res: IResult<_, _> =
        checksum_crlf(ChecksumMode::Required, LineEndingMode::Required).parse(i);
    assert!(res.is_err());

    let error = res.unwrap_err();
    if let Err::Error(error) = error {
        assert_eq!(error.code, ErrorKind::Count)
    } else {
        panic!("Unexpected error")
    }
}

#[test]
fn test_checksum_crlf_small() {
    let i = "*1\r\n";
    let res: IResult<_, _> =
        checksum_crlf(ChecksumMode::Required, LineEndingMode::Required).parse(i);
    assert!(res.is_err());

    let error = res.unwrap_err();
    if let Err::Error(error) = error {
        assert_eq!(error.code, ErrorKind::Eof)
    } else {
        panic!("Unexpected error")
    }
}

#[test]
fn test_checksum_crlf_non_hex() {
    let i = "*1z\r\n";
    let res: IResult<_, _> =
        checksum_crlf(ChecksumMode::Required, LineEndingMode::Required).parse(i);
    assert!(res.is_err());

    let error = res.unwrap_err();
    if let Err::Error(error) = error {
        assert_eq!(error.code, ErrorKind::IsA)
    } else {
        panic!("Unexpected error")
    }
}

#[test]
fn test_checksum_crlf_no_crlf() {
    let i = "*1z";
    let res: IResult<_, _> =
        checksum_crlf(ChecksumMode::Required, LineEndingMode::Required).parse(i);
    assert!(res.is_err());

    let error = res.unwrap_err();
    if let Err::Error(error) = error {
        assert_eq!(error.code, ErrorKind::CrLf)
    } else {
        panic!("Unexpected error")
    }
}

#[test]
fn test_checksum_crlf_no_checksum() {
    let i = "\r\n";
    let res: IResult<_, _> =
        checksum_crlf(ChecksumMode::Required, LineEndingMode::Required).parse(i);
    assert!(res.is_err());

    let error = res.unwrap_err();
    if let Err::Error(error) = error {
        assert_eq!(error.code, ErrorKind::Char)
    } else {
        panic!("Unexpected error")
    }
}
