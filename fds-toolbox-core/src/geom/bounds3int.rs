use std::ops;
use std::ops::Index;
use derive_more::{Add, Sub, Mul, Div};
use strum_macros::EnumIter;


impl Default for Dimension3D {
    fn default() -> Self {
        Self::X
    }
}


impl vector3int::Vector3I {
    const ZERO: vector3int::Vector3I = vector3int::Vector3I { x: 0, y: 0, z: 0 };
}


#[derive(Clone, Copy, Default)]
pub struct Bounds3I {
    pub min: vector3int::Vector3I,
    pub max: vector3int::Vector3I,
}

impl Bounds3I {
    pub fn new(min_x: i32, min_y: i32, min_z: i32, max_x: i32, max_y: i32, max_z: i32) -> Bounds3I {
        Bounds3I {
            min: vector3int::Vector3I {
                x: min_x,
                y: min_y,
                z: min_z,
            },
            max: vector3int::Vector3I {
                x: max_x,
                y: max_y,
                z: max_z,
            },
        }
    }
    pub fn area(&self) -> vector3int::Vector3I {
        self.max - self.min
    }
}
