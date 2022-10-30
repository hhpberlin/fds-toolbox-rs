pub mod plot_2d;
pub mod state;
pub mod tab;

use std::collections::HashSet;
use std::rc::Rc;

use druid::{AppLauncher, Lens, PlatformError, Widget, WidgetExt, WindowDesc};
use fds_toolbox_core::formats::csv::devc::Devices;
use fds_toolbox_core::formats::simulation::{Simulation, TimeSeriesIdx};
use fds_toolbox_core::formats::simulations::{GlobalTimeSeriesIdx, Simulations};

use plot_2d::plot_tab::{Plot2DTab, Plot2DTabData};
use state::{FdsToolboxApp, FdsToolboxData};
use tab::Tab;

fn main() -> Result<(), PlatformError> {
    let main_window = WindowDesc::new(ui_builder);

    let simulations = Simulations::new(vec![Simulation {
        devc: Devices::from_reader(
            include_bytes!("../../../demo-house/DemoHaus2_devc.csv").as_ref(),
        )
        .unwrap(),
    }]);
    let simulations = Rc::new(simulations);
    let data = FdsToolboxApp {
        data: FdsToolboxData {
            simulations: simulations.clone(),
        },
        tab_data: Plot2DTabData::new(HashSet::from_iter(
            simulations.simulations[0]
                .devc
                .iter_device_named_ids()
                .map(|x| x.1)
                .map(|x| GlobalTimeSeriesIdx(0, TimeSeriesIdx::Device(x))),
            //     [GlobalTimeSeriesIdx(
            //     0,
            //     TimeSeriesIdx::Device(
            // simulations.simulations[0]
            //     .devc
            //             // .get_device_idx_by_name("T_B05")
            //             // .unwrap(),
            //     ),
            // )]
        )),
    };

    AppLauncher::with_window(main_window)
        .use_simple_logger()
        .configure_env(|_env, _| {
            // env.get_all().for_each(|(k, v)| {
            //     println!("{}: {:?}", k, v);
            // });

            // env.set(
            //     theme::WINDOW_BACKGROUND_COLOR,
            //     Color::rgb8(0x2e, 0x34, 0x36),
            // );
            // env.set(theme::BUTTON_BORDER_RADIUS, 5);
            // env.set(theme::BUTTON_BORDER_WIDTH, 0);
            // env.set(theme::BUTTON_DARK, Color::rgb8(0x4c, 0x56, 0x5a));
            // env.set(theme::BUTTON_LIGHT, Color::rgb8(0xfc, 0x56, 0x5a));
            // env.set(theme::BUTTON_DARK, 0);
        })
        .launch(data)
}

struct LensId;

impl<T> Lens<T, T> for LensId {
    fn with<V, F: FnOnce(&T) -> V>(&self, data: &T, f: F) -> V {
        f(data)
    }

    fn with_mut<V, F: FnOnce(&mut T) -> V>(&self, data: &mut T, f: F) -> V {
        f(data)
    }
}

// struct TupleLens<L0, L1>(L0, L1);

// impl<'a, T0, U0, L0: Lens<T0, U0>, T1, U1, L1: Lens<T1, U1>> Lens<(T0, T1), (&'a U0, &'a U1)> for TupleLens<L0, L1> {
//     fn with<V, F: FnOnce(&(&'a U0, &'a U1)) -> V>(&self, data: &(T0, T1), f: F) -> V {
//         self.0.with(&data.0, |u0| self.1.with(&data.1, |u1| f(&(u0, u1))))
//     }

//     fn with_mut<V, F: FnOnce(&mut (&'a U0, &'a U1)) -> V>(&self, data: &mut (T0, T1), f: F) -> V {
//         self.0.with_mut(&mut data.0, |u0| self.1.with_mut(&mut data.1, |u1| f(&mut (u0, u1))))
//     }
// }

// TODO: Awesome name
// struct TupleLens<L0, L1>(L0, L1);

// impl<T, U0: Clone, L0: Lens<T, U0>, U1: Clone, L1: Lens<T, U1>> Lens<T, (U0, U1)>
//     for TupleLens<L0, L1>
// {
//     fn with<V, F: FnOnce(&(U0, U1)) -> V>(&self, data: &T, f: F) -> V {
//         self.0.with(&data, |u0| {
//             self.1.with(&data, |u1| f(&(u0.clone(), u1.clone())))
//         })
//     }

//     fn with_mut<V, F: FnOnce(&mut (U0, U1)) -> V>(&self, mut data: &mut T, f: F) -> V {
//         self.0.with_mut(&mut data, |u0| {
//             self.1
//                 .with_mut(&mut data, |u1| f(&mut (u0.clone(), u1.clone())))
//         })
//     }
// }

struct TabLens;

impl Lens<FdsToolboxApp, (Plot2DTabData, FdsToolboxData)> for TabLens {
    fn with<V, F: FnOnce(&(Plot2DTabData, FdsToolboxData)) -> V>(
        &self,
        data: &FdsToolboxApp,
        f: F,
    ) -> V {
        f(&(data.tab_data.clone(), data.data.clone()))
    }

    fn with_mut<V, F: FnOnce(&mut (Plot2DTabData, FdsToolboxData)) -> V>(
        &self,
        data: &mut FdsToolboxApp,
        f: F,
    ) -> V {
        f(&mut (data.tab_data.clone(), data.data.clone()))
    }
}

fn ui_builder() -> impl Widget<FdsToolboxApp> {
    // The label text will be computed dynamically based on the current locale and count
    // let text =
    //     LocalizedString::new("hello-counter").with_arg("count", |data: &u32, _env| (*data).into());
    // let label = Label::new(text).padding(5.0).center();
    // let button = Button::new("increment")
    //     .on_click(|_ctx, data, _env| *data += 1)
    //     .padding(5.0);

    let mut tab = Plot2DTab::new();

    tab.build_widget().lens(TabLens)
    // Flex::column().with_child(label).with_child(button)
}
