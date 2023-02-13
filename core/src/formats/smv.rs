//pub type Input<'a> = &'a [u8];
//pub type Result<'a, T> = nom::IResult<Input<'a>, T, ()>;

use std::{
    collections::HashMap,
    num::{ParseFloatError, ParseIntError},
    str::FromStr,
};

use nom::{
    bytes::complete::{tag, take_while1, is_not},
    combinator::{map, opt},
    sequence::tuple,
    IResult, Parser, branch::alt,
};

use super::util::{from_str_ws_preceded, non_ws, ws};
use crate::geom::{Bounds3F, Bounds3I, Vec3F, Vec3I};
use nom::sequence::preceded;

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
    UnknownSection,
}

fn err(pos: &str, kind: ErrorKind) -> Error {
    Error::WrongSyntax { pos, err: kind }
}

/// Convenience macro for parsing to omit tuple() and similar boilerplate
macro_rules! parse {
    ($i:expr => $t:tt $($tt:tt)+) => { parse($i, tuple((parse!(impl $t), $(parse!(impl $tt)),+))) };
    ($i:expr => $t:tt) => { parse($i, parse!(impl $t)) };
    // Implementation detail
    (impl $i:ident) => { $i };
    (impl $t:tt) => { preceded(ws, tag($t)) };
}

macro_rules! from_str_impl {
    ($($t:ident),+) => {
        $(fn $t(i: &str) -> IResult<&str, $t> {
            from_str_ws_preceded(i)
        })+
    };
}

from_str_impl!(f32, i32, u32, usize);

fn vec3f(i: &str) -> IResult<&str, Vec3F> {
    let (i, (x, y, z)) = tuple((f32, f32, f32))(i)?;
    Ok((i, Vec3F::new(x, y, z)))
}

fn vec3i(i: &str) -> IResult<&str, Vec3I> {
    let (i, (x, y, z)) = tuple((i32, i32, i32))(i)?;
    Ok((i, Vec3I::new(x, y, z)))
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

fn full_line_string(i: &str) -> IResult<&str, String> {
    let string = i.trim().to_string();
    // Take empty subslice at the end of the string
    // this makes sure the pointer still points into the original string
    // incase we want to use it for error reporting
    Ok((&i[i.len()..], string))
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

fn repeat<'a, T, Src: FnMut() -> Result<&'a str, Error<'a>>>(
    mut src: Src,
    parse: impl Fn(&mut Src) -> Result<T, Error<'a>>,
) -> Result<Vec<T>, Error<'a>> {
    let n = parse!(src()? => usize)?;
    (0..n).map(|_| parse(&mut src)).collect()
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
        let mut g_vec = None;
        let mut surfdef = None;
        let mut outlines = None;
        let mut t_offset = None;
        let mut ramps = None;
        let mut offset = None;

        let mut lines = lines.filter(|x| !x.trim_start().is_empty());

        // Not using a for loop because we need to peek at the next lines
        // A for loop would consume `lines` by calling .into_iter()
        while let Some(line) = lines.next() {
            if line.trim().len() != line.len() {
                return Err(err(line, ErrorKind::UnexpectedWhitespace));
            }

            let mut next = || lines.next().ok_or(err(line, ErrorKind::MissingLine));

            match line {
                "TITLE" => title = Some(parse!(next()? => full_line_string)?),
                "VERSION" | "FDSVERSION" => fds_version = Some(parse!(next()? => full_line_string)?),
                "ENDF" => end_file = Some(parse!(next()? => full_line_string)?),
                "INPF" => input_file = Some(parse!(next()? => full_line_string)?),
                "REVISION" => revision = Some(parse!(next()? => full_line_string)?),
                "CHID" => chid = Some(parse!(next()? => full_line_string)?),
                "SOLID_HT3D" => solid_ht3d = Some(parse!(next()? => i32)?),
                "CSVF" => {
                    // TODO
                    let name = parse!(next()? => full_line_string)?;
                    let file = parse!(next()? => full_line_string)?;
                    csv_files.insert(name, file);
                }
                "NMESHES" => num_meshes = Some(parse!(next()? => u32)?),
                "HRRPUVCUT" => hrrpuv_cutoff = Some(parse!(next()? => f32)?),
                "VIEWTIMES" => view_times = Some(parse!(next()? => f32 f32 i32)?),
                "ALBEDO" => albedo = Some(parse!(next()? => f32)),
                "IBLANK" => i_blank = Some(parse!(next()? => i32)),
                "GVEC" => g_vec = Some(parse!(next()? => vec3f)),
                "SURFDEF" => surfdef = Some(parse!(next()? => full_line_string)),
                "SURFACE" => {
                    // TODO
                    let name = parse!(next()? => full_line_string)?;
                    let (a, b) = parse!(next()? => f32 f32)?;
                    let (c, bounds) = parse!(next()? => f32 bounds3f)?;
                    let _ = parse!(next()? => "null" ws)?;
                    todo!();
                }
                "MATERIAL" => {
                    // TODO
                    let name = parse!(next()? => full_line_string)?;
                    let rgb = parse!(next()? => vec3f)?;
                    todo!();
                }
                "OUTLINE" => outlines = Some(repeat(next, |next| parse!(next()? => bounds3f))?),
                "TOFFSET" => t_offset = Some(parse!(next()? => vec3f)?),
                "RAMP" => ramps = Some(repeat(next, |next| {
                    // TODO
                    let (_, name) = parse!(next()? => "RAMP:" full_line_string)?;
                    // TODO: next is &mut &mut here, remove double indirection
                    let vals = repeat(next, |next| parse!(next()? => f32 f32))?;
                    Ok((name, vals))
                })?),
                "PROP" => {
                    todo!();
                }
                "DEVICE" => {
                    let name = map(is_not("%"), |x: &str| x.trim().to_string());
                    let (name, _, unit) = parse!(next()? => name "%" full_line_string)?;

                    // TODO: This is a bit ugly
                    let close = map(tuple((tag("%"), ws, tag("null"))), |_| ());
                    let second_bounds = map(tuple((tag("#"), bounds3f)), |(_, x)| x);
                    let second_bounds = map(tuple((ws, opt(second_bounds), close)), |(_, x, _)| x);

                    let (bounds, a, b, second_bounds) = parse!(next()? => bounds3f f32 f32 second_bounds)?; 
                    todo!();
                }
                "OFFSET" => offset = Some(parse!(next()? => vec3f)?),
                line if line.starts_with("GRID") => {
                    let mesh_name = parse!(line => "GRID" ws full_line_string)?;
                    let (bounds, a) = parse!(next()? => bounds3f i32)?;
                }
                _ => return Err(err(line, ErrorKind::UnknownSection)),
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
