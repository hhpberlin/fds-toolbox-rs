use std::{collections::HashMap, sync::Arc, error::Error, io::Read, path::Path, hash::Hash};

use async_trait::async_trait;
use fds_toolbox_core::{common::series::{TimeSeries0, TimeSeries2, TimeSeries3}, file::{Simulation, FileSystem, OsFs, SimulationPath}, formats::{smv, smoke::dim2::slice, csv}};
use moka::sync::Cache;
use thiserror::Error;

// TODO: Remove dead_code. Here for a dark cockpit.
#[allow(dead_code)]
pub struct MokaStore {
    cache: Cache<SimulationDataIdx, SimulationData>,
    simulations: HashMap<SimulationPath<Fs>, Simulation<Fs>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Fs {
    LocalFs(OsFs),
    // TODO: Add sftp, rescale, etc.
}

#[derive(Debug, Error)]
#[error(transparent)]
pub enum FsErr {
    Io(std::io::Error),
}

#[async_trait]
impl FileSystem for Fs {
    // RIP non-utf8 paths.
    // Probably will never be a problem, but fixing it would be nice.
    // TODO: Fix
    type Path = String;
    type PathRef = str;
    // TODO: Make an enum of all the possible types instead of dyn.
    type Error = FsErr;
    // TODO: Make an enum of all the possible types instead of dyn.
    type File = Box<dyn Read>;

    async fn read(&self, path: &Self::PathRef) -> Result<Self::File, Self::Error> {
        match self {
            Fs::LocalFs(fs) => match fs.read(Path::new(path)).await {
                Ok(file) => Ok(Box::new(file)),
                Err(err) => Err(FsErr::Io(err)),
            },
        }
    }
    async fn exists(&self, path: &Self::PathRef) -> Result<bool, Self::Error> {
        match self {
            Fs::LocalFs(fs) => fs.exists(Path::new(path)).await.map_err(FsErr::Io),
        }
    }

    fn file_path(&self, directory: &Self::PathRef, file_name: &str) -> Self::Path {
        match self {
            Fs::LocalFs(fs) => fs.file_path(Path::new(directory), file_name).to_str().expect("Non-UTF8 paths are currently not supported.").to_string(),
        }
    }
}

/*
       devc (0d over t)
       slices (2d over t)
       cpu (1d, fixed size)
       hrr (1d, fixed size over t)

       only ones worth merging i think:
       s3d (3d over t)
       p3d (3d over t, bytes)
*/

// These are opaque types referencing the indicies from the perspective of the current data-layout.
// They might be different on different runs.


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DevcIdx(usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SliceIdx(usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CpuIdx(usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HrrIdx(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct S3dIdx(usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct P3dIdx(usize);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SimulationsDataIdx(SimulationPath<Fs>, SimulationDataIdx);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SimulationDataIdx {
    Devc(DevcIdx),
    Slice(SliceIdx),
    Cpu(CpuIdx),
    Hrr(HrrIdx),
    S3d(S3dIdx),
    P3d(P3dIdx),
}

#[derive(Debug, Clone)]
pub enum SimulationData {
    Devc(Arc<TimeSeries0>),
    Slice(Arc<TimeSeries2>),
    Cpu(Arc<TimeSeries0>),
    Hrr(Arc<TimeSeries0>),
    S3d(Arc<TimeSeries3>),
    P3d(Arc<TimeSeries3>),
}

/* 
    ParseError<Fs::Error, smv::Error>
    ParseError<Fs::Error, slice::Error>
    ParseError<Fs::Error, csv::cpu::Error>
    ParseError<Fs::Error, csv::hrr::Error>
    ParseError<Fs::Error, csv::devc::Error>
 */

#[derive(Debug, Error)]
#[error(transparent)]
pub enum SimulationDataError {
    #[error("Invalid simulation key")]
    InvalidSimulationKey,
    // Io(std::io::Error),
    Fs(FsErr),
    Smv(smv::Error),
    Slice(slice::Error),
    Cpu(csv::cpu::Error),
    Hrr(csv::hrr::Error),
    Devc(csv::devc::Error),
}

impl MokaStore {
    pub fn new(max_capacity: u64) -> Self {
        Self {
            cache: Cache::builder()
                // Up to 10,000 entries.
                .max_capacity(10_000)
                // Create the cache.
                .build(),
            simulations: HashMap::new(),
        }
    }

    pub async fn get_direct(&self, idx: SimulationsDataIdx) -> Result<SimulationData, SimulationDataError> {
        let simulation = self.simulations.get(&idx.0).ok_or(SimulationDataError::InvalidSimulationKey)?;
        match idx.1 {
            SimulationDataIdx::Devc(idx) => match simulation.csv_devc().await {
                Ok(devc) => Ok(SimulationData::Devc(Arc::new(devc))),
                Err(err) => Err(SimulationDataError::Devc(err)),
            },
            SimulationDataIdx::Slice(idx) => match simulation.slice(idx.0) {
                
            },
            SimulationDataIdx::Cpu(idx) => todo!(),
            SimulationDataIdx::Hrr(idx) => todo!(),
            SimulationDataIdx::S3d(idx) => todo!(),
            SimulationDataIdx::P3d(idx) => todo!(),
        }
    }
}
