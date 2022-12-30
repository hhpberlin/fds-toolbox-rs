use std::{io::Read, num::ParseFloatError, str::FromStr};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uom::{
    si::f32::{MassRate, Power, Time},
    str::ParseQuantityError,
};
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct HRRStep {
    #[serde(rename = "Time")]
    time: Time,
    #[serde(rename = "HRR")]
    heat_release_rate: Power,
    q_radi: Power,
    q_conv: Power,
    q_cond: Power,
    q_diff: Power,
    q_pres: Power,
    q_part: Power,
    q_geom: Power,
    q_enth: Power,
    q_total: Power,
    #[serde(rename = "MLR_FUEL")]
    mass_flow_rate_fuel: MassRate,
    #[serde(rename = "MLR_TOTAL")]
    mass_flow_rate_total: MassRate,
}

pub enum HRRStepDataType {
    HeatReleaseRate,
    QRadi,
    QConv,
    QCond,
    QDiff,
    QPres,
    QPart,
    QGeom,
    QEnth,
    QTotal,
    MassFlowRateFuel,
    MassFlowRateTotal,
}

#[derive(Error, Debug)]
pub enum HRRStepsParseError {
    #[error("Missing units header (first line)")]
    MissingUnitsLine,
    #[error("Missing names header (second line)")]
    MissingNamesLine,

    #[error("Parsing error in units header CSV (first line)")]
    ParsingErrorUnitsCsv(csv::Error),
    #[error("Parsing error in names header CSV (second line)")]
    ParsingErrorNamesCsv(csv::Error),

    #[error("Parsing error in units header (first line, column {0}: {1} '{2}')")]
    ParsingErrorUnits(usize, ParseQuantityError, String),
    #[error("Parsing error in names header (second line, column {0}: {1} not known)")]
    ParsingErrorNames(usize, String),

    #[error("Missing names (second line)")]
    MissingNames,

    #[error(
        "Number of units and names don't match ({units_len} units, {names_len} names, expected 13)"
    )]
    InvalidUnitsAndNamesCount { units_len: usize, names_len: usize },

    #[error("Wrong number of values (line {0}: {1} columns, expected 13)")]
    WrongValueCount(usize, usize),

    #[error("CSV parsing error (line {0}: {1})")]
    ParsingErrorCsv(usize, csv::Error),
    #[error("Float parsing error (line {0}, column {1}: {2})")]
    ParsingError(usize, usize, ParseFloatError),
}

macro_rules! force_unit {
    ($type:ident, $buf:ident, $factors:ident, $idx:expr) => {
        $type {
            value: $factors[$idx].1 * $buf[$factors[$idx].0],
            units: std::marker::PhantomData,
            dimension: std::marker::PhantomData,
        }
    };
}

impl HRRStep {
    pub fn from_reader(rdr: impl Read) -> Result<Vec<Self>, HRRStepsParseError> {
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
            Some(val) => val.map_err(HRRStepsParseError::ParsingErrorUnitsCsv)?,
            None => return Err(HRRStepsParseError::MissingUnitsLine),
        };
        let names = match rdr.next() {
            Some(val) => val.map_err(HRRStepsParseError::ParsingErrorNamesCsv)?,
            None => return Err(HRRStepsParseError::MissingNamesLine),
        };

        let units_len = units.len();
        let names_len = names.len();
        if units_len != names_len || units_len != 13 {
            return Err(HRRStepsParseError::InvalidUnitsAndNamesCount {
                units_len,
                names_len,
            });
        }

        let mut factors = [(0, 0 as f32); 13];
        let mut visited = [false; 13];
        let mut buf: String = String::with_capacity(8);

        fn get_fac<T: FromStr<Err = ParseQuantityError>>(
            txt: &str,
            i: usize,
        ) -> Result<T, HRRStepsParseError> {
            T::from_str(txt)
                .map_err(|e| HRRStepsParseError::ParsingErrorUnits(i, e, txt.to_string()))
        }

