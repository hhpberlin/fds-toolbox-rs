use std::{fmt::Debug, ops::Range, str::FromStr};

use winnow::{
    bytes::{take_while0, take_while1, take_till1},
    character::space0,
    sequence::preceded,
    stream::{AsChar, Stream, StreamIsPartial},
    IResult, Parser, Located,
};

pub fn non_ws<I>(i: I) -> IResult<I, I::Slice>
where
    I: StreamIsPartial + Stream,
    <I as Stream>::Token: AsChar,
{
    take_till1(AsChar::is_space)
        .context("non_ws")
        .parse_next(i)
}

pub fn word<I>(i: I) -> IResult<I, I::Slice>
where
    I: StreamIsPartial + Stream,
    <I as Stream>::Token: AsChar,
{
    preceded(space0, non_ws).parse_next(i)
}

pub fn from_str<I, T: FromStr>(i: I, context: impl Debug + Clone) -> IResult<I, T>
where
    I: StreamIsPartial + Stream,
    <I as Stream>::Token: AsChar,
{
    non_ws
        .map_res(|x: I::Slice| x.parse::<T>())
        .context(context)
        .parse_next(i)
}

// pub fn substr_to_span(full: &str, substr: &str) -> Range<usize> {
//     let offset = full.offset(substr);
//     offset..offset + substr.len()
// }

macro_rules! from_str_impl {
    ($($t:ident),+) => {
        $(pub fn $t<I>(i: I) -> IResult<I, $t>
        where
            I: StreamIsPartial + Stream,
            <I as Stream>::Token: AsChar,
        {
            from_str(i, stringify!($t))
        })+
    };
}

from_str_impl!(f32, i32, u32, usize);
