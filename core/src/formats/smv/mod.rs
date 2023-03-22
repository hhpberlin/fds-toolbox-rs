mod err;
mod util;

mod mesh;

#[cfg(test)]
mod tests;

use miette::SourceCode;
use tracing::instrument;
use std::collections::HashMap;
use util::*;

use winnow::{
    branch::alt,
    bytes::{tag, take_till0, take_till1},
    character::{line_ending, multispace0, not_line_ending, space0},
    combinator::opt,
    error::{ContextError, ErrMode, ParseError},
    multi::count,
    sequence::{delimited, preceded, terminated},
    IResult, Parser,
};

use super::util::{f32, i32, non_ws, u32, usize, InputLocator};
use crate::geom::{Bounds3F, Bounds3I, Vec3F};

macro_rules! ws_separated {
    ($($t:expr),*) => {
        crate::trace_callsite!(crate::ws_separated!($($t),*))
    }
}

#[derive(Debug)]
pub struct Smv {
    title: String,
    fds_version: String,
    end_version: String,
    input_file: String,
    revision: String,
    chid: String,
    solid_ht3d: Option<i32>,
    meshes: Vec<mesh::Mesh>,
    devices: HashMap<String, Device>,
    hrrpuv_cutoff: f32,
    smoke_albedo: f32,
    /// From dump.f90: "Parameter passed to smokeview (in .smv file) to control generation of blockages"
    i_blank: bool,
    gravity_vec: Vec3F,
    surfaces: Vec<Surface>,
    ramps: Vec<Ramp>,
    outlines: Vec<Bounds3F>,
    default_surface_id: String,
    viewtimes: ViewTimes,
    xyz_files: Vec<String>,
    time_range: Option<TimeRange>,
    heat_of_combustion: Option<f32>,
    reaction_fuel: Option<String>,
}

#[derive(Debug)]
pub struct TimeRange {
    time_start: f32,
    time_end: f32,
}

#[derive(Debug)]
pub struct ViewTimes {
    time_end: f32,
    num_frames: i32,
}

#[derive(Debug)]
pub struct Surface {
    id: String,
    /// TMPM in FDS
    /// From doccomment: "Melting temperature of water, conversion factor (K)"
    water_melting_temp: f32,
    material_emissivity: f32,
    surface_type: i32,
    texture_width: f32,
    texture_height: f32,
    rgb: Vec3F,
    transparency: f32,
    texture: Option<String>,
}

#[derive(Debug)]
pub struct Material {
    name: String,
    rgb: Vec3F,
}

#[derive(Debug)]
pub struct Device {
    id: String,
    quantity: String,
    position: Vec3F,
    orientation: Vec3F,
    state_index: i32,
    bounds: Option<Bounds3F>,
    activations: Vec<DeviceActivation>,
    property_id: String,
}

#[derive(Debug)]
pub struct DeviceActivation {
    // TODO: Find out what these names mean and give them better names
    a: i32,
    b: f32,
    c: i32,
}

#[derive(Debug)]
pub enum Smoke3DType {
    // TODO: Find out what these names mean and give them better names
    F,
    G,
}

#[derive(Debug)]
pub struct Smoke3D {
    mesh_index: i32,
    file_name: String,
    quantity: Quantity,
    smoke_type: Smoke3DType,
    mass_extinction_coefficient: Option<f32>,
}

#[derive(Debug)]
pub struct Slice {
    mesh_index: i32,
    file_name: String,
    quantity: String,
    name: String,
    unit: String,
    cell_centered: bool,
    // TODO: Find all options
    slice_type: String,
    bounds: Bounds3I,
    id: Option<String>,
}

#[derive(Debug)]
pub struct Plot3D {
    file_name: String,
    mesh_index: i32,
    quantities: [Quantity; 5],
}

#[derive(Debug)]
pub struct Property {
    name: String,
    smv_ids: Vec<String>,
    smv_props: Vec<String>,
}

#[derive(Debug)]
pub struct Quantity {
    label: String,
    bar_label: String,
    unit: String,
}

#[derive(Debug)]
pub struct Ramp {
    name: String,
    values: Vec<RampValue>,
}

