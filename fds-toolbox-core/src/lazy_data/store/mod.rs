mod map_cache;
mod cached_error;

// #[cfg(test)]
// mod store_test;

use std::{future::{IntoFuture, Future}, pin::Pin};

pub type BoxFut<'a, O> = Pin<Box<dyn Future<Output = O> + Send + 'a>>;

pub trait FutureCache<Key, Value> {
    fn from_future_source<Fut: IntoFuture<Output = Value>>(source: impl Fn(Key) -> Fut + Send + Sync) -> Self;
    fn request(&self, key: Key) -> PotentialResult<Value>;
}

pub type PotentialResult<T> = Result<T, Missing>;

pub enum Missing {
    InFlight { progress: f32 },
    Requested,
    RequestError(Box<dyn std::error::Error>),
    InvalidKey,
}

