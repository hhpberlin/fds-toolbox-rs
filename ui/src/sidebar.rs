pub struct Sidebar {
    state: pure::State,
    caches: HashMap<ArrayStats<f32>, Cache>,
}

impl Sidebar {
    pub fn new() -> Self {
        Self {
            state: pure::State::new(),
            caches: HashMap::new(),
        }
    }
}

use std::collections::HashMap;

use iced::canvas::{Frame, LineCap, Path, Stroke};
use iced::pure::widget::canvas::{self, Cache, Program};
use iced::{pure, Background, Color, Length, Padding, Point, Size, Vector};

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

#[derive(Debug, PartialEq, Eq, Hash)]
enum SidebarId {
    Devc,
}

#[derive(Debug, Clone, Copy)]
pub enum SidebarMessage {
    DevcSelected,
}

trait SidebarEntry<Data, Child> {
    type Iter: Iterator<Item = Child>;
    fn get_data(&self) -> &Data;
    fn children(&self) -> Self::Iter;
}

impl Sidebar {
    fn sidebar_content<'a>(
        data: &'a FdsToolboxData,
    ) -> impl Iterator<
        Item = SidebarBlock<'a, impl Iterator<Item = ArrayStatsVis<'a>> + 'a, SidebarId>,
    > + 'a {
        let devc = data
            .simulations
            .iter()
            .flat_map(|sim| sim.devc.devices.iter())
            .map(|devc| ArrayStatsVis {
                name: &devc.name,
                stats: &devc.stats,
                range: &devc.range,
                cache: &self.caches[&devc.stats],
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

mod style {
    use iced::{
        pure::widget::{button, toggler},
        Color, Vector,
    };

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
                background: iced::Color::from_rgb8(0xFF, 0xFF, 0xFF),
                background_border: None,
                foreground: Color::from_rgb8(0x00, 0x00, 0x00),
                foreground_border: None,
            }
        }

        fn active(&self, is_active: bool) -> iced::toggler::Style {
            iced::toggler::Style {
                background: iced::Color::from_rgb8(0x0F, 0xFF, 0xFF),
                background_border: None,
                foreground: Color::from_rgb8(0x00, 0x00, 0x00),
                foreground_border: None,
            }
        }
    }
}
