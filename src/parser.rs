use nom::{crlf, digit, ErrorKind, IResult};
use std::error;
use std::io;
use std::error::Error;
use std::str::FromStr;
use std::str;

use super::*;


#[derive(Debug)]
pub enum ParseError {
    Io(io::Error),
    Incomplete,
    StartTime,
    ArrowBetweenTimes,
    EndTime,
    Unknown,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ParseError: {}", self.description())
    }
}

impl error::Error for ParseError {
    fn description(&self) -> &str {
        match *self {
            ParseError::Io(ref err) => err.description(),
            ParseError::Incomplete => "SRT data are missing",
            ParseError::StartTime => "Wrong start time format",
            ParseError::EndTime => "Wrong end time format",
            ParseError::ArrowBetweenTimes => "Expected '-->' between start and end time",
            ParseError::Unknown => "Unknown error",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

impl From<io::Error> for ParseError {
    fn from(err: io::Error) -> ParseError {
        ParseError::Io(err)
    }
}

impl From<ErrorKind> for ParseError {
    fn from(err: ErrorKind) -> ParseError {
        match err {
            ErrorKind::Custom(1) => ParseError::StartTime,
            ErrorKind::Custom(2) => ParseError::ArrowBetweenTimes,
            ErrorKind::Custom(3) => ParseError::EndTime,
            _ => ParseError::Unknown,
        }
    }
}

named!(parse_srt<Srt>,
  do_parse!(
    subs: many1!(parse_sub) >>
    (Srt{subs: subs})
  )
);

pub fn parse_srt_from_slice(input: &[u8]) -> Result<Srt, ParseError> {
    match parse_srt(input) {
        IResult::Error(_) => Err(ParseError::Unknown), //TODO from custom to ParseError
        IResult::Incomplete(_) => Err(ParseError::Incomplete),
        IResult::Done(_, srt) => Ok(srt),
    }
}

named!(parse_time<Time>,
  do_parse!(
    h: map_res!(map_res!(digit, str::from_utf8), u8::from_str)   >>
    char!(':') >>
    m: map_res!(map_res!(digit, str::from_utf8), u8::from_str)   >>
    char!(':') >>
    s: map_res!(map_res!(digit, str::from_utf8), u8::from_str)   >>
    char!(',') >>
    ms: map_res!(map_res!(digit, str::from_utf8), u16::from_str)   >>
    (Time{hours: h, minutes: m, seconds: s, milliseconds: ms})
  )
);

named!(parse_sub<SubTitle>,
  do_parse!(
    idx: map_res!(map_res!(digit, str::from_utf8), u32::from_str)   >>
    alt!(tag!(b"\n") | crlf) >>
    ti: add_return_error!(ErrorKind::Custom(1), parse_time) >>
    add_return_error!(ErrorKind::Custom(2), tag_s!(" --> ")) >>
    to: add_return_error!(ErrorKind::Custom(3), parse_time) >>
    alt!(tag!(b"\n") | crlf) >>
// TODO: take_until_and_consume("\n\n" | "\r\n\r\n" | EOF)
    txt: map_res!(map_res!(take_until_and_consume!("\n\n"), str::from_utf8), String::from_str) >>
    (SubTitle{index: idx, start_time: ti, end_time: to, text: txt})
  )
);

#[cfg(test)]
mod tests {
    use super::*;
    use nom::IResult::*;
    use nom::{Needed, ErrorKind};

    #[test]
    fn test_parse_time() {
        let empty: &[u8] = b"";
        let ok = b"01:42:05,123";
        let error = b"01,42:05,123";
        let incomplete = b"01:42:05";

        assert_eq!(parse_time(ok), Done(empty, Time{
            hours: 1,
            minutes: 42,
            seconds: 5,
            milliseconds: 123,
            }));
        assert_eq!(parse_time(error), Error(ErrorKind::Char));
        assert_eq!(parse_time(incomplete), Incomplete(Needed::Size(incomplete.len()+1)));
    }

    #[test]
    fn test_parse_sub() {
        let empty: &[u8] = b"";
        let ok = b"1\n01:42:05,123 --> 01:42:06,456\nFirst line\nSecond line\n\n";
        let error_1 = b"1\n01:42:05,,123 --> 01:42:06,456\nFirst line\nSecond line\n\n";
        let error_2 = b"1\n01:42:05,123 -a-> 01:42:06,456\nFirst line\nSecond line\n\n";
        let error_3 = b"1\n01:42:05,123 --> 01::42:06,456\nFirst line\nSecond line\n\n";
        let incomplete = b"01:42:05";

        assert_eq!(parse_sub(ok), Done(empty, SubTitle{
            index: 1,
            start_time: Time{hours: 1, minutes: 42, seconds: 5, milliseconds: 123},
            end_time: Time{hours: 1, minutes: 42, seconds: 6, milliseconds: 456},
            text: String::from_str("First line\nSecond line").unwrap(),
            }));
        assert_eq!(parse_sub(error_1), Error(ErrorKind::Custom(1)));
        assert_eq!(parse_sub(error_2), Error(ErrorKind::Custom(2)));
        assert_eq!(parse_sub(error_3), Error(ErrorKind::Custom(3)));
    }
}
