use std::{io::Read, num::ParseFloatError, str::FromStr, sync::Arc};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uom::{si::f32::Time, str::ParseQuantityError};

use crate::common::series::{Series, Series1, Series1View, TimeSeries0View, TimeSeriesView};

// TODO: Use 2d-array instead?

#[derive(Debug, Serialize, Deserialize)]
pub struct Devices {
    pub time_in_seconds: Arc<Series1>,
    devices: Vec<DeviceReadings>,
    // devices_by_name: HashMap<String, DeviceIdx>,
}

pub struct Device {
    pub time_in_seconds: Arc<Series1>,
    pub readings: DeviceReadings,
}

// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
// pub struct DeviceIdx(usize);

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceReadings {
    pub unit: String,
    pub name: String,
    pub values: Series1,
}

impl DeviceReadings {
    pub fn view<'a>(&'a self, time_in_seconds: Series1View<'a>) -> TimeSeries0View<'a> {
        TimeSeriesView::new(time_in_seconds, self.values.view(), &self.unit, &self.name)
    }
}

#[derive(Error, Debug)]
pub enum Error {
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

    #[error("Number of units and names don't match ({units_len} units, {names_len} names)")]
    InvalidUnitsAndNamesCount { units_len: usize, names_len: usize },

    #[error("Wrong number of values (line {0}: {1} columns, expected {2})")]
    WrongValueCount(usize, usize, usize),

    #[error("CSV parsing error (line {0}: {1})")]
    ParsingErrorCsv(usize, csv::Error),
    #[error("Float parsing error (line {0}, column {1}: {2})")]
    ParsingError(usize, usize, ParseFloatError),
}

impl Devices {
    pub fn from_reader(rdr: impl Read) -> Result<Self, Error> {
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
            Some(val) => val.map_err(Error::ParsingErrorUnitsCsv)?,
            None => return Err(Error::MissingUnitsLine),
        };
        let names = match rdr.next() {
            Some(val) => val.map_err(Error::ParsingErrorNamesCsv)?,
            None => return Err(Error::MissingNamesLine),
        };

        let units_len = units.len();
        let names_len = names.len();
        if units_len != names_len || units_len < 1 {
            return Err(Error::InvalidUnitsAndNamesCount {
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
                    .map_err(|x| Error::InvalidTimeUnit(x, val.to_string()))?
                    .value
            }
            None => return Err(Error::MissingTimeUnit),
        };

        let units: Vec<_> = units_iter.zip(names.iter().skip(1)).collect();

        let mut times = Vec::new();
        let mut devices: Vec<_> = (0..units.len()).map(|_| Vec::new()).collect();

        let len = devices.len();

        for (i, x) in rdr.enumerate() {
            let x = x.map_err(|x| Error::ParsingErrorCsv(i + 2, x))?;
            let mut x = x.iter();
            let time: f32 = match x.next() {
                Some(val) => {
                    if val.is_empty() {
                        continue;
                    }
                    val.parse().map_err(|x| Error::ParsingError(i + 2, 0, x))?
                }
                None => return Err(Error::WrongValueCount(i + 2, 0, len)),
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
                        .map_err(|x| Error::ParsingError(i + 2, j + 1, x))?;
                    devices[j].push(val);
                }
                j += 1;
            }
            if j != len {
                if j == 0 {
                    continue;
                }
                return Err(Error::WrongValueCount(i + 2, j, len));
            }
        }

        let devices = units
            .into_iter()
            .zip(devices.into_iter())
            .map(|((unit, name), values)| DeviceReadings {
                name: name.to_string(),
                unit: unit.to_string(),
                values: Series::from_vec(values),
            })
            .collect::<Vec<_>>();

        // let devices_by_name = devices
        //     .iter()
        //     .enumerate()
        //     .map(|(i, x)| (x.name.clone(), DeviceIdx(i)))
        //     .collect::<HashMap<_, _>>();

        let times = Series::from_vec(times);