        for (i, (unit, name)) in units.iter().zip(names.iter()).enumerate() {
            // TODO: Is this really the best way to do this?
            // The get_fac::<>()?.value invocations are *not* type-checked due to producing a simple f32, so be careful here
            buf.clear();
            buf.push_str("1 ");
            buf.push_str(unit);
            let factor = match name {
                "Time" => (0, get_fac::<Time>(&buf, i)?.value),
                "HRR" => (1, get_fac::<Power>(&buf, i)?.value),
                "Q_RADI" => (2, get_fac::<Power>(&buf, i)?.value),
                "Q_CONV" => (3, get_fac::<Power>(&buf, i)?.value),
                "Q_COND" => (4, get_fac::<Power>(&buf, i)?.value),
                "Q_DIFF" => (5, get_fac::<Power>(&buf, i)?.value),
                "Q_PRES" => (6, get_fac::<Power>(&buf, i)?.value),
                "Q_PART" => (7, get_fac::<Power>(&buf, i)?.value),
                "Q_GEOM" => (8, get_fac::<Power>(&buf, i)?.value),
                "Q_ENTH" => (9, get_fac::<Power>(&buf, i)?.value),
                "Q_TOTAL" => (10, get_fac::<Power>(&buf, i)?.value),
                "MLR_FUEL" => (11, get_fac::<MassRate>(&buf, i)?.value),
                "MLR_TOTAL" => (12, get_fac::<MassRate>(&buf, i)?.value),
                _ => return Err(HRRStepsParseError::ParsingErrorNames(i, name.to_string())),
            };
            factors[factor.0] = (i, factor.1);
            visited[factor.0] = true;
        }

        if !visited.iter().all(|x| *x) {
            return Err(HRRStepsParseError::MissingNames);
        }

        // HRRStep::deserialize(deserializer);
        let mut buf = [0f32; 13];

        let mut steps = Vec::new();

        for (i, x) in rdr.enumerate() {
            // TODO: Read directly into fields instead of using buf,
            //       reverse usage of factors basically, target idx instead of source

            let x = x.map_err(|x| HRRStepsParseError::ParsingErrorCsv(i + 2, x))?;

            let mut j = 0;
            for x in x.iter() {
                if x.is_empty() {
                    continue;
                }
                if j < buf.len() {
                    buf[j] = x
                        .parse::<f32>()
                        .map_err(|x| HRRStepsParseError::ParsingError(i + 2, j, x))?;
                }
                j += 1;
            }
            if j != buf.len() {
                if j == 0 {
                    continue;
                }
                return Err(HRRStepsParseError::WrongValueCount(i + 2, j));
            }

            steps.push(HRRStep {
                time: force_unit!(Time, buf, factors, 0),
                heat_release_rate: force_unit!(Power, buf, factors, 1),
                q_radi: force_unit!(Power, buf, factors, 2),
                q_conv: force_unit!(Power, buf, factors, 3),
                q_cond: force_unit!(Power, buf, factors, 4),
                q_diff: force_unit!(Power, buf, factors, 5),
                q_pres: force_unit!(Power, buf, factors, 6),
                q_part: force_unit!(Power, buf, factors, 7),
                q_geom: force_unit!(Power, buf, factors, 8),
                q_enth: force_unit!(Power, buf, factors, 9),
                q_total: force_unit!(Power, buf, factors, 10),
                mass_flow_rate_fuel: force_unit!(MassRate, buf, factors, 11),
                mass_flow_rate_total: force_unit!(MassRate, buf, factors, 12),
            });
        }

