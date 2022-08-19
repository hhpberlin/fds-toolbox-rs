use fds_toolbox_core::formats::{arr_meta::Range, self};
use std::fmt::Debug;

type BoxCoordIter<'a> = Box<dyn Iterator<Item = (f32, f32)> + 'a>;
type Plot2DDataBoxed<'a> = Plot2DData<BoxCoordIter<'a>>;

pub trait Plottable2D: Debug {
    // TODO: Tracking https://github.com/rust-lang/rust/issues/63063 to avoid alloc
    // type IntoIter: Iterator<Item = (f32, f32)>;
    // type IntoIter = impl Iterator<Item = (f32, f32)>;
    fn plot_data<'a>(&'a self) -> Option<Plot2DDataBoxed<'a>>;

    fn store_static(&self) -> Option<Vec<(f32, f32)>> {
        let data = self.plot_data()?;
        let vec: Vec<_> = data.data.collect();
        Some(Plot2DData::new(vec., x_range, y_range))
    }
}

pub struct Plot2DData<Iter: Iterator<Item = (f32, f32)>> {
    pub data: Iter,
    pub x_range: Range<f32>,
    pub y_range: Range<f32>,
}

impl<Iter: Iterator<Item = (f32, f32)>> Debug for Plot2DData<Iter> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Plot2DData")
            .field("x_range", &self.x_range)
            .field("y_range", &self.y_range)
            .finish()
    }
}

impl<Iter: Iterator<Item = (f32, f32)>> Plot2DData<Iter> {
    pub fn new(data: Iter, x_range: Range<f32>, y_range: Range<f32>) -> Self {
        Self {
            data,
            x_range,
            y_range,
        }
    }

    pub fn from_iter<'a>(data: impl IntoIterator<IntoIter = Iter> + Copy) -> Option<Self> {
        Self::from_fn_iter(|| data.into_iter(), || data.into_iter())
    }

    pub fn from_fn_iter<'a, RIter: Iterator<Item = (f32, f32)>>(range_data: impl Fn() -> RIter, data: impl FnOnce() -> Iter) -> Option<Self> {
        // TODO: This iterates twice for no good reason, as long as the compiler isn't smart enough to optimize it away anyways
        let x_range = Range::from_iter_val(range_data().map(|(x, _)| x))?;
        let y_range = Range::from_iter_val(range_data().map(|(_, y)| y))?;
        Some(Self::new(data(), x_range, y_range))
    }

    pub fn boxed<'a>(self) -> Plot2DDataBoxed<'a>
        where Iter: 'a
    {
        Plot2DData {
            data: Box::new(self.data),
            x_range: self.x_range,
            y_range: self.y_range,
        }
    }
}

impl<'a> Plot2DDataBoxed<'a> {
    pub fn from_iter_box<Iter: Iterator<Item = (f32, f32)> + 'a>(data: impl IntoIterator<IntoIter = Iter> + Copy) -> Option<Self> {
        Self::from_fn_iter_box(|| data.into_iter())
    }

    pub fn from_fn_iter_box<Iter: Iterator<Item = (f32, f32)> + 'a>(data: impl Fn() -> Iter) -> Option<Self> {
        Self::from_fn_iter(&data, || Box::new(data()) as Box<dyn Iterator<Item = (f32, f32)>>)
    }
}

impl Plottable2D for Vec<(f32, f32)> {
    fn plot_data<'a>(&'a self) -> Option<Plot2DDataBoxed<'a>> {
        Plot2DDataBoxed::from_fn_iter_box(|| self.iter().copied())
    }
}

impl Plottable2D for formats::csv::devc::Device<'_> {
    fn plot_data<'a>(&'a self) -> Option<Plot2DDataBoxed<'a>> {
        Plot2DDataBoxed::from_fn_iter_box(|| self.iter_f32())
    }
}