slint::include_modules!();

use std::sync::{Mutex, RwLock};

use fds_toolbox_core::formats::simulations::Simulations;
use plotters::prelude::*;
use slint::{SharedPixelBuffer, Rgb8Pixel};

struct State {
    shared_pixel_buffer: SharedPixelBuffer<Rgb8Pixel>,
    // plotters_backend: BitMapBackend<'static>,
    simulations: Simulations,
}

impl State {
    fn new() -> Self {
        let shared_pixel_buffer = SharedPixelBuffer::new(800, 600);
        // let plotters_backend = BitMapBackend::with_buffer(
        //     shared_pixel_buffer.make_mut_bytes(),
        //     (shared_pixel_buffer.width(), shared_pixel_buffer.height()),
        // );
        Self {
            shared_pixel_buffer,
            // plotters_backend,
            simulations: Simulations::empty(),
        }
    }
}

fn main() {
    let ui = AppWindow::new();

    let ui_handle = ui.as_weak();
    ui.on_render_plot(render_plot);

    ui.run();
}

lazy_static::lazy_static! {
    static ref STATE: RwLock<State> = RwLock::new(State::new());
}

fn pdf(x: f64, y: f64, a: f64) -> f64 {
    const SDX: f64 = 0.1;
    const SDY: f64 = 0.1;
    let x = x as f64 / 10.0;
    let y = y as f64 / 10.0;
    a * (-x * x / 2.0 / SDX / SDX - y * y / 2.0 / SDY / SDY).exp()
}

fn render_plot(pitch: f32, yaw: f32, amplitude: f32) -> slint::Image {
    let mut pixel_buffer = BUFFER.lock().unwrap().clone(); // SharedPixelBuffer::new(640, 480);
    let size = (pixel_buffer.width(), pixel_buffer.height());

    let backend = BitMapBackend::with_buffer(pixel_buffer.make_mut_bytes(), size);

    // Plotters requires TrueType fonts from the file system to draw axis text - we skip that for
    // WASM for now.
    #[cfg(target_arch = "wasm32")]
    let backend = wasm_backend::BackendWithoutText { backend };

    let root = backend.into_drawing_area();

    root.fill(&WHITE).expect("error filling drawing area");

    let mut chart = ChartBuilder::on(&root)
        .build_cartesian_3d(-3.0..3.0, 0.0..6.0, -3.0..3.0)
        .expect("error building coordinate system");
    chart.with_projection(|mut p| {
        p.pitch = pitch as f64;
        p.yaw = yaw as f64;
        p.scale = 0.7;
        p.into_matrix() // build the projection matrix
    });

    chart.configure_axes().draw().expect("error drawing");

    chart
        .draw_series(
            SurfaceSeries::xoz(
                (-15..=15).map(|x| x as f64 / 5.0),
                (-15..=15).map(|x| x as f64 / 5.0),
                |x, y| pdf(x, y, amplitude as f64),
            )
            .style_func(&|&v| {
                (&HSLColor(240.0 / 360.0 - 240.0 / 360.0 * v / 5.0, 1.0, 0.7)).into()
            }),
        )
        .expect("error drawing series");

    root.present().expect("error presenting");
    drop(chart);
    drop(root);

    slint::Image::from_rgb8(pixel_buffer)
}