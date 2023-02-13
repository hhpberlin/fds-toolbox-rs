//pub type Input<'a> = &'a [u8];
//pub type Result<'a, T> = nom::IResult<Input<'a>, T, ()>;

use std::{
    collections::HashMap,
    num::{ParseFloatError, ParseIntError},
};

use nom::{
    bytes::complete::{is_not, tag},
    combinator::{map, map_res, opt},
    sequence::tuple,
    IResult, Parser,
};

use super::util::{from_str_ws_preceded, non_ws, ws};
use crate::geom::{
    Bounds3, Bounds3F, Bounds3I, Dim3D, Surfaces3, Vec2, Vec2F, Vec2I, Vec3, Vec3F, Vec3I, Vec3U,
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
struct Obst {
    name: Option<String>,
    id: u32,
    is_hole: bool,
    bounds: Bounds3F,
    texture_origin: Vec3F,
    // TODO: Map to actual surface type
    side_surfaces: Surfaces3<i32>,
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
    WrongNumberOfValues { expected: usize, got: usize },
    TrailingCharacters,
    UnknownSection,
    MismatchedIndex { expected: usize, got: usize },
    UnexpectedMeshIdSign(i32),
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
    map(tuple((f32, f32, f32)), Vec3::from)(i)
}
fn vec3i(i: &str) -> IResult<&str, Vec3I> {
    map(tuple((i32, i32, i32)), Vec3::from)(i)
}
fn vec3u(i: &str) -> IResult<&str, Vec3U> {
    map(tuple((u32, u32, u32)), Vec3::from)(i)
}

fn vec2f(i: &str) -> IResult<&str, Vec2F> {
    map(tuple((f32, f32)), Vec2::from)(i)
}
fn vec2i(i: &str) -> IResult<&str, Vec2I> {
    map(tuple((i32, i32)), Vec2::from)(i)
}

fn surfaces3i(i: &str) -> IResult<&str, Surfaces3<i32>> {
    map(
        tuple((i32, i32, i32, i32, i32, i32)),
        |(neg_x, pos_x, neg_y, pos_y, neg_z, pos_z)| Surfaces3 {
            neg_x,
            pos_x,
            neg_y,
            pos_y,
            neg_z,
            pos_z,
        },
    )(i)
}

fn bounds3<T>(i: &str, parser: impl Fn(&str) -> IResult<&str, T>) -> IResult<&str, Bounds3<T>> {
    map(
        tuple((&parser, &parser, &parser, &parser, &parser, &parser)),
        |(min_x, max_x, min_y, max_y, min_z, max_z)| {
            Bounds3::new(
                Vec3::new(min_x, min_y, min_z),
                Vec3::new(max_x, max_y, max_z),
            )
        },
    )(i)
}

fn bounds3f(i: &str) -> IResult<&str, Bounds3F> {
    bounds3(i, f32)
}
fn bounds3i(i: &str) -> IResult<&str, Bounds3I> {
    bounds3(i, i32)
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

fn match_tag<'a>(i: &'a str, tag: &'a str, error: Error<'a>) -> Result<(), Error<'a>> {
    if i.trim().eq(tag) {
        Ok(())
    } else {
        Err(error)
    }
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
    parse: impl Fn(&mut Src, usize) -> Result<T, Error<'a>>,
) -> Result<Vec<T>, Error<'a>> {
    let n = parse!(src()? => usize)?;
    repeat_n(src, parse, n)
}

fn repeat_n<'a, T, Src: FnMut() -> Result<&'a str, Error<'a>>>(
    mut src: Src,
    parse: impl Fn(&mut Src, usize) -> Result<T, Error<'a>>,
    n: usize,
) -> Result<Vec<T>, Error<'a>> {
    (0..n).map(|i| parse(&mut src, i)).collect()
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

    /// Checks if the current line matches the given tag or returns a fitting error
    /// 
    /// # Arguments
    /// 
    /// * `header` - The header of the current section, used for error messages
    /// * `next` - The next function to get the next line
    /// * `tag` - The tag to match
    fn parse_subsection<'a, Src: FnMut() -> Result<&'a str, Error<'a>>>(
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
    ) -> Result<(), Error<'a>> {
        let (_, _mesh_name) = parse!(header => "GRID" full_line_string)?;
        let (dimensions, _a) = parse!(next()? => vec3u i32)?;

        Self::parse_subsection(header, &mut next, "PDIM")?;
        let (bounds, _something) = parse!(next()? => bounds3f vec2f)?;

        let parse_trn = |mut next: &mut Src, dim: Dim3D| {
            // TODO: I'm not too fond of hardcoding the dimension names like this
            Self::parse_subsection(header, &mut next, ["TRNX", "TRNY", "TRNZ"][dim as usize])?;

            // TODO: Why is this a thing? This is just copied from fdsreader right now but idk why it's there
            let n = parse!(next()? => usize)?;
            for _ in 0..n {
                let _ = next()?;
            }

            repeat_n(&mut next, |next, line| {
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
            }, dimensions[dim] as usize)
        };

        let trn = Vec3::new(
            parse_trn(&mut next, Dim3D::X)?,
            parse_trn(&mut next, Dim3D::Y)?,
            parse_trn(&mut next, Dim3D::Z)?,
        );

        Self::parse_subsection(header, &mut next, "OBST")?;
        let obsts = repeat(&mut next, |next, _| {
            let next = next()?;

            // The id is signed, but the sign only represents if it's a hole or not
            // The absolute values are the actual id
            let id = map_res(i32, |x| match x.signum() {
                -1 => Ok((true, x.unsigned_abs())),
                1 => Ok((false, x.unsigned_abs())),
                _ => Err(err(
                    // TODO: I don't like this, spans should be tracked more nicely
                    next.split_whitespace().nth(6).unwrap_or(next),
                    ErrorKind::UnexpectedMeshIdSign(x),
                )),
            });

            // There may be a name appended at the end of the line after a "!"
            let name = opt(map(tuple((ws, tag("!"), full_line_string)), |(_, _, x)| x));

            // The texture origin is optional, if it's not present the default value is used
            // TODO: As per the TODO above, should this be a global offset or the default value?
            let texture_origin = map(opt(vec3f), |x| x.unwrap_or(default_texture_origin));

            let (bounds, (is_hole, id), side_surfaces, texture_origin, name) =
                parse!(next => bounds3f id surfaces3i texture_origin name)?;

            Ok(Obst {
                bounds,
                is_hole,
                id,
                side_surfaces,
                texture_origin,
                name,
            })
        });

        Ok(())
    }
}
