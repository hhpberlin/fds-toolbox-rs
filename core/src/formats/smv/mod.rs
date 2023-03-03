//pub type Input<'a> = &'a [u8];
//pub type Result<'a, T> = nom::IResult<Input<'a>, T, ()>;

mod util;
use miette::Diagnostic;
use thiserror::Error;
use util::*;
mod mesh;
#[cfg(test)]
mod tests;

use std::collections::HashMap;

use winnow::{
    branch::alt,
    bytes::{tag, take_till0, take_till1},
    character::{line_ending, multispace0, space0},
    combinator::{opt, success, value},
    dispatch,
    error::{ContextError, ErrMode, ParseError},
    multi::count,
    sequence::{preceded, terminated},
    stream::{AsChar, Stream},
    IResult, Located, Parser,
};

use super::util::{f32, i32, non_ws, u32, usize, word, InputLocator};
use crate::{
    geom::{Bounds3, Bounds3F, Bounds3I, Surfaces3, Vec2, Vec2F, Vec2I, Vec3, Vec3F, Vec3I, Vec3U},
    ws_separated,
};
use util::*;

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

mod err;

impl Simulation {
    pub fn parse(file: &str) -> Result<Self, err::Error> {
        let parser = SimulationParser {
            located_parser: InputLocator::new(file),
        };
        parser.parse()
    }
}
struct SimulationParser<'a> {
    pub located_parser: InputLocator<'a>,
}

/// Convenience function for applying a parser and storing the remaining input into the reference.
///
/// ```
/// let mut input = "lorem ipsum";
/// parse(&mut input, "lorem").unwrap();
/// assert_eq!(input, " ipsum");
/// ```
fn parse<'ptr, I, O, E>(input: &'ptr mut I, parser: impl Parser<I, O, E>) -> Result<O, ErrMode<E>> {
    let (remaining, value) = parser.parse_next(*input)?;
    *input = remaining;
    Ok(value)
}

fn parse_line<'ptr, 'input, O, E: ParseError<&'input str> + ContextError<&'input str>>(
    input: &'ptr mut &'input str,
    parser: impl Parser<&'input str, O, E>,
) -> Result<O, ErrMode<E>> {
    parse(input, line(parser))
}

fn line<'a, O, E: ParseError<&'a str> + ContextError<&'a str>>(
    parser: impl Parser<&'a str, O, E>,
) -> impl Parser<&'a str, O, E> {
    terminated(parser, line_ending).context("line")
}

/// Parses an entire line, but leaves the line ending in the input.
///
/// ```
/// assert_eq!(full_line.parse_next("lorem\nipsum"), Ok(("ipsum", "\nlorem")));
/// ```
fn full_line(i: &str) -> IResult<&str, &str> {
    take_till1(|c| c == '\r' || c == '\n').parse_next(i)
}

fn repeat<'input, O, E>(
    parser: impl Parser<&'input str, O, winnow::error::Error<&'input str>>,
) -> impl FnMut(&'input str) -> IResult<&'input str, Vec<O>, winnow::error::Error<&'input str>> {
    let mut parser = parser.context("repeat");
    move |input| {
        // let (input, num) = line(usize).parse_next(input)?;
        let (input, num) = line(usize).parse_next(input)?;
        let (input, vec) = count(parser.by_ref(), num).parse_next(input)?;
        Ok((input, vec))
    }
}

// fn repeat_line<'a, O, E>(
//     input: &'a mut &'a str,
//     parser: impl Parser<&'a str, O, winnow::error::Error<&'a str>>,
// ) -> Result<Vec<O>, winnow::error::ErrMode<winnow::error::Error<&'a str>>> {
//     parse(input, repeat)
// }