        Ok(Devices {
            time_in_seconds: times,
            devices,
            // devices_by_name,
        })
    }

    pub fn get_device_by_name(&self, name: &str) -> Option<&DeviceReadings> {
        self.devices.iter().find(|x| x.name == name)
    }

    // pub fn get_device_by_name(&self, name: &str) -> Option<&DeviceReadings> {
    //     let idx = self.get_device_idx_by_name(name)?;
    //     self.get_device_by_idx(idx)
    // }

    // pub fn get_device_by_idx(&self, name: DeviceIdx) -> Option<&DeviceReadings> {
    //     self.devices.get(name.0)
    // }

    // pub fn get_device_idx_by_name(&self, name: &str) -> Option<DeviceIdx> {
    //     self.devices_by_name.get(name).copied()
    // }

    // pub fn iter_device_named_ids(&self) -> impl Iterator<Item = (&str, DeviceIdx)> {
    //     self.devices_by_name.iter().map(|(k, v)| (k.as_str(), *v))
    // }

    // pub fn iter_devices(&self) -> impl Iterator<Item = &DeviceReadings> {
    //     self.devices.iter()
    // }

    // pub fn enumerate_devices(&self) -> impl Iterator<Item = (DeviceIdx, &DeviceReadings)> {
    //     self.devices
    //         .iter()
    //         .enumerate()
    //         .map(|(i, x)| (DeviceIdx(i), x))
    // }
}

// impl TimeSeriesViewSource<DeviceIdx, f32, Ix1> for Devices {
//     fn get_time_series(&self, id: DeviceIdx) -> PotentialResult<TimeSeries0View> {
//         self.get_device_by_idx(id)
//             .map(|x| x.view(self.time_in_seconds.view()))
//             .ok_or(Missing::InvalidKey)
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_parsing() {
        let devices = Devices::from_reader(
            r#"s, m3/s, C, 1/m
        Time,   Zuluft_1,   Abluft_1,   T_B01
        0.0E+0, 1.2E+3,     -2.3E-2,    4.1E-12
        "#
            .as_bytes(),
        )
        .unwrap();

        assert_eq!(devices.time_in_seconds.iter().collect::<Vec<_>>(), [0.0e0]);
        assert_eq!(devices.devices.len(), 3);

        // assert_eq!(devices.devices.iter().find(|x| x.name == "Zuluft_1").unwrap().unit, "m3/s");
        // assert_eq!(devices.devices.iter().find(|x| x.name == "Abluft_1").unwrap().unit, "C");
        // assert_eq!(devices.devices.iter().find(|x| x.name == "T_B01").unwrap().unit, "1/m");

        assert_eq!(devices.get_device_by_name("Zuluft_1").unwrap().unit, "m3/s");
        assert_eq!(devices.get_device_by_name("Abluft_1").unwrap().unit, "C");
        assert_eq!(devices.get_device_by_name("T_B01").unwrap().unit, "1/m");

        // TODO: Check names properly
        // The reason this is not done yet is that the names are not stored
        // in the same order as in the source file, but instead in a HashMap.

        // assert_eq!(devices.devices[0].name, "Zuluft_1");
        // assert_eq!(devices.devices[1].name, "Abluft_1");
        // assert_eq!(devices.devices[2].name, "T_B01");

        assert_eq!(
            devices.get_device_by_name("Zuluft_1").unwrap().values[0],
            1.2e3
        );
        assert_eq!(
            devices.get_device_by_name("Abluft_1").unwrap().values[0],
            -2.3e-2
        );
        assert_eq!(
            devices.get_device_by_name("T_B01").unwrap().values[0],
            4.1e-12
        );
    }

    #[test]
    fn time_unit() {
        let devices = Devices::from_reader(
            r#"h
        Time
        1"#
            .as_bytes(),
        )
        .unwrap();
        assert_eq!(devices.time_in_seconds[0], 3600.0);
    }

    #[test]
    #[should_panic]
    fn time_unit_error() {
        Devices::from_reader(
            r#"one of the time-units of all time
        Time
        1"#
            .as_bytes(),
        )
        .unwrap();
    }

    #[test]
    #[should_panic]
    fn missing_names() {
        Devices::from_reader(r#"s"#.as_bytes()).unwrap();
    }

    #[test]
    #[should_panic]
    fn invalid_number() {
        Devices::from_reader(
            r#"s
        Time
        a 'number'"#
                .as_bytes(),
        )
        .unwrap();
    }
}
