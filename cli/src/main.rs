// use fds_toolbox_core::file::ParsedFile;

use std::path::PathBuf;

use clap::{arg, Parser};
use color_eyre::eyre;

use fds_toolbox_core::file::{OsFs, Simulation};
use fds_toolbox_lazy_data::{fs::AnyFs, moka::MokaStore};
// use fds_toolbox_lazy_data::{memman::MEMORY_MANAGER, sim::CachedSimulation, moka::{MokaStore, DevcIdx}, fs::AnyFs};
use futures::FutureExt;

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
        AnyFs::LocalFs(OsFs),
        args.smv
            .parent()
            .ok_or(eyre::eyre!("Missing Directory"))?
            .to_str()
            .unwrap()
            .to_owned(),
        args.smv.to_str().unwrap(),
    )
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
    let sim_idx = moka.get_idx_by_path(&sim.path);
    dbg!(moka.get_devc(sim_idx).await?);
    dbg!(moka.get_devc(sim_idx).now_or_never());

    Ok(())
}