impl SimulationParser<'_> {
    fn parse<'a>(&self) -> Result<Simulation, err::Error> {
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

        let mut input = self.located_parser.full_input;

        while !input.is_empty() {
            let (input, word) = preceded(multispace0, non_ws).parse_next(input)?;
            // let (input, addendum) =
            //     terminated(alt((full_line.map(Some), success(None))), line_ending)
            //         .parse_next(input)?;

            // dispatch! {

            // }

            if let Ok((input, _)) = line_ending::<_, ()>.parse_next(input) {
                match word {
                    "TITLE" => title = Some(parse_line(&mut input, full_line)?),
                    "VERSION" | "FDSVERSION" => {
                        fds_version = Some(parse_line(&mut input, full_line)?)
                    }
                    "ENDF" => end_file = Some(parse_line(&mut input, full_line)?),
                    "INPF" => input_file = Some(parse_line(&mut input, full_line)?),
                    "REVISION" => revision = Some(parse_line(&mut input, full_line)?),
                    "CHID" => chid = Some(parse_line(&mut input, full_line)?),
                    "SOLID_HT3D" => solid_ht3d = Some(parse_line(&mut input, i32)?),
                    "CSVF" => {
                        let name = parse_line(&mut input, full_line)?;
                        let file_name = parse_line(&mut input, full_line)?;
                        csv_files.insert(name, file_name);
                    }
                    "NMESHES" => num_meshes = Some(parse_line(&mut input, u32)?),
                    "HRRPUVCUT" => hrrpuv_cutoff = Some(parse_line(&mut input, f32)?),
                    "VIEWTIMES" => {
                        view_times = Some(parse_line(&mut input, ws_separated!(f32, f32, i32))?)
                    }
                    "ALBEDO" => albedo = Some(parse_line(&mut input, f32)?),
                    "IBLANK" => i_blank = Some(parse_line(&mut input, i32)?),
                    "GVEC" => g_vec = Some(parse_line(&mut input, vec3f)?),
                    "SURFDEF" => surfdef = Some(parse_line(&mut input, full_line)?),
                    "SURFACE" => {
                        let name = parse_line(&mut input, full_line)?;
                        let (tmpm, material_emissivity) =
                            parse_line(&mut input, ws_separated!(f32, f32))?;
                        let (surface_type, texture_width, texture_height, rgb, transparency) =
                            parse_line(&mut input, ws_separated!(i32, f32, f32, vec3f, f32))?;
                        let texture =
                            parse(&mut input, alt(("null".value(None), full_line.map(Some))))?;
                        surfaces.push(Surface {
                            name: name.to_string(),
                            tmpm,
                            material_emissivity,
                            surface_type,
                            texture_width,
                            texture_height,
                            rgb,
                            transparency,
                            texture: texture.map(str::to_string),
                        });
                    }
                    "MATERIAL" => {
                        let name = parse_line(&mut input, full_line)?;
                        let rgb = parse_line(&mut input, vec3f)?;
                        materials.push(Material {
                            name: name.to_string(),
                            rgb,
                        });
                    }
                    "OUTLINE" => outlines = Some(parse(&mut input, repeat(line(bounds3f)))?),
                    // TODO: This is called offset but fdsreader treats it as the default texture origin
                    //       Check if it actually should be the default value or if it should be a global offset
                    "TOFFSEF" => default_texture_origin = Some(parse_line(&mut input, vec3f)?),
                    "RAMP" => {
                        let ramp = (line(("RAMP:", full_line)), repeat(ws_separated!(f32, f32)));
                        ramps = Some(parse(&mut input, repeat(ramp))?)
                    }
                    "PROP" => {
                        // TODO: This is probably wrong, based on a sample size of two
                        parse(
                            &mut input,
                            (line("null"), line("1"), line("sensor"), line("0")),
                        )?;
                        todo!()
                    }
                    "DEVICE" => {
                        let name = take_till0("%").map(str::trim);
                        let unit = preceded("%", full_line);
                        let (name, unit) = parse_line(&mut input, (name, unit))?;

                        // TODO: This is a bit ugly
                        let close = ws_separated!("%", "null").recognize();

                        // TODO: idk what this is
                        let bounds = preceded("#", bounds3f);
                        let bounds = terminated(opt(bounds), close);

                        // TODO: what are a and b?
                        let (position, orientation, a, b, bounds) =
                            parse_line(&mut input, ws_separated!(vec3f, vec3f, i32, i32, bounds))?;

                        let name = name.to_string();
                        devices.insert(
                            name.clone(),
                            Device {
                                name,
                                unit: unit.to_string(),
                                position,
                                orientation,
                                a,
                                b,
                                bounds,
                                activations: Vec::new(),
                            },
                        );
                    }
                }
            } else {
                match word {
                    "GRID" => {
                        let default_texture_origin = default_texture_origin
                            .ok_or(err::Error::MissingSection { name: "TOFFSET" })?;
                        // let mesh = mesh::parse_mesh(line, default_texture_origin, next)?;
                        // meshes.push(mesh);
                        todo!()
                    }
                    "SMOKF3D" | "SMOKG3D" => {
                        let num = parse_line(&mut input, i32)?;

                        let smoke_type = match word {
                            "SMOKF3D" => Smoke3DType::F,
                            "SMOKG3D" => Smoke3DType::G,
                            _ => unreachable!(),
                        };

                        let file_name = parse_line(&mut input, full_line)?;
                        let quantity = parse_line(&mut input, full_line)?;
                        let name = parse_line(&mut input, full_line)?;
                        let unit = parse_line(&mut input, full_line)?;

                        smoke3d.push(Smoke3D {
                            num,
                            smoke_type,
                            file_name: file_name.to_string(),
                            quantity: quantity.to_string(),
                            name: name.to_string(),
                            unit: unit.to_string(),
                        });
                    }
                    "SLCF" | "SLCC" => {
                        let (mesh_index, _, slice_type, _, bounds) =
                            parse_line(&mut input, ws_separated!(i32, "#", non_ws, "&", bounds3i))?;

                        let cell_centered = match word {
                            "SLCC" => true,
                            "SLCF" => false,
                            _ => unreachable!(),
                        };

                        let file_name = parse_line(&mut input, full_line)?;
                        let quantity = parse_line(&mut input, full_line)?;
                        let name = parse_line(&mut input, full_line)?;
                        let unit = parse_line(&mut input, full_line)?;

                        slices.push(Slice {
                            mesh_index,
                            slice_type: slice_type.to_string(),
                            cell_centered,
                            bounds,
                            file_name: file_name.to_string(),
                            quantity: quantity.to_string(),
                            name: name.to_string(),
                            unit: unit.to_string(),
                        });
                    }
                    "DEVICE_ACT" => {
                        let device = parse_line(&mut input, full_line)?;

                        let Some(device) = devices.get_mut(device) else {
                            return Err(err::Error::InvalidKey { 
                                key: self.located_parser.span_from_substr(device), 
                                key_type: "DEVICE_ACT",
                             });
                        };

                        let (a, b, c) = parse_line(&mut input, ws_separated!(i32, f32, i32))?;

                        device.activations.push(DeviceActivation { a, b, c });
                    }
                    "OPEN_VENT" | "CLOSE_VENT" => {
                        let mesh_index = parse_line(&mut input, i32)?;
                        let (_a, _b) = parse_line(&mut input, ws_separated!(i32, f32))?;

                        let open = match word {
                            "OPEN_VENT" => true,
                            "CLOSE_VENT" => false,
                            _ => unreachable!(),
                        };

                        todo!()
                    }
                    "PL3D" => {
                        let (_a, num) = parse_line(&mut input, ws_separated!(f32, i32))?;

                        let file_name = parse_line(&mut input, full_line)?;
                        let quantity = parse_line(&mut input, full_line)?;
                        let name = parse_line(&mut input, full_line)?;
                        let unit = parse_line(&mut input, full_line)?;

                        // TODO: lines missing
                        todo!();
                        pl3d.push(Plot3D {
                            num,
                            file_name: file_name.to_string(),
                            quantity: quantity.to_string(),
                            name: name.to_string(),
                            unit: unit.to_string(),
                        });
                    }
                }
            }
        }

        Ok(Simulation {
            title: title
                .ok_or(err::Error::MissingSection { name: "TITLE" })?
                .to_string(),
            fds_version: fds_version
                .ok_or(err::Error::MissingSection { name: "FDSVERSION" })?
                .to_string(),
            end_version: end_file
                .ok_or(err::Error::MissingSection { name: "ENDF" })?
                .to_string(),
            input_file: input_file
                .ok_or(err::Error::MissingSection { name: "INPF" })?
                .to_string(),
            revision: revision
                .ok_or(err::Error::MissingSection { name: "REVISION" })?
                .to_string(),
            chid: chid
                .ok_or(err::Error::MissingSection { name: "CHID" })?
                .to_string(),
            solid_ht3d: solid_ht3d.ok_or(err::Error::MissingSection { name: "SOLID_HT3D" })?,
        })
    }
}
