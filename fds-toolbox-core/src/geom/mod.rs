// pub mod bounds3int;
// pub mod vector3int;

use std::ops::Index;

use derive_more::{Add, Constructor, Mul, Sub, Sum};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Dimension3D {
    X,
    Y,
    Z,
}

impl Dimension3D {
    pub fn iter() -> impl Iterator<Item = Dimension3D> {
        [Dimension3D::X, Dimension3D::Y, Dimension3D::Z].into_iter()
    }
}

#[derive(Add, Sub, Mul, Sum, Constructor, Default, PartialEq, Eq, Debug, Copy, Clone)]
pub struct Point2<T> {
    pub x: T,
    pub y: T,
}

// TODO: Is 32-bit enough?
pub type Point2I = Point2<i32>;
pub type Point2U = Point2<u32>;

impl Point2I {
    pub const ZERO: Point2I = Point2I { x: 0, y: 0 };
    pub const ONE: Point2I = Point2I { x: 1, y: 1 };
}

#[derive(Add, Sub, Mul, Sum, Constructor, Default, PartialEq, Eq, Debug, Copy, Clone)]
pub struct Point3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

// TODO: Is 32-bit enough?
pub type Point3I = Point3<i32>;
pub type Point3U = Point3<u32>;

impl Point3I {
    pub const ZERO: Point3I = Point3I { x: 0, y: 0, z: 0 };
    pub const ONE: Point3I = Point3I { x: 1, y: 1, z: 1 };
}

impl<T> Point3<T> {
    pub fn iter<'a>(&self) -> impl Iterator<Item = T> + 'a
    where
        T: Copy + 'a,
    {
        [self.x, self.y, self.z].into_iter()
    }

    pub fn enumerate<'a>(&self) -> impl Iterator<Item = (Dimension3D, T)> + 'a
    where
        T: Copy + 'a,
    {
        Dimension3D::iter().zip(self.iter())
    }
}

impl<T> Index<Dimension3D> for Point3<T> {
    type Output = T;

    fn index(&self, i: Dimension3D) -> &T {
        match i {
            Dimension3D::X => &self.x,
            Dimension3D::Y => &self.y,
            Dimension3D::Z => &self.z,
        }
    }
}

// TODO: Should this really derive Default?
#[derive(Constructor, Default, PartialEq, Eq, Debug, Copy, Clone)]
pub struct Bounds3<T> {
    pub min: Point3<T>,
    pub max: Point3<T>,
}

pub type Bounds3I = Bounds3<i32>;

impl Bounds3I {
    pub fn area(&self) -> Point3U {
        Point3::new(
            i32::abs_diff(self.min.x, self.max.x),
            i32::abs_diff(self.min.y, self.max.y),
            i32::abs_diff(self.min.z, self.max.z),
        )
    }

    pub fn iter(&self) -> impl Iterator<Item = Point3I> {
        let min = self.min;
        let max = self.max;
        (min.x..=max.x).flat_map(move |x| {
            (min.y..=max.y).flat_map(move |y| (min.z..=max.z).map(move |z| Point3::new(x, y, z)))
        })
    }
}
