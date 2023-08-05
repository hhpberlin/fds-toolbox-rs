use std::time::Duration;

use fds_toolbox_core::geom::Vec3I;

use crate::moka::{MokaStore, SimulationIdx};

fn get_closest() {}

struct RSet {
    thing: Vec<(Vec3I, Duration)>,
}

fn get_rset<X, Y, Z>(
    moka: &MokaStore,
    x: impl Fn() -> X,
    y: impl Fn() -> Y,
    z: impl Fn() -> Z,
    threshold: f32,
) -> Result<(), ()>
where
    X: IntoIterator<Item = usize>,
    Y: IntoIterator<Item = usize>,
    Z: IntoIterator<Item = usize>,
{
    // for x in x().into_iter() {
    //     for y in y().into_iter() {
    //         for z in z().into_iter() {
    //             let x = moka.get(idx)
    //         }
    //     }
    // }
    Ok(())
}
