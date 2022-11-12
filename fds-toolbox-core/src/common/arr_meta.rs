use std::{
    hash::Hash,
    ops::{Add, Mul, Sub},
};

use serde::{Deserialize, Serialize};

use super::range::RangeIncl;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArrayStats<Num, NumDivisible = Num, NumSq = Num> {
    pub range: RangeIncl<Num>,
    pub mean: NumDivisible,
    pub variance: NumSq,
    pub std_dev: NumDivisible,
}

impl Hash for ArrayStats<f32> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.range.hash(state);
        state.write_u32(self.mean.to_bits());
        state.write_u32(self.variance.to_bits());
        state.write_u32(self.std_dev.to_bits());
    }
}

impl<N: Default, M: Default, S: Default> Default for ArrayStats<N, M, S> {
    fn default() -> Self {
        Self {
            range: RangeIncl::default(),
            mean: M::default(),
            variance: S::default(),
            std_dev: M::default(),
        }
    }
}

impl<
        N: PartialOrd + Copy,
        M: Add<Output = M> + Sub<Output = M> + Mul<Output = S> + From<N> + Copy,
        S: Add<Output = S> + Sub<Output = S> + Copy,
    > ArrayStats<N, M, S>
{
    pub fn new(
        mut data: impl Iterator<Item = N>,
        div: fn(M, usize) -> M,
        div_sq: fn(S, usize) -> S,
        sqrt: fn(S) -> M,
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
        let variance = (div_sq(sum_sq, count)) - (mean * mean);
        // TODO: Implement using Welford's algorithm
        // TODO: This NaNs for negatives
        let std_dev = sqrt(variance);
        Some(Self {
            range: RangeIncl { min, max },
            mean,
            variance,
            std_dev,
        })
    }

    pub fn bounds(mut iter: impl Iterator<Item = Self>) -> Option<RangeIncl<N>> {
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
        Self::new(data, |a, b| a / b as f32, |a, b| a / b as f32, |a: f32| a.sqrt())
    }
}

// impl ArrayStats<Time, Time, Time> {
//     pub fn new_time_f32(data: impl Iterator<Item = Time>) -> Option<Self> {
//         Self::new(data, |a, b| a / b, |a, b| a / b, |a| a.sqrt())l
//     }
// }

impl ArrayStats<u8, f32, f32> {
    pub fn new_u8(data: impl Iterator<Item = u8>) -> Option<Self> {
        Self::new(data, |a, b| a / b as f32, |a, b| a / b as f32, |a| a.sqrt())
    }
}
