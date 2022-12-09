use fds_toolbox_core::common::series::TimeSeriesViewSource;
use ndarray::Ix2;

use super::{
    cartesian::{Cartesian2df32, CartesianDrawer},
    ids::IdSource,
};

pub struct Heatmap<Id, DataSrc: TimeSeriesViewSource<Id, f32, Ix2>, IdSrc: IdSource<Id = Id>> {
    data_source: DataSrc,
    id_source: IdSrc,
}

impl<Id: Copy, DataSrc: TimeSeriesViewSource<Id, f32, Ix2>, IdSrc: IdSource<Id = Id>>
    CartesianDrawer for Heatmap<Id, DataSrc, IdSrc>
{
    fn draw<DB: plotters_iced::DrawingBackend>(
        &self,
        _chart: &mut plotters::prelude::ChartContext<DB, Cartesian2df32>,
        _state: &super::cartesian::State,
    ) {
    }
}

impl<Id, DataSrc: TimeSeriesViewSource<Id, f32, Ix2>, IdSrc: IdSource<Id = Id>>
    Heatmap<Id, DataSrc, IdSrc>
{
    pub fn new(data_source: DataSrc, id_source: IdSrc) -> Self {
        Self {
            data_source,
            id_source,
        }
    }
}
