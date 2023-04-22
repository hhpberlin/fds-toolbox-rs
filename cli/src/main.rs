// use fds_toolbox_core::file::ParsedFile;

use std::path::PathBuf;

use clap::{arg, Parser};
use color_eyre::eyre;
use fds_toolbox_core::file::{FileSystem, OsFs, Simulation, SimulationPath};
use fds_toolbox_lazy_data::moka::{
    DevcIdx, Fs, MokaStore, SimulationData, SimulationDataIdx, SimulationsDataIdx,
};

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

    // let sim = Simulation::parse_smv(
    //     OsFs,
    //     args.smv
    //         .parent()
    //         .ok_or(eyre::eyre!("Missing Directory"))?
    //         .to_path_buf(),
    //     &args.smv,
    // )
    // .await?;
    // .map_err(eyre!("Parsing bruh moment"))?;

    let moka = MokaStore::new(10_000);

    assert!(args.smv.extension().unwrap() == "smv");

    let path = SimulationPath::new(
        OsFs,
        args.smv
            .parent()
            .ok_or(eyre::eyre!("Missing Directory"))?
            .to_path_buf(),
        args.smv.file_stem().unwrap().to_str().unwrap().to_string(),
    );
    let path = path.map(Fs::LocalFs, |p| p.to_string_lossy().to_string());

    let sim = moka.get_sim(path.clone()).await?;

    dbg!(&sim.path);

    dbg!(sim.csv_cpu().await?);
    // dbg!(sim.csv_devc().await?.time_in_seconds.stats);
    // dbg!(sim.csv_hrr().await?.len());

    // devc(&sim).await?;

    // dbg!(moka.get_devc(path.clone(), DevcIdx(2)).await?);

    Ok(())
}

async fn devc<Fs: FileSystem>(sim: &Simulation<Fs>) -> color_eyre::Result<()> {
    let devc = sim.csv_devc().await?;
    devc.devices.iter().for_each(|d| {
        // println!("{}", d.name, d.values.stats);
    });
    Ok(())
}
