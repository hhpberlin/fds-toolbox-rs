use std::net::SocketAddr;

pub(crate) mod lazy_data;
pub(crate) mod sync;
use lazy_data::remote::quic_remote::QuicRemote;

use crate::lazy_data::remote::quic_remote::ConnectionInfo;

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let _remote = QuicRemote::connect(ConnectionInfo {
        remote_addr: SocketAddr::from(([127, 0, 0, 1], 5000)),
        local_addr: SocketAddr::from(([127, 0, 0, 1], 5001)),
        server_name: "localhost",
    })
    .await
    .unwrap();

    // let comp: dyn CompressionAlgorithm = BrotliCompression;
}

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
