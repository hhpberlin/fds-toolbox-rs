//pub type Input<'a> = &'a [u8];
//pub type Result<'a, T> = nom::IResult<Input<'a>, T, ()>;

use std::{collections::HashMap, io::Read, num::ParseIntError, str::FromStr};

use nom_locate::LocatedSpan;
use thiserror::Error;

type Span<'a> = LocatedSpan<&'a str>;

#[derive(Debug)]
struct Simulation {
    title: String,
    fds_version: String,
    end_version: String,
    input_file: String,
    revision: String,
    chid: String,
    solid_ht3d: f64, // TODO: Float or int?
}

#[derive(Debug, Error)]
enum Error<'a> {
    #[error("Unexpected whitespace, expected no whitespace at the start of the line: {0}")]
    UnexpectedWhitespace(Span<'a>),
    #[error("Missing line after: {0}")]
    MissingLine(Span<'a>),
    #[error("Failed to parse invalid number: {0}")]
    InvalidInt(Span<'a>, ParseIntError),
}

fn parse<'a, T: FromStr<Err = SourceErr>, SourceErr: Into<FnInErr>, FnInErr, FnOutErr: Into<TargetErr>, TargetErr>(
    i: Span<'a>,
    f: impl FnOnce(Span<'a>, FnInErr) -> TargetErr,
) -> Result<T, TargetErr> {
    i.fragment().parse().map_err(|x: SourceErr| f(i, x.into()).into())
}

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

        for line in lines.filter(|x| !x.trim_start().is_empty()) {
            // let line = line.trim_end();
            if line.trim_start().len() != line.len() {
                return Err(Error::UnexpectedWhitespace(line));
            }

            let next = || {
                lines
                    .next()
                    .ok_or_else(|| Error::MissingLine(line))
                    .map(|x| x. x.trim())
            };

            match *line.fragment() {
                "TITLE" => title = Some(next()?),
                "FDSVERSION" => fds_version = Some(next()?),
                "ENDF" => end_file = Some(next()?),
                "INPF" => input_file = Some(next()?),
                "REVISION" => revision = Some(next()?),
                "CHID" => chid = Some(next()?),
                "SOLID_HT3D" => {
                    solid_ht3d = Some(parse(next()?, Error::InvalidInt)?)
                }
                "CSVF" => {
                    // TODO
                    let name = next()?;
                    let file = next()?;
                    csv_files.insert(name, file);
                }
                "NMESHES" => {
                    num_meshes = Some(next()?.parse().map_err(|x| Error::InvalidInt(x))?)
                }
            }
        }

        Ok(())
    }
}
