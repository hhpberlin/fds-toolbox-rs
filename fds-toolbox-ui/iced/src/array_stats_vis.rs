use fds_toolbox_core::common::arr_meta::ArrayStats;
use iced::{
    canvas::{Cache, LineCap, Path, Stroke},
    pure,
    // pure::{
    //     self,
    //     widget::{
    //         canvas::{Cache, Program},
    //         Canvas,
    //     },
    // },
    Color,
    Point,
    Size,
};

#[derive(Debug, Copy, Clone)]
pub struct ArrayStatsVis<'a> {
    stats: &'a ArrayStats<f32>,
    cache: &'a Cache,
}

impl<'a> ArrayStatsVis<'a> {
    // pub fn new(stats: &ArrayStats<f32>) -> Self {
    //     Self { stats, cache: Cache::new() }
    // }
    pub fn new(stats: &'a ArrayStats<f32>, cache: &'a Cache) -> Self {
        Self { stats, cache }
    }

    pub fn view_pure<Message: Copy + 'a>(&'a self) -> pure::Element<'a, Message> {
        pure::widget::Canvas::new(self).into()
    }

    // pub fn view_new<'a, Message: Copy + 'static>(
    //     stats: &'a ArrayStats<f32>,
    //     cache: &'a Cache,
    // ) -> iced::Element<'a, Message> {
    //     ArrayStatsVis { stats, cache }.view()
    // }

    pub fn view<Message: Copy + 'static>(&'a self) -> iced::Element<'a, Message> {
        iced::Canvas::new(self).into()
    }
}

impl<Message> iced::canvas::Program<Message> for &ArrayStatsVis<'_> {
    fn draw(
        &self,
        bounds: iced::Rectangle,
        _cursor: iced::canvas::Cursor,
    ) -> Vec<iced::canvas::Geometry> {
        draw(self, bounds)
    }
}

impl<Message> pure::widget::canvas::Program<Message> for ArrayStatsVis<'_> {
    type State = ();

    fn draw(
        &self,
        state: &Self::State,
        bounds: iced::Rectangle,
        cursor: iced::canvas::Cursor,
    ) -> Vec<iced::canvas::Geometry> {
        draw(&self, bounds)
    }
}

fn draw(
    data: &ArrayStatsVis<'_>,
    bounds: iced::Rectangle,
    // _cursor: iced::canvas::Cursor,
) -> Vec<iced::canvas::Geometry> {
    let Size {
        width: w,
        height: h,
    } = bounds.size();

    if w == 0.0 || h == 0.0 {
        return vec![];
    }

    // let mut frame = Frame::new(bounds.size());
    let vis = data.cache.draw(bounds.size(), |frame| {
        // let background = Path::rectangle(Point::ORIGIN, frame.size());
        // frame.fill(&background, Color::TRANSPARENT);

        let map = move |s| {
            let res = data.stats.range.map(s) * w;
            // if !res.is_finite() || res.is_nan() { return vec![]; } // Guard against divisions by very small numbers
            if !res.is_finite() || res.is_nan() {
                0.0
            } else {
                res
            }
        };
        // dbg!(bounds);

        let range = Path::rectangle(
            Point::new(map(data.stats.range.min), 0.0),
            Size::new(map(data.stats.range.max), h),
        );
        frame.fill(&range, Color::from_rgb8(0x66, 0x66, 0x66));

        let mean_stroke = Stroke {
            width: 2.0,
            color: Color::from_rgb8(0x00, 0x00, 0x00),
            line_cap: LineCap::Round,
            ..Stroke::default()
        };

        let mean_pos = map(data.stats.mean);
        let mean = Path::line(Point::new(mean_pos, 0.0), Point::new(mean_pos, h));

        let stddev_stroke = Stroke {
            width: 2.0,
            color: Color::from_rgb8(0xF0, 0x16, 0x16),
            line_cap: LineCap::Round,
            ..Stroke::default()
        };

        let std_dev = map(data.stats.variance.abs().sqrt()); // TODO
        let std_dev = Path::line(
            Point::new(mean_pos - std_dev, h / 2.0),
            Point::new(mean_pos + std_dev, h / 2.0),
        );

        frame.stroke(&std_dev, stddev_stroke);
        frame.stroke(&mean, mean_stroke);
    });

    vec![vis]
}