#[derive(Debug)]
pub struct RampValue {
    independent: f32,
    dependent: f32,
}

impl Smv {
    pub fn parse_with_warn(
        file: &str,
        // TODO: Not too fond of passing miette::Report around
        warn: Option<Box<dyn Fn(miette::Report) + '_>>,
    ) -> Result<Self, miette::Report> {
        let parser = SimulationParser {
            located_parser: InputLocator::new(file),
        };
        // So the closures can be move
        let parser = &parser;

        let map_err = move |err| parser.map_err(err, file.to_string());
        // TODO: Avoid `to_string` call for owned input
        parser
            // .parse(warn.map(|warn| Box::new(|e| warn(map_err(e))) as Box<dyn Fn(err::Error)>))
            .parse(warn.map(|warn| Box::new(move |e| warn(map_err(e))) as Box<dyn Fn(err::Error)>))
            .map_err(map_err)
    }

    pub fn parse(file: &str) -> Result<Self, miette::Report> {
        Self::parse_with_warn(file, None)
    }

    /// Convenience function for parsing a simulation and printing errors to stderr.
    #[allow(clippy::print_stderr,
        // TODO: Track https://github.com/rust-lang/rust/issues/54503 to uncomment this
        // reason = "Printing to stderr is intended here, this is a convenience function for tests"
    )]
    pub fn parse_with_warn_stdout(file: &str) -> Result<Self, miette::Report> {
        Self::parse_with_warn(file, Some(Box::new(|e| eprintln!("{:?}", e))))
    }
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
    not_line_ending
        .parse_next(input)
        .map(|(i, o)| (i, o.trim()))
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

struct SimulationParser<'a> {
    pub located_parser: InputLocator<'a>,
}

