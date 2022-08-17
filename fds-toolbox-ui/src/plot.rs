use fds_toolbox_core::formats::arr_meta::Range;
use iced::{Element, Length, canvas::{Cache, Frame, Geometry}, Size};
use plotters::prelude::*;
use plotters_iced::{Chart, ChartWidget, DrawingBackend, ChartBuilder};

pub enum ChartMessage {

}

struct MyChart<Iter: IntoIterator<Item = (f32, f32)>> {
    cache: Cache,
    data: Iter,
    x_range: Range<f32>,
    y_range: Range<f32>,
}

impl<Iter: Iterator<Item = (f32, f32)>> Chart<ChartMessage> for MyChart<Iter> {
    fn build_chart<DB:DrawingBackend>(&self, mut chart: ChartBuilder<DB>) {
        let mut chart = chart
            .x_label_area_size(0)
            .y_label_area_size(28)
            .margin(20)
            .build_cartesian_2d(self.x_range.into_range(), self.y_range.into_range())
            .expect("failed to build chart");

        let color = Palette99::pick(4).mix(0.9);

        chart
            .draw_series(
                LineSeries::new(
                    self.data.into_iter(),
                    color.stroke_width(2),
                ),
            )
            .expect("failed to draw chart data");
    }
}

impl<Iter: Iterator<Item = (f32, f32)>> MyChart<Iter> {
    #[inline]
    fn draw<F: Fn(&mut Frame)>(&self, bounds: Size, draw_fn: F) -> Geometry {
        self.cache.draw(bounds, draw_fn)
    }
    
    fn view(&mut self)->Element<ChartMessage> {
        ChartWidget::new(self)
            .width(Length::Units(200))
            .height(Length::Units(200))
            .into()
    }
}