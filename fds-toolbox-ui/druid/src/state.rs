use std::rc::Rc;

use druid::{Data, Lens};
use fds_toolbox_core::formats::simulations::Simulations;

use crate::plot_2d::plot_tab::Plot2DTabData;

#[derive(Clone, Data, Lens)]
pub struct FdsToolboxApp {
    pub data: FdsToolboxData,
    pub tab_data: Plot2DTabData,
}

#[derive(Clone, Data, Lens)]
pub struct FdsToolboxData {
    pub simulations: Rc<Simulations>,
}

// pub struct
