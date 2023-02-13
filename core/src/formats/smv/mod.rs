//pub type Input<'a> = &'a [u8];
//pub type Result<'a, T> = nom::IResult<Input<'a>, T, ()>;

mod util;
use util::*;

use std::{
    collections::HashMap,
    num::{ParseFloatError, ParseIntError},
};

use nom::{
    bytes::complete::{is_not, tag},
    combinator::{map, map_res, opt, success},
    sequence::tuple,
    IResult, Parser,
};

use super::util::{from_str_ws_preceded, non_ws, ws};
use crate::{geom::{
    Bounds3, Bounds3F, Bounds3I, Dim3D, Surfaces3, Vec2, Vec2F, Vec2I, Vec3, Vec3F, Vec3I, Vec3U,
}, parse};
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
struct Obst {
    name: Option<String>,
    id: u32,
    is_hole: bool,
    bounds: Bounds3F,
    texture_origin: Vec3F,
    // TODO: Map to actual surface type
    side_surfaces: Surfaces3<i32>,
    bounds_idx: Bounds3I,
    // TODO: Introduce rgb struct?
    rgb: Option<Vec3F>,
    color_index: i32,
    block_type: i32,
}

#[derive(Debug)]
struct Vent {
    bounds: Bounds3F,
    vent_index: i32,
    surface: i32,
    texture_origin: Option<Vec3F>,
    bounds_idx: Bounds3I,
    color_index: i32,
    draw_type: i32,
    // TODO: Introduce 4D vector or rgba struct?
    rgba: Option<(Vec3F, f32)>,
}

#[derive(Debug)]
struct CircularVent {
    bounds: Bounds3F,
    vent_index: i32,
    surface: i32,
    origin: Vec3F,
    radius: f32,
    bounds_idx: Bounds3I,
    color_index: i32,
    draw_type: i32,
    rgba: Option<(Vec3F, f32)>,
}

#[derive(Debug)]
struct Mesh {
    name: String,
    dimensions: Vec3U,
    bounds: Bounds3F,
    obsts: Vec<Obst>,
    trn: Vec3<Vec<f32>>,
    vents: Vec<Vent>,
    circular_vents: Vec<CircularVent>,
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
    // An obst with an id of 0 was given (or .signum() returned a value other than -1, 0, or 1)
    UnexpectedObstIdSign(i32),
    // Rgb should only be given iff color_index == -3
    InvalidObstColor {
        color_index: i32,
        rgb: Option<Vec3F>,
    },
    InvalidVentTextureOrigin {
        i: usize,
        num_vents: usize,
        num_dummies: usize,
        texture_origin: Option<Vec3F>,
    },
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
        let mut offset = None;

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
                "OFFSET" => offset = Some(parse!(next()? => vec3f)?),
                line if line.starts_with("GRID") => {
                    todo!();
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

/// Checks if the current line matches the given tag or returns a fitting error
///
/// # Arguments
///
/// * `header` - The header of the current section, used for error messages
/// * `next` - The next function to get the next line
/// * `tag` - The tag to match
fn parse_subsection_hdr<'a, Src: FnMut() -> Result<&'a str, Error<'a>>>(
    header: &'a str,
    mut next: Src,
    tag: &'static str,
) -> Result<(), Error<'a>> {
    let err = Error::MissingSubSection {
        parent: header,
        name: tag,
    };
    match next() {
        Ok(next_line) => match_tag(next_line, tag, err),
        Err(_) => Err(err),
    }
}

