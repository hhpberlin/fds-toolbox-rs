use std::ops::{Add, Div, Mul, Sub};

use serde::{Deserialize, Serialize};

use super::range::Range;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ArrayStats<Num, NumDivisible = Num> {
    pub range: Range<Num>,
    pub mean: NumDivisible,
    pub variance: NumDivisible,
    pub std_dev: NumDivisible,
}

impl<N: Default, M: Default> Default for ArrayStats<N, M> {
    fn default() -> Self {
        Self {
            range: Range::default(),
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
            range: Range { min, max },
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
        let mut range = first.range;
        for stats in iter {
            range = range.max(stats.range);
        }
        Some(range)
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
