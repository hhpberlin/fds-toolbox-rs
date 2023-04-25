use std::{error::Error, hash::Hash, sync::Arc};

use fds_toolbox_core::{
    common::series::TimeSeries3,
    file::{self, ParseError, Simulation, SimulationPath},
    formats::{
        csv::{self, cpu::CpuData, devc::DeviceList, hrr::HrrStep},
        smoke::dim2::slice::{self, Slice},
    },
};
use get_size::GetSize;
use moka::future::Cache;
use thiserror::Error;

use crate::fs::{AnyFs, FsErr};

// TODO: Remove dead_code. Here for a dark cockpit.
#[allow(dead_code)]
// TODO: Hand impl Debug
pub struct MokaStore {
    cache: Cache<SimulationsDataIdx, SimulationData>,
    // simulations: HashMap<SimulationPath<Fs>, Simulation<Fs>>,
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

/// Indexes into the simulation data of any simulation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SimulationsDataIdx(pub SimulationPath<AnyFs>, pub SimulationDataIdx);

/// Indexes into the simulation data of a single simulation (one .smv and associated files).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SimulationDataIdx {
    /// The simulation itself, i.e. the data in its .smv.
    Simulation,
    Devc(DevcIdx),
    Slice(SliceIdx),
    Cpu(CpuIdx),
    Hrr(HrrIdx),
    S3d(S3dIdx),
    P3d(P3dIdx),
}

#[derive(Debug, Clone)]
pub enum SimulationData {
    // TODO: This technically creates multiple sources of truths since the path is also stored in the Simulation.
    Simulation(Arc<Simulation<AnyFs>>),
    Devc(Arc<DeviceList>),
    Cpu(Arc<Option<CpuData>>),
    Hrr(Arc<Vec<HrrStep>>),
    Slice(Arc<Slice>),
    S3d(Arc<TimeSeries3>),
    P3d(Arc<TimeSeries3>),
}

impl SimulationData {
    fn ref_count(&self) -> usize {
        match self {
            SimulationData::Simulation(x) => Arc::strong_count(x),
            SimulationData::Devc(x) => Arc::strong_count(x),
            SimulationData::Slice(x) => Arc::strong_count(x),
            SimulationData::Cpu(x) => Arc::strong_count(x),
            SimulationData::Hrr(x) => Arc::strong_count(x),
            SimulationData::S3d(x) => Arc::strong_count(x),
            SimulationData::P3d(x) => Arc::strong_count(x),
        }
    }

    fn size(&self) -> usize {
        match self {
            SimulationData::Simulation(x) => x.get_size(),
            SimulationData::Devc(x) => x.get_size(),
            SimulationData::Slice(x) => x.get_size(),
            SimulationData::Cpu(x) => x.get_size(),
            SimulationData::Hrr(x) => x.get_size(),
            SimulationData::S3d(x) => x.get_size(),
            SimulationData::P3d(x) => x.get_size(),
        }
    }
}

#[derive(Debug, Error)]
#[error(transparent)]
pub enum SimulationDataError {
    #[error("Invalid simulation key")]
    InvalidSimulationKey,
    // Io(std::io::Error),
    Fs(#[from] FsErr),
    Io(#[from] std::io::Error),
    Smv(#[from] file::SmvErr),
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

macro_rules! get_thing {
    (fn $name:ident < $t:tt > ( $idx_ty:ty ) -> $data_ty:ty $({ $f:expr })?) => {
        pub async fn $name(
            &self,
            path: SimulationPath<AnyFs>,
            idx: $idx_ty,
        ) -> Result<Arc<$data_ty>, Arc<SimulationDataError>> {
            match self.get(SimulationsDataIdx(path, SimulationDataIdx::$t(idx))).await {
                Ok(SimulationData::$t(sim)) => Ok($($f)? (sim)),
                // TODO: Proper error handling, eviction, etc.
                Ok(_) => unreachable!("Found wrong data type for given index."),
                Err(err) => Err(err),
            }
        }
    };
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
            // simulations: HashMap::new(),
        }
    }

    async fn get_sim(
        &self,
        path: SimulationPath<AnyFs>,
    ) -> Result<Arc<Simulation<AnyFs>>, Arc<SimulationDataError>> {
        let sim = self
            .cache
            .try_get_with(
                SimulationsDataIdx(path.clone(), SimulationDataIdx::Simulation),
                async {
                    let sim = path.parse().await;
                    match sim {
                        Ok(sim) => Ok(SimulationData::Simulation(Arc::new(sim))),
                        Err(err) => Err::<_, SimulationDataError>(err.into()),
                    }
                },
            )
            .await;

        match sim {
            Ok(SimulationData::Simulation(sim)) => Ok(sim),
            // TODO: Proper error handling, eviction, etc.
            Ok(_) => unreachable!("Found wrong data type for given index."),
            Err(err) => Err(err),
        }
    }

    // pub async fn get_smv(
    //     &self,
    //     path: SimulationPath<Fs>,
    // ) -> Result<Arc<Smv>, Arc<SimulationDataError>> {
    //     self.get_sim(path).await?.smv
    // }

    get_thing!(fn get_devc <Devc >(DevcIdx ) ->  DeviceList );
    get_thing!(fn get_slice<Slice>(SliceIdx) ->  Slice      );
    get_thing!(fn get_cpu  <Cpu  >(CpuIdx  ) ->  Option<CpuData>);
    get_thing!(fn get_hrr  <Hrr  >(HrrIdx  ) ->  Vec<HrrStep>);
    get_thing!(fn get_s3d  <S3d  >(S3dIdx  ) ->  TimeSeries3);
    get_thing!(fn get_p3d  <P3d  >(P3dIdx  ) ->  TimeSeries3);

    pub async fn get(
        &self,
        idx: SimulationsDataIdx,
    ) -> Result<SimulationData, Arc<SimulationDataError>> {
        let simulation = self.get_sim(idx.0.clone()).await?;
        // .ok_or(SimulationDataError::InvalidSimulationKey)?;

        fn convert<T, E: Into<SimulationDataError>>(
            res: Result<T, E>,
            f: impl FnOnce(Arc<T>) -> SimulationData,
        ) -> Result<SimulationData, SimulationDataError> {
            match res {
                Ok(x) => Ok(f(Arc::new(x))),
                Err(err) => Err(err.into()),
            }
        }

        let fut = async {
            match &idx.1 {
                SimulationDataIdx::Simulation => Ok(SimulationData::Simulation(simulation)),
                SimulationDataIdx::Devc(_idx) => {
                    convert(simulation.csv_devc().await, SimulationData::Devc)
                }
                SimulationDataIdx::Slice(idx) => {
                    convert(simulation.slice(idx.0).await, SimulationData::Slice)
                }
                SimulationDataIdx::Cpu(_idx) => {
                    convert(simulation.csv_cpu().await, SimulationData::Cpu)
                }
                // match simulation.csv_cpu().await {
                //     Ok(x) => Ok(SimulationData::Cpu(x.map(Arc::new))),
                //     Err(err) => Err(err.into()),
                // },
                SimulationDataIdx::Hrr(_idx) => {
                    convert(simulation.csv_hrr().await, SimulationData::Hrr)
                }
                SimulationDataIdx::S3d(_idx) => todo!(),
                SimulationDataIdx::P3d(_idx) => todo!(),
            }
        };

        self.cache.try_get_with(idx.clone(), fut).await
    }
}
