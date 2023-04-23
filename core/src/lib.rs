// TODO: Re-enable and fix
// #![warn(clippy::pedantic)]

// #![warn(clippy::nursery)]
// #![warn(clippy::cargo)]
#![warn(clippy::complexity)]
#![warn(clippy::correctness)]
#![warn(clippy::perf)]
#![warn(clippy::style)]
#![warn(clippy::suspicious)]
#![warn(clippy::print_stdout)]
#![warn(clippy::print_stderr)]
// #![warn(clippy::todo)]
// #![warn(clippy::unimplemented)]
// #![warn(clippy::dbg_macro)]
// #![warn(clippy::unreachable)]
// #![warn(clippy::panic)]

// #![warn(clippy::unwrap_used)]
// #![warn(clippy::expect_used)]

// TODO: Remove this and remove dead-code once prototyping is done
#![allow(dead_code)]

pub mod common;
pub mod formats;
pub(crate) mod geom;
// pub(crate) mod lazy_data;

pub mod file;
pub(crate) mod sync;

// #[tokio::main]
// async fn main() {
//     println!("Hello, world!");

//     let _remote = QuicRemote::connect(ConnectionInfo {
//         remote_addr: SocketAddr::from(([127, 0, 0, 1], 5000)),
//         local_addr: SocketAddr::from(([127, 0, 0, 1], 5001)),
//         server_name: "localhost",
//     })
//     .await
//     .unwrap();

//     // let comp: dyn CompressionAlgorithm = BrotliCompression;
// }

// pub struct FdsSimulation {
//     pub chid: String,
//     pub meshes: Vec<FdsMesh>,
//     pub surfaces: Vec<FdsSurface>,
//     pub ventilations: Vec<FdsVentilation>,
//     pub slices: Vec<FdsSlice>,
//     pub data_3d: Vec<FdsData3D>,
//     pub isosurfaces: Vec<FdsIsoSurface>,
//     pub particles: Vec<particles>,
//     pub devices: Vec<FdsDevice>,
//     pub evacs: Vec<FdsEvac>,
//     pub smoke3d: Vec<FdsSmoke3D>,
// }
