use iced::widget::{Column, Text};
use iced::{executor, Application, Command, Container, Element, Length, Row, Settings};
use iced_aw::{TabBar, TabLabel};
use tabs::{FdsToolboxTab, FdsToolboxTabMessage, Tab};

mod panes;
mod sidebar;
mod tabs;
mod treeview;

pub fn main() -> iced::Result {
    FdsToolbox::run(Settings::default())
}

#[derive(Debug)]
struct FdsToolbox {
    active_tab: usize,
    tabs: Vec<FdsToolboxTab>,
    data: FdsToolboxData,
    tree: TreeView<'static, usize>,
}

#[derive(Debug)]
struct FdsToolboxData {
    // simulations: Vec<Simulation>,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    TabSelected(usize),
    TabClosed(usize),
    TabMessage(FdsToolboxTabMessage),
}

impl FdsToolbox {
    pub fn active_tab(&mut self) -> Option<&mut FdsToolboxTab> {
        self.tabs.get_mut(self.active_tab)
    }
}

impl Application for FdsToolbox {
    type Message = Message;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            FdsToolbox {
                active_tab: 0,
                tabs: Vec::new(),
                data: FdsToolboxData {},
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "FDS Toolbox".to_string()
    }

    fn update(&mut self, _message: Self::Message) -> Command<Self::Message> {
        Command::none()
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        let sidebar = ;

        let tab_bar: Element<'_, Self::Message> = match self.tabs.len() {
            0 => Column::new().into(),
            _ => self
                .tabs
                .iter()
                .fold(
                    TabBar::new(self.active_tab, Message::TabSelected),
                    |tab_bar, tab| {
                        let tab_label = <FdsToolboxTab as Tab<FdsToolboxData>>::title(tab);
                        tab_bar.push(TabLabel::Text(tab_label))
                    },
                )
                .on_close(Message::TabClosed)
                .tab_width(Length::Shrink)
                .spacing(5)
                .padding(5)
                .text_size(32)
                .into(),
        };

        let content = match self.tabs.get_mut(self.active_tab) {
            Some(tab) => tab.view(&mut self.data),
            None => Text::new("No tabs open").into(),
        };

        Row::new()
            .push(sidebar)
            .push(
                Column::new().push(tab_bar).push(
                    Container::new(content.map(Message::TabMessage))
                        .width(Length::Fill)
                        .height(Length::Fill),
                ),
            )
            .into()
    }
}
