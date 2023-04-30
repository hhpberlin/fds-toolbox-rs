use std::{
    hash::Hash,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use dashmap::DashMap;
use fds_toolbox_core::file::{FileSystem, ParseError, SimulationPath};
use get_size::GetSize;

use crate::{cached::Cached, sim::CachedSimulation};

pub struct Simulations<Fs: FileSystem + Eq + Hash = crate::fs::AnyFs> {
    simulations: DashMap<SimulationIdx, Cached<Arc<CachedSimulation<Fs>>>>,
    /// Maps a simulation path to a simulation index.
    /// Used to avoid having to compare full paths when indexing into `simulations`.
    by_path: DashMap<SimulationPath<Fs>, SimulationIdx>,
    idx_cntr: AtomicUsize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SimulationIdx(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BySimulation<T>(pub SimulationIdx, pub T);

impl<Fs: FileSystem + Eq + Hash> Simulations<Fs> {
    pub fn new() -> Self {
        Self {
            simulations: DashMap::new(),
            by_path: DashMap::new(),
            idx_cntr: AtomicUsize::new(0),
        }
    }

    fn get_new_idx(&self) -> SimulationIdx {
        let idx = self.idx_cntr.fetch_add(1, Ordering::SeqCst);
        SimulationIdx(idx)
    }

    fn add(&self, sim: Cached<Arc<CachedSimulation<Fs>>>) -> SimulationIdx {
        let idx = self.get_new_idx();
        self.simulations.insert(idx, sim);
        idx
    }

    pub fn add_by_path(&self, path: SimulationPath<Fs>) -> SimulationIdx
    where
        Fs: FileSystem + GetSize,
        Fs::Path: GetSize,
    {
        self.add(Cached::from_fut_enrolled::<ParseError<_, _>>(
            Box::pin(async move {
                let sim = path.parse().await?;
                Ok(Arc::new(CachedSimulation::new(Arc::new(sim), None)))
            }),
            None,
        ))
    }

    pub fn enumerate_simulations(
        &self,
    ) -> impl Iterator<Item = BySimulation<Cached<Arc<CachedSimulation<Fs>>>>> + '_ {
        self.simulations.iter().map(|entry| {
            let idx = entry.key();
            let sim = entry.value();
            BySimulation(*idx, sim.clone())
        })
    }

    pub fn iter_simulations(&self) -> impl Iterator<Item = Cached<Arc<CachedSimulation<Fs>>>> + '_ {
        self.simulations.iter().map(|entry| entry.value().clone())
    }
}

impl<Fs: FileSystem + Eq + Hash> Default for Simulations<Fs>
where
    Fs: FileSystem + GetSize,
    Fs::Path: GetSize,
{
    fn default() -> Self {
        Self::new()
    }
}
