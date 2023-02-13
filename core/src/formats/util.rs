use std::str::FromStr;

use nom::{
    bytes::complete::take_while1,
    combinator::map_res,
    error::ParseError,
    sequence::{tuple, Tuple, preceded}, character::complete::space1,
};

// These handle non-ascci whitespace as well, as opposed to the nom whitespace parsers
pub fn non_ws(i: &str) -> nom::IResult<&str, &str> {
    take_while1(|c: char| !c.is_whitespace())(i)
}

pub fn ws(i: &str) -> nom::IResult<&str, &str> {
    take_while1(|c: char| c.is_whitespace())(i)
}

pub fn from_str<T: FromStr>(i: &str) -> nom::IResult<&str, T> {
    map_res(non_ws, |x: &str| x.parse::<T>())(i)
}

pub fn from_str_ws_preceded<T: FromStr>(i: &str) -> nom::IResult<&str, T> {
    preceded(ws, from_str)(i)
}

