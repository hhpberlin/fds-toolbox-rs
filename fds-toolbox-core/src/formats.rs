use self::csv::devc::Devices;

pub mod csv;
pub mod out;
pub mod slcf;
pub mod smv;
pub mod arr_meta;

#[derive(Debug)]
pub struct Simulation {
    pub devc: Devices,
}
