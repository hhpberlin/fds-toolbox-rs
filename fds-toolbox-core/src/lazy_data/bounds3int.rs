use std::ops;
use std::ops::Index;
use ndarray::Dim;

pub enum Dimension3D {
    X, Y, Z
}

#[derive(Copy)]
pub struct Vector3Int {
    x: i32,
    y: i32,
    z: i32,
}

impl Vector3Int {
    const ZERO: Vector3Int = Vector3Int{x: 0, y: 0, z: 0};
}

impl Index<Dimension3D> for Vector3Int {
    type Output = i32;
    fn index(& self, i: Dimension3D) -> i32{
       match i {
           Dimension3D::X => self.x,
           Dimension3D::Y => self.y,
           Dimension3D::Z => self.z
       } 
    }
}

impl ops::Sub<Vector3Int> for Vector3Int {
    type Output = Vector3Int;
    fn sub(self, rhs: Vector3Int) -> Self::Output {
        Vector3Int{x: self.x - rhs.x, y: self.y - rhs.y, z: self.z - rhs.z}
    }
}
impl ops::add<vector3int> for vector3int {
    type Output = vector3int;
    fn add(self, rhs: vector3int) -> self::Output {
        vector3int{x: self.x + rhs.x, y: self.y + rhs.y, z: self.z + rhs.z}
    }
}
impl ops::Mul<vector3int> for vector3int {
    type Output = vector3int;
    fn mul(self, rhs: vector3int) -> Self::Output {
        vector3int{x: self.x * rhs.x, y: self.y * rhs.y, z: self.z * rhs.z}
    }
}
impl ops::Mul<f32> for vector3int {
    type Output = vector3int;
    fn mul(self, rhs: f32) -> Self::Output {
        vector3int{x: self.x * rhs, y: self.y * rhs, z: self.z * rhs}
    }
}

#[derive(Copy)]
pub struct Bounds3Int {
    min: Vector3Int,
    max: Vector3Int,
}

impl Bounds3Int {
    fn area(&self) -> Vector3Int {
        self.max - self.min
    }
}