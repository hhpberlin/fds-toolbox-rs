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
    branch::alt,
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

#[derive(Debug)]
struct Surface {
    name: String,
    // TODO: What is this? Better name
    tmpm: f32,
    material_emissivity: f32,
    surface_type: i32,
    texture_width: f32,
    texture_height: f32,
    rgb: Vec3F,
    transparency: f32,
    texture: Option<String>,
}

#[derive(Debug)]
struct Material {
    name: String,
    rgb: Vec3F,
}

#[derive(Debug)]
struct Device {
    name: String,
    unit: String,
    position: Vec3F,
    orientation: Vec3F,
    a: i32,
    b: i32,
    bounds: Option<Bounds3F>,
    activations: Vec<DeviceActivation>,
}

#[derive(Debug)]
struct DeviceActivation {
    // TODO: Find out what these names mean and give them better names
    a: i32,
    b: f32,
    c: i32,
}

#[derive(Debug)]
enum Smoke3DType {
    // TODO: Find out what these names mean and give them better names
    F,
    G,
}

#[derive(Debug)]
struct Smoke3D {
    num: i32,
    file_name: String,
    quantity: String,
    name: String,
    unit: String,
    smoke_type: Smoke3DType,
}

#[derive(Debug)]
struct Slice {
    mesh_index: i32,
    file_name: String,
    quantity: String,
    name: String,
    unit: String,
    cell_centered: bool,
    // TODO: Find all options
    slice_type: String,
    bounds: Bounds3I,
}

#[derive(Debug)]
struct Plot3D {
    num: i32,
    file_name: String,
    quantity: String,
    name: String,
    unit: String,
}

