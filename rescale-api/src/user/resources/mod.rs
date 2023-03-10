// /// Information about a Rescale coretype.
// #[derive(Debug, Deserialize)]
// struct Coretype {
//     /// Indicates whether this coretype has SSD drives.
//     has_ssd: bool,

//     /// A unique identifier for this coretype, used in other requests.
//     code: String,

//     /// TODO.
//     compute: String,

//     /// The display name of the coretype.
//     name: String,

//     /// Indicates if this coretype is deprecated. If true, it cannot be used to submit jobs.
//     is_deprecated: bool,

//     /// Price per core hour for on-demand jobs.
//     price: f32,

//     /// Indicates if this coretype can be used for remote visualization clusters.
//     remote_viz_allowed: bool,

//     /// The amount of per-core storage in GB.
//     storage: i32,

//     /// The price per core hour for low priority jobs.
//     low_priority_price: String,

//     /// Indicates if jobs submitted with this coretype must specify a maximum walltime.
//     walltime_required: bool,

//     /// An integer for sorting coretype displays.
//     display_order: Option<i32>,

//     /// Indicates the amount of IO available on this coretype, e.g. 10 GB/s.
//     io: String,

//     /// The amount of per-core memory in MB.
//     memory: i32,

//     /// The number of cores on instances of this coretype, with an entry for each instance size.
//     cores: Vec<i32>,

//     /// Indicates if this is a primary (default) coretype.
//     is_primary: bool,

//     /// Processor information.
//     processor_info: String,

//     /// The same as the storage property.
//     storage_io: String,

//     /// A description of this coretype.
//     description: String,
// }
