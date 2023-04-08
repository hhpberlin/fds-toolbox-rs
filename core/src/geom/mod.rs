// pub mod bounds3int;
// pub mod vector3int;

use std::ops::Index;

use derive_more::{Add, Constructor, Mul, Sub, Sum};

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Dim3D {
    X = 0,
    Y = 1,
    Z = 2,
}

pub struct Dim3DSigned {
    pub is_positive: bool,
    pub dim: Dim3D,
}

impl Dim3DSigned {
    pub fn new(is_positive: bool, dim: Dim3D) -> Self {
        Self { is_positive, dim }
    }

    pub fn iter() -> impl Iterator<Item = Dim3DSigned> {
        [
            Dim3DSigned::new(true, Dim3D::X),
            Dim3DSigned::new(false, Dim3D::X),
            Dim3DSigned::new(true, Dim3D::Y),
            Dim3DSigned::new(false, Dim3D::Y),
            Dim3DSigned::new(true, Dim3D::Z),
            Dim3DSigned::new(false, Dim3D::Z),
        ]
        .into_iter()
    }

    pub fn as_u8(&self) -> u8 {
        // TODO: Is this correct?
        (self.dim as u8) + if self.is_positive { 0 } else { 3 }
    }
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

pub type Vec2F = Vec2<f32>;

impl Vec2I {
    pub const ZERO: Vec2I = Vec2I { x: 0, y: 0 };
    pub const ONE: Vec2I = Vec2I { x: 1, y: 1 };
}

impl<T> From<(T, T)> for Vec2<T> {
    fn from((x, y): (T, T)) -> Self {
        Vec2 { x, y }
    }
}

impl<T> From<Vec2<T>> for (T, T) {
    fn from(v: Vec2<T>) -> Self {
        (v.x, v.y)
    }
}

#[derive(Add, Sub, Mul, Sum, Constructor, Default, PartialEq, Eq, Debug, Copy, Clone, Hash)]
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
#[derive(Constructor, Default, PartialEq, Eq, Debug, Copy, Clone, Hash)]
pub struct Bounds3<T> {
    pub min: Vec3<T>,
    pub max: Vec3<T>,
}

impl<T> Bounds3<T> {
    pub fn from_fds_notation(min_x: T, max_x: T, min_y: T, max_y: T, min_z: T, max_z: T) -> Self {
        Bounds3::new(
            Vec3::new(min_x, min_y, min_z),
            Vec3::new(max_x, max_y, max_z),
        )
    }

    pub fn from_fds_notation_tuple(tuple: (T, T, T, T, T, T)) -> Self {
        Self::from_fds_notation(tuple.0, tuple.1, tuple.2, tuple.3, tuple.4, tuple.5)
    }
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

// Not using Bounds3 so bounds for arithmetic operations can be applied to Bounds3 later, without restricting this ones type arguments
#[derive(Constructor, Default, PartialEq, Eq, Debug, Copy, Clone)]
pub struct Surfaces3<T> {
    pub neg_x: T,
    pub pos_x: T,
    pub neg_y: T,
    pub pos_y: T,
    pub neg_z: T,
    pub pos_z: T,
}