enum Error<'a> {
    WrongSyntax { pos: &'a str, err: ErrorKind },
    Nom(nom::Err<nom::error::Error<&'a str>>),
    NomV(nom::Err<nom::error::VerboseError<&'a str>>),
    // TODO: Using enum instead of a &str worth it?
    MissingSection { name: &'static str },
    MissingSubSection { parent: &'a str, name: &'static str },
    InvalidKey { parent: &'a str, key: &'a str },
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
    MismatchedIndex { expected: usize, got: usize },
    Mesh(mesh::ErrorKind),
    InvalidSection,
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
        let mut surfaces = Vec::new();
        let mut materials = Vec::new();
        let mut devices = HashMap::new();
        let mut smoke3d = Vec::new();
        let mut slices = Vec::new();
        let mut pl3d = Vec::new();

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
                    let name = parse!(next()? => full_line_string)?;
                    let (tmpm, material_emissivity) = parse!(next()? => f32 f32)?;
                    let (surface_type, texture_width, texture_height, rgb, transparency) =
                        parse!(next()? => i32 f32 f32 vec3f f32)?;
                    let texture = alt((map(tag("null"), |_| None), map(full_line_string, Some)));
                    let texture = parse!(next()? => texture)?;
                    let surface = Surface {
                        name,
                        tmpm,
                        material_emissivity,
                        surface_type,
                        texture_width,
                        texture_height,
                        rgb,
                        transparency,
                        texture,
                    };
                    surfaces.push(surface);
                }
                "MATERIAL" => {
                    // TODO
                    let name = parse!(next()? => full_line_string)?;
                    let rgb = parse!(next()? => vec3f)?;
                    materials.push(Material { name, rgb });
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
                    // TODO: This is probably wrong, based on a sample size of two
                    let _ = parse!(next()? => "null")?;
                    let _ = parse!(next()? => "1")?;
                    let _ = parse!(next()? => "sensor")?;
                    let _ = parse!(next()? => "0")?;
                }
                "DEVICE" => {
                    let name = map(is_not("%"), |x: &str| x.trim().to_string());
                    let unit = map(tuple((tag("%"), full_line_string)), |(_, x)| x);
                    let (name, unit) = parse!(next()? => name unit)?;

                    // TODO: This is a bit ugly
                    let close = map(tuple((tag("%"), ws, tag("null"))), |_| ());

                    // TODO: idk what this is
                    let bounds = map(tuple((tag("#"), bounds3f)), |(_, x)| x);
                    let bounds = map(tuple((ws, opt(bounds), close)), |(_, x, _)| x);

                    // TODO: what are a and b?
                    let (position, orientation, a, b, bounds) =
                        parse!(next()? => vec3f vec3f i32 i32 bounds)?;

                    devices.insert(name.clone(), Device {
                        name,
                        unit,
                        position,
                        orientation,
                        a,
                        b,
                        bounds,
                        activations: Vec::new(),
                    });
                }
                line => {
                    let Some(line_first) = line.split_whitespace().next() else {
                        return Err(err(line, ErrorKind::InvalidSection));
                    };

                    match line_first {
                        "GRID" => {
                            let default_texture_origin = default_texture_origin
                                .ok_or(Error::MissingSection { name: "TOFFSET" })?;
                            let mesh = mesh::parse_mesh(line, default_texture_origin, next)?;
                            meshes.push(mesh);
                        }
                        "SMOKF3D" | "SMOKG3D" => {
                            let tag = tag(line_first);
                            let (_, num) = parse!(line => tag i32)?;

                            let smoke_type = match line_first {
                                "SMOKF3D" => Smoke3DType::F,
                                "SMOKG3D" => Smoke3DType::G,
                                _ => unreachable!(),
                            };

                            let file_name = parse!(next()? => full_line_string)?;
                            let quantity = parse!(next()? => full_line_string)?;
                            let name = parse!(next()? => full_line_string)?;
                            let unit = parse!(next()? => full_line_string)?;

                            smoke3d.push(Smoke3D {
                                smoke_type,
                                num,
                                file_name,
                                quantity,
                                name,
                                unit,
                            });
                        }
                        "SLCF" | "SLCC" => {
                            // TODO: a lot, this is completely different from fdsreaders implementation
                            let tag = tag(line_first);
                            let (_, mesh_index, _, slice_type, _, bounds) =
                                parse!(line => tag i32 "#" string "&" bounds3i)?;

                            let cell_centered = match line_first {
                                "SLCF" => false,
                                "SLCC" => true,
                                _ => unreachable!(),
                            };

                            let file_name = parse!(next()? => full_line_string)?;
                            let quantity = parse!(next()? => full_line_string)?;
                            let name = parse!(next()? => full_line_string)?;
                            let unit = parse!(next()? => full_line_string)?;

                            slices.push(Slice {
                                mesh_index,
                                slice_type,
                                bounds,
                                cell_centered,
                                file_name,
                                quantity,
                                name,
                                unit,
                            });
                        }
                        "DEVICE_ACT" => {
                            let (_, device) = parse!(line => "DEVICE_ACT" full_line_string)?;

                            let Some(device) = devices.get_mut(&device) else {
                                return Err(Error::InvalidKey {
                                    parent: line,
                                    key: line.split,
                                });
                            };

                            let (a, b, c) = parse!(line => i32 f32 i32)?;

                            device.activations.push(DeviceActivation { a, b, c });
                        }
                        "OPEN_VENT" | "CLOSE_VENT" => {
                            let tag = tag(line_first);
                            let (_, mesh_index) = parse!(line => tag i32)?;
                            let (_a, _b) = parse!(line => i32 f32)?;

                            // TODO
                        }
                        "PL3D" => {
                            let (_, _a, num) = parse!(line => "PL3D" f32 i32)?;

                            let file_name = parse!(next()? => full_line_string)?;
                            let quantity = parse!(next()? => full_line_string)?;
                            let name = parse!(next()? => full_line_string)?;
                            let unit = parse!(next()? => full_line_string)?;

                            // TODO: lines missing

                            pl3d.push(Plot3D {
                                num,
                                file_name,
                                quantity,
                                name,
                                unit,
                            });
                        }
                        _ => return Err(err(line, ErrorKind::UnknownSection)),
                    }
                }
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
