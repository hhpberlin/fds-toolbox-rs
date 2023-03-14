use winnow::{
    stream::{AsChar, Stream, StreamIsPartial},
    IResult, Parser,
};

use crate::geom::{
    Bounds3, Bounds3F, Bounds3I, Surfaces3, Vec2F, Vec2I, Vec2U, Vec3F, Vec3I, Vec3U,
};

use super::super::util::{f32, i32, u32};

/// Takes any amount of winnow parsers and returns a parser that parses them in sequence,
/// separated by any amount of whitespace.

#[macro_export]
macro_rules! ws_separated {
    () => {
        winnow::character::space0
            .context(concat!("ws_separated!()"))
    };
    ($($t:expr),*) => {
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

/// Adds the current file, line and column to the given parser as context.
#[macro_export]
macro_rules! trace_callsite {
    ($t:expr) => {
        $t.context(concat!(file!(), ":", line!(), ":", column!()))
    };
}

/// Implements a parser that parses a white space separated sequence of parsers
/// and converts it to the given type with the given closure.
///
/// If no function is given, the [`From`] trait is used.
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
