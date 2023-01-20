use std::{future::{IntoFuture, Future}, hash::Hash};

use dashmap::DashMap;


// #[async_trait]
// pub trait KVCache<Key, Value> {
//     async fn get(&self, key: &Key) -> Result<Value>;
// }

pub trait FutureCache<Key, Value> {
    fn from_future_source<Fut: IntoFuture<Output = Value>>(source: impl Fn(Key) -> Fut) -> Self;
    fn request(&self, key: Key) -> PotentialResult<Value>;
}

pub type PotentialResult<T> = Result<T, Missing>;

pub enum Missing {
    InFlight { progress: f32 },
    Requested,
    RequestError(Box<dyn std::error::Error>),
    InvalidKey,
}

pub struct MapCache<Key, Value> {
    source: Box<dyn Fn(Key) -> Box<dyn Future<Output = Value>>>,
    cache: DashMap<Key, PotentialResult<Value>>,
}

impl<Key: Eq + Hash + Clone, Value: Clone> FutureCache<Key, Value> for MapCache<Key, Value> {
    fn from_future_source<Fut: IntoFuture<Output = Value>>(source: impl Fn(Key) -> Fut) -> Self {
        Self {
            source: Box::new(move |key| Box::new(source(key).into_future())),
            cache: DashMap::new(),
        }
    }

    fn request(&self, key: Key) -> PotentialResult<Value> {
        self.launch_request(key)
    }
}

impl<Key, Value> MapCache<Key, Value> {
    fn lock_request(&self, key: &Key) -> &dyn Future<Output = Value> {
        let cached = self.cache.get(key);
    }

    fn launch_request(&self, key: Key) {
        let source = self.source;
        let cache = self.cache.clone();
        tokio::spawn(async move {
            let result = source(key).await;
            cache.insert(key, PotentialResult::Result(result));
        });
    }
}