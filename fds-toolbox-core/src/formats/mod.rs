use self::csv::devc::Devices;

pub mod csv;
pub mod out;
pub mod slcf;
pub mod smv;

#[derive(Debug)]
pub struct Simulation {
    pub devc: Devices,
}

pub enum SimulationData2D<'a> {
    Device(&'a str),
}
