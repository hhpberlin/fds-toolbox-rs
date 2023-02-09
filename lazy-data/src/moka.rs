use std::sync::Arc;

use fds_toolbox_core::{formats::{simulation::{TimeSeriesIdx, SliceSeriesIdx}, simulations::SimulationIdx}, common::series::{TimeSeries0, TimeSeries2}};
use moka::sync::Cache;

#[derive(Debug)]
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


}