#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::future::IntoFuture;

use eframe::egui;
use fds_toolbox_core::file::{OsFs, SimulationPath};
use fds_toolbox_lazy_data::{
    fs::AnyFs,
    moka::{MokaStore, SimulationIdx},
};
use tokio::sync::mpsc;
use tracing::error;

fn main() -> Result<(), eframe::Error> {
    // env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();
    let options = eframe::NativeOptions {
        drag_and_drop_support: true,
        initial_window_size: Some(egui::vec2(320.0, 240.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Native file dialogs and drag-and-drop files",
        options,
        Box::new(|_cc| {
            Box::new(FdsToolboxApp {
                store: MokaStore::new(10_000),
                app_state: Default::default(),
                message_channel: mpsc::channel(100),
            })
        }),
    )
}

// #[derive(Default)]
struct FdsToolboxApp {
    // dropped_files: Vec<egui::DroppedFile>,
    // picked_path: Option<String>,
    pub store: MokaStore,
    pub app_state: AppState,
    pub message_channel: (mpsc::Sender<Message>, mpsc::Receiver<Message>),
}

// TODO: Better name
struct AppSync {
    pub store: MokaStore,
    pub channel: mpsc::Sender<Message>,
}

#[derive(Default)]
struct AppState {
    pub active_simulations: Vec<SimulationIdx>,
}

enum Message {
    OpenSimulationPath(SimulationPath<AnyFs>),
    OpenSimulationIdx(SimulationIdx),
}

impl FdsToolboxApp {
    pub fn get_sync(&self) -> AppSync {
        AppSync {
            store: self.store.clone(),
            channel: self.message_channel.0.clone(),
        }
    }

    pub fn spawn<F>(&self, f: impl FnOnce() -> F + Send + 'static)
    where
        F: IntoFuture<Output = Option<Message>> + Send,
        F::IntoFuture: Send + 'static,
    {
        // let sync = self.get_sync();
        let channel = self.message_channel.0.clone();
        tokio::spawn(async move {
            if let Some(message) = f().into_future().await {
                channel.send(message).await.unwrap();
            }
        });
    }

    fn handle_message(&mut self, message: Message) {
        match message {
            Message::OpenSimulationPath(sim_path) => {
                let store = self.store.clone();
                self.spawn(move || async move {
                    let sim_idx = store.get_idx_by_path(&sim_path).0;
                    Some(Message::OpenSimulationIdx(sim_idx))
                });
            }
            Message::OpenSimulationIdx(sim_idx) => {
                self.app_state.active_simulations.push(sim_idx);
            }
        }
    }
}

impl eframe::App for FdsToolboxApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(msg) = self.message_channel.1.try_recv() {
            self.handle_message(msg);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Drag-and-drop files onto the window!");

            if ui.button("Open file…").clicked() {
                self.spawn(move || async move {
                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        let file = rfd::AsyncFileDialog::new()
                            .add_filter("Smokeview", &["smv"])
                            .pick_file()
                            .await;
                        let Some(file) = file else {
                            return None;
                        };
                        let path = file.path();
                        let Some(dir) = path.parent() else {
                            error!("Could not get parent directory of file {:?}", path);
                            return None;
                        };
                        let Some((path, dir)) = path.to_str().zip(dir.to_str()) else {
                            error!("Could not convert path to string: {:?}", path);
                            return None;
                        };
                        let (path, dir) = (path.to_string(), dir.to_string());

                        let sim_path = SimulationPath::new_full(AnyFs::LocalFs(OsFs), dir, path);
                        Some(Message::OpenSimulationPath(sim_path))
                    } else {
                        None
                    }
                });
            }
        });

        // preview_files_being_dropped(ctx);

        // // Collect dropped files:
        // ctx.input(|i| {
        //     if !i.raw.dropped_files.is_empty() {
        //         self.dropped_files = i.raw.dropped_files.clone();
        //     }
        // });
    }
}

// /// Preview hovering files:
// fn preview_files_being_dropped(ctx: &egui::Context) {
//     use egui::*;
//     use std::fmt::Write as _;

//     if !ctx.input(|i| i.raw.hovered_files.is_empty()) {
//         let text = ctx.input(|i| {
//             let mut text = "Dropping files:\n".to_owned();
//             for file in &i.raw.hovered_files {
//                 if let Some(path) = &file.path {
//                     write!(text, "\n{}", path.display()).ok();
//                 } else if !file.mime.is_empty() {
//                     write!(text, "\n{}", file.mime).ok();
//                 } else {
//                     text += "\n???";
//                 }
//             }
//             text
//         });

//         let painter =
//             ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("file_drop_target")));

//         let screen_rect = ctx.screen_rect();
//         painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(192));
//         painter.text(
//             screen_rect.center(),
//             Align2::CENTER_CENTER,
//             text,
//             TextStyle::Heading.resolve(&ctx.style()),
//             Color32::WHITE,
//         );
//     }
// }
