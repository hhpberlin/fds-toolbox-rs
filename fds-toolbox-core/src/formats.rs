use self::csv::devc::Devices;

pub mod arr_meta;
pub mod csv;
pub mod out;
pub mod slcf;
pub mod smv;

#[derive(Debug)]
pub struct Simulation {
    pub devc: Devices,
}
