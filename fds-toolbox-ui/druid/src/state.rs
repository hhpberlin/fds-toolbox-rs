use std::rc::Rc;

use druid::{Data, Lens};
use fds_toolbox_core::formats::simulations::Simulations;

#[derive(Clone, Data, Lens)]
pub struct FdsToolboxApp {
    pub simulations: Rc<Simulations>,
}
