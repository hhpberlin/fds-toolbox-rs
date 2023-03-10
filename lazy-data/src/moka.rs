use std::sync::Arc;

use fds_toolbox_core::{
    common::series::{TimeSeries0, TimeSeries2},
    formats::{
        simulation::{SliceSeriesIdx, TimeSeriesIdx},
        simulations::SimulationIdx,
    },
};
use moka::sync::Cache;

#[derive(Debug)]
// TODO: Remove dead_code. Here for a dark cockpit.
#[allow(dead_code)]

pub struct MokaStore {
    s0d: Cache<SimulationIdx<TimeSeriesIdx>, Arc<TimeSeries0>>,
    s2d: Cache<SimulationIdx<SliceSeriesIdx>, Arc<TimeSeries2>>,
    // s3d: Cache<SimulationIdx<VolumeSeriesIdx>, TimeSeries3>,
}

impl MokaStore {
    pub fn new(max_capacity: u64) -> Self {
        Self {
            s0d: Cache::new(max_capacity),
            s2d: Cache::new(max_capacity),
            // s3d: Cache::new(max_capacity),
        }
    }

    pub fn get(&self) {
        //self.s0d.get_with(key, init)
    }
}
