// use fds_toolbox_core::file::ParsedFile;

use std::path::PathBuf;

use clap::{arg, Parser};
use color_eyre::eyre;
use fds_toolbox_core::file::{OsFs, Simulation};

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

    dbg!(sim.path);

    Ok(())
}
