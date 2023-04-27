use std::{sync::Arc, time::Duration};

use fds_toolbox_core::{
    common::series::TimeSeries3,
    file::{FileSystem, Simulation},
    formats::{
        csv::{cpu::CpuData, devc::DeviceList, hrr::HrrStep},
        smoke::dim2::slice::Slice,
    },
};
use get_size::GetSize;

use crate::{
    cached::{Cached, CachedError},
    memman::CachedData,
};

#[derive(Debug)]
pub struct CachedSimulation<Fs: FileSystem> {
    sim: Arc<Simulation<Fs>>,
    devc: Cached<Arc<DeviceList>>,
    cpu: Cached<Arc<Option<CpuData>>>,
    hrr: Cached<Arc<Vec<HrrStep>>>,
    slice: Vec<Cached<Arc<Slice>>>,
    smoke3d: Vec<Cached<Arc<TimeSeries3>>>,
    plot3d: Vec<Cached<Arc<TimeSeries3>>>,
    refresh_interval: Option<Duration>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SliceIdx(usize);

impl<Fs: FileSystem> GetSize for CachedSimulation<Fs>
where
    Fs: FileSystem + GetSize,
    Fs::Path: GetSize,
{
    fn get_size(&self) -> usize {
        self.sim.get_size()
            + self.devc.get_size()
            + self.cpu.get_size()
            + self.hrr.get_size()
            + self.slice.get_size()
            + self.smoke3d.get_size()
            + self.plot3d.get_size()
    }
}

impl<Fs: FileSystem + 'static> CachedSimulation<Fs> {
    pub fn new(sim: Arc<Simulation<Fs>>, refresh_interval: Option<Duration>) -> Self {
        let n_slices = sim.smv.slices.len();
        let n_smoke3d = sim.smv.smoke3d.len();
        let n_plot3d = sim.smv.plot3d.len();

        Self {
            sim,
            devc: Cached::empty_enrolled(refresh_interval),
            cpu: Cached::empty_enrolled(refresh_interval),
            hrr: Cached::empty_enrolled(refresh_interval),
            slice: vec![Cached::empty_enrolled(refresh_interval); n_slices],
            smoke3d: vec![Cached::empty_enrolled(refresh_interval); n_smoke3d],
            plot3d: vec![Cached::empty_enrolled(refresh_interval); n_plot3d],
            refresh_interval,
        }
    }

    pub async fn get_devc(&self) -> Result<Arc<DeviceList>, CachedError> {
        let sim = self.sim.clone();
        self.devc
            .get_cached(move || Box::pin(async move { sim.csv_devc().await.map(Arc::new) }))
            .await
    }

    pub async fn get_cpu(&self) -> Result<Arc<Option<CpuData>>, CachedError> {
        let sim = self.sim.clone();
        self.cpu
            .get_cached(move || Box::pin(async move { sim.csv_cpu().await.map(Arc::new) }))
            .await
    }

    pub async fn get_hrr(&self) -> Result<Arc<Vec<HrrStep>>, CachedError> {
        let sim = self.sim.clone();
        self.hrr
            .get_cached(move || Box::pin(async move { sim.csv_hrr().await.map(Arc::new) }))
            .await
    }

    pub async fn get_slice(&self, idx: usize) -> Result<Arc<Slice>, CachedError> {
        let sim = self.sim.clone();
        self.slice[idx]
            .get_cached(move || Box::pin(async move { sim.slice(idx).await.map(Arc::new) }))
            .await
    }

    // pub async fn get_smoke3d(&self, idx: usize) -> Result<Arc<TimeSeries3>, CachedError> {
    //     let sim = self.sim.clone();
    //     self.smoke3d[idx].get_cached(move || Box::pin(async move {
    //         sim.smoke3d(idx).await.map(Arc::new)
    //     })).await
    // }

    // pub async fn get_plot3d(&self, idx: usize) -> Result<Arc<TimeSeries3>, CachedError> {
    //     let sim = self.sim.clone();
    //     self.plot3d[idx].get_cached(move || Box::pin(async move {
    //         sim.plot3d(idx).await.map(Arc::new)
    //     })).await
    // }
}
