// use ariadne::Report;

use std::fs;

use super::Simulation;

#[test]
fn parses_successfully() {
    let input = include_str!("../../../../demo-house/DemoHaus2.smv");
    // let input = include_str!("/vol/big/hhp file stuff big/LCBH_FDS674_M16_EG08_v02__Cloned__run1/LCBH_FDS674_M16_EG08_v02.smv");
    // let input = include_str!("/vol/big/hhp file stuff big/22B0062_BIZ_SZ02_UGEG_Koffer_M16_v01_run1/22B0062_BIZ_SZ02-UGEG-Koffer_M16_v01.smv");
    let sim = Simulation::parse_with_warn_stdout(input).unwrap();
    assert_eq!(sim.chid, "DemoHaus2");
}

#[test]
fn parses_a_lot_of_stuff() {
    // Recurse through all directories in a folder where I have some simulations and parse all .smv files
    // TODO: Make this not depend on my specific file structure - environment var or something?

    fn visit_dirs(dir: &std::path::Path, cb: &dyn Fn(&std::path::Path)) -> std::io::Result<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    visit_dirs(&path, cb)?;
                } else {
                    cb(&path);
                }
            }
        }
        Ok(())
    }

    visit_dirs(
        std::path::Path::new("/vol/big/hhp file stuff big"),
        &|path| {
            if path.extension().unwrap() == "smv" {
                let input = fs::read_to_string(path).unwrap();
                println!("-- Parsing {:?}", path);
                println!(" > {} bytes", input.len());
                let sim = Simulation::parse_with_warn_stdout(&input);
                sim.unwrap();
            }
        },
    )
    .unwrap();
}