fn parse_mesh<'a, Src: FnMut() -> Result<&'a str, Error<'a>>>(
    header: &'a str,
    default_texture_origin: Vec3F,
    mut next: Src,
) -> Result<Mesh, Error<'a>> {
    let (_, mesh_name) = parse!(header => "GRID" full_line_string)?;
    let (dimensions, _a) = parse!(next()? => vec3u i32)?;

    parse_subsection_hdr(header, &mut next, "PDIM")?;
    let (bounds, _something) = parse!(next()? => bounds3f vec2f)?;

    let parse_trn = |mut next: &mut Src, dim: Dim3D| {
        // TODO: I'm not too fond of hardcoding the dimension names like this
        parse_subsection_hdr(header, &mut next, ["TRNX", "TRNY", "TRNZ"][dim as usize])?;

        // TODO: Why is this a thing? This is just copied from fdsreader right now but idk why it's there
        let n = parse!(next()? => usize)?;
        for _ in 0..n {
            let _ = next()?;
        }

        repeat_n(
            &mut next,
            |next, line| {
                let next = next()?;
                let (i, v) = parse!(next => usize f32)?;
                if i != line {
                    return Err(err(
                        next.split_whitespace().next().unwrap_or(next),
                        ErrorKind::MismatchedIndex {
                            expected: line,
                            got: i,
                        },
                    ));
                }
                Ok(v)
            },
            dimensions[dim] as usize,
        )
    };

    let trn = Vec3::new(
        parse_trn(&mut next, Dim3D::X)?,
        parse_trn(&mut next, Dim3D::Y)?,
        parse_trn(&mut next, Dim3D::Z)?,
    );

    parse_subsection_hdr(header, &mut next, "OBST")?;
    let obsts = parse_obsts(&mut next, default_texture_origin)?;

    parse_subsection_hdr(header, &mut next, "VENT")?;
    let vents = parse_vents(&mut next)?;

    parse_subsection_hdr(header, &mut next, "CVENT")?;
    let circular_vents = parse_circular_vents(&mut next)?;

    // TODO: fdsreader doesn't parse this, but it's in the .smv file I'm referencing
    parse_subsection_hdr(header, &mut next, "OFFSET")?;
    let _offset = parse!(next()? => vec3f)?;

    Ok(Mesh {
        name: mesh_name,
        dimensions,
        bounds,
        trn,
        obsts,
        vents,
        circular_vents,
    })
}

