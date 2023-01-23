use std::{
    hash::Hash,
    sync::{Arc, Weak},
    time::{Duration, Instant},
};

use color_eyre::Report;
use dashmap::DashMap;
use tokio::sync::broadcast;
use tracing::debug;

use super::BoxFut;

#[derive(Clone)]
pub struct MapCache<Key, Value>
where
    Key: Clone + Send + Sync + Eq + Hash + 'static,
    Value: Clone + Send + Sync + 'static,
{
    inner: DashMap<Key, Cached<Value>>,
}

#[derive(Debug, Clone)]
struct Cached<Value>
where
    Value: Clone + Send + Sync + 'static,
{
    cached_value: Option<(Instant, Value)>,
    // TODO: Is color_eyre::Report appropriate here? And should it really be Arc?
    inflight: Option<Weak<broadcast::Sender<Result<Value, Arc<Report>>>>>,
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
    pub fn new(refresh_interval: Duration) -> Self {
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

            if let Some((fetched_at, value)) = inner.cached_value.as_ref() {
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
        Ok(rx
            .recv()
            .await
            .map_err(|err| color_eyre::eyre::eyre!("In-Flight request panicked: {:#}", err))??)
    }
}
