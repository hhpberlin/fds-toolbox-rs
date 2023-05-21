use std::path::PathBuf;

use clap::{arg, Parser};
use color_eyre::eyre;

use fds_toolbox_core::file::{OsFs, Simulation, SimulationPath};
use fds_toolbox_lazy_data::{fs::AnyFs, moka::MokaStore};
use plotters::prelude::*;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the .smv file
    #[arg(short, long, value_name = "FILE")]
    smv: PathBuf,
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let args = Cli::parse();

    dbg!(&args.smv);

    let sim = Simulation::parse_smv(SimulationPath::new(
        AnyFs::LocalFs(OsFs),
        args.smv
            .parent()
            .ok_or(eyre::eyre!("Missing Directory"))?
            .to_str()
            .unwrap()
            .to_owned(),
        args.smv.to_str().unwrap(),
    ))
    .await?;

    // let sim = CachedSimulation::new(Arc::new(sim), None);

    // MEMORY_MANAGER.print_stats();

    // dbg!(sim.get_devc().get().await);

    // MEMORY_MANAGER.print_stats();

    // // let aaa = sim.get_devc();
    // // pin!(aaa);
    // // let aaa = aaa;
    // // dbg!(aaa.poll(Context::));

    // MEMORY_MANAGER.flush_all();
    // println!("Flushed all");
    // MEMORY_MANAGER.print_stats();

    let moka = MokaStore::new(10000);
    let sim_idx = moka.get_idx_by_path(&sim.path).0;
    // dbg!(moka.devc().try_get_or_spawn(sim_idx, ()));
    // tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    // dbg!(moka.devc().try_get(sim_idx, ()));
    // dbg!(moka.devc().get(sim_idx, ()).await?);
    // dbg!(moka.devc().get(sim_idx, ()).now_or_never());

    dbg!(moka
        .devc()
        .get(sim_idx, ())
        .await
        .unwrap()
        .enumerate_device_readings()
        .map(|x| (x.0, &x.1.name, &x.1.unit,))
        .map(|x| x.2)
        .collect::<Vec<_>>());

    // let b = BitMapBackend::new("test.png", (1024, 768));
    // let a = b.into_drawing_area();
    // a.fill(&WHITE)?;
    // let mut chart = ChartBuilder::on(&a)
    //     .caption("Test", ("sans-serif", 50))
    //     .margin(5)
    //     .x_label_area_size(30)
    //     .y_label_area_size(30)
    //     .build_cartesian_2d(50f32..100f32, 0f32..80f32)?;
    // chart.configure_mesh().draw()?;
    // chart.draw_series(LineSeries::new(
    //     (0..100).map(|x| (x as f32, x as f32)),
    //     &RED,
    // ))?;
    // a.present()?;

    Ok(())
}
