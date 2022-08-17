use std::ops::{Add, Div, Mul, Sub};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ArrayStats<N, M = N> {
    pub min: N,
    pub max: N,
    pub mean: M,
    pub variance: M,
    pub std_dev: M,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Range<N> {
    pub min: N,
    pub max: N,
}

impl<N: Default, M: Default> Default for ArrayStats<N, M> {
    fn default() -> Self {
        Self {
            min: N::default(),
            max: N::default(),
            mean: M::default(),
            variance: M::default(),
            std_dev: M::default(),
        }
    }
}

impl<
        N: PartialOrd + Copy,
        M: Add<Output = M> + Sub<Output = M> + Div<Output = M> + Mul<Output = M> + From<N> + Copy,
    > ArrayStats<N, M>
{
    pub fn new(
        mut data: impl Iterator<Item = N>,
        div: fn(M, usize) -> M,
        sqrt: fn(M) -> M,
    ) -> Option<Self> {
        let first = match data.next() {
            Some(value) => value,
            None => return None,
        };
        let first_m = M::from(first);
        let mut min = first;
        let mut max = first;
        let mut sum = first_m;
        let mut sum_sq = first_m * first_m;
        let mut count = 1usize;
        let data = data.peekable();
        for value in data {
            if value < min {
                min = value;
            }
            if value > max {
                max = value;
            }
            let value_m = M::from(value);
            sum = sum + value_m;
            sum_sq = sum_sq + (value_m * value_m);
            count += 1;
        }
        let mean = div(sum, count);
        let variance = (div(sum_sq, count)) - (mean * mean);
        // TODO: Implement using Welford's algorithm
        // TODO: This NaNs for negatives
        let std_dev = sqrt(variance);
        Some(Self {
            min,
            max,
            mean,
            variance,
            std_dev,
        })
    }

    pub fn bounds(mut iter: impl Iterator<Item = Self>) -> Option<Range<N>> {
        let first = match iter.next() {
            Some(value) => value,
            None => return None,
        };
        let mut range = Range {
            min: first.min,
            max: first.max,
        };
        for stats in iter {
            if stats.min < range.min {
                range.min = stats.min;
            }
            if stats.max > range.max {
                range.max = stats.max;
            }
        }
        Some(range)
    }
}

impl<N> Range<N> {
    pub fn new(min: N, max: N) -> Self {
        Self { min, max }
    }
}

impl<N: Sub + Copy> Range<N>
where
    <N as Sub>::Output: Div<<N as Sub>::Output>,
{
    pub fn width(&self) -> <N as Sub>::Output {
        self.max - self.min
    }

    pub fn map(&self, value: N) -> <<N as Sub>::Output as Div>::Output {
        (value - self.min) / self.width()
    }
}

impl ArrayStats<f32> {
    pub fn new_f32(data: impl Iterator<Item = f32>) -> Option<Self> {
        Self::new(data, |a, b| a / b as f32, |a| a.sqrt())
    }
}

impl ArrayStats<u8, f32> {
    pub fn new_u8(data: impl Iterator<Item = u8>) -> Option<Self> {
        Self::new(data, |a, b| a / b as f32, |a| a.sqrt())
    }
}
