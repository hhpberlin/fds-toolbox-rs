use crate::common::series::{TimeSeriesViewSource, TimeSeriesView};

use super::csv::devc::{Devices, DeviceIdx};

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

// pub enum Idx {

// }

#[derive(Debug, Clone, Copy)]
pub enum TimeSeriesIdx {
    Device(DeviceIdx),
}