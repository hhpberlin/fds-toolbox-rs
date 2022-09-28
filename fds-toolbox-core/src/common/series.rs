use std::{borrow::Borrow, ops::Index};

use ndarray::{Array1, ArrayView1};
use serde::{Deserialize, Serialize};

use super::arr_meta::ArrayStats;

#[derive(Debug, Serialize, Deserialize)]
pub struct Series {
    data: Array1<f32>,
    stats: ArrayStats<f32>,
}

impl Series {
    pub fn new(data: Array1<f32>) -> Self {
        // TODO: Should we be storing Option directly instead? Does default really make sense here?
        let stats = ArrayStats::new_f32(data.iter().copied()).unwrap_or_default();
        Self { data, stats }
    }

    pub fn from_vec(data: Vec<f32>) -> Self {
        Self::new(Array1::from_vec(data))
    }

    pub fn view(&self) -> SeriesView {
        SeriesView {
            data: self.data.view(),
            stats: self.stats,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = f32> + '_ {
        self.data.iter().copied()
    }
}

impl Index<usize> for Series {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct SeriesView<'a> {
    pub data: ArrayView1<'a, f32>,
    pub stats: ArrayStats<f32>, // TODO: Should we borrow this instead?
}

impl<'a> SeriesView<'a> {
    pub fn new(data: ArrayView1<'a, f32>, stats: ArrayStats<f32>) -> Self {
        Self { data, stats }
    }

    pub fn iter(&self) -> impl Iterator<Item = f32> + '_ {
        self.data.iter().copied()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TimeSeries {
    time_in_seconds: Series,
    values: Series,
    unit: String,
    name: String,
}

impl TimeSeries {
    pub fn new(name: String, unit: String, time_in_seconds: Series, values: Series) -> Self {
        assert_eq!(time_in_seconds.data.len(), values.data.len());
        Self {
            name,
            unit,
            time_in_seconds,
            values,
        }
    }

    pub fn view(&self) -> TimeSeriesView {
        TimeSeriesView {
            name: self.name.borrow(),
            unit: self.unit.borrow(),
            time_in_seconds: self.time_in_seconds.view(),
            values: self.values.view(),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (f32, f32)> + '_ {
        self.time_in_seconds
            .iter()
            .zip(self.values.iter())
            .map(|(t, v)| (t, v))
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct TimeSeriesView<'a> {
    pub time_in_seconds: SeriesView<'a>,
    pub values: SeriesView<'a>,
    pub unit: &'a str,
    pub name: &'a str,
}

impl<'a> TimeSeriesView<'a> {
    pub fn new(
        time_in_seconds: SeriesView<'a>,
        values: SeriesView<'a>,
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

    pub fn iter(&self) -> impl Iterator<Item = (f32, f32)> + '_ {
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

pub trait TimeSeriesViewSource<Id> {
    fn get_time_series(&self, id: Id) -> Option<TimeSeriesView>;

    // fn get_time_series_iter(&self, ids: impl Iterator<Item = Id>) -> impl Iterator<Item = TimeSeriesView> {
    //     ids.filter_map(move |id| self.get_time_series(id))
    // }
}

impl TimeSeriesViewSource<()> for TimeSeries {
    fn get_time_series(&self, _: ()) -> Option<TimeSeriesView> {
        Some(self.view())
    }
}
