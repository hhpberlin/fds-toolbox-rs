/* File to parse:

HRRPUVCUT
     1
     66.66667

RAMP
    3
 RAMP: RSRVD TEMPERATURE PROFILE
    2
-0.30000      1.0000
  21.300      1.0000
 RAMP: RSRVD PRESSURE PROFILE
    2
-0.30000      0.0000
  21.300      1.0000
 RAMP: Burner_RAMP_Q
    4
  0.0000      0.0000
  300.00      1.0000
  700.00      1.0000
  1200.0      0.0000

  */

use std::str::FromStr;

use nom::{
    bytes::complete::{tag, take_while1},
    character::complete::{line_ending, space0, space1, not_line_ending},
    combinator::map_res,
    multi::count,
    number::complete::float,
    sequence::tuple,
    IResult, Parser,
};

struct Stuff {
    hrrpuvcut: Vec<f32>,
    ramp: Vec<Ramp>,
}

struct Ramp {
    name: String,
    values: Vec<(f32, f32)>,
}

fn nl(i: &str) -> IResult<&str, ()> {
    tuple((line_ending, space0)).map(|(_, _)| ()).parse(i)
}

// Parse the file using nom combinators
fn parse_all(i: &str) -> IResult<&str, Stuff> {
    let (i, hrrpuvcut) = parse_hrrpuvcut(i)?;
    let (i, ramp) = parse_ramp(i)?;
    Ok((i, Stuff { hrrpuvcut, ramp }))
}

fn parse_hrrpuvcut(i: &str) -> IResult<&str, Vec<f32>> {
    let (i, _) = tag("HRRPUVCUT")(i)?;
    let (i, _) = nl(i)?;
    let (i, num) = int(i)?;
    let (i, _) = nl(i)?;
    let (i, values) = count(float, num)(i)?;
    let (i, _) = nl(i)?;
    Ok((i, values))
}

fn int<I: FromStr>(i: &str) -> IResult<&str, I> {
    map_res(
        take_while1(|c: char| c.is_ascii_digit() || "-+".contains(c)),
        |x: &str| x.parse::<I>(),
    )(i)
}

fn parse_ramp(i: &str) -> IResult<&str, Vec<Ramp>> {
    let (i, _) = tag("RAMP")(i)?;
    let (i, _) = nl(i)?;
    let (i, num) = int(i)?;
    let (i, _) = nl(i)?;
    let (i, ramps) = count(parse_ramp_block, num)(i)?;
    Ok((i, ramps))
}

fn parse_ramp_block(i: &str) -> IResult<&str, Ramp> {
    let (i, (name, _, _)) = tuple((tag("RAMP:"), space1, not_line_ending))(i)?;
    let (i, _) = nl(i)?;
    let (i, num) = int(i)?;
    let (i, _) = nl(i)?;
    let (i, values) = count(tuple((float, space1, float)).map(|(a, _, b)| (a, b)), num)(i)?;
    let (i, _) = nl(i)?;
    Ok((i, Ramp { name: name.to_string(), values }))
}
