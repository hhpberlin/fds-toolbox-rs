use serde::{Deserialize, Serialize};

use self::{cpu::CpuInfo, devc::DeviceList, hrr::HrrStep};

pub mod cpu;
pub mod devc;
pub mod hrr;

// TODO: There's `mass` and `ctrl` csv files as well apparently
#[derive(Debug, Serialize, Deserialize)]
pub struct CsvData {
    cpu_entries: Vec<CpuInfo>,
    heat_release_rate_entries: Vec<HrrStep>,
    device_lists: Vec<DeviceList>,
}
