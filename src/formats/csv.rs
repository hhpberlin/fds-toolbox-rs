use serde::{Deserialize, Serialize};

use uom::si::f32::{Time};

use self::{hrr::HRRStep, devc::Devices};

pub mod hrr;
pub mod devc;

#[derive(Debug, Serialize, Deserialize)]
pub struct CsvData {
    cpu_entries: Vec<CpuData>,
    heat_release_rate_entries: Vec<HRRStep>,
    device_lists: Vec<Devices>,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct CpuData {
    mpi_rank: u32,
    main_time: Time,
    divg_time: Time,
    mass_time: Time,
    velo_time: Time,
    pres_time: Time,
    wall_time: Time,
    dump_time: Time,
    part_time: Time,
    radi_time: Time,
    fire_time: Time,
    evac_time: Time,
    hvac_time: Time,
    comm_time: Time,
    total_time: Time,
}
