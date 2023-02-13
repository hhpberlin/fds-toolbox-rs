use super::{util::*, err, Error, ErrorKind as SupErrorKind};

use std::{
    collections::HashMap,
    num::{ParseFloatError, ParseIntError},
};

use nom::{
    bytes::complete::{is_not, tag},
    combinator::{map, opt, success, map_res},
    sequence::tuple,
    IResult, Parser,
};

use super::super::util::{from_str_ws_preceded, non_ws, ws};
use crate::{
    geom::{Bounds3, Bounds3F, Bounds3I, Surfaces3, Vec2, Vec2F, Vec2I, Vec3, Vec3F, Vec3I, Vec3U, Dim3D},
    parse,
};
use nom::sequence::preceded;

#[derive(Debug)]
pub struct Obst {
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
pub struct CircularVent {
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
pub struct Mesh {
    name: String,
    dimensions: Vec3U,
    bounds: Bounds3F,
    obsts: Vec<Obst>,
    trn: Vec3<Vec<f32>>,
    vents: Vec<Vent>,
    circular_vents: Vec<CircularVent>,
}

#[derive(Debug)]
pub enum ErrorKind {
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

pub(super) fn parse_mesh<'a, Src: FnMut() -> Result<&'a str, Error<'a>>>(
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
                        SupErrorKind::MismatchedIndex {
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
                    SupErrorKind::Mesh(ErrorKind::UnexpectedObstIdSign(x)),
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
                    SupErrorKind::Mesh(ErrorKind::InvalidObstColor { color_index, rgb }),
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
                    SupErrorKind::Mesh(ErrorKind::InvalidVentTextureOrigin {
                        i,
                        num_vents,
                        num_dummies,
                        texture_origin,
                    }),
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
