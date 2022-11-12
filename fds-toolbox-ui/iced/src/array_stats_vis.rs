use fds_toolbox_core::common::arr_meta::ArrayStats;
use iced::{
    widget::{
        canvas::{Cache, Geometry, LineCap, Path, Program, Stroke, self},
    },
    Color, Element, Point, Size, Theme,
};

#[derive(Debug, Copy, Clone)]
pub struct ArrayStatsVis<'a> {
    stats: &'a ArrayStats<f32>,
    cache: Option<&'a Cache>,
}

impl<'a> ArrayStatsVis<'a> {
    pub fn new(stats: &'a ArrayStats<f32>, cache: Option<&'a Cache>) -> Self {
        Self { stats, cache }
    }

    pub fn view<Message: Copy + 'a>(&'a self) -> Element<'a, Message> {
        canvas(self).into()
    }
}

// pub fn view<'a, Message: Copy + 'a>(
//     stats: &'a ArrayStats<f32>,
//     cache: &'a Cache,
// ) -> Element<'a, Message> {
//     ArrayStatsVis::new(stats, cache).view()
// }

impl<Message> Program<Message> for ArrayStatsVis<'_> {
    type State = ();

    fn draw(
        &self,
        state: &Self::State,
        theme: &Theme,
        bounds: iced::Rectangle,
        cursor: iced::widget::canvas::Cursor,
    ) -> Vec<Geometry> {
        draw(self, bounds)
    }
}

fn draw(
    data: &ArrayStatsVis<'_>,
    bounds: iced::Rectangle,
    // _cursor: iced::canvas::Cursor,
) -> Vec<Geometry> {
    let Size {
        width: w,
        height: h,
    } = bounds.size();

    if w == 0.0 || h == 0.0 {
        return vec![];
    }

    // let mut frame = Frame::new(bounds.size());
    let vis = data.cache.unwrap_or_else(|| &Cache::new()).draw(bounds.size(), |frame| {
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
            line_cap: LineCap::Round,
            ..Stroke::default()
        };

        let mean_pos = map(data.stats.mean);
        let mean = Path::line(Point::new(mean_pos, 0.0), Point::new(mean_pos, h));

        let stddev_stroke = Stroke {
            width: 2.0,
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
