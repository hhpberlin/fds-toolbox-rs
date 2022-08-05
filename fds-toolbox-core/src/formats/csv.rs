use serde::{Deserialize, Serialize};

use self::{cpu::CpuData, devc::Devices, hrr::HRRStep};

pub mod cpu;
pub mod devc;
pub mod hrr;

#[derive(Debug, Serialize, Deserialize)]
pub struct CsvData {
    cpu_entries: Vec<CpuData>,
    heat_release_rate_entries: Vec<HRRStep>,
    device_lists: Vec<Devices>,
}
