use plotters::style::{Color, ShapeStyle};

pub struct LabeledSeries {
    name: String,
    // color: Box<dyn Into<ShapeStyle>>,
    data: Box<dyn Iterator<Item = (f32, f32)>>,
}

pub trait SeriesSource {
    fn iter_series(&self) -> Box<dyn Iterator<Item = LabeledSeries>>;
}