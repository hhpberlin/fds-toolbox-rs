use serde::{Deserialize, Serialize};

use super::{arr_meta::ArrayStats, range::RangeIncl};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ArrayStats2D<Time, Num, NumDivisible = Num> {
    pub time_range: RangeIncl<Time>,
    pub stats: ArrayStats<Num, NumDivisible>,
}

impl<T: Default, N: Default, M: Default> Default for ArrayStats2D<T, N, M> {
    fn default() -> Self {
        Self {
            time_range: RangeIncl::default(),
            stats: ArrayStats::default(),
        }
    }
}

impl<T, N, M> ArrayStats2D<T, N, M> {
    fn new(time_range: RangeIncl<T>, stats: ArrayStats<N, M>) -> Self {
        Self { time_range, stats }
    }
}

// trait FnExt<Iter: Iterator>: Fn() -> Iter {}
// impl<Iter: Iterator, F: Fn() -> Iter> FnExt<Iter> for F {}

// impl<Iter: Iterator, F: FnExt<Iter>> IntoIterator for &F {
//     fn into_iter(self) -> Self {
//         self()
//     }
// }

// impl<T: PartialOrd + Copy, N, M> FromIterator<(T, N)> for ArrayStats2D<T, N, M> {
//     fn from_iter<I: IntoIterator<Item = (T, N)>>(iter: I) -> Self {
//         let mut time_range = None;
//         let iter = iter.into_iter().map(|(time, value)| {
//             match time_range {
//                 None => {
//                     time_range = Some(Range::new(time, time));
//                 }
//                 Some(ref mut time_range) => {
//                     time_range.expand(time);
//                 }
//             }
//             value
//         });
//         let stats = ArrayStats::from_iter(iter);

//     }
// }
