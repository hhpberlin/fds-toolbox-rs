

use ndarray::Ix1;
use serde::{Deserialize, Serialize};

use crate::common::series::{TimeSeries1View, TimeSeriesViewSource};

use super::{
    csv::devc::{DeviceIdx, Devices},
    slcf::Slice,
};

#[derive(Debug)]
pub struct Simulation {
    pub devc: Devices,
    pub slcf: Vec<Slice>,
}

impl TimeSeriesViewSource<TimeSeriesIdx, f32, Ix1> for Simulation {
    fn get_time_series(&self, idx: TimeSeriesIdx) -> Option<TimeSeries1View> {
        match idx {
            TimeSeriesIdx::Device(idx) => self.devc.get_time_series(idx),
        }
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
pub struct SliceIdx(pub u32);
