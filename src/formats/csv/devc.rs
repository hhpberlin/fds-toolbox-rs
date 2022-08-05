use std::{io::Read, num::ParseFloatError, str::FromStr};

use serde::{Serialize, Deserialize};
use thiserror::Error;
use uom::{si::f32::Time, str::ParseQuantityError};

// TODO: Use nd-array instead?

#[derive(Debug, Serialize, Deserialize)]
pub struct Devices {
    times: Vec<Time>,
    devices: Vec<DeviceReadings>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceReadings {
    unit: String,
    name: String,
    values: Vec<f32>,
}

#[derive(Error, Debug)]
pub enum DevicesParsingError {
    #[error("Missing units header (first line)")]
    MissingUnitsLine,
    #[error("Missing names header (second line)")]
    MissingNamesLine,

    #[error("Missing time unit (first line)")]
    MissingTimeUnit,
    #[error("Invalid time unit (first line, {0} '{1}')")]
    InvalidTimeUnit(ParseQuantityError, String),

    #[error("Parsing error in units header CSV (first line)")]
    ParsingErrorUnitsCsv(csv::Error),
    #[error("Parsing error in names header CSV (second line)")]
    ParsingErrorNamesCsv(csv::Error),

    #[error(
        "Number of units and names don't match ({units_len} units, {names_len} names)"
    )]
    InvalidUnitsAndNamesCount { units_len: usize, names_len: usize },

    #[error("Wrong number of values (line {0}: {1} columns, expected {2})")]
    WrongValueCount(usize, usize, usize),

    #[error("CSV parsing error (line {0}: {1})")]
    ParsingErrorCsv(usize, csv::Error),
    #[error("Float parsing error (line {0}, column {1}: {2})")]
    ParsingError(usize, usize, ParseFloatError),
}

impl Devices {
    pub fn from_reader(rdr: impl Read) -> Result<Self, DevicesParsingError> {
        let rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .trim(csv::Trim::All)
            // To allow an empty line at the end of the file
            // Currently skips any empty line (except in the 2 header lines)
            // Empty records are also skipped
            .flexible(true)
            .from_reader(rdr);

        let mut rdr = rdr.into_records();

        let units = match rdr.next() {
            Some(val) => val.map_err(DevicesParsingError::ParsingErrorUnitsCsv)?,
            None => return Err(DevicesParsingError::MissingUnitsLine),
        };
        let names = match rdr.next() {
            Some(val) => val.map_err(DevicesParsingError::ParsingErrorNamesCsv)?,
            None => return Err(DevicesParsingError::MissingNamesLine),
        };

        let units_len = units.len();
        let names_len = names.len();
        if units_len != names_len || units_len < 1 {
            return Err(DevicesParsingError::InvalidUnitsAndNamesCount {
                units_len,
                names_len,
            });
        }

        let mut units_iter = units.iter();
        let time_fac = match units_iter.next() {
            Some(val) => {
                let mut s = "1 ".to_string();
                s.push_str(val);
                Time::from_str(&s)
                    .map_err(|x| DevicesParsingError::InvalidTimeUnit(x, val.to_string()))?
            },
            None => return Err(DevicesParsingError::MissingTimeUnit),
        };
        
        let mut times = Vec::new();
        let mut devices = units_iter
            .zip(names.iter().skip(1))
            .map(|(unit, name)| {
                DeviceReadings {
                    unit: unit.to_string(),
                    name: name.to_string(),
                    values: Vec::new(),
                }
            }).collect::<Vec<_>>();

        let len = devices.len();

        for (i, x) in rdr.enumerate() {
            let x = x.map_err(|x| DevicesParsingError::ParsingErrorCsv(i+2, x))?;
            let mut x = x.iter();
            let time: f32 = match x.next() {
                Some(val) => {
                    if val.is_empty() { continue; }
                    val.parse().map_err(|x| DevicesParsingError::ParsingError(i+2, 0, x))?
                },
                None => return Err(DevicesParsingError::WrongValueCount(i+2, 0, len)),
            };
            times.push(time_fac * time);

            let mut j = 0;
            for x in x {
                if x.is_empty() {
                    continue;
                }
                if j < len {
                    let val = x
                        .parse::<f32>()
                        .map_err(|x| DevicesParsingError::ParsingError(i+2, j+1, x))?;
                    devices[j].values.push(val);
                }
                j+=1;
            }
            if j != len {
                if j == 0 { continue; }
                return Err(DevicesParsingError::WrongValueCount(i+2, j, len));
            }
        }

        Ok(Devices {
            times,
            devices,
        })
    }
}

#[cfg(test)]
mod tests {
    use uom::si::time::{second, hour};

    use super::*;

    #[test]
    fn basic_parsing() {
        let devices = Devices::from_reader(r#"s, m3/s, C, 1/m
        Time,   Zuluft_1,   Abluft_1,   T_B01
        0.0E+0, 1.2E+3,     -2.3E-2,    4.1E-12
        "#.as_bytes()).unwrap();

        assert_eq!(devices.times.iter().map(|x| x.get::<second>()).collect::<Vec<_>>(), [0.0e0]);
        assert_eq!(devices.devices.len(), 3);

        assert_eq!(devices.devices[0].unit, "m3/s");
        assert_eq!(devices.devices[1].unit, "C");
        assert_eq!(devices.devices[2].unit, "1/m");

        assert_eq!(devices.devices[0].name, "Zuluft_1");
        assert_eq!(devices.devices[1].name, "Abluft_1");
        assert_eq!(devices.devices[2].name, "T_B01");

        assert_eq!(devices.devices[0].values, [1.2e3]);
        assert_eq!(devices.devices[1].values, [-2.3e-2]);
        assert_eq!(devices.devices[2].values, [4.1e-12]);
    }

    #[test]
    fn time_unit() {
        let devices = Devices::from_reader(r#"h
        Time
        1"#.as_bytes()).unwrap();
        assert_eq!(devices.times[0].get::<hour>(), 1.0);
    }

    #[test]
    #[should_panic]
    fn time_unit_error() {
        Devices::from_reader(r#"one of the time-units of all time
        Time
        1"#.as_bytes()).unwrap();
    }

    #[test]
    #[should_panic]
    fn missing_names() {
        Devices::from_reader(r#"s"#.as_bytes()).unwrap();
    }

    #[test]
    #[should_panic]
    fn invalid_number() {
        Devices::from_reader(r#"s
        Time
        a 'number'"#.as_bytes()).unwrap();
    }
}