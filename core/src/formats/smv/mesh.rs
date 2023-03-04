use super::*;

use std::{
    collections::HashMap,
    num::{ParseFloatError, ParseIntError},
};

use miette::{Diagnostic, SourceSpan};
use thiserror::Error;
use winnow::{
    character::space0,
    combinator::{opt, rest},
    IResult, Located, Parser,
};

use super::super::util::{f32, i32, non_ws, u32, usize, word};
use super::util::*;
use crate::{
    geom::{
        Bounds3, Bounds3F, Bounds3I, Dim3D, Surfaces3, Vec2, Vec2F, Vec2I, Vec3, Vec3F, Vec3I,
        Vec3U,
    },
    ws_separated,
};

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
    offset: Vec3F,
    dimensions: Vec3U,
    rgb: Vec3F,
    bounds: Bounds3F,
    obsts: Vec<Obst>,
    trn: Vec3<Vec<f32>>,
    vents: Vec<Vent>,
    circular_vents: Vec<CircularVent>,
}

#[derive(Debug, Error, Diagnostic)]
#[error("oops!")]
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

// /// Checks if the current line matches the given tag or returns a fitting error
// ///
// /// # Arguments
// ///
// /// * `header` - The header of the current section, used for error messages
// /// * `next` - The next function to get the next line
// /// * `tag` - The tag to match
// fn parse_subsection_hdr<'a>(
//     header: SourceSpan,
//     mut input: &'a str,
//     tag: &'static str,
// ) -> Result<(), err::Error> {
//     if let Ok(next_line) = parse_line(&mut input, tag) {
//         if next_line.trim().eq(tag) {
//             Ok(())
//         } else {
//             Err(err::Error::MissingSubSection {
//                 parent: header,
//                 name: tag,
//                 // found: next_line,
//                 found: None,
//             })
//         }
//     } else {
//         Err(err::Error::MissingSubSection {
//             parent: header,
//             name: tag,
//             found: None,
//         })
//     }
// }

