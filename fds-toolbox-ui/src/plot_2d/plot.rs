use std::fmt::Debug;

use fds_toolbox_core::formats::{arr_meta::Range, csv::devc::Device};
use iced::{
    canvas::{Cache, Frame, Geometry},
    Element, Length, Size,
};
use plotters::prelude::*;
use plotters_iced::{Chart, ChartBuilder, ChartWidget, DrawingBackend};
// use uom::si::f32::Time;

#[derive(Debug, Clone, Copy)]
pub enum ChartMessage {}

#[derive(Debug)]
pub struct Plot2D {
    cache: Cache,
    data: Vec<Box<dyn Plottable2D>>,
}

type BoxCoordIter = Box<dyn Iterator<Item = (f32, f32)>>;
type Plot2DDataBoxed = Plot2DData<BoxCoordIter>;

pub struct Plot2DData<Iter: Iterator<Item = (f32, f32)>> {
    data: Iter,
    x_range: Range<f32>,
    y_range: Range<f32>,
}

impl<Iter: Iterator<Item = (f32, f32)>> Debug for Plot2DData<Iter> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Plot2DData")
            .field("x_range", &self.x_range)
            .field("y_range", &self.y_range)
            .finish()
    }
}

impl Plottable2D for Vec<(f32, f32)> {
    fn plot_data<'a>(&'a self) -> Option<Plot2DData<Box<dyn Iterator<Item = (f32, f32)> + 'a>>> {
        let (x_range, y_range) = match (Range::range_iter(self.iter().map(|(x, _)| *x)), Range::range_iter(self.iter().map(|(_, y)| *y))) {
            (Some(x_range), Some(y_range)) => (x_range, y_range),
            _ => return None,
        };
        Some(Plot2DData {
            data: Box::new(self.iter().copied()),
            x_range,
            y_range,
        })
    }
}

pub trait Plottable2D: Debug {
    // TODO: Tracking https://github.com/rust-lang/rust/issues/63063 to avoid alloc
    // type IntoIter: Iterator<Item = (f32, f32)>;
    // type IntoIter = impl Iterator<Item = (f32, f32)>;
    fn plot_data<'a>(&'a self) -> Option<Plot2DData<Box<dyn Iterator<Item = (f32, f32)> + 'a>>>;
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

        let x_range = Range::max_iter(data.iter().map(|x| x.x_range));
        let y_range = Range::max_iter(data.iter().map(|x| x.y_range));

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

pub fn get_range<X: Copy + PartialOrd, Y: Copy + PartialOrd>(
    mut iter: impl Iterator<Item = (X, Y)>,
) -> Option<(Range<X>, Range<Y>)> {
    let first = iter.next()?;
    let xr = Range::new(first.0, first.0);
    let yr = Range::new(first.1, first.1);
    Some(iter.fold((xr, yr), |(xr, yr), (x, y)| (xr.expand(x), yr.expand(y))))
}

impl Plot2D {
    pub fn from_(data: Vec<(f32, f32)>) -> Self {
        // let r = get_range(data.iter().copied());
        // let data = r.map(|(x_range, y_range)| Plot2DData {
        //     data: Box::new(data.iter().copied()) as BoxCoordIter,
        //     x_range,
        //     y_range,
        // });
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
