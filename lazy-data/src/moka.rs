use std::{collections::HashMap, error::Error, hash::Hash, sync::Arc};

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
use parking_lot::RwLock;
use thiserror::Error;
use tracing::error;

use crate::fs::{AnyFs, FsErr};

// TODO: Remove dead_code. Here for a dark cockpit.
#[allow(dead_code)]
// TODO: Hand impl Debug
#[derive(Debug)]
pub struct MokaStore {
    cache: Cache<SimulationsDataIdx, SimulationData>,
    // simulations: DashMap<SimulationPath<AnyFs>, SimulationIdx>,
    // // simulations: DashMap<SimulationIdx, SimulationPath<AnyFs>>,
    // idx_cntr: AtomicUsize,
    idx_map: RwLock<IdxMap>,
}

#[derive(Debug)]
struct IdxMap {
    idx_to_path: HashMap<SimulationIdx, SimulationPath<AnyFs>>,
    path_to_idx: HashMap<SimulationPath<AnyFs>, SimulationIdx>,
    cntr: usize,
}

// impl Debug for MokaStore {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("MokaStore")
//             .field("cache", &self.cache)
//             .finish()
//     }
// }

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
pub struct DeviceIdx(usize);

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
pub struct SimulationsDataIdx(pub SimulationIdx, pub SimulationDataIdx);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SimulationIdx(usize);

/// Indexes into the simulation data of a single simulation (one .smv and associated files).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SimulationDataIdx {
    /// The simulation itself, i.e. the data in its .smv.
    Simulation,
    DevciceList,
    Slice(SliceIdx),
    Cpu,
    Hrr(HrrIdx),
    S3d(S3dIdx),
    P3d(P3dIdx),
}

#[derive(Debug, Clone)]
pub enum SimulationData {
    // TODO: This technically creates multiple sources of truths since the path is also stored in the Simulation.
    Simulation(Arc<Simulation<AnyFs>>),
    DevciceList(Arc<DeviceList>),
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
            SimulationData::DevciceList(x) => Arc::strong_count(x),
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
            SimulationData::DevciceList(x) => x.get_size(),
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
    (fn $name:ident < $t:tt > ( $($idx_ty:ty)? ) -> $data_ty:ty $({ $f:expr })?) => {
        pub async fn $name(
            &self,
            idx: SimulationIdx,
            $(inner_idx: $idx_ty)*
        ) -> Result<Arc<$data_ty>, Arc<SimulationDataError>> {
            match self.get(SimulationsDataIdx(idx, SimulationDataIdx::$t $((get_thing!(discard $idx_ty keep inner_idx)))*)).await {
                Ok(SimulationData::$t(sim)) => Ok($($f)? (sim)),
                // TODO: Proper error handling, eviction, etc.
                Ok(_) => unreachable!("Found wrong data type for given index."),
                Err(err) => Err(err),
            }
        }
    };
    (discard $_d:tt keep $k:tt) => { $k};
}

impl IdxMap {
    fn new() -> Self {
        Self {
            cntr: 0,
            idx_to_path: HashMap::new(),
            path_to_idx: HashMap::new(),
        }
    }

    fn insert(&mut self, path: &SimulationPath<AnyFs>) -> SimulationIdx {
        let idx = SimulationIdx(self.cntr);
        self.cntr += 1;

        self.idx_to_path.insert(idx, path.clone());
        self.path_to_idx.insert(path.clone(), idx);

        idx
    }

    fn try_get_by_path(&self, path: &SimulationPath<AnyFs>) -> Option<SimulationIdx> {
        self.path_to_idx.get(path).copied()
    }

    fn try_get_by_idx(&self, idx: SimulationIdx) -> Option<&SimulationPath<AnyFs>> {
        self.idx_to_path.get(&idx)
    }

    fn get_by_path_mut(&mut self, path: &SimulationPath<AnyFs>) -> SimulationIdx {
        match self.try_get_by_path(path) {
            Some(idx) => idx,
            None => self.insert(path),
        }
    }

    fn get_by_path_rw_lock(this: &RwLock<Self>, path: &SimulationPath<AnyFs>) -> SimulationIdx {
        if let Some(idx) = this.read().try_get_by_path(path) {
            return idx;
        }
        this.write().get_by_path_mut(path)
    }

    fn get_path_by_idx(&self, idx: SimulationIdx) -> Option<&SimulationPath<AnyFs>> {
        self.idx_to_path.get(&idx)
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
            // simulations: HashMap::new(),
            // simulations: DashMap::new(),
            // idx_cntr: AtomicUsize::new(0),
            idx_map: RwLock::new(IdxMap::new()),
        }
    }

    pub fn get_idx_by_path(&self, path: &SimulationPath<AnyFs>) -> SimulationIdx {
        IdxMap::get_by_path_rw_lock(&self.idx_map, path)
    }

    fn get_path_by_idx(&self, idx: SimulationIdx) -> Option<SimulationPath<AnyFs>> {
        self.idx_map.read().get_path_by_idx(idx).cloned()
    }

    pub async fn get_sim(
        &self,
        idx: SimulationIdx,
    ) -> Result<Arc<Simulation<AnyFs>>, Arc<SimulationDataError>> {
        let sim = self
            .cache
            .try_get_with(
                SimulationsDataIdx(idx, SimulationDataIdx::Simulation),
                async {
                    let Some(path) = self.get_path_by_idx(idx) else {
                        return Err(SimulationDataError::InvalidSimulationKey);
                    };

                    match path.clone().parse().await {
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

    get_thing!(fn get_devc<DevciceList>() -> DeviceList);
    get_thing!(fn get_cpu<Cpu>() -> Option<CpuData>);

    get_thing!(fn get_slice<Slice>(SliceIdx) -> Slice);
    get_thing!(fn get_hrr<Hrr>(HrrIdx) -> Vec<HrrStep>);
    get_thing!(fn get_s3d<S3d>(S3dIdx) -> TimeSeries3);
    get_thing!(fn get_p3d<P3d>(P3dIdx) -> TimeSeries3);

    pub async fn get(
        &self,
        idx: SimulationsDataIdx,
    ) -> Result<SimulationData, Arc<SimulationDataError>> {
        let simulation = self.get_sim(idx.0).await?;
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
                SimulationDataIdx::DevciceList => {
                    convert(simulation.csv_devc().await, SimulationData::DevciceList)
                }
                SimulationDataIdx::Slice(idx) => {
                    convert(simulation.slice(idx.0).await, SimulationData::Slice)
                }
                SimulationDataIdx::Cpu => convert(simulation.csv_cpu().await, SimulationData::Cpu),
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

    pub fn evict(&self, idx: SimulationIdx) {
        match self.cache.invalidate_entries_if(move |k, _| k.0 == idx) {
            Ok(_) => (),
            Err(err) => error!("Failed to evict simulation data: {}", err),
        }
    }
}
