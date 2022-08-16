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

use iced::pure;

use iced::pure::Pure;

use iced::Element;

use super::FdsToolboxData;

use fds_toolbox_core::formats::arr_meta::ArrayStats;

#[derive(Debug)]
struct SidebarBlock<'a, Iter: Iterator, Id> {
    title: &'a str,
    id: Id,
    content: Iter,
}

#[derive(Debug)]
struct DevcSidebar<'a> {
    name: &'a str,
    meta: &'a ArrayStats<f32>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum SidebarId {
    Devc,
}

#[derive(Debug, Clone, Copy)]
pub enum SidebarMessage {
    DevcSelected,
    Scroll(f32),
}

impl Sidebar {
    fn sidebar_content<'a>(
        data: &'a FdsToolboxData,
    ) -> impl Iterator<Item = SidebarBlock<'a, impl Iterator<Item = DevcSidebar<'a>> + 'a, SidebarId>> + 'a
    {
        let devc = data
            .simulations
            .iter()
            .flat_map(|sim| sim.devc.devices.iter())
            .map(|devc| DevcSidebar {
                name: &devc.name,
                meta: &devc.meta,
            });

        vec![SidebarBlock {
            title: "DEVC",
            id: SidebarId::Devc,
            content: devc,
        }]
        .into_iter()
    }

    pub fn view_sidebar<'a>(&'a mut self, data: &'a FdsToolboxData) -> Element<'a, SidebarMessage> {
        Pure::new(&mut self.state, Self::view_sidebar_pure(data)).into()
    }

    fn view_sidebar_pure(data: &FdsToolboxData) -> pure::Element<'_, SidebarMessage> {
        let mut col = pure::column();

        for block in Self::sidebar_content(data) {
            let mut content = pure::column()
                .push(
                    pure::button(pure::text(block.title).size(20))
                        .on_press(SidebarMessage::DevcSelected),
                )
                // .spacing(5)
                .padding(10);

            for elem in block.content {
                content = content.push(
                    pure::button(pure::text(elem.name).size(12))
                        .on_press(SidebarMessage::DevcSelected),
                );
            }

            col = col.push(content);
        }

        pure::scrollable(col).into()
    }
}
