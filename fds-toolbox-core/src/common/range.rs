use std::{
    hash::Hash,
    ops::{Add, Div, Mul, Sub, RangeInclusive},
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RangeIncl<N> {
    pub min: N,
    pub max: N,
}

impl Hash for RangeIncl<f32> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u32(self.min.to_bits());
        state.write_u32(self.max.to_bits());
    }
}

impl<N> RangeIncl<N> {
    pub fn new(min: N, max: N) -> Self {
        Self { min, max }
    }
}

impl<N: Default> Default for RangeIncl<N> {
    fn default() -> Self {
        Self {
            min: N::default(),
            max: N::default(),
        }
    }
}

impl<N: Sub<Output = N> + Div<Output = N> + Copy> RangeIncl<N> {
    pub fn width(&self) -> <N as Sub>::Output {
        self.max - self.min
    }

    pub fn center(&self) -> <N as Div>::Output
    where
        N: Add<Output = N> + From<u8>,
    {
        (self.max + self.min) / N::from(2)
    }

    pub fn map(&self, value: N) -> <<N as Sub>::Output as Div>::Output {
        (value - self.min) / self.width()
    }

    // pub fn unmap(&self, value: N) -> <<N as Sub>::Output as Div>::Output {
    //     (value + self.min) / self.width()
    // }

    pub fn zoom(&mut self, center: N, factor: N)
    where
        N: Mul<Output = N> + Add<Output = N> + PartialOrd,
    {
        self.min = center + (self.min - center) * factor;
        self.max = center + (self.max - center) * factor;
        if self.min > self.max {
            (self.min, self.max) = (self.max, self.min);
        }
    }

    pub fn pan(&mut self, delta: N)
    where
        N: Add<Output = N>,
    {
        self.min = self.min + delta;
        self.max = self.max + delta;
    }
}

impl<N> RangeIncl<N> {
    pub fn into_range(self) -> std::ops::Range<N> {
        self.min..self.max
    }

    pub fn into_range_inclusive(self) -> std::ops::RangeInclusive<N> {
        self.min..=self.max
    }
}

impl<N: PartialOrd + Copy> RangeIncl<N> {
    pub fn expand(&self, new: N) -> Self {
        Self::new(
            if self.min < new { self.min } else { new },
            if self.max > new { self.max } else { new },
        )
    }

    pub fn max(&self, new: RangeIncl<N>) -> Self {
        Self::new(
            if self.min < new.min {
                self.min
            } else {
                new.min
            },
            if self.max > new.max {
                self.max
            } else {
                new.max
            },
        )
    }

    pub fn from_iter_val(iter: impl IntoIterator<Item = N>) -> Option<RangeIncl<N>> {
        iter.into_iter().fold(None, |acc, n| match acc {
            Some(acc) => Some(acc.expand(n)),
            None => Some(RangeIncl::new(n, n)),
        })
    }

    pub fn from_iter_range(iter: impl IntoIterator<Item = RangeIncl<N>>) -> Option<RangeIncl<N>> {
        iter.into_iter().fold(None, |acc, range| match acc {
            Some(acc) => Some(acc.max(range)),
            None => Some(range),
        })
    }

    pub fn contains(&self, value: N) -> bool {
        self.min <= value && value <= self.max
    }
}

impl<N> From<RangeIncl<N>> for std::ops::Range<N> {
    fn from(range: RangeIncl<N>) -> Self {
        range.into_range()
    }
}

impl<N: Clone> From<RangeInclusive<N>> for RangeIncl<N> {
    fn from(range: RangeInclusive<N>) -> Self {
        Self::new(range.start().clone(), range.end().clone())
    }
}