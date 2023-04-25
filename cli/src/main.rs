// use fds_toolbox_core::file::ParsedFile;

use std::{path::PathBuf, sync::Arc};

use clap::{arg, Parser};
use color_eyre::eyre;

use fds_toolbox_core::file::{OsFs, Simulation};
use fds_toolbox_lazy_data::{memman::MEMORY_MANAGER, sim::CachedSimulation};

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

    let sim = CachedSimulation::new(Arc::new(sim), None);

    MEMORY_MANAGER.print_stats();

    dbg!(sim.get_cpu().await);

    MEMORY_MANAGER.print_stats();
    MEMORY_MANAGER.flush_all();
    println!("Flushed all");
    MEMORY_MANAGER.print_stats();

    Ok(())
}
