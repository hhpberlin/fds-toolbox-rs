use std::{cell::RefCell, collections::VecDeque};

use fds_toolbox_core::{common::series::{TimeSeriesViewSource, PotentialResult, TimeSeriesView}, formats::{simulations::SimulationIdx, simulation::TimeSeriesIdx}};
use ndarray::{Ix1, Dimension};

struct Store<Id, Ix: Dimension> {
    request_queue: RefCell<VecDeque<Id>>,
    // data: DashMap<Id, PotentialResult<TimeSeriesView<f64, Ix, f64>>>,
    _phantom: std::marker::PhantomData<Ix>,
}

impl<Id, Ix: Dimension> Store<Id, Ix> {
    
}

impl TimeSeriesViewSource<SimulationIdx<TimeSeriesIdx>, f32, Ix1> for Store<SimulationIdx<TimeSeriesIdx>, Ix1> {
    fn get_time_series(&self, idx: SimulationIdx<TimeSeriesIdx>) -> PotentialResult<TimeSeriesView<f32, Ix1, f32>> {
        let SimulationIdx(idx, inner) = idx;
        // self.simulations.get(idx)?.get_time_series(inner)
        todo!()
    }
}