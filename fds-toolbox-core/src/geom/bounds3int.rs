use std::ops;
use std::ops::Index;
use strum_macros::EnumIter;

#[derive(Clone, Copy, EnumIter, PartialEq)]
pub enum Dimension3D {
    X,
    Y,
    Z,
}

impl Default for Dimension3D {
    fn default() -> Self {
        Self::X
    }
}

#[derive(Clone, Copy, Default)]
pub struct Vector3I {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Vector3I {
    const ZERO: Vector3I = Vector3I { x: 0, y: 0, z: 0 };
}

impl Index<Dimension3D> for Vector3I {
    type Output = i32;

    fn index(&self, i: Dimension3D) -> &i32 {
        match i {
            Dimension3D::X => &self.x,
            Dimension3D::Y => &self.y,
            Dimension3D::Z => &self.z,
        }
    }
}

impl ops::Sub<Vector3I> for Vector3I {
    type Output = Vector3I;
    fn sub(self, rhs: Vector3I) -> Self::Output {
        Vector3I {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}
impl ops::Add<Vector3I> for Vector3I {
    type Output = Vector3I;
    fn add(self, rhs: Vector3I) -> Self::Output {
        Vector3I {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}
impl ops::Mul<Vector3I> for Vector3I {
    type Output = Vector3I;
    fn mul(self, rhs: Vector3I) -> Self::Output {
        Vector3I {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        }
    }
}
impl ops::Mul<i32> for Vector3I {
    type Output = Vector3I;
    fn mul(self, rhs: i32) -> Self::Output {
        Vector3I {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct Bounds3I {
    pub min: Vector3I,
    pub max: Vector3I,
}

impl Bounds3I {
    pub fn new(min_x: i32, min_y: i32, min_z: i32, max_x: i32, max_y: i32, max_z: i32) -> Bounds3I {
        Bounds3I {
            min: Vector3I {
                x: min_x,
                y: min_y,
                z: min_z,
            },
            max: Vector3I {
                x: max_x,
                y: max_y,
                z: max_z,
            },
        }
    }
    pub fn area(&self) -> Vector3I {
        self.max - self.min
    }
}
