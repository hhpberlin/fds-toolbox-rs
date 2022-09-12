use std::ops::{Add, Div, Mul, Sub};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Range<N> {
    pub min: N,
    pub max: N,
}

impl<N> Range<N> {
    pub fn new(min: N, max: N) -> Self {
        Self { min, max }
    }
}

impl<N: Default> Default for Range<N> {
    fn default() -> Self {
        Self {
            min: N::default(),
            max: N::default(),
        }
    }
}

impl<N: Sub<Output = N> + Div<Output = N> + Copy> Range<N> {
    pub fn width(&self) -> <N as Sub>::Output {
        self.max - self.min
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
        let width = self.width();
        let new_width = width / factor;
        let offset = (width - new_width) * self.map(center);
        self.min = self.min + offset;
        self.max = self.max - offset;
        if self.min > self.max {
            (self.min, self.max) = (self.max, self.min);
        }
    }
}

impl<N> Range<N> {
    pub fn into_range(self) -> std::ops::Range<N> {
        self.min..self.max
    }

    pub fn into_range_inclusive(self) -> std::ops::RangeInclusive<N> {
        self.min..=self.max
    }
}

impl<N: PartialOrd + Copy> Range<N> {
    pub fn expand(&self, new: N) -> Self {
        Self::new(
            if self.min < new { self.min } else { new },
            if self.max > new { self.max } else { new },
        )
    }

    pub fn max(&self, new: Range<N>) -> Self {
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

    pub fn from_iter_val(iter: impl IntoIterator<Item = N>) -> Option<Range<N>> {
        iter.into_iter().fold(None, |acc, n| match acc {
            Some(acc) => Some(acc.expand(n)),
            None => Some(Range::new(n, n)),
        })
    }

    pub fn from_iter_range(iter: impl IntoIterator<Item = Range<N>>) -> Option<Range<N>> {
        iter.into_iter().fold(None, |acc, range| match acc {
            Some(acc) => Some(acc.max(range)),
            None => Some(range),
        })
    }
}

// trait RangeExt<N>: Iterator {
//     fn expand(&self, new: N) -> Range<N>
//         where Self::Item == N;
//     {

//     }
//     fn max(&self, new: Self) -> Self;
// }
// impl<N: PartialOrd + Copy, I: Iterator<Item = Range<N>>> I {}
// }

impl<N> From<Range<N>> for std::ops::Range<N> {
    fn from(range: Range<N>) -> Self {
        range.into_range()
    }
}
