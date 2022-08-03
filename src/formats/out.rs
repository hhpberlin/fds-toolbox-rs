use chrono::{DateTime, Duration, Utc};
use uom::si::f32::{Power, MassRate, Time};

use crate::geom::geom::Point3;

pub struct FdsOut {
    job_id: String,
    job_title: String,
    fds_version: FdsVersion,
    mpi_enabled: bool,
    open_mp_enabled: bool,
    mpi_version: String,
    mpi_library_version: String,
    mpi_process_count: u32,
    open_mp_threads: u32,
    start_date: DateTime<Utc>,
    sim_start_time: Time,
    sim_end_time: Time,
    is_completed: bool,
    wallclock_total_elapsed_time: Duration,
    wallclock_time_stepping_time: Duration,
    steps: Vec<Step>,
    cpu_entries: Vec<CpuData>,
    heat_release_rate_entries: Vec<HRRStep>,
    device_lists: Vec<Devices>,
}

pub struct Step {
    number: u32,
    time_calculated: DateTime<Utc>,
    sim_step_size: Duration,
    sim_elapsed_time: Duration,
    pressure_iterations: u32,
    max_velocity_error_mesh_number: u32,
    max_velocity_error: PositionedValue<f32>,
    file_start_index: u32,
    mesh_steps: Vec<MeshStep>,
}

pub struct MeshStep {
    total_heat_release_rate: Power,
    radiation_loss: Power,
    min_divergence: PositionedValue<f32>,
    max_divergence: PositionedValue<f32>,
    max_cfl_number: PositionedValue<f32>,
    max_vn_number: PositionedValue<f32>,
}

pub struct Devices {
    times: Vec<Time>,
    devices: Vec<DeviceReadings>,
}

pub struct DeviceReadings {
    unit: String,
    name: String,
    values: Vec<f32>,
}

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

pub struct HRRStep {
    time: Time,
    heat_release_rate: Power,
    q_radi: Power,
    q_conv: Power,
    q_cond: Power,
    q_diff: Power,
    q_pres: Power,
    q_part: Power,
    q_geom: Power,
    q_enth: Power,
    q_total: Power,
    mass_flow_rate_fuel: MassRate,
    mass_flow_rate_total: MassRate,
}

pub struct PositionedValue<T> {
    pos: Point3<u32>,
    value: T,
}

pub struct FdsVersion {
    major: FdsMajorVersion,
    version_text: String,
    revision: String,
    build_date: DateTime<Utc>,
    compiler: String, // TODO: Enum?
    compilation_date: DateTime<Utc>,
}

pub enum FdsMajorVersion {
    Fds5,
    Fds6,
}
