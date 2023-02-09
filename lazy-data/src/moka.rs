use fds_toolbox_core::{formats::{simulation::{TimeSeriesIdx, SliceSeriesIdx}, simulations::SimulationIdx}, common::series::{TimeSeries0, TimeSeries2}};
use moka::sync::Cache;

pub struct MokaStore {
    s0d: Cache<SimulationIdx<TimeSeriesIdx>, TimeSeries0>,
    s2d: Cache<SimulationIdx<SliceSeriesIdx>, TimeSeries2>,
    // s3d: Cache<SimulationIdx<VolumeSeriesIdx>, TimeSeries3>,
}

