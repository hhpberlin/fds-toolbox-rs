use std::ops::{Add, Div, Mul, Sub};

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ArrayMeta<N, M = N> {
    pub min: N,
    pub max: N,
    pub mean: M,
    pub variance: M,
}

impl<N: Default, M: Default> Default for ArrayMeta<N, M> {
    fn default() -> Self {
        Self {
            min: N::default(),
            max: N::default(),
            mean: M::default(),
            variance: M::default(),
        }
    }
}

impl<N: PartialOrd + Copy, M: Add<Output = M> + Sub<Output = M> + Div<Output = M> + Mul<Output = M> + From<N> + Copy> ArrayMeta<N, M> {
    pub fn new(mut data: impl Iterator<Item = N>, div: fn(M, usize) -> M) -> Option<Self> {
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
        let mut data = data.peekable();
        while let Some(value) = data.next() {
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
        Some(Self {
            min,
            max,
            mean,
            variance,
        })
    }
}