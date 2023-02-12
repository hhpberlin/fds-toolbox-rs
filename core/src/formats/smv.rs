//pub type Input<'a> = &'a [u8];
//pub type Result<'a, T> = nom::IResult<Input<'a>, T, ()>;

use std::{
    collections::HashMap,
    num::{ParseFloatError, ParseIntError},
};

use nom_locate::LocatedSpan;
use thiserror::Error;

use crate::geom::Vec3F;

type Span<'a> = LocatedSpan<&'a str>;

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

#[derive(Debug, Error)]
enum Error<'a> {
    #[error("Unexpected whitespace, expected no whitespace at the start of the line: {0}")]
    UnexpectedWhitespace(Span<'a>),
    #[error("Missing line after: {0}")]
    MissingLine(Span<'a>),
    #[error("Failed to parse invalid number: {0}")]
    InvalidInt(Span<'a>, ParseIntError),
    #[error("Failed to parse invalid number: {0}")]
    InvalidFloat(Span<'a>, ParseFloatError),
    // TODO: Using enum worth it?
    #[error("Missing section: {0}")]
    MissingSection(&'static str),
    #[error("Wrong number of values {1}, expected {2}: {0}")]
    WrongNumberOfValues(Span<'a>, usize, usize),
}

// TODO: Track spans for inside a line
// Currently, the span only tracks the entire line
// because split_whitespace is only implemented for &str, not Span
macro_rules! parse {
    ($i:expr ; $sp:expr => $t:ident else $err: expr) => {
        $i.parse::<$t>().map_err(|e| $err($sp, e))?
    };
    ($i:expr ; $sp:expr => ( $($t:ident),+ )) => {
        {
            // Counts the number of types ($t)
            // TODO: Track https://github.com/rust-lang/rust/issues/83527 for cleaner solution
            let param_count = 0 $(+ { let _ = stringify!($t); 1 })+;

            let mut parts = $i.split_whitespace();
            let mut idx = 0;

            ($(
                {
                    let part = parts.next().ok_or(Error::WrongNumberOfValues($sp, idx, param_count))?;
                    idx += 1;
                    parse!(part ; $sp => $t)
                }
            ),+)

            // if parts.next().is_some() {
            //     return Err(Error::WrongNumberOfValues($i, idx, params));
            // }
        }
    };
    ($i:expr ; $sp:expr => f32) => { parse!($i ; $sp => f32 else Error::InvalidFloat) };
    ($i:expr ; $sp:expr => i32) => { parse!($i ; $sp => i32 else Error::InvalidInt) };
    ($i:expr ; $sp:expr => u32) => { parse!($i ; $sp => u32 else Error::InvalidInt) };
    ($i:expr ; $sp:expr => str) => { $i.fragment().trim_start() };
    ($i:expr ; $sp:expr => Vec3F) => { Vec3F::from(parse!($i ; $sp => (f32, f32, f32))) };
    ($i:expr ; $sp:expr => $t:ident) => { compile_error!(concat!("Unknown type: ", stringify!($t))) };
    ($sp:expr => $t:ident) => { parse!($sp.fragment() ; $sp => $t) };
    ($sp:expr => ( $($t:ident),+ )) => { parse!($sp.fragment() ; $sp => ( $($t),+ )) };
}



// fn split_whitespace_span(i: Span<'_>) -> impl Iterator<Item = Span<'_>> {
//     // i.fragment().split_whitespace().map(move |x| i.(x))
//     i.split_whitespace().map(move |x| x.)
// }

impl Simulation {
    pub fn parse<'a>(lines: impl Iterator<Item = Span<'a>>) -> Result<Self, Error<'a>> {
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

        let mut lines = lines.filter(|x| !x.trim_start().is_empty());

        // Not using a for loop because we need to peek at the next lines
        // A for loop would consume `lines` by calling .into_iter()
        while let Some(line) = lines.next() {
            // let line = line.trim_end();
            if line.trim().len() != line.len() {
                return Err(Error::UnexpectedWhitespace(line));
            }

            let mut next = || {
                lines.next().ok_or(Error::MissingLine(line))
                // .map(|x| x.trim())
            };
            let next_line = next()?;

            match *line.fragment() {
                "TITLE" => title = Some(next_line.trim_start()),
                "VERSION" | "FDSVERSION" => fds_version = Some(next_line.trim_start()),
                "ENDF" => end_file = Some(next_line.trim_start()),
                "INPF" => input_file = Some(next_line.trim_start()),
                "REVISION" => revision = Some(next_line.trim_start()),
                "CHID" => chid = Some(next_line.trim_start()),
                "SOLID_HT3D" => solid_ht3d = Some(parse!(next_line => i32)),
                "CSVF" => {
                    // TODO
                    let name = next()?;
                    let file = next()?;
                    csv_files.insert(name, file);
                }
                "NMESGES" => num_meshes = Some(parse!(next_line => u32)),
                // "NMESHES" => num_meshes = Some(parse(next()?, Error::InvalidInt)?),
                // "HRRPUVCUT" => hrrpuv_cutoff = Some(parse(next()?, Error::InvalidFloat)?),
                "VIEWTIMES" => view_times = Some(parse!(next_line => (f32, f32, i32))),
                "ALBEDO" => albedo = Some(parse!(next_line => f32)),
                "IBLANK" => i_blank = Some(parse!(next_line => i32)),
                "GVEC" => g_vec = Some(parse!(next_line => Vec3F)),
                _ => unimplemented!("Unknown line: {}", line),
            }
        }

        Ok(Simulation {
            title: title.ok_or(Error::MissingSection("TITLE"))?.to_string(),
            fds_version: fds_version
                .ok_or(Error::MissingSection("FDSVERSION"))?
                .to_string(),
            end_version: end_file.ok_or(Error::MissingSection("ENDF"))?.to_string(),
            input_file: input_file.ok_or(Error::MissingSection("INPF"))?.to_string(),
            revision: revision
                .ok_or(Error::MissingSection("REVISION"))?
                .to_string(),
            chid: chid.ok_or(Error::MissingSection("CHID"))?.to_string(),
            solid_ht3d: solid_ht3d.ok_or(Error::MissingSection("SOLID_HT3D"))?,
        })
    }
}
