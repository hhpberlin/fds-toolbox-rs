pub mod map_cache;
// pub(crate) mod sync;

// #[cfg(test)]
// mod store_test;

use std::{
    error,
    future::{Future, IntoFuture},
    pin::Pin,
};

pub type BoxFut<'a, O> = Pin<Box<dyn Future<Output = O> + Send + 'a>>;

pub trait FutureCache<'a, Key, Value> {
    fn from_future_source<F, E>(source: F) -> Self
    where
        F: Fn(Key) -> BoxFut<'a, Result<Value, E>> + Send + Sync + 'a,
        E: std::error::Error + Sync + Send + 'static;

    fn request(&self, key: Key) -> PotentialResult<Value>;
}

pub type PotentialResult<T> = Result<T, Missing>;

pub enum Missing {
    InFlight { progress: f32 },
    Requested,
    RequestError(Box<dyn std::error::Error>),
    InvalidKey,
}
