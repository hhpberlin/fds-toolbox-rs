// use fds_toolbox_core::file::ParsedFile;

use std::path::PathBuf;

use clap::{arg, Parser};
use color_eyre::eyre;
use fds_toolbox_core::file::{OsFs, Simulation, FileSystem};

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

    let sim = Simulation::parse_smv(
        OsFs,
        args.smv
            .parent()
            .ok_or(eyre::eyre!("Missing Directory"))?
            .to_path_buf(),
        &args.smv,
    )
    .await?;
    // .map_err(eyre!("Parsing bruh moment"))?;

    dbg!(&sim.path);

    dbg!(sim.csv_cpu().await?);
    // dbg!(sim.csv_devc().await?.time_in_seconds.stats);
    // dbg!(sim.csv_hrr().await?.len());

    devc(sim).await?;

    Ok(())
}

async fn devc<Fs: FileSystem>(sim: Simulation<Fs>) -> color_eyre::Result<()> {
    let devc = sim.csv_devc().await?;
    devc.devices.iter().for_each(|d| {
        println!("{}", d.name);
    });
    Ok(())
}