        Ok(steps)
    }

    pub fn get_data(
        elems: &[Self],
        data_type: HRRStepDataType,
        // TODO: We can have a little heap allocation, as a treat
    ) -> Box<dyn Iterator<Item = (Time, f32)> + '_> {
        match data_type {
            HRRStepDataType::HeatReleaseRate => {
                Box::new(elems.iter().map(|x| (x.time, x.heat_release_rate.value)))
            }
            HRRStepDataType::QRadi => Box::new(elems.iter().map(|x| (x.time, x.q_radi.value))),
            HRRStepDataType::QConv => Box::new(elems.iter().map(|x| (x.time, x.q_conv.value))),
            HRRStepDataType::QCond => Box::new(elems.iter().map(|x| (x.time, x.q_cond.value))),
            HRRStepDataType::QDiff => Box::new(elems.iter().map(|x| (x.time, x.q_diff.value))),
            HRRStepDataType::QPres => Box::new(elems.iter().map(|x| (x.time, x.q_pres.value))),
            HRRStepDataType::QPart => Box::new(elems.iter().map(|x| (x.time, x.q_part.value))),
            HRRStepDataType::QGeom => Box::new(elems.iter().map(|x| (x.time, x.q_geom.value))),
            HRRStepDataType::QEnth => Box::new(elems.iter().map(|x| (x.time, x.q_enth.value))),
            HRRStepDataType::QTotal => Box::new(elems.iter().map(|x| (x.time, x.q_total.value))),
            HRRStepDataType::MassFlowRateFuel => {
                Box::new(elems.iter().map(|x| (x.time, x.mass_flow_rate_fuel.value)))
            }
            HRRStepDataType::MassFlowRateTotal => {
                Box::new(elems.iter().map(|x| (x.time, x.mass_flow_rate_total.value)))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use uom::si::{mass_rate::kilogram_per_second, power::kilowatt, time::second};

    use super::*;

    #[test]
    fn basic_parsing() {
        let hrrs = HRRStep::from_reader(r#"s,kW,kW,kW,kW,kW,kW,kW,kW,kW,kW,kg/s,kg/s
        Time,HRR,Q_RADI,Q_CONV,Q_COND,Q_DIFF,Q_PRES,Q_PART,Q_GEOM,Q_ENTH,Q_TOTAL,MLR_FUEL,MLR_TOTAL
         0.0000000E+000, 0.0000000E+000,-8.0996608E-001,-4.3266538E-006, 0.0000000E+000, 0.0000000E+000, 0.0000000E+000, 0.0000000E+000, 0.0000000E+000, 0.0000000E+000,-8.0997040E-001, 0.0000000E+000, 0.0000000E+000
         1.0206207E+000, 1.3223356E-001,-4.4154689E-002, 3.3198851E-004,-1.1500706E-004,-1.6679039E-005, 0.0000000E+000, 0.0000000E+000, 0.0000000E+000, 2.2088911E-002, 8.8279171E-002, 6.8489026E-006, 6.8489026E-006
        "#.as_bytes()).unwrap();

        assert_eq!(hrrs.len(), 2);

        assert_eq!(hrrs[0].time, Time::new::<second>(0.0));
        assert_eq!(hrrs[0].heat_release_rate, Power::new::<kilowatt>(0.0));
        assert_eq!(hrrs[0].q_radi, Power::new::<kilowatt>(-8.099_661E-1));
        assert_eq!(hrrs[0].q_conv, Power::new::<kilowatt>(-4.326_653_7E-6));
        assert_eq!(hrrs[0].q_cond, Power::new::<kilowatt>(0.0));
        assert_eq!(hrrs[0].q_diff, Power::new::<kilowatt>(0.0));
        assert_eq!(hrrs[0].q_pres, Power::new::<kilowatt>(0.0));
        assert_eq!(hrrs[0].q_part, Power::new::<kilowatt>(0.0));
        assert_eq!(hrrs[0].q_geom, Power::new::<kilowatt>(0.0));
        assert_eq!(hrrs[0].q_enth, Power::new::<kilowatt>(0.0));
        assert_eq!(hrrs[0].q_total, Power::new::<kilowatt>(-8.099_704E-1));
        assert_eq!(
            hrrs[0].mass_flow_rate_fuel,
            MassRate::new::<kilogram_per_second>(0.0)
        );
        assert_eq!(
            hrrs[0].mass_flow_rate_total,
            MassRate::new::<kilogram_per_second>(0.0)
        );

        assert_eq!(hrrs[1].time, Time::new::<second>(1.020_620_7));
        assert_eq!(
            hrrs[1].heat_release_rate,
            Power::new::<kilowatt>(1.322_335_6E-1)
        );
        assert_eq!(hrrs[1].q_radi, Power::new::<kilowatt>(-4.415_469E-2));
        assert_eq!(hrrs[1].q_conv, Power::new::<kilowatt>(3.319_885E-4));
        assert_eq!(hrrs[1].q_cond, Power::new::<kilowatt>(-1.150_070_6E-4));
        assert_eq!(hrrs[1].q_diff, Power::new::<kilowatt>(-1.667_904E-5));
        assert_eq!(hrrs[1].q_pres, Power::new::<kilowatt>(0.0000000E+000));
        assert_eq!(hrrs[1].q_part, Power::new::<kilowatt>(0.0000000E+000));
        assert_eq!(hrrs[1].q_geom, Power::new::<kilowatt>(0.0000000E+000));
        assert_eq!(hrrs[1].q_enth, Power::new::<kilowatt>(2.208_891_1E-2));
        assert_eq!(hrrs[1].q_total, Power::new::<kilowatt>(8.827_917E-2));
        assert_eq!(
            hrrs[1].mass_flow_rate_fuel,
            MassRate::new::<kilogram_per_second>(6.848_902_6E-6)
        );
        assert_eq!(
            hrrs[1].mass_flow_rate_total,
            MassRate::new::<kilogram_per_second>(6.848_902_6E-6)
        );
    }
}
