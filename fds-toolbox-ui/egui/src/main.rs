use eframe::egui;
use egui::{
    plot::{Legend, Line, Plot, PlotPoints},
    Align, Layout, ScrollArea,
};
use fds_toolbox_core::{
    common::series::TimeSeriesViewSource,
    formats::{
        csv::devc::Devices,
        simulation::{Simulation, TimeSeriesIdx},
        simulations::{GlobalTimeSeriesIdx, Simulations},
    },
};

fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "My egui App",
        native_options,
        Box::new(|cc| Box::new(FdsToolboxApp::new(cc))),
    );
}

struct FdsToolboxApp {
    ids: Vec<GlobalTimeSeriesIdx>,
    search: String,
    simulations: Simulations,
}

impl FdsToolboxApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.

        let simulations = Simulations::new(vec![Simulation {
            devc: Devices::from_reader(
                include_bytes!("../../../demo-house/DemoHaus2_devc.csv").as_ref(),
            )
            .unwrap(),
        }]);
        let id = GlobalTimeSeriesIdx(
            0,
            TimeSeriesIdx::Device(
                simulations[0]
                    .devc
                    .get_device_idx_by_name("AST_1OG_Glaswand_N2")
                    .unwrap(),
            ),
        );

        Self {
            ids: vec![id],
            simulations,
            search: "".to_string(),
        }
    }
}

impl eframe::App for FdsToolboxApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    ui.with_layout(Layout::top_down(Align::TOP), |ui| {
                        ui.text_edit_singleline(&mut self.search);

                        for (sidx, simulation) in self.simulations.simulations.iter().enumerate() {
                            ui.label(sidx.to_string());

                            let mut devc: Vec<_> =
                                simulation.devc.iter_device_named_ids().collect();
                            devc.sort_by_key(|x| x.0);
                            for (name, didx) in devc {
                                if !name.contains(self.search.as_str()) {
                                    continue;
                                }
                                let mut checked = self.ids.contains(&GlobalTimeSeriesIdx(
                                    sidx,
                                    TimeSeriesIdx::Device(didx),
                                ));
                                if ui.checkbox(&mut checked, name).changed() {
                                    if checked {
                                        self.ids.push(GlobalTimeSeriesIdx(
                                            sidx,
                                            TimeSeriesIdx::Device(didx),
                                        ));
                                    } else {
                                        self.ids.retain(|id| {
                                            id != &GlobalTimeSeriesIdx(
                                                sidx,
                                                TimeSeriesIdx::Device(didx),
                                            )
                                        });
                                    }
                                }
                            }

                            if sidx < self.simulations.simulations.len() - 1 {
                                ui.separator();
                            }
                        }
                    });
                });

                ui.separator();

                let plot = Plot::new("2d_plot").legend(Legend::default());
                plot.show(ui, |plot_ui| {
                    for series in self.ids.iter().filter_map(|id| self.simulations.get_time_series(*id)) {
                        plot_ui.line(Line::new(PlotPoints::from_iter(
                            series
                                .iter()
                                .map(|(x, y)| [x as f64, y as f64]),
                        ))
                        .name(format!("{} ({})", series.name, series.unit)));
                    }
                });

                ui.separator();
            });
        });
    }
}
