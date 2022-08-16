pub struct Sidebar {
    state: pure::State,
}

impl Sidebar {
    pub fn new() -> Self {
        Self {
            state: pure::State::new(),
        }
    }
}

use iced::canvas::{Cache, Path, Frame, LineCap, Stroke};
use iced::pure::widget::canvas::{Program, self};
use iced::{pure, Vector, Length, Padding, Color, Point, Background, Size};

use iced::pure::Pure;

use iced::pure::widget::{button, Canvas};
use iced::Element;

use super::FdsToolboxData;

use fds_toolbox_core::formats::arr_meta::{ArrayStats, Range};

#[derive(Debug)]
struct SidebarBlock<'a, Iter: Iterator, Id> {
    title: &'a str,
    id: Id,
    content: Iter,
}

#[derive(Debug)]
struct ArraySidebarElement<'a> {
    name: &'a str,
    stats: &'a ArrayStats<f32>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum SidebarId {
    Devc,
}

#[derive(Debug, Clone, Copy)]
pub enum SidebarMessage {
    DevcSelected,
}

impl Sidebar {
    fn sidebar_content<'a>(
        data: &'a FdsToolboxData,
    ) -> impl Iterator<
        Item = SidebarBlock<'a, impl Iterator<Item = ArraySidebarElement<'a>> + 'a, SidebarId>,
    > + 'a {
        let devc = data
            .simulations
            .iter()
            .flat_map(|sim| sim.devc.devices.iter())
            .map(|devc| ArraySidebarElement {
                name: &devc.name,
                stats: &devc.stats,
            });

        vec![SidebarBlock {
            title: "DEVC",
            id: SidebarId::Devc,
            content: devc,
        }]
        .into_iter()
    }

    pub(crate) fn view_sidebar<'a>(
        &'a mut self,
        data: &'a FdsToolboxData,
    ) -> Element<'a, SidebarMessage> {
        Pure::new(&mut self.state, Self::view_sidebar_pure(data)).into()
    }

    fn view_sidebar_pure(data: &FdsToolboxData) -> pure::Element<'_, SidebarMessage> {
        let mut col = pure::column();

        for block in Self::sidebar_content(data) {
            let mut content = pure::column()
                .push(
                    pure::button(pure::text(block.title).size(20))
                        .on_press(SidebarMessage::DevcSelected)
                        .style(style::SidebarStyle::Title)
                        .width(Length::Fill),
                )
                // .spacing(5)
                .padding(10);

            for elem in block.content {
                content = content.push(elem.view(SidebarMessage::DevcSelected));
            }

            col = col.push(content);
        }

        pure::scrollable(col).into()
    }
}

impl ArraySidebarElement<'_> {
    fn view<'a, Message: Copy + 'a>(&self, m: Message) -> pure::Element<'a, Message> {
        pure::button(
            pure::row()
                .push(pure::text(self.name).size(20))
                .push(
                    Canvas::new(ArrayStatsVis{ stats: self.stats.clone(), range: Range::new(0.0, 100.0) } )
                    .width(Length::FillPortion(3))
                    .height(Length::Units(20))
                )
            )
            .on_press(m)
            .style(style::SidebarStyle::Element)
            .width(Length::FillPortion(5))
            .padding([5, 5, 5, 15])
            .into()
    }
}

#[derive(Debug)]
struct ArrayStatsVis {
    stats: ArrayStats<f32>,
    range: Range<f32>,
    // cache: Cache,
}

impl<Message> Program<Message> for ArrayStatsVis {
    type State = ();

    fn draw(
        &self,
        state: &Self::State,
        bounds: iced::Rectangle,
        cursor: iced::canvas::Cursor,
    ) -> Vec<iced::canvas::Geometry> {
        let mut frame = Frame::new(bounds.size());
        // let vis = self.cache.draw(bounds.size(), |frame| {
            // let background = Path::rectangle(Point::ORIGIN, frame.size());
            // frame.fill(&background, Color::TRANSPARENT);
            let Size { width: w, height: h } = bounds.size();

            if w == 0.0 || h == 0.0 { return vec![]; }
            dbg!(self);

            let map = move |s| {
                let res = self.range.map(s) * w;
                // if !res.is_finite() || res.is_nan() { return vec![]; } // Guard against divisions by very small numbers
                if !res.is_finite() || res.is_nan() { 0.0 } else { res }
            };
            // dbg!(bounds);

            let range = Path::rectangle(Point::new(map(self.stats.min), 0.0), Size::new(map(self.stats.max), h));
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
            let std_dev = Path::line(Point::new(mean_pos - std_dev, h/2.0), Point::new(mean_pos + std_dev, h/2.0));

            frame.stroke(&std_dev, stddev_stroke);
            frame.stroke(&mean, mean_stroke);
        // });

        vec![frame.into_geometry()]
    }
}

mod style {
    use iced::{pure::widget::{button, toggler}, Vector, Color};

    pub enum SidebarStyle {
        Title,
        Element,
    }

    impl button::StyleSheet for SidebarStyle {
        fn active(&self) -> iced::button::Style {
            iced::button::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb8(
                    0xFF, 0xFF, 0xFF,
                ))),
                border_radius: 0.0,
                border_width: 0.0,
                border_color: iced::Color::from_rgb8(0x00, 0x00, 0x00),
                text_color: iced::Color::from_rgb8(0x00, 0x00, 0x00),
                shadow_offset: Vector::new(0.0, 0.0),
            }
        }

        fn hovered(&self) -> iced::button::Style {
            iced::button::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb8(
                    0xF0, 0xF0, 0xF0,
                ))),
                ..self.active()
            }
        }

        fn pressed(&self) -> iced::button::Style {
            iced::button::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb8(
                    0x70, 0x70, 0xF0,
                ))),
                ..self.active()
            }
        }
    }

    impl toggler::StyleSheet for SidebarStyle {
        fn hovered(&self, is_active: bool) -> iced::toggler::Style {
            iced::toggler::Style {
                background: iced::Color::from_rgb8(
                    0xFF, 0xFF, 0xFF,
                ),
                background_border: None,
                foreground: Color::from_rgb8(0x00, 0x00, 0x00),
                foreground_border: None,
            }
        }

        fn active(&self, is_active: bool) -> iced::toggler::Style {
            iced::toggler::Style {
                background: iced::Color::from_rgb8(
                    0x0F, 0xFF, 0xFF,
                ),
                background_border: None,
                foreground: Color::from_rgb8(0x00, 0x00, 0x00),
                foreground_border: None,
            }
        }
    }
}
