use chrono::{DateTime, Utc, Duration};

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
    sim_start_time: Duration,
    sim_end_time: Duration,
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
    max_velocity_error: PositionedValue<f32>,
    file_start_index: u32,
    mesh_steps: Vec<MeshStep>,
}

pub struct MeshStep {
    
}

pub struct PositionedValue<T> {
    mesh_number: u32,
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
    Fds5, Fds6,
}