use std::{
    future::{Future, IntoFuture},
    pin::Pin,
};

use dashmap::{DashMap, mapref::entry::Entry};
use tokio::task::JoinHandle;
use std::hash::Hash;

use super::{FutureCache, PotentialResult};

enum PotentialValue<T> {
    Value(T),
    Future(JoinHandle<T>),
}

pub struct MapCache<Key, Value> {
    source: Box<dyn Fn(Key) -> Pin<Box<dyn Future<Output = Value>>> + Send + Sync>,
    cache: DashMap<Key, PotentialValue<Value>>,
}

impl<Key: Eq + Hash + Clone, Value: Clone> FutureCache<Key, Value> for MapCache<Key, Value> {
    fn from_future_source<Fut: IntoFuture<Output = Value>>(source: impl Fn(Key) -> Fut + Send + Sync) -> Self {
        Self {
            source: Box::new(move |key| Box::pin(source(key).into_future())),
            cache: DashMap::new(),
        }
    }

    fn request(&self, key: Key) -> PotentialResult<Value> {
        self.launch_request(key)
    }
}

impl<Key: Eq + Hash + Clone + Send + Sync, Value: Clone + Send + Sync> MapCache<Key, Value> {
    // fn lock_request(&self, key: &Key) -> &dyn Future<Output = Value> {
    //     let cached = self.cache.get(key);
    // }

    async fn launch_request(&self, key: Key) -> &PotentialValue<Value> {
        let key2 = key.clone();
        let entry = self.cache.entry(key).or_insert_with(move ||
            PotentialValue::Future(tokio::spawn(async move {
                let src = self.source;
                let result = src(key2).await;
                result
            }))
        );

        entry.value()
    }
}
