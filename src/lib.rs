#[macro_use]
extern crate nom;

pub mod parser;
use std::fmt;
use std::io::prelude::*;
use std::fs::File;

pub use parser::ParseError;
use parser::parse_srt_from_slice;

#[derive(Debug, PartialEq)]
pub struct Srt {
    subs: Vec<SubTitle>,
}

#[derive(Debug, PartialEq)]
pub struct Time {
    hours: u8,
    minutes: u8,
    seconds: u8,
    milliseconds: u16,
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "{:02}:{:02}:{:02},{:03}",
               self.hours,
               self.minutes,
               self.seconds,
               self.milliseconds)
    }
}

#[derive(Debug, PartialEq)]
pub struct SubTitle {
    index: u32,
    start_time: Time,
    end_time: Time,
    text: String,
}

impl fmt::Display for SubTitle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "{}\n{} --> {}\n{}\n",
               self.index,
               self.start_time,
               self.end_time,
               self.text)
    }
}

pub fn parse_srt_from_file(filename: &str) -> Result<Srt, ParseError> {
    let mut f = try!(File::open(filename));
    let mut buffer = vec![];
    try!(f.read_to_end(&mut buffer));

    parse_srt_from_slice(&buffer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::IResult::*;
    use nom::{Needed, ErrorKind};
    use std::str::FromStr;
    use std::str;

    #[test]
    fn test_parse_srt() {
        let srt = parse_srt_from_file("assets/test.srt").unwrap();

        assert_eq!(srt,
                   Srt {
                       subs: vec![SubTitle {
                                      index: 1,
                                      start_time: Time {
                                          hours: 0,
                                          minutes: 0,
                                          seconds: 1,
                                          milliseconds: 123,
                                      },
                                      end_time: Time {
                                          hours: 0,
                                          minutes: 0,
                                          seconds: 3,
                                          milliseconds: 456,
                                      },
                                      text: String::from_str("First line").unwrap(),
                                  },
                                  SubTitle {
                                      index: 2,
                                      start_time: Time {
                                          hours: 0,
                                          minutes: 1,
                                          seconds: 5,
                                          milliseconds: 0,
                                      },
                                      end_time: Time {
                                          hours: 0,
                                          minutes: 1,
                                          seconds: 6,
                                          milliseconds: 010,
                                      },
                                      text: String::from_str("Second line,\nand third line.")
                                          .unwrap(),
                                  },
                                  SubTitle {
                                      index: 3,
                                      start_time: Time {
                                          hours: 1,
                                          minutes: 42,
                                          seconds: 5,
                                          milliseconds: 123,
                                      },
                                      end_time: Time {
                                          hours: 1,
                                          minutes: 42,
                                          seconds: 6,
                                          milliseconds: 456,
                                      },
                                      text: String::from_str("This is the end!").unwrap(),
                                  }],
                   });
    }
}
