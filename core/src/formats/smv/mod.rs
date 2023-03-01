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
    error::{ErrMode, ParseError, ContextError},
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

fn parse<'a, I, O, E>(input: &'a mut I, parser: impl Parser<I, O, E>) -> Result<O, ErrMode<E>> {
    let (remaining, value) = parser.parse_next(*input)?;
    *input = remaining;
    Ok(value)
}

fn parse_line<'a, O, E: ParseError<&'a str> + ContextError<&'a str>>(
    input: &'a mut &str,
    parser: impl Parser<&'a str, O, E>,
) -> Result<O, ErrMode<E>> {
    parse(input, terminated(parser, line_ending).context("line"))
}

/// Parses an entire line, but leaves the line ending in the input.
///
/// ```
/// assert_eq!(full_line.parse_next("lorem\nipsum"), Ok(("ipsum", "\nlorem")));
/// ```
fn full_line(i: &str) -> IResult<&str, &str> {
    take_till1(|c| c == '\r' || c == '\n').parse_next(i)
}

impl SimulationParser<'_> {
    fn parse<'a>(&self) -> Result<Simulation, err::> {
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
            let (input, addendum) =
                terminated(alt((full_line.map(Some), success(None))), line_ending)
                    .parse_next(input)?;

            match (input, addendum) {
                ("TITLE", None) => title = Some(parse_line(&mut input, full_line)),
            }
        }

        Ok(Simulation {
            title: title.ok_or(err::Error::MissingSection { name: "TITLE" })?,
            fds_version: fds_version.ok_or(err::Error::MissingSection { name: "FDSVERSION" })?,
            end_version: end_file.ok_or(err::Error::MissingSection { name: "ENDF" })?,
            input_file: input_file.ok_or(err::Error::MissingSection { name: "INPF" })?,
            revision: revision.ok_or(err::Error::MissingSection { name: "REVISION" })?,
            chid: chid.ok_or(err::Error::MissingSection { name: "CHID" })?,
            solid_ht3d: solid_ht3d.ok_or(err::Error::MissingSection { name: "SOLID_HT3D" })?,
        })
    }
}
