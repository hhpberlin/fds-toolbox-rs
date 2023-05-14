use std::{collections::HashMap, error::Error, hash::Hash, marker::PhantomData, sync::Arc};

use derive_more::Unwrap;
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
#[derive(Debug, Clone)]
pub struct MokaStore {
    cache: Cache<SimulationsDataIdx, SimulationData>,
    // simulations: DashMap<SimulationPath<AnyFs>, SimulationIdx>,
    // // simulations: DashMap<SimulationIdx, SimulationPath<AnyFs>>,
    // idx_cntr: AtomicUsize,
    idx_map: Arc<RwLock<IdxMap>>,
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

#[derive(Debug, Clone, Unwrap)]
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

    // fn try_get_by_idx(&self, idx: SimulationIdx) -> Option<&SimulationPath<AnyFs>> {
    //     self.idx_to_path.get(&idx)
    // }

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

// trait MokaDataType<Idx> {
//     fn try_into(data: SimulationDataIdx) -> Option<Idx>;
//     fn idx(idx: Idx) -> SimulationDataIdx;
// }

// impl MokaDataType<()> for () {
//     fn try_into(data: SimulationDataIdx) -> Option<()> {
//         match data {
//             SimulationDataIdx::Simulation(_) => Some(()),
//             _ => None,
//         }
//     }

//     fn idx(_: ()) -> SimulationDataIdx {
//         SimulationDataIdx::Simulation
//     }
// }

pub struct DataSrc<'a, Idx, Data>
where
    Data: DataType<Idx = Idx>,
{
    store: &'a MokaStore,
    _idx: PhantomData<Idx>,
    _data: PhantomData<Data>,
}

pub trait DataType
where
    Self: Sized,
{
    type Idx;
    fn make_idx(idx: Self::Idx) -> SimulationDataIdx;
    fn unwrap_data(data: SimulationData) -> Option<Self>;
}

macro_rules! data_type_impl {
    ($ty:ty, $idx:ty, $idx_ident:ident, $idx_variant:expr, $data_variant:tt) => {
        impl DataType for $ty {
            type Idx = $idx;

            fn make_idx($idx_ident: Self::Idx) -> SimulationDataIdx {
                // SimulationDataIdx::$variant(idx)
                $idx_variant
            }

            fn unwrap_data(data: SimulationData) -> Option<Self> {
                match data {
                    SimulationData::$data_variant(x) => Some(x),
                    _ => None,
                }
            }
        }
    };
    ($ty:ty, $variant:tt) => {
        data_type_impl!($ty, (), _idx, SimulationDataIdx::$variant, $variant);
    };
    ($ty:ty, $idx:ty, $variant:tt) => {
        data_type_impl!($ty, $idx, $variant - -idx);
    };
    ($ty:ty, $idx:ty, $variant:tt -- $idx_ident:ident) => {
        data_type_impl!(
            $ty,
            $idx,
            $idx_ident,
            SimulationDataIdx::$variant($idx_ident),
            $variant
        );
    };
}

data_type_impl!(Arc<Simulation<AnyFs>>, Simulation);
data_type_impl!(Arc<DeviceList>, DevciceList);
data_type_impl!(Arc<Option<CpuData>>, Cpu);
data_type_impl!(Arc<Slice>, SliceIdx, Slice);
data_type_impl!(Arc<Vec<HrrStep>>, HrrIdx, Hrr);
// data_type_impl!(Arc<TimeSeries3>, P3dIdx, P3d);
// data_type_impl!(Arc<TimeSeries3>, S3dIdx, S3d);

impl<'a, Idx, Data> DataSrc<'a, Idx, Data>
where
    Data: DataType<Idx = Idx>,
{
    pub fn new(store: &'a MokaStore) -> Self {
        Self {
            store,
            _idx: PhantomData,
            _data: PhantomData,
        }
    }

    pub async fn get(
        &self,
        sim: SimulationIdx,
        idx: Idx,
    ) -> Result<Data, Arc<SimulationDataError>> {
        let idx = SimulationsDataIdx(sim, Data::make_idx(idx));
        let data = self.store.get(idx).await?;
        Data::unwrap_data(data).ok_or_else(|| Arc::new(SimulationDataError::InvalidSimulationKey))
    }

    pub async fn load(&self, sim: SimulationIdx, idx: Idx) -> Result<(), Arc<SimulationDataError>> {
        self.get(sim, idx).await?;
        Ok(())
    }

    pub fn try_get_no_load(&self, sim: SimulationIdx, idx: Idx) -> Option<Data> {
        let idx = SimulationsDataIdx(sim, Data::make_idx(idx));
        let data = self.store.try_get(&idx);
        data.and_then(Data::unwrap_data)
    }

    pub fn try_get(&self, sim: SimulationIdx, idx: Idx) -> Option<Data> {
        let idx = SimulationsDataIdx(sim, Data::make_idx(idx));
        let data = self.store.try_get_or_spawn(idx);
        data.and_then(Data::unwrap_data)
    }

    pub fn make_idx(&self, sim: SimulationIdx, idx: Idx) -> SimulationsDataIdx {
        SimulationsDataIdx(sim, Data::make_idx(idx))
    }

    pub fn unwrap_data(&self, data: SimulationData) -> Option<Data> {
        Data::unwrap_data(data)
    }
}

// impl<'a, Data> DataSrc<'a, (), Data>
// where
//     Data: DataType<Idx = ()>,
// {
//     pub async fn get(&self, sim: SimulationIdx) -> Result<Data, Arc<SimulationDataError>> {
//         self.get(sim, ())
//     }

//     pub fn try_get(&self, sim: SimulationIdx) -> Option<Data> {
//         self.try_get(sim, ())
//     }

//     pub fn try_get_or_spawn(&self, sim: SimulationIdx) -> Option<Data> {
//         self.try_get_or_spawn(sim, ())
//     }
// }

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
            idx_map: Arc::new(RwLock::new(IdxMap::new())),
        }
    }

    pub fn get_idx_by_path(&self, path: &SimulationPath<AnyFs>) -> SimulationIdx {
        IdxMap::get_by_path_rw_lock(&self.idx_map, path)
    }

    pub fn get_path_by_idx(&self, idx: SimulationIdx) -> Option<SimulationPath<AnyFs>> {
        self.idx_map.read().get_path_by_idx(idx).cloned()
    }

    async fn get_sim(
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

        Ok(sim?.unwrap_simulation())
    }

    // pub async fn get_smv(
    //     &self,
    //     path: SimulationPath<Fs>,
    // ) -> Result<Arc<Smv>, Arc<SimulationDataError>> {
    //     self.get_sim(path).await?.smv
    // }

    // pub async fn get_devc(
    //     &self,
    //     idx: SimulationIdx,
    // ) -> Result<Arc<DeviceList>, Arc<SimulationDataError>> {
    //     Ok(self
    //         .get(SimulationsDataIdx(idx, SimulationDataIdx::DevciceList))
    //         .await?
    //         .unwrap())
    // }
    // pub async fn get_cpu(
    //     &self,
    //     idx: SimulationIdx,
    // ) -> Result<Arc<Option<CpuData>>, Arc<SimulationDataError>> {
    //     Ok(self
    //         .get(SimulationsDataIdx(idx, SimulationDataIdx::Cpu))
    //         .await?
    //         .unwrap())
    // }

    // pub async fn get_slice(
    //     &self,
    //     idx: SimulationIdx,
    //     inner_idx: SliceIdx,
    // ) -> Result<Arc<Slice>, Arc<SimulationDataError>> {
    //     Ok(self
    //         .get(SimulationsDataIdx(
    //             idx,
    //             SimulationDataIdx::Slice(get_thing!(discard SliceIdx keep inner_idx)),
    //         ))
    //         .await?
    //         .unwrap())
    // }
    // pub async fn get_hrr(
    //     &self,
    //     idx: SimulationIdx,
    //     inner_idx: HrrIdx,
    // ) -> Result<Arc<Vec<HrrStep>>, Arc<SimulationDataError>> {
    //     Ok(self
    //         .get(SimulationsDataIdx(
    //             idx,
    //             SimulationDataIdx::Hrr(get_thing!(discard HrrIdx keep inner_idx)),
    //         ))
    //         .await?
    //         .unwrap())
    // }
    // pub async fn get_s3d(
    //     &self,
    //     idx: SimulationIdx,
    //     inner_idx: S3dIdx,
    // ) -> Result<Arc<TimeSeries3>, Arc<SimulationDataError>> {
    //     Ok(self
    //         .get(SimulationsDataIdx(
    //             idx,
    //             SimulationDataIdx::S3d(get_thing!(discard S3dIdx keep inner_idx)),
    //         ))
    //         .await?
    //         .unwrap())
    // }
    // pub async fn get_p3d(
    //     &self,
    //     idx: SimulationIdx,
    //     inner_idx: P3dIdx,
    // ) -> Result<Arc<TimeSeries3>, Arc<SimulationDataError>> {
    //     Ok(self
    //         .get(SimulationsDataIdx(
    //             idx,
    //             SimulationDataIdx::P3d(get_thing!(discard P3dIdx keep inner_idx)),
    //         ))
    //         .await?
    //         .unwrap())
    // }

    pub fn sim(&self) -> DataSrc<(), Arc<Simulation<AnyFs>>> {
        DataSrc::new(self)
    }
    pub fn devc(&self) -> DataSrc<(), Arc<DeviceList>> {
        DataSrc::new(self)
    }
    pub fn cpu(&self) -> DataSrc<(), Arc<Option<CpuData>>> {
        DataSrc::new(self)
    }
    pub fn slice(&self) -> DataSrc<SliceIdx, Arc<Slice>> {
        DataSrc::new(self)
    }
    pub fn hrr(&self) -> DataSrc<HrrIdx, Arc<Vec<HrrStep>>> {
        DataSrc::new(self)
    }
    // pub fn s3d(&self) -> DataSrc<S3dIdx, Arc<TimeSeries3>> { DataSrc::new(self) }
    // pub fn p3d(&self) -> DataSrc<P3dIdx, Arc<TimeSeries3>> { DataSrc::new(self) }

    pub fn try_get(&self, idx: &SimulationsDataIdx) -> Option<SimulationData> {
        self.cache.get(idx)
    }

    pub async fn get(
        &self,
        idx: SimulationsDataIdx,
    ) -> Result<SimulationData, Arc<SimulationDataError>> {
        match self.try_get(&idx) {
            Some(data) => Ok(data),
            None => self.get_core(idx).await,
        }
    }

    pub fn try_get_or_spawn(&self, idx: SimulationsDataIdx) -> Option<SimulationData> {
        match self.try_get(&idx) {
            Some(data) => Some(data),
            None => {
                let this = self.clone();
                tokio::spawn(async move { this.get_core(idx).await });
                None
            }
        }
    }

    async fn get_core(
        &self,
        idx: SimulationsDataIdx,
    ) -> Result<SimulationData, Arc<SimulationDataError>> {
        fn convert<T, E: Into<SimulationDataError>>(
            res: Result<T, E>,
            f: impl FnOnce(Arc<T>) -> SimulationData,
        ) -> Result<SimulationData, SimulationDataError> {
            match res {
                Ok(x) => Ok(f(Arc::new(x))),
                Err(err) => Err(err.into()),
            }
        }

        // There is a small chance this will be called twice at once, but it's not a big deal,
        //  as `get_sim` deduplicates the expensive part internally.
        // The reason this isn't inside `fut` is because it returns a `Result` with
        //  its error wrapped in `Arc` but `get_with` would wrap it in another `Arc`.
        let simulation = self.get_sim(idx.0).await?;
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
