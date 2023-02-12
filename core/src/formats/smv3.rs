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

use std::{str::FromStr};

use chumsky::{prelude::*, text::Character, Error};

use super::utils::i32;
struct Stuff {
    hrrpuvcut: Vec<f32>,
    ramp: Vec<Ramp>,
}

struct Ramp {
    name: String,
    values: Vec<(f32, f32)>,
}

fn parse_stuff(input: &str) -> Result<Stuff, Simple<&str>> {
    text::keyword("RAMP")
        .then(text::newline())
        .then(i32())
}