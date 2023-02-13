// pub mod bounds3int;
// pub mod vector3int;

use std::ops::Index;

use derive_more::{Add, Constructor, Mul, Sub, Sum};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Dim3D {
    X,
    Y,
    Z,
}

impl Dim3D {
    pub fn iter() -> impl Iterator<Item = Dim3D> {
        [Dim3D::X, Dim3D::Y, Dim3D::Z].into_iter()
    }
}

#[derive(Add, Sub, Mul, Sum, Constructor, Default, PartialEq, Eq, Debug, Copy, Clone)]
pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}

// TODO: Is 32-bit enough?
pub type Vec2I = Vec2<i32>;
pub type Vec2U = Vec2<u32>;

impl Vec2I {
    pub const ZERO: Vec2I = Vec2I { x: 0, y: 0 };
    pub const ONE: Vec2I = Vec2I { x: 1, y: 1 };
}

#[derive(Add, Sub, Mul, Sum, Constructor, Default, PartialEq, Eq, Debug, Copy, Clone)]
pub struct Vec3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

// TODO: Is 32-bit enough?
pub type Vec3I = Vec3<i32>;
pub type Vec3U = Vec3<u32>;

pub type Vec3F = Vec3<f32>;

impl Vec3I {
    pub const ZERO: Vec3I = Vec3I { x: 0, y: 0, z: 0 };
    pub const ONE: Vec3I = Vec3I { x: 1, y: 1, z: 1 };
}

impl<T> From<(T, T, T)> for Vec3<T> {
    fn from((x, y, z): (T, T, T)) -> Self {
        Vec3 { x, y, z }
    }
}

impl<T> From<Vec3<T>> for (T, T, T) {
    fn from(v: Vec3<T>) -> Self {
        (v.x, v.y, v.z)
    }
}

impl<T> Vec3<T> {
    pub fn iter<'a>(&self) -> impl Iterator<Item = T> + 'a
    where
        T: Copy + 'a,
    {
        [self.x, self.y, self.z].into_iter()
    }

    pub fn enumerate<'a>(&self) -> impl Iterator<Item = (Dim3D, T)> + 'a
    where
        T: Copy + 'a,
    {
        Dim3D::iter().zip(self.iter())
    }
}

impl<T> Index<Dim3D> for Vec3<T> {
    type Output = T;

    fn index(&self, i: Dim3D) -> &T {
        match i {
            Dim3D::X => &self.x,
            Dim3D::Y => &self.y,
            Dim3D::Z => &self.z,
        }
    }
}

// TODO: Should this really derive Default?
#[derive(Constructor, Default, PartialEq, Eq, Debug, Copy, Clone)]
pub struct Bounds3<T> {
    pub min: Vec3<T>,
    pub max: Vec3<T>,
}

pub type Bounds3I = Bounds3<i32>;
pub type Bounds3F = Bounds3<f32>;

impl Bounds3I {
    pub fn area(&self) -> Vec3U {
        Vec3::new(
            i32::abs_diff(self.min.x, self.max.x),
            i32::abs_diff(self.min.y, self.max.y),
            i32::abs_diff(self.min.z, self.max.z),
        )
    }

    pub fn iter(&self) -> impl Iterator<Item = Vec3I> {
        let min = self.min;
        let max = self.max;
        (min.x..=max.x).flat_map(move |x| {
            (min.y..=max.y).flat_map(move |y| (min.z..=max.z).map(move |z| Vec3::new(x, y, z)))
        })
    }
}
