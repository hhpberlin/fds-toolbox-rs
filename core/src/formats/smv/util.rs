use miette::SourceSpan;
use winnow::{
    branch::alt,
    character::{not_line_ending, space0},
    sequence::{preceded, terminated, tuple},
    stream::{AsBStr, AsChar, Compare, Location, Stream, StreamIsPartial},
    IResult, Located, Parser,
};

use crate::geom::{
    Bounds3, Bounds3F, Bounds3I, Surfaces3, Vec2F, Vec2I, Vec2U, Vec3, Vec3F, Vec3I, Vec3U,
};

use super::{
    super::util::{f32, i32, non_ws, u32, usize, word},
    err,
    err::Error,
    err::ErrorKind,
};

// /// Convenience macro for parsing to omit tuple() and similar boilerplate
// #[macro_export]
// macro_rules! parse {
//     ($i:expr => $($t:expr)+) => { parse($i, ws_separated!($($t),+)) };
// }

#[macro_export]
macro_rules! ws_separated {
    // (@step $(lhs:expr),* ; $head:expr, $($tail:expr),*) =>
    // ($($t:expr),+ ; $($t_copy:expr),+) => {
    //     let res = winnow::sequence::preceded(
    //         winnow::character::space0, 
    //         $t.context(concat!("ws_separated.", i, stringify!($t)))
    //     );
    // },
    ($($t:expr),+) => {
        {
            winnow::sequence::terminated(
                (
                    $(
                        winnow::sequence::preceded(
                            winnow::character::space0, 
                            $t
                                // .context(concat!("ws_separated.", stringify!($t)))
                        )
                    ),+
                ),
                winnow::character::space0)
                .context(concat!("ws_separated!(", stringify!($($t),+), ")"))
        }
    };
}

#[macro_export]
macro_rules! trace_callsite {
    ($t:expr) => {
        $t.context(concat!(file!(), ":", line!(), ":", column!()))
    };
}

// fn ws_sep(l: impl List) {
//     tuple()
// }

macro_rules! impl_from {
    ($name:ident ( $($t:expr),+ ) -> $ret:ty { $e:expr }) => {
        pub fn $name<I>(i: I) -> IResult<I, $ret>
        where
            I: StreamIsPartial + Stream,
            <I as Stream>::Token: AsChar,
            <I as Stream>::Slice: AsRef<str>,
        {
            ws_separated!($($t),+)
                .context(stringify!($name))
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

impl_from!(bounds3i(i32, i32, i32, i32, i32, i32) -> Bounds3I { Bounds3::from_fds_notation_tuple });
impl_from!(bounds3f(f32, f32, f32, f32, f32, f32) -> Bounds3F { Bounds3::from_fds_notation_tuple });

// pub fn string<I>(i: I) -> IResult<I, String>
// where
//     I: StreamIsPartial + Stream,
//     <I as Stream>::Token: AsChar,
//     <I as Stream>::Slice: AsRef<str>,
// {
//     non_ws
//         .map(|s: I::Slice| s.as_ref().to_string())
//         .parse_next(i)
// }

// pub fn full_line_str<'a, I>(i: I) -> IResult<I, &'a str>
// where
//     I: StreamIsPartial + Stream + 'a,
//     I: Compare<&'static str> + AsBStr,
//     <I as Stream>::Token: AsChar,
//     <I as Stream>::Slice: AsRef<str>,
// {
//     not_line_ending
//         .map(|s: I::Slice| s.as_ref().trim())
//         .parse_next(i)
// }

// pub fn full_line_string<I>(i: I) -> IResult<I, String>
// where
//     I: StreamIsPartial + Stream,
//     <I as Stream>::Token: AsChar,
//     <I as Stream>::Slice: AsRef<str>,
// {
//     full_line_str.map(|s| s.to_string()).parse_next(i)
// }

// pub(super) fn match_tag<'a>(i: &'a str, tag: &'a str, error: Error) -> Result<(), Error> {
//     if i.trim().eq(tag) {
//         Ok(())
//     } else {
//         Err(error)
//     }
// }

// pub(super) fn parse<'a, I, T, E>(i: I, mut parser: impl Parser<I, T, E>) -> Result<T, Error>
// where
//     I: StreamIsPartial + Stream + Location,
//     <I as Stream>::Token: AsChar,
//     <I as Stream>::Slice: AsRef<str>,
//     Error: From<winnow::Err<E>>,
//     E: winnow::error::ParseError<I>,
// {
//     let parser = parser.with_span();
//     let (i, (o, s)) = parser.parse_next(i)?;

//     // TODO
//     // if i.eof_offset() !=  {
//     Ok(o)
//     // } else {
//     //     Err(err(s.into(), ErrorKind::TrailingCharacters))
//     // }
// }

// pub(super) fn repeat<'a, T, Src: FnMut() -> Result<Located<&'a str>, err::Error>>(
//     mut src: Src,
//     parse: impl Fn(&mut Src, usize) -> Result<T, err::Error>,
// ) -> Result<Vec<T>, err::Error>
// {
//     let n = parse!(src()? => usize)?;
//     repeat_n(src, parse, n)
// }

// pub(super) fn repeat_n<'a, T, Src: FnMut() -> Result<Located<&'a str>, err::Error>>(
//     mut src: Src,
//     parse: impl Fn(&mut Src, usize) -> Result<T, err::Error>,
//     n: usize,
// ) -> Result<Vec<T>, err::Error> {
//     (0..n).map(|i| parse(&mut src, i)).collect()
// }