impl<'a> SimulationParser<'a> {
    /// Converts the given [`err::Error`] into a pretty-printable [`miette::Report`].
    fn map_err<Src: SourceCode + Send + Sync + 'static>(
        &self,
        err: err::Error,
        owned_input: Src,
    ) -> miette::Report {
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
        miette::Report::new(err).with_source_code(owned_input)
    }

    /// Checks if the given value matches the given constant,
    /// if not it returns [`err::Error::InvalidFloatConstant`] with given `&str` as location.
    /// The signature of `val` matches the return of [`Parser::with_recognized`].
    fn f32_const(&self, val: (f32, &str), const_val: f32) -> Result<(), err::Error> {
        let (val, str) = val;
        if val == const_val {
            Ok(())
        } else {
            Err(err::Error::InvalidFloatConstant {
                span: self.located_parser.span_from_substr(str),
                expected: const_val,
            })
        }
    }

    /// Checks if the given value matches the given constant,
    /// if not it returns [`err::Error::InvalidIntConstant`] with given `&str` as location.
    /// The signature of `val` matches the return of [`Parser::with_recognized`].
    fn i32_const(&self, val: (i32, &str), const_val: i32) -> Result<(), err::Error> {
        let (val, str) = val;
        if val == const_val {
            Ok(())
        } else {
            Err(err::Error::InvalidIntConstant {
                span: self.located_parser.span_from_substr(str),
                expected: const_val,
            })
        }
    }

    /// Parses the input as ".smv", calling `warn` for any non-critical errors if `warn` is not `None`.
    // TODO: Should non-critical errors have a separate type? It would make sense but duplicate some code.
    #[instrument]
    fn parse(&self, warn: Option<Box<dyn Fn(err::Error) + '_>>) -> Result<Smv, err::Error> {
        // For reference, the SMV file is written by `dump.f90` in FDS.
        // Search for `WRITE(LU_SMV` to find the relevant parts of the code.

        let mut title = None;
        let mut fds_version = None;
        let mut end_file = None;
        let mut input_file = None;
        let mut revision = None;
        let mut chid = None;
        let mut csv_files = HashMap::new();
        let mut solid_ht3d: Option<i32> = None; // TODO: Type
        let mut num_meshes: Option<usize> = None;
        let mut hrrpuv_cutoff: Option<f32> = None;
        let mut heat_of_combustion = None;
        let mut reaction_fuel = None;

        let mut time_range = None;

        let mut viewtimes = None;

        let mut smoke_albedo = None;
        let mut i_blank = None;
        let mut gravity_vec = None;
        let mut default_surface_id = None;
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

        let mut xyz_files = Vec::new();

        let mut input = self.located_parser.full_input;

        // TODO: Error on unexpected section repetitions (2 titles for example)

        while !input.is_empty() {
            let word = parse(&mut input, preceded(multispace0, non_ws))?;

            if parse(&mut input, line_ending::<_, ()>).is_ok() {
                match word {
                    "TITLE" => title = Some(parse_line(&mut input, full_line)?),
                    "VERSION" | "FDSVERSION" => {
                        let mut ver =
                            parse(&mut input, terminated(full_line, line_ending))?.to_string();
                        // TODO: This is very cursed and weird
                        //       Why is there even a case where the version is two lines long
                        //       Who decided that's okay, like morally speaking
                        if let Ok::<_, ErrMode<()>>(line) =
                            parse(&mut input, terminated(full_line, line_ending))
                        {
                            if !line.is_empty() {
                                ver = format!("{ver}\n{line}");
                            }
                        }
                        fds_version = Some(ver);
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
                    "NMESHES" => num_meshes = Some(parse_line(&mut input, usize)?),
                    "HRRPUVCUT" => {
                        // This line is hardcoded as 1 in FDS
                        let _ = parse_line(&mut input, "1")?;
                        hrrpuv_cutoff = Some(parse_line(&mut input, f32)?)
                    }
                    "TIMES" => {
                        let (time_start, time_end) =
                            parse_line(&mut input, ws_separated!(f32, f32))?;
                        time_range = Some(TimeRange {
                            time_start,
                            time_end,
                        });
                    }
                    "VIEWTIMES" => {
                        let (start, time_end, num_frames) =
                            parse_line(&mut input, ws_separated!(f32.with_recognized(), f32, i32))?;
                        // Hardcoded to 0 in FDS
                        self.f32_const(start, 0.)?;
                        viewtimes = Some(ViewTimes {
                            time_end,
                            num_frames,
                        });
                    }
                    "ALBEDO" => smoke_albedo = Some(parse_line(&mut input, f32)?),
                    // Always 0 or 1
                    // TODO: Error if not 0 or 1
                    "IBLANK" => i_blank = Some(parse_line(&mut input, u32)? == 1),
                    "GVEC" => gravity_vec = Some(parse_line(&mut input, vec3f)?),
                    "SURFDEF" => default_surface_id = Some(parse_line(&mut input, full_line)?),
                    "SURFACE" => {
                        let name = parse_line(&mut input, full_line)?;
                        let (water_melting_temp, material_emissivity) =
                            parse_line(&mut input, ws_separated!(f32, f32))?;
                        let (surface_type, texture_width, texture_height, rgb, transparency) =
                            parse_line(&mut input, ws_separated!(i32, f32, f32, vec3f, f32))?;
                        let texture = parse(
                            &mut input,
                            alt((tag("null").value(None), full_line.map(Some))),
                        )?;
                        surfaces.push(Surface {
                            id: name.to_string(),
                            water_melting_temp,
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
                    "TOFFSET" => default_texture_origin = Some(parse_line(&mut input, vec3f)?),
                    "RAMP" => {
                        let ramp = (
                            line(preceded("RAMP:", full_line)),
                            repeat(line(ws_separated!(f32, f32)).map(
                                |(independent, dependent)| RampValue {
                                    independent,
                                    dependent,
                                },
                            )),
                        )
                            .map(|(name, values)| Ramp {
                                name: name.trim().to_string(),
                                values,
                            });
                        ramps = Some(parse(&mut input, repeat(ramp))?);
                    }
                    "PROP" => {
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
                        let device_id = take_till0("%").map(str::trim);
                        let quant = preceded("%", full_line);
                        let (device_id, quantity) = parse_line(&mut input, (device_id, quant))?;

                        // TODO: This is a bit ugly
                        let close = preceded((space0, "%"), full_line);

                        // TODO: idk what this is
                        let bounds = preceded("#", bounds3f);

                        // TODO: what are a and b?
                        let (position, orientation, state_index, zero, bounds, property_id) =
                            parse_line(
                                &mut input,
                                ws_separated!(
                                    vec3f,
                                    vec3f,
                                    i32,
                                    i32.with_recognized(),
                                    opt(bounds),
                                    close
                                ),
                            )?;

                        self.i32_const(zero, 0)?;

                        let id = device_id.to_string();
                        devices.insert(
                            id.clone(),
                            Device {
                                id,
                                quantity: quantity.to_string(),
                                position,
                                orientation,
                                state_index,
                                bounds,
                                activations: Vec::new(),
                                property_id: property_id.to_string(),
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
                    "XYZ" => {
                        let file_name = parse_line(&mut input, full_line)?;
                        xyz_files.push(file_name.to_string());
                    }
                    "HoC" => {
                        let one = parse_line(&mut input, i32.with_recognized())?;
                        self.i32_const(one, 1)?;
                        heat_of_combustion = Some(parse_line(&mut input, f32)?);
                    }
                    "FUEL" => {
                        let one = parse_line(&mut input, i32.with_recognized())?;
                        self.i32_const(one, 1)?;
                        reaction_fuel = Some(parse_line(&mut input, full_line)?);
                    }
                    // Quietly discard some sections
                    // TODO: Parse these sections
                    "FACE" | "CADGEOM" | "VERT" | "CLASS_OF_PARTICLES" => {
                        input = self.skip_section(input, &None, word)?;
                    }
                    _ => {
                        input = self.skip_section(input, &warn, word)?;
                    }
                }
            } else {
                match word {
                    // TODO: I can't find any reference to "SMOKG3D" in the current `dump.f90` source code, only "SMOKF3D".
                    "SMOKF3D" | "SMOKG3D" => {
                        let (mesh_index, mass_extinction_coefficient) =
                            parse_line(&mut input, ws_separated!(i32, opt(f32)))?;

                        let smoke_type = match word {
                            "SMOKF3D" => Smoke3DType::F,
                            "SMOKG3D" => Smoke3DType::G,
                            _ => unreachable!(),
                        };

                        let file_name = parse_line(&mut input, full_line)?;
                        let quantity = parse(&mut input, quantity)?;

                        smoke3d.push(Smoke3D {
                            mesh_index,
                            smoke_type,
                            file_name: file_name.to_string(),
                            quantity,
                            mass_extinction_coefficient,
                        });
                    }
                    "SLCF" | "SLCC" => {
                        let id = preceded("%", take_till1("&").map(str::trim));
                        // From Fortran:
                        // ' ! ',SL%SLCF_INDEX, CC_VAL
                        // Not present in DemoHaus2
                        let stuff = preceded("!", ws_separated!(i32, opt(i32)));
                        let (mesh_index, _, slice_type, id, _, bounds, _stuff) = parse_line(
                            &mut input,
                            ws_separated!(i32, "#", non_ws, opt(id), "&", bounds3i, opt(stuff)),
                        )?;

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
                            file_name: file_name.to_string(),
                            quantity: quantity.to_string(),
                            name: name.to_string(),
                            unit: unit.to_string(),
                            cell_centered,
                            slice_type: slice_type.to_string(),
                            bounds,
                            id: id.map(ToString::to_string),
                        });
                    }
                    // TODO: I can't find any reference to "DEVICE_ACT" in the current `dump.f90` source code.
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
                        let _mesh_index = parse_line(&mut input, i32)?;
                        let (_a, _b) = parse_line(&mut input, ws_separated!(i32, f32))?;

                        let _open = match word {
                            "OPEN_VENT" => true,
                            "CLOSE_VENT" => false,
                            _ => unreachable!(),
                        };

                        // todo!()
                    }
                    "PL3D" => {
                        let (_time, mesh_index) = parse_line(&mut input, ws_separated!(f32, i32))?;

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
                    // "BNDF" => {
                    //     let (mesh_index, one) = parse_line(&mut input, ws_separated!(u32, i32.with_recognized()));
                    //     i32_const(one, 1)?;

                    // }
                    // Quietly discard some sections
                    // TODO: Parse these sections
                    "PRT5" | "ISOG" | "HIDE_OBST" | "SHOW_OBST" | "BNDF" => {
                        input = self.skip_section(input, &None, word)?;
                    }
                    _ => {
                        input = self.skip_section(input, &warn, word)?;
                    }
                }
            }
        }

        let num_meshes = num_meshes.ok_or(err::Error::MissingSection { name: "NMESHES" })?;
        if meshes.len() != num_meshes {
            return Err(err::Error::WrongNumberOfMeshes {
                expected: num_meshes,
                found: meshes.len(),
            });
        }

        let title = title
            .ok_or(err::Error::MissingSection { name: "TITLE" })?
            .to_string();
        let fds_version = fds_version.ok_or(err::Error::MissingSection { name: "FDSVERSION" })?;
        let end_version = end_file
            .ok_or(err::Error::MissingSection { name: "ENDF" })?
            .to_string();
        let input_file = input_file
            .ok_or(err::Error::MissingSection { name: "INPF" })?
            .to_string();
        let revision = revision
            .ok_or(err::Error::MissingSection { name: "REVISION" })?
            .to_string();
        let chid = chid
            .ok_or(err::Error::MissingSection { name: "CHID" })?
            .to_string();
        // let solid_ht3d = solid_ht3d.ok_or(err::Error::MissingSection { name: "SOLID_HT3D" })?;

        let hrrpuv_cutoff =
            hrrpuv_cutoff.ok_or(err::Error::MissingSection { name: "HRRPUVCUT" })?;
        let viewtimes = viewtimes.ok_or(err::Error::MissingSection { name: "VIEWTIMES" })?;
        let smoke_albedo = smoke_albedo.ok_or(err::Error::MissingSection { name: "ALBEDO" })?;
        let i_blank = i_blank.ok_or(err::Error::MissingSection { name: "IBLANK" })?;
        let gravity_vec = gravity_vec.ok_or(err::Error::MissingSection { name: "GVEC" })?;

        let ramps = ramps.ok_or(err::Error::MissingSection { name: "RAMP" })?;
        let outlines = outlines.ok_or(err::Error::MissingSection { name: "OUTLINE" })?;

        let default_surface_id = default_surface_id
            .ok_or(err::Error::MissingSection { name: "SURFDEF" })?
            .to_string();

        let reaction_fuel = reaction_fuel.map(str::to_string);

        Ok(Smv {
            title,
            fds_version,
            end_version,
            input_file,
            revision,
            chid,
            solid_ht3d,
            meshes,
            devices,
            hrrpuv_cutoff,
            smoke_albedo,
            i_blank,
            gravity_vec,
            ramps,
            outlines,
            default_surface_id,
            surfaces,
            viewtimes,
            time_range,
            xyz_files,
            heat_of_combustion,
            reaction_fuel,
        })
    }

    fn skip_section<'b>(
        &'b self,
        mut input: &'b str,
        warn: &Option<Box<dyn Fn(err::Error) + 'b>>,
        word: &'b str,
    ) -> Result<&'b str, err::Error> {
        // Skip the current line
        // Incase of parsing something like "MESH <name>", if name is all caps, it would falsely be parsed as a section
        let Ok(_) = parse_line::<_, ()>(&mut input, full_line) else {
            return Err(err::Error::UnexpectedEndOfInput {
                span: self.located_parser.span_from_substr(word),
            });
        };

        // Parse lines until finding the next section
        // Next section is determined as the first line that starts with an all-caps word
        while let Ok::<_, ErrMode<()>>((remainder, word)) =
            terminated(not_line_ending, line_ending).parse_next(input)
        {
            if word.chars().all(|c| c.is_ascii_uppercase()) {
                break;
            }
            input = remainder;
        }

        // If we want to fail on unknown sections instead, uncomment this
        // return Err(err::Error::UnknownSection {
        //     section: self.located_parser.span_from_substr(word),
        // });

        // TODO: Track https://github.com/rust-lang/rust/issues/91345 and use .inspect
        warn.iter().for_each(|x| {
            x(err::Error::UnknownSection {
                section: self.located_parser.span_from_substr(word),
            })
        });

        Ok(input)
    }
}
