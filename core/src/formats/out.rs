use chrono::{DateTime, Duration, Utc};
use uom::si::f32::{Power, Time};

use crate::geom::Vec3;

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
}

pub struct Step {
    number: u32,
    time_calculated: DateTime<Utc>,
    sim_step_size: Time,
    sim_elapsed_time: Time,
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

pub struct PositionedValue<T> {
    pos: Vec3<u32>,
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
