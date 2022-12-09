use std::ops;
use std::ops::Index;

#[derive(Clone, Copy)]
pub enum Dimension3D {
    X,
    Y,
    Z,
}

#[derive(Clone, Copy)]
pub struct Vector3I {
    x: i32,
    y: i32,
    z: i32,
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

#[derive(Clone, Copy)]
pub struct Bounds3I {
    min: Vector3I,
    max: Vector3I,
}

impl Bounds3I {
    fn area(&self) -> Vector3I {
        self.max - self.min
    }
}
