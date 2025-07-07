use nmea0183_parser::{ChecksumMode, Error, IResult, LineEndingMode, nmea0183};
use nom::{
    Finish, Parser,
    error::{ErrorKind, ParseError},
};

fn parse_content(i: &str) -> IResult<&str, ()> {
    if i.is_empty() {
        return Err(nom::Err::Error(Error::from_error_kind(i, ErrorKind::Eof)));
    }

    Ok((i, ()))
}

fn print(result: IResult<&str, ()>) {
    match result.finish() {
        Ok((remaining, _)) => {
            println!("Parsed successfully, remaining input: '{}'", remaining);
        }
        Err(e) => {
            println!("Parsing error occurred: {:?}", e);
        }
    }
}

fn main() {
    let parser_factory = nmea0183(ChecksumMode::Optional, LineEndingMode::Required);
    let mut parser = parser_factory(parse_content);

    let result = parser.parse("$GPGGA,123456,data*41\r\n");
    print(result);

    let result = parser.parse("$\r\n");
    print(result);
}
