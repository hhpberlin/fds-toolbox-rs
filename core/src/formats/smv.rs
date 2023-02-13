//pub type Input<'a> = &'a [u8];
//pub type Result<'a, T> = nom::IResult<Input<'a>, T, ()>;

use std::{
    collections::HashMap,
    num::{ParseFloatError, ParseIntError},
    str::FromStr,
};

use nom::{bytes::complete::take_while1, sequence::tuple, IResult, Parser, combinator::map};

use crate::geom::{Bounds3F, Bounds3I, Vec3F};

use super::util::{from_str_ws_preceded, non_ws};

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

enum Error<'a> {
    WrongSyntax { pos: &'a str, err: ErrorKind },
    Nom(nom::Err<nom::error::Error<&'a str>>),
    NomV(nom::Err<nom::error::VerboseError<&'a str>>),
    // TODO: Using enum instead of a &str worth it?
    MissingSection { name: &'static str },
}

impl<'a> From<nom::Err<nom::error::Error<&'a str>>> for Error<'a> {
    fn from(err: nom::Err<nom::error::Error<&'a str>>) -> Self {
        Error::Nom(err)
    }
}

impl<'a> From<nom::Err<nom::error::VerboseError<&'a str>>> for Error<'a> {
    fn from(err: nom::Err<nom::error::VerboseError<&'a str>>) -> Self {
        Error::NomV(err)
    }
}

#[derive(Debug)]
enum ErrorKind {
    UnexpectedWhitespace,
    MissingLine,
    InvalidInt(ParseIntError),
    InvalidFloat(ParseFloatError),
    // TODO: Expected is currently a lower bound, not an exact value because of the way the macro is written
    WrongNumberOfValues { expected: usize, got: usize },
    TrailingCharacters,
}

fn err(pos: &str, kind: ErrorKind) -> Error {
    Error::WrongSyntax { pos, err: kind }
}

/// Parses a line into values as separated by whitespace.
macro_rules! parse {
    ($i:expr => ($($t:ident),+)) => {
        parse($i, tuple(($($t),+)))
    };
    ($i:expr => $t:ident) => {
        parse($i, $t)
    };
}

macro_rules! from_str_impl {
    ($($t:ident),+) => {
        $(fn $t(i: &str) -> IResult<&str, $t> {
            from_str_ws_preceded(i)
        })+
    };
}

from_str_impl!(f32, i32, u32);

fn vec3f(i: &str) -> IResult<&str, Vec3F> {
    let (i, (x, y, z)) = tuple((f32, f32, f32))(i)?;
    Ok((i, Vec3F::new(x, y, z)))
}

fn bounds3f(i: &str) -> IResult<&str, Bounds3F> {
    let (i, (min_x, max_x, min_y, max_y, min_z, max_z)) = tuple((f32, f32, f32, f32, f32, f32))(i)?;
    Ok((
        i,
        Bounds3F::new(
            Vec3F::new(min_x, min_y, min_z),
            Vec3F::new(max_x, max_y, max_z),
        ),
    ))
}

fn string(i: &str) -> IResult<&str, String> {
    map(non_ws, |s| s.to_string())(i)
}

fn parse<'a, T, E>(i: &'a str, mut parser: impl Parser<&'a str, T, E>) -> Result<T, Error<'a>>
where
    Error<'a>: From<nom::Err<E>>,
{
    let (i, o) = parser.parse(i)?;

    if i.is_empty() {
        Ok(o)
    } else {
        Err(err(i, ErrorKind::TrailingCharacters))
    }
}

impl Simulation {
    pub fn parse<'a>(lines: impl Iterator<Item = &'a str>) -> Result<Self, Error<'a>> {
        let mut title = None;
        let mut fds_version = None;
        let mut end_file = None;
        let mut input_file = None;
        let mut revision = None;
        let mut chid = None;
        let mut csv_files = HashMap::new();
        let mut solid_ht3d: Option<i32> = None; // TODO: Type
        let mut num_meshes: Option<u32> = None;
        let mut hrrpuv_cutoff: Option<f32> = None;
        // TODO: Find out what these values are
        let mut view_times: Option<(f32, f32, i32)> = None;
        let mut albedo = None;
        let mut i_blank = None;
        // let mut g_vec = None;
        let mut surfdef = None;

        let mut lines = lines.filter(|x| !x.trim_start().is_empty());

        // Not using a for loop because we need to peek at the next lines
        // A for loop would consume `lines` by calling .into_iter()
        while let Some(line) = lines.next() {
            if line.trim().len() != line.len() {
                return Err(err(line, ErrorKind::UnexpectedWhitespace));
            }

            let mut next = || lines.next().ok_or(err(line, ErrorKind::MissingLine));
            let next_line = next()?;

            match line {
                "TITLE" => title = Some(parse!(next_line => string)?),
                "VERSION" | "FDSVERSION" => fds_version = Some(parse!(next_line => string)?),
                "ENDF" => end_file = Some(parse!(next_line => string)?),
                "INPF" => input_file = Some(parse!(next_line => string)?),
                "REVISION" => revision = Some(parse!(next_line => string)?),
                "CHID" => chid = Some(parse!(next_line => string)?),
                "SOLID_HT3D" => solid_ht3d = Some(parse!(next_line => i32)?),
                "CSVF" => {
                    // TODO
                    let name = parse!(next()? => string)?;
                    let file = parse!(next()? => string)?;
                    csv_files.insert(name, file);
                }
                "NMESGES" => num_meshes = Some(parse!(next_line => u32)?),
                // "NMESHES" => num_meshes = Some(parse(next()?, Error::InvalidInt)?),
                // "HRRPUVCUT" => hrrpuv_cutoff = Some(parse(next()?, Error::InvalidFloat)?),
                "VIEWTIMES" => view_times = Some(parse!(next_line => (f32, f32, i32))?),
                "ALBEDO" => albedo = Some(parse!(next_line => f32)),
                "IBLANK" => i_blank = Some(parse!(next_line => i32)),
                // "GVEC" => g_vec = Some(parse!(next_line => Vec3F)),
                "SURFDEF" => surfdef = Some(parse!(next_line => string)),
                "SURFACE" => {
                    let name = parse!(next_line => string);
                    // let (a, b) = parse!(next()? => (f32, f32));
                }
                _ => unimplemented!("Unknown line: {}", line),
            }
        }

        Ok(Simulation {
            title: title.ok_or(Error::MissingSection { name: "TITLE" })?,
            fds_version: fds_version.ok_or(Error::MissingSection { name: "FDSVERSION" })?,
            end_version: end_file.ok_or(Error::MissingSection { name: "ENDF" })?,
            input_file: input_file.ok_or(Error::MissingSection { name: "INPF" })?,
            revision: revision.ok_or(Error::MissingSection { name: "REVISION" })?,
            chid: chid.ok_or(Error::MissingSection { name: "CHID" })?,
            solid_ht3d: solid_ht3d.ok_or(Error::MissingSection { name: "SOLID_HT3D" })?,
        })
    }
}
