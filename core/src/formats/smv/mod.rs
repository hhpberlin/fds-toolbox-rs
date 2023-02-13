//pub type Input<'a> = &'a [u8];
//pub type Result<'a, T> = nom::IResult<Input<'a>, T, ()>;

mod util;
use util::*;
mod mesh;

use std::{
    collections::HashMap,
    num::{ParseFloatError, ParseIntError},
};

use nom::{
    bytes::complete::{is_not, tag},
    combinator::{map, opt, success},
    sequence::tuple,
    IResult, Parser,
};

use super::util::{from_str_ws_preceded, non_ws, ws};
use crate::{
    geom::{Bounds3, Bounds3F, Bounds3I, Surfaces3, Vec2, Vec2F, Vec2I, Vec3, Vec3F, Vec3I, Vec3U},
    parse,
};
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
    MissingSubSection { parent: &'a str, name: &'static str },
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
    WrongNumberOfValues {
        expected: usize,
        got: usize,
    },
    TrailingCharacters,
    UnknownSection,
    MismatchedIndex {
        expected: usize,
        got: usize,
    },
    Mesh(mesh::ErrorKind),
}

fn err(pos: &str, kind: ErrorKind) -> Error {
    Error::WrongSyntax { pos, err: kind }
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
        let mut default_texture_origin = None;
        let mut ramps = None;
        let mut meshes = Vec::new();

        let mut lines = lines.filter(|x| !x.trim_start().is_empty());

        // Not using a for loop because we need to peek at the next lines
        // A for loop would consume `lines` by calling .into_iter()
        while let Some(line) = lines.next() {
            let mut next = || lines.next().ok_or(err(line, ErrorKind::MissingLine));

            match line {
                "TITLE" => title = Some(parse!(next()? => full_line_string)?),
                "VERSION" | "FDSVERSION" => {
                    fds_version = Some(parse!(next()? => full_line_string)?)
                }
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
                    let _name = parse!(next()? => full_line_string)?;
                    let (_a, _b) = parse!(next()? => f32 f32)?;
                    let (_c, _bounds) = parse!(next()? => f32 bounds3f)?;
                    let _ = parse!(next()? => "null" ws)?;
                    todo!();
                }
                "MATERIAL" => {
                    // TODO
                    let _name = parse!(next()? => full_line_string)?;
                    let _rgb = parse!(next()? => vec3f)?;
                    todo!();
                }
                "OUTLINE" => outlines = Some(repeat(next, |next, _| parse!(next()? => bounds3f))?),
                // TODO: This is called offset but fdsreader treats it as the default texture origin
                //       Check if it actually should be the default value or if it should be a global offset
                "TOFFSET" => default_texture_origin = Some(parse!(next()? => vec3f)?),
                "RAMP" => {
                    ramps = Some(repeat(next, |next, _| {
                        // TODO
                        let (_, name) = parse!(next()? => "RAMP:" full_line_string)?;
                        // TODO: next is &mut &mut here, remove double indirection
                        let vals = repeat(next, |next, _| parse!(next()? => f32 f32))?;
                        Ok((name, vals))
                    })?)
                }
                "PROP" => {
                    todo!();
                }
                "DEVICE" => {
                    let name = map(is_not("%"), |x: &str| x.trim().to_string());
                    let (_name, _, _unit) = parse!(next()? => name "%" full_line_string)?;

                    // TODO: This is a bit ugly
                    let close = map(tuple((tag("%"), ws, tag("null"))), |_| ());
                    let second_bounds = map(tuple((tag("#"), bounds3f)), |(_, x)| x);
                    let second_bounds = map(tuple((ws, opt(second_bounds), close)), |(_, x, _)| x);

                    let (_bounds, _a, _b, _second_bounds) =
                        parse!(next()? => bounds3f f32 f32 second_bounds)?;
                    todo!();
                }
                line if line.starts_with("GRID") => {
                    let default_texture_origin =
                        default_texture_origin.ok_or(Error::MissingSection { name: "TOFFSET" })?;
                    let mesh = mesh::parse_mesh(line, default_texture_origin, next)?;
                    meshes.push(mesh);
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