impl SimulationParser<'_> {
    /// Parses a single line and matches it against `tag`.
    /// Returns the line if it matches, otherwise returns an error
    /// referencing the found line and the given `section`.
    fn subsection_hdr<'this: 'data, 'data>(
        &'this self,
        tag: &'static str,
        section: &'this str,
    ) -> impl Fn(&'data str) -> Result<(&'data str, &'data str), err::Error> + 'data {
        move |mut input| {
            let line = parse(
                &mut input,
                line(full_line.context(tag)).context("subsection_hdr"),
            )?;
            if line.trim().eq(tag) {
                return Ok((input, line));
            }
            Err(err::Error::MissingSubSection {
                parent: self.located_parser.span_from_substr(section),
                name: tag,
                found: Some(self.located_parser.span_from_substr(line)),
            })
        }
    }

    pub(super) fn parse_mesh<'this: 'data, 'data>(
        &'this self,
        default_texture_origin: Vec3F,
    ) -> impl Fn(&'data str) -> Result<(&str, Mesh), err::Error> {
        move |mut input| {
            let offset = parse_line(&mut input, vec3f)?;

            let (_, mesh_name) = parse_line(&mut input, ws_separated!("GRID", full_line))?;
            let (dimensions, _a) = parse_line(&mut input, ws_separated!(vec3u, i32))?;

            // Capture `self` and `mesh_name`
            let subsection_hdr = |tag| self.subsection_hdr(tag, mesh_name);

            parse_fn(&mut input, subsection_hdr("PDIM"))?;

            let (bounds, rgb) = parse_line(&mut input, ws_separated!(bounds3f, vec3f))?;

            let trn = |dim: Dim3D, name: &'static str| {
                move |mut input| {
                    let header = parse_fn(&mut input, subsection_hdr(name))?;

                    // TODO: Why is this a thing? This is just copied from fdsreader right now but idk why it's there
                    let n = parse_line(&mut input, usize)?;
                    for _ in 0..n {
                        // cast the line to the void
                        let _ = parse_line(&mut input, full_line)?;
                    }

                    let len = dimensions[dim] as usize;
                    let mut vec = Vec::with_capacity(len);

                    for line in 0..=len {
                        let ((i, i_str), v) =
                            parse_line(&mut input, ws_separated!(usize.with_recognized(), f32))?;
                        if i != line {
                            return Err(err::Error::SuspiciousIndex {
                                inside_subsection: self.located_parser.span_from_substr(header),
                                index: self.located_parser.span_from_substr(i_str),
                                expected: line,
                            });
                        }
                        vec.push(v);
                    }

                    Ok((input, vec))
                }
            };

            let trn = Vec3::new(
                parse_fn(&mut input, trn(Dim3D::X, "TRNX"))?,
                parse_fn(&mut input, trn(Dim3D::Y, "TRNY"))?,
                parse_fn(&mut input, trn(Dim3D::Z, "TRNZ"))?,
            );

            parse_fn(&mut input, subsection_hdr("OBST"))?;
            let obsts = parse_fn(&mut input, self.parse_obsts(default_texture_origin))?;

            parse_fn(&mut input, subsection_hdr("VENT"))?;
            let vents = parse_fn(&mut input, self.parse_vents())?;

            parse_fn(&mut input, subsection_hdr("CVENT"))?;
            let circular_vents = parse_fn(&mut input, self.parse_circular_vents())?;

            Ok((
                input,
                Mesh {
                    name: mesh_name.to_string(),
                    offset,
                    dimensions,
                    rgb,
                    bounds,
                    obsts,
                    trn,
                    vents,
                    circular_vents,
                },
            ))
        }
    }

    fn parse_obsts<'this: 'data, 'data>(
        &'this self,
        default_texture_origin: Vec3<f32>,
    ) -> impl Fn(&'data str) -> Result<(&'data str, Vec<Obst>), err::Error> {
        move |mut input| {
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
            let num_obsts = parse_line(&mut input, usize)?;

            let obsts = (0..num_obsts)
                .map(|_| {
                    // The id is signed, but the sign only represents if it's a hole or not
                    // The absolute values are the actual id
                    let id = i32
                        .with_recognized()
                        .map_res(|(x, x_str)| match x.signum() {
                            -1 => Ok((true, x.unsigned_abs())),
                            1 => Ok((false, x.unsigned_abs())),
                            _ => Err(err::Error::UnexpectedObstIdSign {
                                number: self.located_parser.span_from_substr(x_str),
                                signum: x.signum(),
                            }),
                        });

                    // There may be a name appended at the end of the line after a "!"
                    let name = opt(preceded("!", full_line));

                    // The texture origin is optional, if it's not present the default value is used
                    // TODO: As per the TODO above, should this be a global offset or the default value?
                    let texture_origin = opt(vec3f).map(|x| x.unwrap_or(default_texture_origin));

                    // Full line looks like this:
                    // bounds3f (6xf32) id (i32) surfaces3i (6xi32) optional[texture_origin (3xf32)] optional[name (string)]
                    let (bounds, (is_hole, id), side_surfaces, texture_origin, name) = parse_line(
                        &mut input,
                        ws_separated!(bounds3f, id, surfaces3i, texture_origin, name),
                    )?;

                    Ok(HalfObst {
                        bounds,
                        is_hole,
                        id,
                        side_surfaces,
                        texture_origin,
                        name: name.map(str::to_string),
                    })
                })
                .collect::<Result<Vec<_>, err::Error>>()?;

            assert_eq!(obsts.len(), num_obsts);

            let obsts = obsts
                .into_iter()
                .map(|obst| {
                    let rgb = opt(vec3f.with_recognized());
                    let (bounds_idx, (color_index, color_index_str), block_type, rgb) = parse_line(
                        &mut input,
                        ws_separated!(bounds3i, i32.with_recognized(), i32, rgb),
                    )?;

                    if (color_index == -3) != rgb.is_some() {
                        return Err(err::Error::InvalidObstColor {
                            color_index: self.located_parser.span_from_substr(color_index_str),
                            rgb: rgb.map(|(_, x)| self.located_parser.span_from_substr(x)),
                        });
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
                        rgb: rgb.map(|(x, _)| x),
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok((input, obsts))
        }
    }

    fn parse_vents<'this: 'data, 'data>(
        &'this self,
    ) -> impl Fn(&'data str) -> Result<(&'data str, Vec<Vent>), err::Error> {
        |mut input| {
            let (num_vents_total, num_dummies) =
                parse_line(&mut input, ws_separated!(usize, usize))?;
            let num_non_dummies = num_vents_total - num_dummies;

            struct HalfVent {
                bounds: Bounds3F,
                vent_index: i32,
                surface: i32,
                texture_origin: Option<Vec3F>,
            }

            let vents = (0..num_vents_total)
                .map(|vent_line_number: usize| {
                    let texture_origin = opt(vec3f.with_recognized());
                    let ((bounds, vent_index, surface, texture_origin), line) = parse_line(
                        &mut input,
                        ws_separated!(bounds3f, i32, i32, texture_origin).with_recognized(),
                    )?;

                    if (vent_line_number < num_non_dummies) != texture_origin.is_some() {
                        return Err(err::Error::VentTextureOrigin {
                            vent: self.located_parser.span_from_substr(line),
                            num_vents_total,
                            num_non_dummies,
                            vent_line_number,
                            texture_origin: texture_origin
                                .map(|(_, x)| self.located_parser.span_from_substr(x)),
                        });
                    }

                    Ok(HalfVent {
                        bounds,
                        vent_index,
                        surface,
                        texture_origin: texture_origin.map(|(x, _)| x),
                    })
                })
                .collect::<Result<Vec<_>, err::Error>>()?;

            assert_eq!(vents.len(), num_vents_total);

            let vents = vents
                .into_iter()
                .map(|vent| {
                    let rgba = opt(ws_separated!(vec3f, f32));
                    let (bounds_idx, color_index, draw_type, rgba) =
                        parse_line(&mut input, ws_separated!(bounds3i, i32, i32, rgba))?;

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
                .collect::<Result<Vec<_>, err::Error>>()?;

            Ok((input, vents))
        }
    }

    fn parse_circular_vents<'this: 'data, 'data>(
        &'this self,
    ) -> impl Fn(&'data str) -> Result<(&'data str, Vec<CircularVent>), err::Error> {
        move |mut input| {
            let num_vents = parse_line(&mut input, usize)?;

            struct HalfCircularVent {
                bounds: Bounds3F,
                vent_index: i32,
                surface: i32,
                origin: Vec3F,
                radius: f32,
            }

            let vents = (0..num_vents)
                .map(|_| {
                    let (bounds, vent_index, surface, origin, radius) =
                        parse_line(&mut input, ws_separated!(bounds3f, i32, i32, vec3f, f32))?;

                    Ok(HalfCircularVent {
                        bounds,
                        vent_index,
                        surface,
                        origin,
                        radius,
                    })
                })
                .collect::<Result<Vec<_>, err::Error>>()?;

            assert_eq!(vents.len(), num_vents);

            let vents = vents
                .into_iter()
                .map(|vent| {
                    let rgba = opt(ws_separated!(vec3f, f32));
                    let (bounds_idx, color_index, draw_type, rgba) =
                        parse_line(&mut input, ws_separated!(bounds3i, i32, i32, rgba))?;

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
                .collect::<Result<Vec<_>, err::Error>>()?;

            Ok((input, vents))
        }
    }
}

// pub(super) fn parse_mesh<'a, Src: FnMut() -> Result<Located<&'a str>, err::Error>>(
//     header: Located<&'a str>,
//     default_texture_origin: Vec3F,
//     mut next: Src,
// ) -> Result<Mesh, err::Error> {
//     let (_, mesh_name) = parse!(header => "GRID" full_line_string)?;
//     let header: SourceSpan = rest.span().parse_next(header)?.1.into();
//     let (dimensions, _a) = parse!(next()? => vec3u i32)?;

//     parse_subsection_hdr(header, &mut next, "PDIM")?;
//     let (bounds, _something) = parse!(next()? => bounds3f vec2f)?;

//     let parse_trn = |mut next: &mut Src, dim: Dim3D| {
//         // TODO: I'm not too fond of hardcoding the dimension names like this
//         parse_subsection_hdr(header, &mut next, ["TRNX", "TRNY", "TRNZ"][dim as usize])?;

//         // TODO: Why is this a thing? This is just copied from fdsreader right now but idk why it's there
//         let n = parse!(next()? => usize)?;
//         for _ in 0..n {
//             let _ = next()?;
//         }

//         repeat_n(
//             &mut next,
//             |next, line| {
//                 let next = next()?;
//                 let (i, v) = parse!(next => usize f32)?;
//                 if i != line {
//                     return Err(err(
//                         todo!(),
//                         // next.split_whitespace().next().unwrap_or(next),
//                         SupErrorKind::MismatchedIndex {
//                             expected: line,
//                             got: i,
//                         },
//                     ));
//                 }
//                 Ok(v)
//             },
//             dimensions[dim] as usize,
//         )
//     };

//     let trn = Vec3::new(
//         parse_trn(&mut next, Dim3D::X)?,
//         parse_trn(&mut next, Dim3D::Y)?,
//         parse_trn(&mut next, Dim3D::Z)?,
//     );

//     parse_subsection_hdr(header, &mut next, "OBST")?;
//     let obsts = parse_obsts(&mut next, default_texture_origin)?;

//     parse_subsection_hdr(header, &mut next, "VENT")?;
//     let vents = parse_vents(&mut next)?;

//     parse_subsection_hdr(header, &mut next, "CVENT")?;
//     let circular_vents = parse_circular_vents(&mut next)?;

//     // TODO: fdsreader doesn't parse this, but it's in the .smv file I'm referencing
//     parse_subsection_hdr(header, &mut next, "OFFSET")?;
//     let _offset = parse!(next()? => vec3f)?;

//     Ok(Mesh {
//         name: mesh_name,
//         dimensions,
//         bounds,
//         trn,
//         obsts,
//         vents,
//         circular_vents,
//     })
// }
