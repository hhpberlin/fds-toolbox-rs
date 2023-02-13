use nom::{IResult, combinator::map, sequence::tuple, Parser};

use crate::geom::{Vec3F, Vec3I, Vec3U, Vec2F, Vec2I, Vec2, Vec3, Surfaces3, Bounds3, Bounds3F, Bounds3I};

use super::{err, ErrorKind, Error, super::util::{non_ws, from_str_ws_preceded}};

/// Convenience macro for parsing to omit tuple() and similar boilerplate
#[macro_export]
macro_rules! parse {
    ($i:expr => $t:tt $($tt:tt)+) => { parse($i, tuple((parse!(impl $t), $(parse!(impl $tt)),+))) };
    ($i:expr => $t:tt) => { parse($i, parse!(impl $t)) };
    // Implementation detail
    (impl $i:ident) => { $i };
    (impl $t:tt) => { preceded(ws, tag($t)) };
}

macro_rules! from_str_impl {
    ($($t:ident),+) => {
        $(pub fn $t(i: &str) -> IResult<&str, $t> {
            from_str_ws_preceded(i)
        })+
    };
}

from_str_impl!(f32, i32, u32, usize);

pub fn vec3f(i: &str) -> IResult<&str, Vec3F> {
    map(tuple((f32, f32, f32)), Vec3::from)(i)
}
pub fn vec3i(i: &str) -> IResult<&str, Vec3I> {
    map(tuple((i32, i32, i32)), Vec3::from)(i)
}
pub fn vec3u(i: &str) -> IResult<&str, Vec3U> {
    map(tuple((u32, u32, u32)), Vec3::from)(i)
}

pub fn vec2f(i: &str) -> IResult<&str, Vec2F> {
    map(tuple((f32, f32)), Vec2::from)(i)
}
pub fn vec2i(i: &str) -> IResult<&str, Vec2I> {
    map(tuple((i32, i32)), Vec2::from)(i)
}

pub fn surfaces3i(i: &str) -> IResult<&str, Surfaces3<i32>> {
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

pub fn bounds3<T>(i: &str, parser: impl Fn(&str) -> IResult<&str, T>) -> IResult<&str, Bounds3<T>> {
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

pub fn bounds3f(i: &str) -> IResult<&str, Bounds3F> {
    bounds3(i, f32)
}
pub fn bounds3i(i: &str) -> IResult<&str, Bounds3I> {
    bounds3(i, i32)
}

pub fn string(i: &str) -> IResult<&str, String> {
    map(non_ws, |s| s.to_string())(i)
}

pub fn full_line_string(i: &str) -> IResult<&str, String> {
    let string = i.trim().to_string();
    // Take empty subslice at the end of the string
    // this makes sure the pointer still points into the original string
    // incase we want to use it for error reporting
    Ok((&i[i.len()..], string))
}

pub(super) fn match_tag<'a>(i: &'a str, tag: &'a str, error: Error<'a>) -> Result<(), Error<'a>> {
    if i.trim().eq(tag) {
        Ok(())
    } else {
        Err(error)
    }
}

pub(super) fn parse<'a, T, E>(i: &'a str, mut parser: impl Parser<&'a str, T, E>) -> Result<T, Error<'a>>
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

pub(super) fn repeat<'a, T, Src: FnMut() -> Result<&'a str, Error<'a>>>(
    mut src: Src,
    parse: impl Fn(&mut Src, usize) -> Result<T, Error<'a>>,
) -> Result<Vec<T>, Error<'a>> {
    let n = parse!(src()? => usize)?;
    repeat_n(src, parse, n)
}

pub(super) fn repeat_n<'a, T, Src: FnMut() -> Result<&'a str, Error<'a>>>(
    mut src: Src,
    parse: impl Fn(&mut Src, usize) -> Result<T, Error<'a>>,
    n: usize,
) -> Result<Vec<T>, Error<'a>> {
    (0..n).map(|i| parse(&mut src, i)).collect()
}