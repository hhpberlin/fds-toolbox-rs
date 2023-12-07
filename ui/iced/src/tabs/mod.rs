use std::cell::RefCell;

use fds_toolbox_lazy_data::moka::SimulationIdx;



#[derive(Debug, Clone)]
pub enum TabMessage {
    Replace(Tab),
    Plot(crate::plotters::cartesian::Message),
}

#[derive(Debug, Clone)]
pub enum Tab {
    Home,
    Overview(SimulationIdx),
    Plot(RefCell<crate::plotters::cartesian::State>),
}