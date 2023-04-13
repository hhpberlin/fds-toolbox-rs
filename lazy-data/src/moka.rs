use std::{collections::HashMap, error::Error, hash::Hash, io::Read, path::Path, sync::Arc};

use async_trait::async_trait;
use fds_toolbox_core::{
    common::series::TimeSeries3,
    file::{FileSystem, OsFs, ParseError, Simulation, SimulationPath},
    formats::{
        csv::{self, cpu::{CpuInfo, CpuData}, devc::DeviceList, hrr::HRRStep},
        smoke::dim2::slice::{self, Slice},
        smv::{self, Smv},
    },
};
use get_size::GetSize;
use moka::future::Cache;
use thiserror::Error;

// TODO: Remove dead_code. Here for a dark cockpit.
#[allow(dead_code)]
// TODO: Hand impl Debug
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
            Fs::LocalFs(fs) => fs
                .file_path(Path::new(directory), file_name)
                .to_str()
                .expect("Non-UTF8 paths are currently not supported.")
                .to_string(),
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
    Smv(Arc<Smv>),
    Devc(Arc<DeviceList>),
    Cpu(Option<Arc<CpuData>>),
    Hrr(Arc<Vec<HRRStep>>),
    Slice(Arc<Slice>),
    S3d(Arc<TimeSeries3>),
    P3d(Arc<TimeSeries3>),
}

impl SimulationData {
    fn ref_count(&self) -> usize {
        match self {
            SimulationData::Smv(x) => Arc::strong_count(x),
            SimulationData::Devc(x) => Arc::strong_count(x),
            SimulationData::Slice(x) => Arc::strong_count(x),
            SimulationData::Cpu(x) => x.as_ref().map(Arc::strong_count).unwrap_or(1),
            SimulationData::Hrr(x) => Arc::strong_count(x),
            SimulationData::S3d(x) => Arc::strong_count(x),
            SimulationData::P3d(x) => Arc::strong_count(x),
        }
    }

    fn size(&self) -> usize {
        match self {
            SimulationData::Smv(x) => x.get_size(),
            SimulationData::Devc(x) => x.get_size(),
            SimulationData::Slice(x) => x.get_size(),
            SimulationData::Cpu(x) => x.get_size(),
            SimulationData::Hrr(x) => x.get_size(),
            SimulationData::S3d(x) => x.get_size(),
            SimulationData::P3d(x) => x.get_size(),
        }
    }
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
    Fs(#[from] FsErr),
    Io(#[from] std::io::Error),
    Smv(#[from] smv::Error),
    Slice(#[from] slice::Error),
    Cpu(#[from] csv::cpu::Error),
    Hrr(#[from] csv::hrr::Error),
    Devc(#[from] csv::devc::Error),
}

impl<ParseErr: Error> From<ParseError<FsErr, ParseErr>> for SimulationDataError
where
    SimulationDataError: From<ParseErr>,
{
    fn from(value: ParseError<FsErr, ParseErr>) -> Self {
        match value {
            ParseError::Fs(err) => SimulationDataError::Fs(err),
            ParseError::Io(err) => SimulationDataError::Io(err),
            ParseError::Parse(err) => SimulationDataError::from(err),
        }
    }
}

impl MokaStore {
    pub fn new(max_capacity: u64) -> Self {
        Self {
            cache: Cache::builder()
                // Up to 10,000 entries.
                .max_capacity(max_capacity)
                // Create the cache.
                .weigher(|_k, v: &SimulationData| {
                    // This is a rather arbitrary way of weighing the values.

                    let s = v.size();

                    // Evicting values with references just loses the value and doesn't free the memory, so we weigh them higher to prevent that.
                    let r = v.ref_count();

                    s.ilog2() * (r as u32)
                })
                .build(),
            simulations: HashMap::new(),
        }
    }

    pub async fn get_direct(
        &self,
        idx: SimulationsDataIdx,
    ) -> Result<SimulationData, SimulationDataError> {
        let simulation = self
            .simulations
            .get(&idx.0)
            .ok_or(SimulationDataError::InvalidSimulationKey)?;

        fn convert<T, E: Into<SimulationDataError>>(
            res: Result<T, E>,
            f: impl FnOnce(Arc<T>) -> SimulationData,
        ) -> Result<SimulationData, SimulationDataError> {
            match res {
                Ok(x) => Ok(f(Arc::new(x))),
                Err(err) => Err(err.into()),
            }
        }

        match idx.1 {
            SimulationDataIdx::Devc(_idx) => convert(simulation.csv_devc().await, SimulationData::Devc),
            SimulationDataIdx::Slice(idx) => {
                convert(simulation.slice(idx.0).await, SimulationData::Slice)
            }
            SimulationDataIdx::Cpu(_idx) => {
                match simulation.csv_cpu().await {
                    Ok(x) => Ok(SimulationData::Cpu(x.map(Arc::new))),
                    Err(err) => Err(err.into()),
                }
            },
            SimulationDataIdx::Hrr(_idx) => convert(simulation.csv_hrr().await, SimulationData::Hrr),
            SimulationDataIdx::S3d(_idx) => todo!(),
            SimulationDataIdx::P3d(_idx) => todo!(),
        }
    }

    pub fn get(&self, _idx: SimulationDataIdx) {
        // self.cache.get_with(key, init)
        // self.cache.entry(idx).or_insert_with(init)
        // self.cache.get
    }
}
