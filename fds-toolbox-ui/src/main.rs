use modular_ui_iced::pane;

pub fn main() -> iced::Result {
    // MainWindow::run(Settings::default())
    pane::main()
}

// struct MainWindow {

// }

// impl Sandbox for MainWindow {
//     fn new() -> Self {
//         MainWindow {

//         }
//     }

//     fn title(&self) -> String {
//         format!("{} - FDS Toolbox", "..")
//     }

//     type Message = ();

//     fn update(&mut self, message: Self::Message) {

//     }

//     fn view(&mut self) -> iced::Element<'_, Self::Message> {
//         let mut controls = Row::new();
//     }
// }
