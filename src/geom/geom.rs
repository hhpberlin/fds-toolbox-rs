use derive_more::*;

#[derive(Add, Sub, Default, PartialEq, Eq, Debug, Copy, Clone)]
pub struct Point3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl Point3<f64> {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
}