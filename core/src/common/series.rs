use std::{borrow::Borrow, ops::Index};

use get_size::GetSize;
use ndarray::{Array, ArrayView, Axis, Dimension, Ix1, Ix2, Ix3, Ix4, RemoveAxis};
use serde::{Deserialize, Serialize};

use super::arr_meta::ArrayStats;

// TODO: Manually implement (Partial)Eq to assure stats are checked first to avoid reading the entire array if possible
#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct Series<T, Ix: Dimension> {
    data: Array<T, Ix>,
    pub stats: ArrayStats<T>,
}

impl<T, Ix: Dimension> GetSize for Series<T, Ix> {
    fn get_size(&self) -> usize {
        // TODO: This does not account for a little bit of overhead from the `Array` struct itself
        self.data.len() * std::mem::size_of::<T>() + std::mem::size_of::<ArrayStats<T>>()
    }
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
        SeriesView::new(self.data.view(), self.stats)
    }

    pub fn iter(&self) -> impl Iterator<Item = T> + '_ {
        self.data.iter().copied()
    }
}

impl Series1 {
    pub fn from_vec(data: Vec<f32>) -> Self {
        Array::from_vec(data).into()
    }
}

impl<Ix: Dimension> From<Array<f32, Ix>> for Series<f32, Ix> {
    fn from(data: Array<f32, Ix>) -> Self {
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

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub struct SeriesView<'a, T: Copy, Ix: Dimension, Ref: 'a = ()> {
    pub data: ArrayView<'a, T, Ix>,
    // TODO: Should we borrow this instead?
    // TODO: Should this recompute for the subset?
    pub stats: ArrayStats<T>,
    base_ref: Option<Ref>,
}

pub type Series1View<'a, T = f32> = SeriesView<'a, T, Ix1>;
pub type Series2View<'a, T = f32> = SeriesView<'a, T, Ix2>;

impl<'a, T: Copy, Ix: Dimension, Ref> SeriesView<'a, T, Ix, Ref> {
    pub fn new(data: ArrayView<'a, T, Ix>, stats: ArrayStats<T>) -> Self {
        Self {
            data,
            stats,
            base_ref: None,
        }
    }

    pub fn new_with_ref(data: ArrayView<'a, T, Ix>, stats: ArrayStats<T>, base_ref: Ref) -> Self {
        Self {
            data,
            stats,
            base_ref: Some(base_ref),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = T> + '_ {
        self.data.iter().copied()
    }

    pub fn map<IxOut: Dimension>(
        &self,
        f: impl FnOnce(&Self) -> ArrayView<'a, T, IxOut>,
    ) -> SeriesView<'a, T, IxOut> {
        SeriesView::new(f(self), self.stats)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TimeSeries<Value: Copy, Ix: Dimension, Time: Copy = f32> {
    pub time_in_seconds: Series1<Time>,
    /// Axis 0 is time
    pub values: Series<Value, Ix>,
    unit: String,
    name: String,
}

// Can't use derive here because it doesn't understand that `Ix` does not need to impl `GetSize`
impl<Value: Copy, Ix: Dimension, Time: Copy> GetSize for TimeSeries<Value, Ix, Time> {
    fn get_heap_size(&self) -> usize {
        self.time_in_seconds.get_heap_size()
            + self.values.get_heap_size()
            + self.unit.get_heap_size()
            + self.name.get_heap_size()
    }
}

pub type TimeSeries0<Value = f32, Time = f32> = TimeSeries<Value, Ix1, Time>;
pub type TimeSeries2<Value = f32, Time = f32> = TimeSeries<Value, Ix3, Time>;
pub type TimeSeries3<Value = f32, Time = f32> = TimeSeries<Value, Ix4, Time>;

impl<Value: Copy, Ix: Dimension, Time: Copy> TimeSeries<Value, Ix, Time> {
    pub fn new(
        name: String,
        unit: String,
        time_in_seconds: Series1<Time>,
        values: Series<Value, Ix>,
    ) -> Self {
        assert_eq!(time_in_seconds.data.len(), values.data.len_of(Axis(0)));
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
            // .map(|(t, v)| (t, v))
    }

    pub fn len(&self) -> usize {
        self.time_in_seconds.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct TimeSeriesView<'a, Value: Copy, Ix: Dimension, Time: Copy = f32> {
    pub time_in_seconds: Series1View<'a, Time>,
    /// Axis 0 is time
    pub values: SeriesView<'a, Value, Ix>,
    pub unit: &'a str,
    pub name: &'a str,
}

pub type TimeSeries0View<'a, Value = f32, Time = f32> = TimeSeriesView<'a, Value, Ix1, Time>;
pub type TimeSeries2View<'a, Value = f32, Time = f32> = TimeSeriesView<'a, Value, Ix3, Time>;
pub type TimeSeries3View<'a, Value = f32, Time = f32> = TimeSeriesView<'a, Value, Ix4, Time>;

impl<'a, Value: Copy, Time: Copy> TimeSeriesView<'a, Value, Ix1, Time> {
    pub fn iter(&self) -> impl Iterator<Item = (Time, Value)> + '_ {
        self.time_in_seconds
            .iter()
            .zip(self.values.iter())
            // .map(|(t, v)| (t, v))
    }

    pub fn iter_windows<E>(
        &self,
        window_size: usize,
    ) -> impl Iterator<Item = (ArrayView<Time, Ix1>, ArrayView<Value, Ix1>)> + '_ {
        self.time_in_seconds
            .data
            .windows(window_size)
            .into_iter()
            .zip(self.values.data.windows(window_size))
            // .map(|(t, v)| (t, v))
    }
}

impl<'a, Value: Copy, Ix: Dimension, Time: Copy> TimeSeriesView<'a, Value, Ix, Time> {
    pub fn new(
        time_in_seconds: Series1View<'a, Time>,
        values: SeriesView<'a, Value, Ix>,
        unit: &'a str,
        name: &'a str,
    ) -> Self {
        assert_eq!(time_in_seconds.data.len(), values.data.len_of(Axis(0)));
        Self {
            time_in_seconds,
            values,
            unit,
            name,
        }
    }

    pub fn view_frame(
        &'a self,
        frame_num: usize,
    ) -> Option<TimeSeriesFrame<'a, Value, Ix::Smaller, Time>>
    where
        Ix: RemoveAxis,
    {
        let len = self.values.data.len_of(Axis(0));
        if frame_num >= len {
            None
        } else {
            let frame = SeriesView::new(
                self.values.data.index_axis(Axis(0), frame_num),
                self.values.stats,
            );
            Some(TimeSeriesFrame::new(
                self.time_in_seconds.data[frame_num],
                frame,
                self.unit,
                self.name,
            ))
        }
    }

    pub fn frame_panic(&'a self, index: usize) -> TimeSeriesFrame<'a, Value, Ix::Smaller, Time>
    where
        Self: 'a,
        Ix: RemoveAxis,
    {
        self.view_frame(index).expect("Indexed out of bounds")
    }
}

// impl<'a, Value: Copy, Ix: Dimension + RemoveAxis, Time: Copy> Index<usize>
//     for TimeSeriesView<'a, Value, Ix, Time>
// {
//     type Output = TimeSeriesFrame<'a, Value, Ix::Smaller, Time>;

//     fn index(&self, index: usize) -> &Self::Output {
//         &self.frame(index).expect("Indexed out of bounds")
//     }
// }

#[derive(Debug, Clone, Copy, Serialize)]
pub struct TimeSeriesFrame<'a, Value: Copy, Ix: Dimension, Time: Copy = f32> {
    pub time_in_seconds: Time,
    pub values: SeriesView<'a, Value, Ix>,
    pub unit: &'a str,
    pub name: &'a str,
}

impl<'a, Value: Copy, Ix: Dimension, Time: Copy> TimeSeriesFrame<'a, Value, Ix, Time> {
    pub fn new(
        time_in_seconds: Time,
        values: SeriesView<'a, Value, Ix>,
        unit: &'a str,
        name: &'a str,
    ) -> Self {
        Self {
            time_in_seconds,
            values,
            unit,
            name,
        }
    }
}

// pub type TimeSeries0Frame<'a, Value = f32, Time = f32> = TimeSeriesFrame<'a, Value, Ix0, Time>;
// pub type TimeSeries1Frame<'a, Value = f32, Time = f32> = TimeSeriesFrame<'a, Value, Ix1, Time>;
pub type TimeSeries2Frame<'a, Value = f32, Time = f32> = TimeSeriesFrame<'a, Value, Ix2, Time>;

// impl<'a, Value: Copy, Time: Copy> TimeSeriesView1<'a, Value, Time> {
//     fn iter(&self) -> Self::IntoIter {
//         self.into_iter()
//     }
// }

// impl<'a, Value: Copy, Time: Copy> IntoIterator for &TimeSeriesView1<'a, Value, Time> {
//     type Item = (Time, Value);
//     type IntoIter = impl Iterator<Item = (Time, Value)>;

//     fn into_iter(self) -> Self::IntoIter {
//         self.time_in_seconds
//             .iter()
//             .zip(self.values.iter())
//             .map(|(t, v)| (t, v))
//     }
// }

// impl<'a, Value: Copy, Time: Copy> IntoIterator for &TimeSeriesView2<'a, Value, Time> {
//     type Item = (Time, Value);
//     type IntoIter = impl Iterator<Item = (Time, Value)>;

//     fn into_iter(self) -> Self::IntoIter {
//         self.iter()
//     }
// }

// impl<'a, Value: Copy, Time: Copy> TimeSeriesView2<'a, Value, Time> {
//     fn (self) -> Self::IntoIter {
//         self.iter()
//     }
// }

// TODO: Name this better
pub type PotentialResult<T> = Result<T, Missing>;

pub enum Missing {
    InFlight { progress: f32 },
    Requested,
    RequestError(Box<dyn std::error::Error>),
    InvalidKey,
}

pub trait TimeSeriesViewSource<Id, Value: Copy = f32, Ix: Dimension = Ix1, Time: Copy = f32> {
    // fn get_time_series(&self, id: Id) -> Option<TimeSeriesView<Value, Ix, Time>>;

    fn get_time_series(&self, id: Id) -> PotentialResult<TimeSeriesView<Value, Ix, Time>>;

    // fn get_time_series_iter(&self, ids: impl Iterator<Item = Id>) -> impl Iterator<Item = TimeSeriesView> {
    //     ids.filter_map(move |id| self.get_time_series(id))
    // }
}

pub trait TimeSeriesSourceAsync<Id, Value: Copy = f32, Ix: Dimension = Ix1, Time: Copy = f32> {
    type Error: std::error::Error;
    async fn get_time_series(&self, id: Id) -> Result<TimeSeries<Value, Ix, Time>, Self::Error>;
}

// pub type TimeSeries1ViewSource<Id, Value = f32, Time = f32> = TimeSeriesViewSource<Id, Value, Ix1, Time>;

impl<Id, T: TimeSeriesViewSource<Id, Value, Ix, Time>, Value: Copy, Ix: Dimension, Time: Copy>
    TimeSeriesViewSource<Id, Value, Ix, Time> for &T
{
    fn get_time_series(&self, id: Id) -> PotentialResult<TimeSeriesView<Value, Ix, Time>> {
        (*self).get_time_series(id)
    }
}

impl<Value: Copy, Ix: Dimension, Time: Copy> TimeSeriesViewSource<(), Value, Ix, Time>
    for TimeSeries<Value, Ix, Time>
{
    fn get_time_series(&self, _: ()) -> PotentialResult<TimeSeriesView<Value, Ix, Time>> {
        Ok(self.view())
    }
}
