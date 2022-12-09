use std::{borrow::Borrow, ops::Index};

use ndarray::{Array, ArrayView, Dimension, Ix1, Ix2};
use serde::{Deserialize, Serialize};

use super::arr_meta::ArrayStats;

#[derive(Debug, Serialize, Deserialize)]
pub struct Series<T, Ix: Dimension> {
    data: Array<T, Ix>,
    pub stats: ArrayStats<T>,
}

pub type Series1<T = f32> = Series<T, Ix1>;
pub type Series2<T = f32> = Series<T, Ix2>;

impl<T: Copy, Ix: Dimension> Series<T, Ix> {
    pub fn new(data: Array<T, Ix>, stats: ArrayStats<T>) -> Self {
        Self { data, stats }
    }

    // pub fn from_vec(data: Vec<T>, stats: ArrayStats<T>) -> Self {
    //     Self::new(Array::from_vec(data), stats)
    // }

    pub fn view(&self) -> SeriesView<T, Ix> {
        SeriesView {
            data: self.data.view(),
            stats: self.stats,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = T> + '_ {
        self.data.iter().copied()
    }
}

impl Series1 {
    pub fn from_vec(data: Vec<f32>) -> Self {
        Self::new_f32(Array::from_vec(data))
    }

    pub fn new_f32(data: Array<f32, Ix1>) -> Self {
        // TODO: Should we be storing Option directly instead? Does default really make sense here?
        let stats = ArrayStats::new_f32(data.iter().copied()).unwrap_or_default();
        Self { data, stats }
    }
}

impl<T, Ix: Dimension> Index<Ix> for Series<T, Ix> {
    type Output = T;

    fn index(&self, index: Ix) -> &Self::Output {
        &self.data[index]
    }
}

impl<T> Index<usize> for Series1<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct SeriesView<'a, T: Copy, Ix: Dimension> {
    pub data: ArrayView<'a, T, Ix>,
    pub stats: ArrayStats<T>, // TODO: Should we borrow this instead?
}

pub type Series1View<'a, T = f32> = SeriesView<'a, T, Ix1>;

impl<'a, T: Copy, Ix: Dimension> SeriesView<'a, T, Ix> {
    pub fn new(data: ArrayView<'a, T, Ix>, stats: ArrayStats<T>) -> Self {
        Self { data, stats }
    }

    pub fn iter(&self) -> impl Iterator<Item = T> + '_ {
        self.data.iter().copied()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TimeSeries<Value: Copy, Ix: Dimension, Time: Copy = f32> {
    time_in_seconds: Series1<Time>,
    values: Series<Value, Ix>,
    unit: String,
    name: String,
}

impl<Value: Copy, Ix: Dimension, Time: Copy> TimeSeries<Value, Ix, Time> {
    pub fn new(
        name: String,
        unit: String,
        time_in_seconds: Series1<Time>,
        values: Series<Value, Ix>,
    ) -> Self {
        assert_eq!(time_in_seconds.data.len(), values.data.len());
        Self {
            name,
            unit,
            time_in_seconds,
            values,
        }
    }

    pub fn view(&self) -> TimeSeriesView<Value, Ix, Time> {
        TimeSeriesView {
            name: self.name.borrow(),
            unit: self.unit.borrow(),
            time_in_seconds: self.time_in_seconds.view(),
            values: self.values.view(),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (Time, Value)> + '_ {
        self.time_in_seconds
            .iter()
            .zip(self.values.iter())
            .map(|(t, v)| (t, v))
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct TimeSeriesView<'a, Value: Copy, Ix: Dimension, Time: Copy = f32> {
    pub time_in_seconds: Series1View<'a, Time>,
    pub values: SeriesView<'a, Value, Ix>,
    pub unit: &'a str,
    pub name: &'a str,
}

pub type TimeSeries1View<'a, Value = f32, Time = f32> = TimeSeriesView<'a, Value, Ix1, Time>;

impl<'a, Value: Copy, Ix: Dimension, Time: Copy> TimeSeriesView<'a, Value, Ix, Time> {
    pub fn new(
        time_in_seconds: Series1View<'a, Time>,
        values: SeriesView<'a, Value, Ix>,
        unit: &'a str,
        name: &'a str,
    ) -> Self {
        assert_eq!(time_in_seconds.data.len(), values.data.len());
        Self {
            time_in_seconds,
            values,
            unit,
            name,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (Time, Value)> + '_ {
        self.time_in_seconds
            .iter()
            .zip(self.values.iter())
            .map(|(t, v)| (t, v))
    }
}

// impl<'a> IntoIterator<Item = (f32, f32)> for TimeSeriesView<'a> {
//     type IntoIter = impl Iterator<Item = (f32, f32)>;

//     fn into_iter(self) -> Self::IntoIter {
//         self.iter()
//     }
// }

pub trait TimeSeriesViewSource<Id, Value: Copy = f32, Ix: Dimension = Ix1, Time: Copy = f32> {
    fn get_time_series(&self, id: Id) -> Option<TimeSeriesView<Value, Ix, Time>>;

    // fn get_time_series_iter(&self, ids: impl Iterator<Item = Id>) -> impl Iterator<Item = TimeSeriesView> {
    //     ids.filter_map(move |id| self.get_time_series(id))
    // }
}

// pub type TimeSeries1ViewSource<Id, Value = f32, Time = f32> = TimeSeriesViewSource<Id, Value, Ix1, Time>;

impl<Id, T: TimeSeriesViewSource<Id, Value, Ix, Time>, Value: Copy, Ix: Dimension, Time: Copy>
    TimeSeriesViewSource<Id, Value, Ix, Time> for &T
{
    fn get_time_series(&self, id: Id) -> Option<TimeSeriesView<Value, Ix, Time>> {
        (*self).get_time_series(id)
    }
}

impl<Value: Copy, Ix: Dimension, Time: Copy> TimeSeriesViewSource<(), Value, Ix, Time>
    for TimeSeries<Value, Ix, Time>
{
    fn get_time_series(&self, _: ()) -> Option<TimeSeriesView<Value, Ix, Time>> {
        Some(self.view())
    }
}
