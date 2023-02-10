//pub type Input<'a> = &'a [u8];
//pub type Result<'a, T> = nom::IResult<Input<'a>, T, ()>;

use std::{collections::HashMap, io::Read};

use nom_locate::LocatedSpan;

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

enum SimulationParsingError {
    UnexpectedWhitespace(Span<'static>),
}

impl Simulation {
    pub fn parse(lines: impl Iterator<Item = &str>) -> Result<Self, ()> {
        let mut title = None;
        let mut fds_version = None;
        let mut end_file = None;
        let mut input_file = None;
        let mut revision = None;
        let mut chid = None;
        let mut csv_files = HashMap::new();
        let mut solid_ht3d = None;

        for line in lines {
            // let line = line.trim_end();
            if line.trim_start().len() != line.len() {
                return Err(());
            }

            let next = || {
                lines.next().ok_or(())?.trim()
            };

            match line {
                "TITLE" => title = Some(next),
                "FDS VERSION" => fds_version = Some(line),
            }
        }
    }
}
