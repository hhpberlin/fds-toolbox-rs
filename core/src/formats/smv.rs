//pub type Input<'a> = &'a [u8];
//pub type Result<'a, T> = nom::IResult<Input<'a>, T, ()>;

use std::{collections::HashMap, num::{ParseIntError, ParseFloatError}, str::FromStr};

use nom::{IResult, sequence::{tuple, Tuple}, bytes::complete::take_while, combinator::{map_res, map}};
use nom_locate::LocatedSpan;
use thiserror::Error;

type Span<'a> = LocatedSpan<&'a str>;

#[derive(Debug)]
struct Simulation {
    title: String,
    fds_version: String,
    end_version: String,
    input_file: String,
    revision: String,
    chid: String,
    solid_ht3d: i32, // TODO: Is this the correct type?
}

#[derive(Debug, Error)]
enum Error<'a> {
    #[error("Unexpected whitespace, expected no whitespace at the start of the line: {0}")]
    UnexpectedWhitespace(Span<'a>),
    #[error("Missing line after: {0}")]
    MissingLine(Span<'a>),
    #[error("Failed to parse invalid number: {0}")]
    InvalidInt(Span<'a>, ParseIntError),
    #[error("Failed to parse invalid number: {0}")]
    InvalidFloat(Span<'a>, ParseFloatError),
    // TODO: Using enum worth it?
    #[error("Missing section: {0}")]
    MissingSection(&'static str),
    // // TODO
    // #[error("Nom error: {0}")]
    // NomError(#[from] nom::Err<Span<'static>>),
    #[error("Wrong number of values {1}, expected {2}: {0}")]
    WrongNumberOfValues(Span<'a>, usize, usize),
}

// fn parse<'a, T: FromStr<Err = SourceErr>, SourceErr, TargetErr>(
//     i: Span<'a>,
//     f: impl FnOnce(Span<'a>, SourceErr) -> TargetErr,
// ) -> Result<T, TargetErr> {
//     i.fragment().trim().parse().map_err(|x| f(i, x))
// }

macro_rules! parse {
    ($i:expr => $t:ty | $err:expr) => {
        $i.fragment().parse::<$t>().map_err(|e| $err($i, e))?
    };
    ($i:expr => ($($t:ident)+)) => {
        {
            let parts = $i.fragment().split_whitespace();
            let parts = parts.enumerate();

            ($(
                parse!($i => $t)
            ),+)
        }
    };
    ($i:expr => f32) => { parse!($i => f32 | Error::InvalidFloat) };
    ($i:expr => i32) => { parse!($i => i32 | Error::InvalidInt) };
    ($i:expr => u32) => { parse!($i => u32 | Error::InvalidInt) };
    ($i:expr => str) => { $i.fragment().trim_start() };
    ($i:expr => $t:ident) => { compile_error!(concat!("Unknown type: ", stringify!($t))) };
}


// macro_rules! assign_all {
//     ($i:expr, $($t:expr)*) => {
//         $(
//             $t = $i();
//         )*
//     };
// }

impl Simulation {
    pub fn parse<'a>(lines: impl Iterator<Item = Span<'a>>) -> Result<Self, Error<'a>> {
        let mut title = None;
        let mut fds_version = None;
        let mut end_file = None;
        let mut input_file = None;
        let mut revision = None;
        let mut chid = None;
        let mut csv_files = HashMap::new();
        let mut solid_ht3d: Option<i32> = None; // TODO: Type
        let mut num_meshes: Option<u32> = None;

        let mut lines = lines.filter(|x| !x.trim_start().is_empty());

        // Not using a for loop because we need to peek at the next lines
        // A for loop would consume `lines` by calling .into_iter()
        while let Some(line) = lines.next() {
            // let line = line.trim_end();
            if line.trim().len() != line.len() {
                return Err(Error::UnexpectedWhitespace(line));
            }

            let mut next = || {
                lines.next().ok_or(Error::MissingLine(line))
                // .map(|x| x.trim())
            };
            let next_line = next()?;

            match *line.fragment() {
                "TITLE" => title = Some(next_line.trim_start()),
                "VERSION" | "FDSVERSION" => fds_version = Some(next_line.trim_start()),
                "ENDF" => end_file = Some(next_line.trim_start()),
                "INPF" => input_file = Some(next_line.trim_start()),
                "REVISION" => revision = Some(next_line.trim_start()),
                "CHID" => chid = Some(next_line.trim_start()),
                "SOLID_HT3D" => solid_ht3d = Some(parse!(next_line => i32)),
                "CSVF" => {
                    // TODO
                    let name = next()?;
                    let file = next()?;
                    csv_files.insert(name, file);
                }
                "NMESGES" => num_meshes = Some(parse!(next_line => u32)),
                // "NMESHES" => num_meshes = Some(parse(next()?, Error::InvalidInt)?),
                // "HRRPUVCUT" => hrrpuv_cutoff = Some(parse(next()?, Error::InvalidFloat)?),
                // "VIEWTIMES" => ,
                _ => unimplemented!("Unknown line: {}", line),
            }
        }

        Ok(Simulation {
            title: title.ok_or(Error::MissingSection("TITLE"))?.to_string(),
            fds_version: fds_version
                .ok_or(Error::MissingSection("FDSVERSION"))?
                .to_string(),
            end_version: end_file.ok_or(Error::MissingSection("ENDF"))?.to_string(),
            input_file: input_file.ok_or(Error::MissingSection("INPF"))?.to_string(),
            revision: revision
                .ok_or(Error::MissingSection("REVISION"))?
                .to_string(),
            chid: chid.ok_or(Error::MissingSection("CHID"))?.to_string(),
            solid_ht3d: solid_ht3d.ok_or(Error::MissingSection("SOLID_HT3D"))?,
        })
    }
}


// fn parse_one<T: FromStr>(i: Span<'_>) -> IResult<Span<'_>, T> {
//     let parse = map_res(not_ws, |s| {
//         s.fragment().parse().map_err(|e| e.into())
//     });
//     map(tuple((ws, parse, ws)), |(_, v, _)| v)(i)
// }

// fn parse_non_ws<T: FromStr>(i: Span<'_>) -> IResult<Span<'_>, T> {
//     map_res(not_ws, |s: Span<'_>| s.fragment().parse())(i)
// }

fn not_ws(i: Span<'_>) -> IResult<Span<'_>, Span<'_>> { take_while(|c: char| !c.is_whitespace())(i) }

fn ws(i: Span<'_>) -> IResult<Span<'_>, Span<'_>> { take_while(|c: char| c.is_whitespace())(i) }