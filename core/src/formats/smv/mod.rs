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
    character::{line_ending, multispace0, not_line_ending, space0},
    combinator::{opt, success, value},
    dispatch,
    error::{ContextError, ErrMode, ParseError},
    multi::count,
    sequence::{delimited, preceded, terminated},
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
    quantity: Quantity,
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
    file_name: String,
    mesh_index: i32,
    quantities: [Quantity; 5],
}

#[derive(Debug)]
struct Property {
    name: String,
    smv_ids: Vec<String>,
    smv_props: Vec<String>,
}

#[derive(Debug)]
struct Quantity {
    label: String,
    bar_label: String,
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

// TODO: Track https://github.com/rust-lang/rust/issues/50784 for doctests of private functions

/// Convenience function for applying a parser and storing the remaining input into the reference.
///
/// ```ignore
/// # use fds_toolbox_core::formats::smv::parse;
/// let mut input = "lorem ipsum";
/// parse(&mut input, "lorem").unwrap();
/// assert_eq!(input, " ipsum");
/// ```
fn parse<I: Copy, O, E>(input: &mut I, mut parser: impl Parser<I, O, E>) -> Result<O, ErrMode<E>> {
    // let (remaining, value) = parser.parse_next(*input)?;
    // *input = remaining;
    // Ok(value)
    parse_fn(input, |i| parser.parse_next(i))
}

fn parse_fn<I: Copy, O, E>(
    input: &mut I,
    mut parser: impl FnMut(I) -> Result<(I, O), E>,
) -> Result<O, E> {
    let (remaining, value) = parser(*input)?;
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
    delimited(space0, parser, (line_ending, multispace0)).context("line")
}

/// Parses an entire line, but leaves the line ending in the input.
///
/// ```ignore
/// # use fds_toolbox_core::formats::smv::parse;
/// assert_eq!(full_line.parse_next("lorem\nipsum"), Ok(("ipsum", "\nlorem")));
/// ```
fn full_line<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, &'a str, E> {
    // TODO: Which one is better between these two?
    not_line_ending.parse_next(input)
    //take_till1(|c| c == '\r' || c == '\n').parse_next(i)
}

fn repeat<'input, O>(
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

#[cfg(test)]
mod test {
    use winnow::error::VerboseError;

    use super::*;

    #[test]
    fn test_parse() {
        let mut input = "lorem ipsum";
        parse::<_, _, VerboseError<_>>(&mut input, "lorem").unwrap();
        assert_eq!(input, " ipsum");
    }

    #[test]
    fn test_line() {
        let mut input = "lorem ipsum\nsit amet\r\ndolor";
        assert_eq!(
            parse::<_, _, VerboseError<_>>(&mut input, line("lorem ipsum")).unwrap(),
            "lorem ipsum",
        );
        assert_eq!(
            parse::<_, _, VerboseError<_>>(&mut input, line(full_line)).unwrap(),
            "sit amet",
        );
        assert_eq!(input, "dolor");
    }

    #[test]
    fn ws_separated() {
        let mut input = "lorem 1 ipsum 5.3";
        assert_eq!(
            parse::<_, _, winnow::error::Error<_>>(
                &mut input,
                ws_separated!("lorem", i32, "ipsum", f32)
            )
            .unwrap(),
            ("lorem", 1, "ipsum", 5.3),
        );
    }
}

// fn repeat_line<'a, O, E>(
//     input: &'a mut &'a str,
//     parser: impl Parser<&'a str, O, winnow::error::Error<&'a str>>,
// ) -> Result<Vec<O>, winnow::error::ErrMode<winnow::error::Error<&'a str>>> {
//     parse(input, repeat)
// }

fn quantity(mut input: &str) -> IResult<&str, Quantity> {
    let label = parse_line(&mut input, full_line)?;
    let bar_label = parse_line(&mut input, full_line)?;
    let unit = parse_line(&mut input, full_line)?;
    Ok((
        input,
        Quantity {
            label: label.to_string(),
            bar_label: bar_label.to_string(),
            unit: unit.to_string(),
        },
    ))
}

