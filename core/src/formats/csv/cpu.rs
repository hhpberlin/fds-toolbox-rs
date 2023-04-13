use std::{collections::HashMap, io::Read};

use csv::ErrorKind;
use get_size::GetSize;
use serde::{Deserialize, Serialize};

use thiserror::Error;
use uom::si::{f32::Time, time::second};

#[derive(Debug, Serialize, Deserialize)]
pub struct CpuInfo {
    pub mpi_rank: u32,
    pub main_time: Time,
    pub divg_time: Time,
    pub mass_time: Time,
    pub velo_time: Time,
    pub pres_time: Time,
    pub wall_time: Time,
    pub dump_time: Time,
    pub part_time: Time,
    pub radi_time: Time,
    pub fire_time: Time,
    pub evac_time: Time,
    pub hvac_time: Time,
    pub comm_time: Time,
    pub total_time: Time,
}

// Can't use derive because no default implementation for `uom` types exists,
// and derive impl tries calling `GetSize` functions for all members.
impl GetSize for CpuInfo {}

#[derive(Debug, Serialize, Deserialize, GetSize)]
pub struct CpuData {
    pub info: Vec<CpuInfo>,
    by_mpi_rank: HashMap<u32, usize>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CpuInfoUntyped {
    #[serde(rename = "Rank")]
    mpi_rank: u32,
    #[serde(rename = "MAIN")]
    main_time: f32,
    #[serde(rename = "DIVG")]
    divg_time: f32,
    #[serde(rename = "MASS")]
    mass_time: f32,
    #[serde(rename = "VELO")]
    velo_time: f32,
    #[serde(rename = "PRES")]
    pres_time: f32,
    #[serde(rename = "WALL")]
    wall_time: f32,
    #[serde(rename = "DUMP")]
    dump_time: f32,
    #[serde(rename = "PART")]
    part_time: f32,
    #[serde(rename = "RADI")]
    radi_time: f32,
    #[serde(rename = "FIRE")]
    fire_time: f32,
    #[serde(rename = "EVAC")]
    evac_time: f32,
    #[serde(rename = "HVAC")]
    hvac_time: f32,
    #[serde(rename = "COMM")]
    comm_time: f32,
    #[serde(rename = "Total T_USED (s)")]
    total_time: f32,
}

// #[derive(Error, Debug)]
// pub enum HRRStepsParseError {
//     #[error("Missing units header (first line)")]
//     MissingUnitsLine,
// }

#[derive(Error, Debug)]
#[error(transparent)]
pub struct Error(#[from] csv::Error);

impl CpuData {
    pub fn from_reader(rdr: impl Read) -> Result<Self, Error> {
        let info = CpuInfo::from_reader(rdr)?;

        let by_mpi_rank = info
            .iter()
            .enumerate()
            .map(|(i, c)| (c.mpi_rank, i))
            .collect::<HashMap<_, _>>();

        Ok(Self { info, by_mpi_rank })
    }

    pub fn by_mpi_rank(&self, mpi_rank: u32) -> Option<&CpuInfo> {
        self.by_mpi_rank.get(&mpi_rank).map(|&i| &self.info[i])
    }
}

impl CpuInfo {
    fn from_reader(rdr: impl Read) -> Result<Vec<Self>, Error> {
        let mut rdr = csv::ReaderBuilder::new()
            // .has_headers(false)
            .trim(csv::Trim::All)
            // // To allow an empty line at the end of the file
            // // Currently skips any empty line (except in the 2 header lines)
            // // Empty records are also skipped
            // .flexible(true)
            .from_reader(rdr);

        let mut buf = Vec::new();

        for result in rdr.deserialize() {
            let record: CpuInfoUntyped = match result {
                Ok(record) => record,
                Err(e) => {
                    // Skip empty lines
                    // TODO: Also ignores single-entry lines with content currently
                    if let ErrorKind::UnequalLengths {
                        pos: _,
                        expected_len: _,
                        len: 1,
                    } = e.kind()
                    {
                        continue;
                    }
                    return Err(Error(e));
                }
            };
            let record = CpuInfo {
                mpi_rank: record.mpi_rank,
                main_time: Time::new::<second>(record.main_time),
                divg_time: Time::new::<second>(record.divg_time),
                mass_time: Time::new::<second>(record.mass_time),
                velo_time: Time::new::<second>(record.velo_time),
                pres_time: Time::new::<second>(record.pres_time),
                wall_time: Time::new::<second>(record.wall_time),
                dump_time: Time::new::<second>(record.dump_time),
                part_time: Time::new::<second>(record.part_time),
                radi_time: Time::new::<second>(record.radi_time),
                fire_time: Time::new::<second>(record.fire_time),
                evac_time: Time::new::<second>(record.evac_time),
                hvac_time: Time::new::<second>(record.hvac_time),
                comm_time: Time::new::<second>(record.comm_time),
                total_time: Time::new::<second>(record.total_time),
            };
            buf.push(record);
        }

        Ok(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_parsing() {
        let cpus = CpuInfo::from_reader(r#"Rank,MAIN,DIVG,MASS,VELO,PRES,WELL,DUMP,PART,RADI,FIRE,EVAC,HVAC,COMM,Total T_USED (s)
        1,1.2E3,4.5E6,-1.2E3,3.4E-5,0,123,-123,1e1,2e-1,.1e1,-.2e-0,-.1,01,1e01
        "#.as_bytes()).unwrap();
        assert_eq!(cpus.len(), 1);
        assert_eq!(cpus[0].mpi_rank, 1);
        assert_eq!(cpus[0].main_time, Time::new::<second>(1.2E3));
        assert_eq!(cpus[0].divg_time, Time::new::<second>(4.5E6));
        assert_eq!(cpus[0].mass_time, Time::new::<second>(-1.2E3));
        assert_eq!(cpus[0].velo_time, Time::new::<second>(3.4E-5));
        assert_eq!(cpus[0].pres_time, Time::new::<second>(0f32));
        assert_eq!(cpus[0].wall_time, Time::new::<second>(123f32));
        assert_eq!(cpus[0].dump_time, Time::new::<second>(-123f32));
        assert_eq!(cpus[0].part_time, Time::new::<second>(1e1));
        assert_eq!(cpus[0].radi_time, Time::new::<second>(2e-1));
        assert_eq!(cpus[0].fire_time, Time::new::<second>(0.1e1));
        assert_eq!(cpus[0].evac_time, Time::new::<second>(-0.2e-0));
        assert_eq!(cpus[0].hvac_time, Time::new::<second>(-0.1));
        assert_eq!(cpus[0].comm_time, Time::new::<second>(01f32));
        assert_eq!(cpus[0].total_time, Time::new::<second>(1e01));
    }

    #[test]
    #[should_panic]
    fn missing_headers() {
        CpuInfo::from_reader(
            r#"Rank,MAIN
        1,1.2E3"#
                .as_bytes(),
        )
        .unwrap();
    }
}
