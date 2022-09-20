use serde::{Deserialize, Serialize};

use crate::common::series::{TimeSeriesView, TimeSeriesViewSource};

use super::csv::devc::{DeviceIdx, Devices};

#[derive(Debug)]
pub struct Simulation {
    pub devc: Devices,
}

impl TimeSeriesViewSource<TimeSeriesIdx> for Simulation {
    fn get_time_series(&self, idx: TimeSeriesIdx) -> Option<TimeSeriesView> {
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
