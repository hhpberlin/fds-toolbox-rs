use fds_toolbox_core::formats::arr_meta::Range;
use iced::{Element, Length, canvas::{Cache, Frame, Geometry}, Size};
use plotters::prelude::*;
use plotters_iced::{Chart, ChartWidget, DrawingBackend, ChartBuilder};

#[derive(Debug, Clone, Copy)]
pub enum ChartMessage {

}

pub struct MyChart<Iter>
    where for<'a> &'a Iter: IntoIterator<Item = (f32, f32)>
{
    cache: Cache,
    data: Option<MyChartData<Iter>>,
}

pub struct MyChartData<Iter>
    where for<'a> &'a Iter: IntoIterator<Item = (f32, f32)>
{
    data: Iter,
    x_range: Range<f32>,
    y_range: Range<f32>,
}

impl<Iter> Chart<ChartMessage> for MyChart<Iter>
    where for<'a> &'a Iter: IntoIterator<Item = (f32, f32)>
{
    #[inline]
    fn draw<F: Fn(&mut Frame)>(&self, bounds: Size, draw_fn: F) -> Geometry {
        self.cache.draw(bounds, draw_fn)
    }

    fn build_chart<DB:DrawingBackend>(&self, mut chart: ChartBuilder<DB>) {
        let chart = chart
            .x_label_area_size(0)
            .y_label_area_size(28)
            .margin(20);

        let data = match &self.data {
            Some(data) => data,
            None => return,
        };

        let mut chart = chart
            .build_cartesian_2d(data.x_range.into_range(), data.y_range.into_range())
            .expect("failed to build chart");


        let color = Palette99::pick(4).mix(0.9);

        chart
            .draw_series(
                LineSeries::new(
                    data.data.into_iter(),
                    color.stroke_width(2),
                ),
            )
            .expect("failed to draw chart data");
    }
}

pub fn get_range<T: Copy + PartialOrd>(iter: impl Iterator<Item = (T, T)>) -> Option<(Range<T>, Range<T>)> {
    let first = iter.next()?;
    let xr = Range::new(first.0, first.0);
    let yr = Range::new(first.1, first.1);
    Some(iter.fold((xr, yr), |(xr, yr), (x, y)| (xr.expand(x), yr.expand(y))))
}

impl<Iter> MyChart<Iter>
    where for<'a> &'a Iter: IntoIterator<Item = (f32, f32)>
{
    pub fn from_(data: Iter) -> Self {
        let r = get_range(data.into_iter());
        let data = match r {
            Some((x_range, y_range)) => Some(MyChartData {
                data,
                x_range,
                y_range,
            }),
            None => None,
        };
        Self {
            cache: Cache::new(),
            data,
        }
    }

    pub fn view(&mut self)->Element<ChartMessage> {
        ChartWidget::new(self)
            .width(Length::Units(200))
            .height(Length::Units(200))
            .into()
    }
}