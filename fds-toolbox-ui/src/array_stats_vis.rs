use fds_toolbox_core::formats::arr_meta::{ArrayStats, Range};
use iced::{
    canvas::{LineCap, Path, Stroke},
    pure::{
        self,
        widget::{
            canvas::{Cache, Program},
            Canvas,
        },
    },
    Color, Point, Size,
};

#[derive(Debug)]
pub struct ArrayStatsVis<'a> {
    stats: &'a ArrayStats<f32>,
    range: &'a Range<f32>,
    cache: &'a Cache,
}

impl ArrayStatsVis<'_> {
    pub fn view<'a, Message: Copy + 'a>(&'a self, _m: Message) -> pure::Element<'a, Message> {
        Canvas::new(self).into()
    }
}

// #[derive(Debug)]
// struct ArrayStatsVis<'a> {
//     stats: &'a ArrayStats<f32>,
//     range: Range<f32>,
//     cache: &'a Cache,
// }

impl<Message> Program<Message> for ArrayStatsVis<'_> {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        bounds: iced::Rectangle,
        _cursor: iced::canvas::Cursor,
    ) -> Vec<iced::canvas::Geometry> {
        let Size {
            width: w,
            height: h,
        } = bounds.size();

        if w == 0.0 || h == 0.0 {
            return vec![];
        }

        // let mut frame = Frame::new(bounds.size());
        let vis = self.cache.draw(bounds.size(), |frame| {
            // let background = Path::rectangle(Point::ORIGIN, frame.size());
            // frame.fill(&background, Color::TRANSPARENT);

            let map = move |s| {
                let res = self.range.map(s) * w;
                // if !res.is_finite() || res.is_nan() { return vec![]; } // Guard against divisions by very small numbers
                if !res.is_finite() || res.is_nan() {
                    0.0
                } else {
                    res
                }
            };
            // dbg!(bounds);

            let range = Path::rectangle(
                Point::new(map(self.stats.min), 0.0),
                Size::new(map(self.stats.max), h),
            );
            frame.fill(&range, Color::from_rgb8(0x66, 0x66, 0x66));

            let mean_stroke = Stroke {
                width: 2.0,
                color: Color::from_rgb8(0x00, 0x00, 0x00),
                line_cap: LineCap::Round,
                ..Stroke::default()
            };

            let mean_pos = map(self.stats.mean);
            let mean = Path::line(Point::new(mean_pos, 0.0), Point::new(mean_pos, h));

            let stddev_stroke = Stroke {
                width: 2.0,
                color: Color::from_rgb8(0xF0, 0x16, 0x16),
                line_cap: LineCap::Round,
                ..Stroke::default()
            };

            let std_dev = map(self.stats.variance.abs().sqrt()); // TODO
            let std_dev = Path::line(
                Point::new(mean_pos - std_dev, h / 2.0),
                Point::new(mean_pos + std_dev, h / 2.0),
            );

            frame.stroke(&std_dev, stddev_stroke);
            frame.stroke(&mean, mean_stroke);
        });

        vec![vis]
    }
}
