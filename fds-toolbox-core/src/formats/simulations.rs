use std::ops::Index;

use crate::common::series::{TimeSeriesView, TimeSeriesViewSource};

use super::simulation::{Simulation, TimeSeriesIdx};

#[derive(Debug)]
pub struct Simulations {
    simulations: Vec<Simulation>,
}

impl Simulations {
    pub fn empty() -> Self {
        Self { simulations: vec![] }
    }

    pub fn new(simulations: Vec<Simulation>) -> Self {
        Self { simulations }
    }
}

impl Index<usize> for Simulations {
    type Output = Simulation;

    fn index(&self, index: usize) -> &Self::Output {
        &self.simulations[index]
    }
}

impl TimeSeriesViewSource<GlobalTimeSeriesIdx> for Simulations {
    fn get_time_series(&self, idx: GlobalTimeSeriesIdx) -> Option<TimeSeriesView> {
        let GlobalTimeSeriesIdx(idx, inner) = idx;
        self.simulations.get(idx)?.get_time_series(inner)
    }
}

// TODO: Should this have public fields or be an opaque type instantiated by Simulations?
#[derive(Debug, Clone, Copy)]
pub struct GlobalTimeSeriesIdx(pub usize, pub TimeSeriesIdx);
