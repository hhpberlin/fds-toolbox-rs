use std::{
    hash::Hash,
    sync::{Arc, Weak},
    time::Instant,
};

use color_eyre::Report;
use dashmap::DashMap;
use tokio::sync::broadcast;

use crate::FutureCache;

use super::BoxFut;

pub struct MapCacheFn<'a, K, V>
where
    K: Clone + Send + Sync + Eq + Hash + 'static,
    V: Clone + Send + Sync + 'static,
{
    inner: MapCache<K, V>,
    f: Arc<dyn Fn(K) -> BoxFut<'a, Result<V, Arc<Report>>> + Send + Sync + 'a>,
}

impl<'a, K, V> FutureCache<'a, K, V> for MapCacheFn<'a, K, V>
where
    K: Clone + Send + Sync + Eq + Hash + 'static,
    V: Clone + Send + Sync + 'static,
{
    fn from_future_source<F, E>(source: F) -> Self
    where
        F: Fn(K) -> BoxFut<'a, Result<V, E>> + Send + Sync + 'a,
        E: std::error::Error + Sync + Send + 'static {
        Self {
            inner: MapCache::new(),
            f: Arc::new(move |key| {
                let fut = source(key);
                Box::pin(async move {
                    let res = fut.await.map_err(|e| Arc::new(e.into()));
                    res
                })
            }),
        }
    }

    fn request(&self, key: K) -> crate::PotentialResult<V> {
        self.inner.get_cached(key, self.f.clone()).await
    }
}

#[derive(Clone)]
pub struct MapCache<Key, Value>
where
    Key: Clone + Send + Sync + Eq + Hash + 'static,
    Value: Clone + Send + Sync + 'static,
{
    inner: DashMap<Key, Cached<Value>>,
}

// TODO: Is color_eyre::Report appropriate here? And should it really be Arc?
type EyreResult<Value> = Result<Value, Arc<Report>>;

#[derive(Debug, Clone)]
struct Cached<Value>
where
    Value: Clone + Send + Sync + 'static,
{
    cached_value: Option<(Instant, Value)>,
    inflight: Option<Weak<broadcast::Sender<EyreResult<Value>>>>,
}

impl<T> Default for Cached<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn default() -> Self {
        Self {
            cached_value: None,
            inflight: None,
        }
    }
}

impl<K, V> MapCache<K, V>
where
    K: Clone + Send + Sync + Eq + Hash + 'static,
    V: Clone + Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self {
            inner: Default::default(),
        }
    }

    pub async fn get_cached<F, E>(&self, key: K, f: F) -> Result<V, Arc<Report>>
    where
        F: FnOnce(K) -> BoxFut<'static, Result<V, E>>,
        E: std::error::Error + Sync + Send + 'static,
    {
        let mut rx = {
            // only sync code in this block
            let mut inner = self.inner.entry(key.clone()).or_default();

            if let Some((_fetched_at, value)) = inner.cached_value.as_ref() {
                // TODO
                // if fetched_at.elapsed() < self.refresh_interval {
                return Ok(value.clone());
                // } else {
                // debug!("stale, let's refresh");
                // }
            }

            if let Some(inflight) = inner.inflight.as_ref().and_then(Weak::upgrade) {
                inflight.subscribe()
            } else {
                // there isn't, let's fetch
                let (tx, rx) = broadcast::channel::<Result<V, Arc<Report>>>(1);
                let tx = Arc::new(tx);
                inner.inflight = Some(Arc::downgrade(&tx));
                let inner = self.inner.clone();

                let fut = f(key.clone());

                tokio::spawn(async move {
                    let res = fut.await;

                    {
                        // only sync code in this block
                        let mut inner = inner.get_mut(&key).unwrap();
                        inner.inflight = None;

                        match res {
                            Ok(value) => {
                                inner.cached_value.replace((Instant::now(), value.clone()));
                                let _ = tx.send(Ok(value));
                            }
                            Err(e) => {
                                let _ = tx.send(Err(Arc::new(e.into())));
                            }
                        };
                    }
                });
                rx
            }
        };

        // if we reached here, we're waiting for an in-flight request (we weren't
        // able to serve from cache)
        rx.recv()
            .await
            .map_err(|err| color_eyre::eyre::eyre!("In-Flight request panicked: {:#}", err))?
    }
}

impl<K, V> Default for MapCache<K, V>
where
    K: Clone + Send + Sync + Eq + Hash + 'static,
    V: Clone + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}