fn parse_obsts<'a, Src: FnMut() -> Result<&'a str, Error<'a>>>(
    mut next: Src,
    default_texture_origin: Vec3<f32>,
) -> Result<Vec<Obst>, Error<'a>> {
    // Stores obstacles as they are defined in the first half
    // Since obstacles are defined like this:
    //
    // OBST
    //  2        number of obstacles
    //  1.2 ...  obstacle 1
    //  2.3 ...  obstacle 2
    //  1 2 ...  more info about obstacle 1
    //  3 4 ...  more info about obstacle 2
    //
    struct HalfObst {
        name: Option<String>,
        id: u32,
        is_hole: bool,
        bounds: Bounds3F,
        texture_origin: Vec3F,
        // TODO: Map to actual surface type
        side_surfaces: Surfaces3<i32>,
    }
    let num_obsts = parse!(next()? => usize)?;
    let obsts = repeat_n(
        &mut next,
        |next, _| {
            let next = next()?;

            // The id is signed, but the sign only represents if it's a hole or not
            // The absolute values are the actual id
            let id = map_res(i32, |x| match x.signum() {
                -1 => Ok((true, x.unsigned_abs())),
                1 => Ok((false, x.unsigned_abs())),
                _ => Err(err(
                    // TODO: I don't like this, spans should be tracked more nicely
                    next.split_whitespace().nth(6).unwrap_or(next),
                    ErrorKind::UnexpectedObstIdSign(x),
                )),
            });

            // There may be a name appended at the end of the line after a "!"
            let name = opt(map(tuple((ws, tag("!"), full_line_string)), |(_, _, x)| x));

            // The texture origin is optional, if it's not present the default value is used
            // TODO: As per the TODO above, should this be a global offset or the default value?
            let texture_origin = map(opt(vec3f), |x| x.unwrap_or(default_texture_origin));

            // Full line looks like this:
            // bounds3f (6xf32) id (i32) surfaces3i (6xi32) optional[texture_origin (3xf32)] optional[name (string)]
            let (bounds, (is_hole, id), side_surfaces, texture_origin, name) =
                parse!(next => bounds3f id surfaces3i texture_origin name)?;

            Ok(HalfObst {
                bounds,
                is_hole,
                id,
                side_surfaces,
                texture_origin,
                name,
            })
        },
        num_obsts,
    )?;

    assert_eq!(obsts.len(), num_obsts);

    let obsts = obsts
        .into_iter()
        .map(|obst| {
            let next = next()?;
            let rgb = opt(vec3f);
            let (bounds_idx, color_index, block_type, rgb) = parse!(next => bounds3i i32 i32 rgb)?;

            if (color_index == -3) != rgb.is_some() {
                return Err(err(
                    // TODO: This just passes the whole line
                    next,
                    ErrorKind::InvalidObstColor { color_index, rgb },
                ));
            }

            Ok(Obst {
                bounds: obst.bounds,
                id: obst.id,
                is_hole: obst.is_hole,
                side_surfaces: obst.side_surfaces,
                texture_origin: obst.texture_origin,
                name: obst.name,
                bounds_idx,
                color_index,
                block_type,
                rgb,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(obsts)
}

fn parse_vents<'a, Src: FnMut() -> Result<&'a str, Error<'a>>>(
    mut next: Src,
) -> Result<Vec<Vent>, Error<'a>> {
    let (num_vents, num_dummies) = parse!(next()? => usize usize)?;
    let num_non_dummies = num_vents - num_dummies;

    struct HalfVent {
        bounds: Bounds3F,
        vent_index: i32,
        surface: i32,
        texture_origin: Option<Vec3F>,
    }

    let vents = repeat_n(
        &mut next,
        |next, i| {
            let next = next()?;
            let texture_origin = opt(vec3f);
            let (bounds, vent_index, surface, texture_origin) =
                parse!(next => bounds3f i32 i32 texture_origin)?;

            if (i < num_non_dummies) != texture_origin.is_some() {
                return Err(err(
                    next,
                    ErrorKind::InvalidVentTextureOrigin {
                        i,
                        num_vents,
                        num_dummies,
                        texture_origin,
                    },
                ));
            }

            Ok(HalfVent {
                bounds,
                vent_index,
                surface,
                texture_origin,
            })
        },
        num_vents,
    )?;

    assert_eq!(vents.len(), num_vents);

    let vents = vents
        .into_iter()
        .map(|vent| {
            let next = next()?;
            let rgba = opt(tuple((vec3f, f32)));
            let (bounds_idx, color_index, draw_type, rgba) = parse!(next => bounds3i i32 i32 rgba)?;

            Ok(Vent {
                bounds: vent.bounds,
                vent_index: vent.vent_index,
                surface: vent.surface,
                texture_origin: vent.texture_origin,
                bounds_idx,
                color_index,
                draw_type,
                rgba,
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;

    Ok(vents)
}

fn parse_circular_vents<'a, Src: FnMut() -> Result<&'a str, Error<'a>>>(
    mut next: Src,
) -> Result<Vec<CircularVent>, Error<'a>> {
    let num_vents = parse!(next()? => usize)?;

    struct HalfCircularVent {
        bounds: Bounds3F,
        vent_index: i32,
        surface: i32,
        origin: Vec3F,
        radius: f32,
    }

    let vents = repeat_n(
        &mut next,
        |next, _| {
            let next = next()?;
            let (bounds, vent_index, surface, origin, radius) =
                parse!(next => bounds3f i32 i32 vec3f f32)?;

            Ok(HalfCircularVent {
                bounds,
                vent_index,
                surface,
                origin,
                radius,
            })
        },
        num_vents,
    )?;

    assert_eq!(vents.len(), num_vents);

    let vents = vents
        .into_iter()
        .map(|vent| {
            let next = next()?;
            let rgba = opt(tuple((vec3f, f32)));
            let (bounds_idx, color_index, draw_type, rgba) = parse!(next => bounds3i i32 i32 rgba)?;

            Ok(CircularVent {
                bounds: vent.bounds,
                vent_index: vent.vent_index,
                surface: vent.surface,
                origin: vent.origin,
                radius: vent.radius,
                bounds_idx,
                color_index,
                draw_type,
                rgba,
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;

    Ok(vents)
}