impl SimulationParser<'_> {
    // fn parse2(&self) -> Result
    fn map_err(&self, err: err::Error) -> miette::Report {
        let err = match err {
            err::Error::SyntaxNonDiagnostic {
                remaining_length_bytes,
                kind,
            } => {
                let start = self.located_parser.full_input.len() - remaining_length_bytes;
                let word = self.located_parser.full_input[start..]
                    .split_whitespace()
                    .next()
                    .unwrap_or(&self.located_parser.full_input[start..]);

                err::Error::Syntax {
                    location: self.located_parser.span_from_substr(word),
                    kind,
                }
            }
            err => err,
        };
        miette::Report::new(err).with_source_code(self.located_parser.full_input.to_string())
    }

    fn parse(&self) -> Result<Simulation, err::Error> {
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
        let mut properties = Vec::new();

        let mut input = self.located_parser.full_input;

        while !input.is_empty() {
            let word = parse(&mut input, preceded(multispace0, non_ws))?;
            // let (input, addendum) =
            //     terminated(alt((full_line.map(Some), success(None))), line_ending)
            //         .parse_next(input)?;

            // dispatch! {

            // }

            if let Ok(_) = parse(&mut input, line_ending::<_, ()>) {
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
                    "HRRPUVCUT" => {
                        // TODO: Find out what this is
                        let _ = parse_line(&mut input, i32)?;
                        hrrpuv_cutoff = Some(parse_line(&mut input, f32)?)
                    }
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
                        let texture = parse(
                            &mut input,
                            alt((tag("null").value(None), full_line.map(Some))),
                        )?;
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
                    "TOFFSET" => default_texture_origin = Some(parse_line(&mut input, vec3f)?),
                    "RAMP" => {
                        let ramp = (
                            line(("RAMP:", full_line)),
                            repeat(line(ws_separated!(f32, f32))),
                        );
                        ramps = Some(parse(&mut input, repeat(ramp))?)
                    }
                    "PROP" => {
                        // TODO: This is probably wrong, based on a sample size of two
                        // parse(
                        //     &mut input,
                        //     (line("null"), line("1"), line("sensor"), line("0")),
                        // )?;
                        // todo!()

                        let name = parse_line(&mut input, full_line)?;
                        let smv_ids = parse(&mut input, repeat(line(full_line)))?;
                        let smv_props = parse(&mut input, repeat(line(full_line)))?;

                        properties.push(Property {
                            name: name.to_string(),
                            smv_ids: smv_ids.into_iter().map(str::to_string).collect(),
                            smv_props: smv_props.into_iter().map(str::to_string).collect(),
                        });
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
                    // GRID always preceded by OFFSET
                    "OFFSET" => {
                        let default_texture_origin = default_texture_origin
                            .ok_or(err::Error::MissingSection { name: "TOFFSET" })?;

                        let mesh = parse_fn(&mut input, self.parse_mesh(default_texture_origin))?;
                        meshes.push(mesh);
                    }
                    _ => {
                        return Err(err::Error::UnknownSection {
                            section: self.located_parser.span_from_substr(word),
                        })
                    }
                }
            } else {
                match word {
                    "SMOKF3D" | "SMOKG3D" => {
                        let num = parse_line(&mut input, i32)?;

                        let smoke_type = match word {
                            "SMOKF3D" => Smoke3DType::F,
                            "SMOKG3D" => Smoke3DType::G,
                            _ => unreachable!(),
                        };

                        let file_name = parse_line(&mut input, full_line)?;
                        let quantity = parse(&mut input, quantity)?;

                        smoke3d.push(Smoke3D {
                            num,
                            smoke_type,
                            file_name: file_name.to_string(),
                            quantity
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

                        // todo!()
                    }
                    "PL3D" => {
                        let (time, mesh_index) = parse_line(&mut input, ws_separated!(f32, i32))?;

                        let file_name = parse_line(&mut input, full_line)?;

                        // TODO: use `try_map` (https://github.com/rust-lang/rust/issues/79711) when it's stable
                        // let quantities = [(); 5].try_map(|_| {
                        //     parse(&mut input, quantity)?
                        // });
                        let (q1, q2, q3, q4, q5) = parse(
                            &mut input,
                            (quantity, quantity, quantity, quantity, quantity),
                        )?;
                        let quantities = [q1, q2, q3, q4, q5];

                        pl3d.push(Plot3D {
                            mesh_index,
                            file_name: file_name.to_string(),
                            quantities,
                        });
                    }
                    _ => {
                        return Err(err::Error::UnknownSection {
                            section: self.located_parser.span_from_substr(word),
                        })
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
