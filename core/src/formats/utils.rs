use std::str::FromStr;

use chumsky::{prelude::*, text::Character, Error, Parser};

fn from_str<T: FromStr>(
    p: impl Parser<char, <char as Character>::Collection, Error = Simple<char>> + Copy + Clone,
    name: &'static str,
) -> impl Parser<char, T, Error = Simple<char>> + Copy + Clone
where
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    p.try_map(move |x, span| {
        x.parse::<T>()
            .map_err(|e| Simple::custom(span, format!("Failed to parse {}: {}", name, e)))
    })
}

macro_rules! from_str_parser {
    ($name:ident, $t:ty, $src:expr) => {
        pub fn $name<C: Character, E: Error<C>>(
        ) -> impl Parser<char, $t, Error = Simple<char>> {
            // from_str($src, stringify!($t))
            $src.from_str()
        }
    };
}

fn float() ->  impl Parser<char, <char as Character>::Collection, Error = Simple<char>> {
    let sign = one_of("+-");
    let int = text::int(10);
    let exponent = one_of("eE").then(sign.or_not()).then(int);
    
    sign.or_not().then(int).then(one_of(".").then(int).or_not()).then(exponent.or_not())
}

from_str_parser!(i32, i32, text::int(10));
from_str_parser!(u32, u32, text::int(10));

from_str_parser!(f32, f32, float());
