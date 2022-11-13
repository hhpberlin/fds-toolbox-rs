use fds_toolbox_core::common::arr_meta::ArrayStats;
use iced::{
    widget::{
        canvas::{Cache, Geometry, LineCap, Path, Program, Stroke, stroke::Style},
        Canvas,
    },
    Color, Element, Length, Point, Size, Theme,
};

struct ArrayStatsCanvas<Num, NumDivisible = Num, NumSq = Num>(ArrayStats<Num, NumDivisible, NumSq>);

pub fn array_stats_vis<'a, Message: Copy + 'a>(stats: ArrayStats<f32>) -> Element<'a, Message> {
    Canvas::new(ArrayStatsCanvas(stats))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

impl<Message> Program<Message> for ArrayStatsCanvas<f32> {
    type State = Cache;

    fn draw(
        &self,
        state: &Self::State,
        theme: &Theme,
        bounds: iced::Rectangle,
        _cursor: iced::widget::canvas::Cursor,
    ) -> Vec<Geometry> {
        draw(self.0, state, theme, bounds)
    }
}

fn draw(
    stats: ArrayStats<f32>,
    cache: &Cache,
    theme: &Theme,
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
    let vis = cache.draw(bounds.size(), |frame| {
        if w <= 1.0 || h <= 1.0 {
            return;
        }
        // let background = Path::rectangle(Point::ORIGIN, frame.size());
        // frame.fill(&background, Color::TRANSPARENT);

        let map = move |s| {
            let res = stats.range.map(s) * w;
            // if !res.is_finite() || res.is_nan() { return vec![]; } // Guard against divisions by very small numbers
            if !res.is_finite() || res.is_nan() {
                0.0
            } else {
                res
            }
        };
        // dbg!(bounds);

        let range = Path::rectangle(
            Point::new(map(stats.range.min), 0.0),
            Size::new(map(stats.range.max), h),
        );
        // dbg![&range];
        frame.fill(&range, theme.extended_palette().secondary.base.color);

        let mean_stroke = Stroke {
            style: Style::Solid(theme.extended_palette().danger.base.color),
            width: 2.0,
            line_cap: LineCap::Round,
            ..Stroke::default()
        };

        let mean_pos = map(stats.mean);
        let mean = Path::line(Point::new(mean_pos, 0.0), Point::new(mean_pos, h));

        let stddev_stroke = Stroke {
            style: Style::Solid(theme.extended_palette().primary.base.color),
            width: 2.0,
            line_cap: LineCap::Round,
            ..Stroke::default()
        };

        let std_dev = map(stats.variance.abs().sqrt()); // TODO
        let std_dev = Path::line(
            Point::new(mean_pos - std_dev, h / 2.0),
            Point::new(mean_pos + std_dev, h / 2.0),
        );
        // dbg![&mean];
        // dbg![&std_dev];

        frame.stroke(&std_dev, stddev_stroke);
        frame.stroke(&mean, mean_stroke);
    });

    vec![vis]
}
