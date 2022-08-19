use std::fmt::Debug;

use fds_toolbox_core::formats::{arr_meta::Range, csv::devc::Device};
use iced::{
    canvas::{Cache, Frame, Geometry},
    Element, Length, Size,
};
use plotters::prelude::*;
use plotters_iced::{Chart, ChartBuilder, ChartWidget, DrawingBackend};

use super::plottable::Plottable2D;
// use uom::si::f32::Time;

#[derive(Debug, Clone, Copy)]
pub enum ChartMessage {}

#[derive(Debug)]
pub struct Plot2D {
    cache: Cache,
    data: Vec<Box<dyn Plottable2D>>,
}

impl Chart<ChartMessage> for Plot2D {
    #[inline]
    fn draw<F: Fn(&mut Frame)>(&self, bounds: Size, draw_fn: F) -> Geometry {
        self.cache.draw(bounds, draw_fn)
    }

    fn build_chart<DB: DrawingBackend>(&self, mut chart: ChartBuilder<DB>) {
        let chart = chart.x_label_area_size(30).y_label_area_size(30).margin(20);

        // TODO: Avoid alloc by reusing iterator?
        let data = self.data.iter().map(|x| x.plot_data()).filter_map(|x| x).collect::<Vec<_>>(); 

        let x_range = Range::from_iter_range(data.iter().map(|x| x.x_range));
        let y_range = Range::from_iter_range(data.iter().map(|x| x.y_range));

        let (x_range, y_range) = match (x_range, y_range) {
            (Some(x_range), Some(y_range)) => (x_range, y_range),
            // _ => return,
            _ => (Range::new(0.0, 1.0), Range::new(0.0, 1.0)),
        };

        let mut chart = chart
            .build_cartesian_2d(x_range.into_range(), y_range.into_range())
            .expect("failed to build chart");

        chart.configure_mesh().draw().expect("failed to draw mesh");

        let color = Palette99::pick(4).mix(0.9);

        for data in data.into_iter() {
            chart
                .draw_series(LineSeries::new(
                    data.data,
                    color.stroke_width(2),
                ))
                // .label("y = x^2")
                // .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED))
                .expect("failed to draw chart data");
        }


        // chart
        //     .configure_series_labels()
        //     .background_style(&WHITE.mix(0.8))
        //     .border_style(&BLACK)
        //     .draw()
        //     .expect("failed to draw chart labels");
    }
}

impl Plot2D {
    pub fn from_(data: Vec<(f32, f32)>) -> Self {
        Self {
            cache: Cache::new(),
            data: vec![Box::new(data)],
        }
    }

    pub fn view(&mut self) -> Element<ChartMessage> {
        ChartWidget::new(self)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
