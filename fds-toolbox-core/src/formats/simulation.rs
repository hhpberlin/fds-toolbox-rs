use ndarray::{Ix1, Ix3};
use serde::{Deserialize, Serialize};

use crate::common::series::{TimeSeries0View, TimeSeries2View, TimeSeriesViewSource};

use super::{
    csv::devc::{DeviceIdx, Devices},
    smoke::dim2::slice::Slice,
    // slcf::Slice,
};

#[derive(Debug)]
pub struct Simulation {
    pub devc: Devices,
    pub slcf: Vec<Slice>,
}

impl TimeSeriesViewSource<TimeSeriesIdx, f32, Ix1> for Simulation {
    fn get_time_series(&self, idx: TimeSeriesIdx) -> Option<TimeSeries0View> {
        match idx {
            TimeSeriesIdx::Device(idx) => self.devc.get_time_series(idx),
        }
    }
}

impl TimeSeriesViewSource<SliceSeriesIdx, f32, Ix3> for Simulation {
    fn get_time_series(&self, idx: SliceSeriesIdx) -> Option<TimeSeries2View> {
        self.slcf.get(idx.0).map(|slice| slice.frames.view())
    }
}

pub enum SimulationData2D<'a> {
    Device(&'a str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TimeSeriesIdx {
    Device(DeviceIdx),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SliceSeriesIdx(pub usize);
