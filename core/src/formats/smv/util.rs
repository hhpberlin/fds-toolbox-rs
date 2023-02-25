use winnow::{
    sequence::{preceded, terminated},
    stream::{AsChar, Stream, StreamIsPartial},
    IResult, Parser, character::space0,
};

use crate::geom::{
    Bounds3, Bounds3F, Bounds3I, Surfaces3, Vec2F, Vec2I, Vec2U, Vec3, Vec3F, Vec3I, Vec3U,
};

use super::{
    super::util::{f32, i32, non_ws, u32, usize, word},
    err, Error, ErrorKind,
};

/// Convenience macro for parsing to omit tuple() and similar boilerplate
#[macro_export]
macro_rules! parse {
    ($i:expr => $($t:expr)+) => { parse($i, ws_separated!($($t),+)) };
}

#[macro_export]
macro_rules! ws_separated {
    ($($t:expr),+) => {
        winnow::sequence::terminated(($(winnow::sequence::preceded(winnow::character::space0, $t)),+), winnow::character::space0)
    };
}

macro_rules! impl_from {
    ($name:ident ( $($t:expr),+ ) -> $ret:ty { $e:expr }) => {
        pub fn $name<I>(i: I) -> IResult<I, $ret>
        where
            I: StreamIsPartial + Stream,
            <I as Stream>::Token: AsChar,
        {
            ws_separated!($($t),+)
            .map($e)
            .parse_next(i)
        }
    };
    ($name:ident ( $($t:expr),+ ) -> $ret:ident) => {
        impl_from!($name ( $($t),+ ) -> $ret { $ret::from });
    };
}

impl_from!(vec3f(f32, f32, f32) -> Vec3F);
impl_from!(vec3i(i32, i32, i32) -> Vec3I);
impl_from!(vec3u(u32, u32, u32) -> Vec3U);

impl_from!(vec2f(f32, f32) -> Vec2F);
impl_from!(vec2i(i32, i32) -> Vec2I);
impl_from!(vec2u(u32, u32) -> Vec2U);

impl_from!(surfaces3i(i32, i32, i32, i32, i32, i32) -> Surfaces3<i32> {
    |(neg_x, pos_x, neg_y, pos_y, neg_z, pos_z)| Surfaces3 {
        neg_x,
        pos_x,
        neg_y,
        pos_y,
        neg_z,
        pos_z,
    }
});

impl_from!(bounds3f(i32, i32, i32, i32, i32, i32) -> Bounds3I { Bounds3::from_fds_notation_tuple });
impl_from!(bounds3i(f32, f32, f32, f32, f32, f32) -> Bounds3F { Bounds3::from_fds_notation_tuple });

pub fn string<I>(i: I) -> IResult<I, String>
where
    I: StreamIsPartial + Stream,
    <I as Stream>::Token: AsChar,
{
    non_ws.map(|s: char| s.to_string()).parse_next(i)
}

pub fn full_line_str<I>(i: I) -> IResult<I, I::Slice>
where
    I: StreamIsPartial + Stream,
    <I as Stream>::Token: AsChar,
{
    let string = i.trim();
    // Take empty subslice at the end of the string
    // this makes sure the pointer still points into the original string
    // incase we want to use it for error reporting
    Ok((&i[i.len()..], string))
}

pub fn full_line_string<I>(i: I) -> IResult<I, String>
where
    I: StreamIsPartial + Stream,
    <I as Stream>::Token: AsChar,
{
    full_line_string.map(|s| s.to_string()).parse_next(i)
}

pub(super) fn match_tag<'a>(i: &'a str, tag: &'a str, error: Error) -> Result<(), Error> {
    if i.trim().eq(tag) {
        Ok(())
    } else {
        Err(error)
    }
}

pub(super) fn parse<'a, T, E>(
    src: &'a str,
    i: &'a str,
    mut parser: impl Parser<&'a str, T, E>,
) -> Result<T, Error>
where
    Error: From<winnow::Err<E>>,
{
    let (i, o) = parser.parse(i)?;

    if i.is_empty() {
        Ok(o)
    } else {
        Err(err(src, i, ErrorKind::TrailingCharacters))
    }
}

pub(super) fn repeat<'a, T, Src: FnMut() -> Result<&'a str, Error>>(
    mut src: Src,
    parse: impl Fn(&mut Src, usize) -> Result<T, Error>,
) -> Result<Vec<T>, Error> {
    let n = parse!(src()? => usize)?;
    repeat_n(src, parse, n)
}

pub(super) fn repeat_n<'a, T, Src: FnMut() -> Result<&'a str, Error>>(
    mut src: Src,
    parse: impl Fn(&mut Src, usize) -> Result<T, Error>,
    n: usize,
) -> Result<Vec<T>, Error> {
    (0..n).map(|i| parse(&mut src, i)).collect()
}

/// Checks if the current line matches the given tag or returns a fitting error
///
/// # Arguments
///
/// * `header` - The header of the current section, used for error messages
/// * `next` - The next function to get the next line
/// * `tag` - The tag to match
pub(super) fn parse_subsection_hdr<'a, Src: FnMut() -> Result<&'a str, Error>>(
    header: &'a str,
    mut next: Src,
    tag: &'static str,
) -> Result<(), Error> {
    let err = Error::MissingSubSection {
        parent: header,
        name: tag,
    };
    match next() {
        Ok(next_line) => match_tag(next_line, tag, err),
        Err(_) => Err(err),
    }
}
