use std::{fmt::Debug, hash::Hash, pin::Pin, sync::Arc, sync::Weak, time::Duration};

use futures::Future;
use parking_lot::Mutex;

use tokio::{sync::broadcast, time::Instant};
use tracing::debug;

use crate::memman::MEMORY_MANAGER;

pub type BoxFut<'a, O> = Pin<Box<dyn Future<Output = O> + Send + 'a>>;

// The following code is stolen from fasterthanlime

#[derive(Debug, Clone, thiserror::Error)]
#[error("stringified error: {inner}")]
pub struct CachedError {
    inner: String,
}

impl CachedError {
    pub fn new<E: std::fmt::Display>(e: E) -> Self {
        Self {
            inner: e.to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Cached<T>(pub(crate) Arc<CachedInner<T>>)
where
    T: Clone + Send + Sync + 'static;

impl<T> Cached<T>
where
    T: Clone + Send + Sync + 'static,
{
    pub fn empty(refresh_interval: Option<Duration>) -> Self {
        Self(Arc::new(CachedInner {
            mutex: Mutex::new(CachedValue {
                last_fetched: None,
                inflight: None,
                last_accessed: None,
            }),
            refresh_interval,
        }))
    }
}

impl<T: get_size::GetSize + Send + Sync + 'static> Cached<Arc<T>> {
    pub fn empty_enrolled(refresh_interval: Option<Duration>) -> Self {
        let cached = Self::empty(refresh_interval);
        cached.enroll();
        cached
    }
}

#[derive(Debug)]
pub struct CachedInner<T>
where
    T: Clone + Send + Sync + 'static,
{
    mutex: Mutex<CachedValue<T>>,
    refresh_interval: Option<Duration>,
}

#[derive(Debug)]
struct CachedValue<T>
where
    T: Clone + Send + Sync + 'static,
{
    last_fetched: Option<(Instant, T)>,
    last_accessed: Option<Instant>,
    inflight: Option<Weak<broadcast::Sender<Result<T, CachedError>>>>,
}

// impl<T> Cached<T> where
// T: Clone + Send + Sync + 'static,
// {
//     pub async fn get_cached<F, E>(&self, f: F) -> Result<T, CachedError>
//     where
//         F: FnOnce() -> BoxFut<'static, Result<T, E>>,
//         E: std::fmt::Display + 'static,
//     {
//         self.0.get_cached(f).await
//     }

//     pub fn try_get_sync(&self) -> Option<Result<T, CachedError>> {
//         self.0.try_get_sync()
//     }

//     pub fn get_last_accessed(&self) -> Option<Instant> {
//         self.0.get_last_accessed()
//     }
// }

impl<T> std::ops::Deref for Cached<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Target = CachedInner<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> Cached<T>
where
    T: Clone + Send + Sync + 'static,
{
    pub async fn get_cached<F, E>(&self, f: F) -> Result<T, CachedError>
    where
        F: FnOnce() -> BoxFut<'static, Result<T, E>>,
        E: std::fmt::Display + 'static,
    {
        let mut rx = {
            let mut inner = self.mutex.lock();
            inner.last_accessed = Some(Instant::now());

            if let Some((fetched_at, value)) = inner.last_fetched.as_ref() {
                let Some(refresh_interval) = self.refresh_interval else {
                    return Ok(value.clone());
                };

                let elapsed = fetched_at.elapsed();

                if elapsed < refresh_interval {
                    return Ok(value.clone());
                } else {
                    debug!(elapsed = ?elapsed, refresh_interval = ?refresh_interval, "Cache is stale, let's refresh");
                }
            }

            if let Some(inflight) = inner.inflight.as_ref().and_then(Weak::upgrade) {
                inflight.subscribe()
            } else {
                let (tx, rx) = broadcast::channel::<Result<T, CachedError>>(1);
                let tx = Arc::new(tx);
                inner.inflight = Some(Arc::downgrade(&tx));
                let inner = self.clone();

                let fut = f();

                tokio::spawn(async move {
                    let res = fut.await;

                    {
                        // only sync code in this block
                        let mut inner = inner.mutex.lock();
                        inner.inflight = None;

                        match res {
                            Ok(value) => {
                                inner.last_fetched.replace((Instant::now(), value.clone()));
                                let _ = tx.send(Ok(value));
                            }
                            Err(e) => {
                                let _ = tx.send(Err(CachedError {
                                    inner: e.to_string(),
                                }));
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
            .map_err(|_| CachedError::new("in-flight request died"))?
    }
}



impl<T> CachedInner<T>
where
    T: Clone + Send + Sync + 'static,
{
    pub fn try_get_sync(&self) -> Option<Result<T, CachedError>> {
        let mut inner = self.mutex.lock();
        inner.last_accessed = Some(Instant::now());

        let Some((fetched_at, value)) = inner.last_fetched.as_ref() else {
            return None;
        };

        let Some(refresh_interval) = self.refresh_interval else {
                return Some(Ok(value.clone()));
            };

        let elapsed = fetched_at.elapsed();

        if elapsed < refresh_interval {
            Some(Ok(value.clone()))
        } else {
            debug!(elapsed = ?elapsed, refresh_interval = ?refresh_interval, "Cache is stale, ignoring value");
            None
        }
    }

    pub fn get_last_accessed(&self) -> Option<Instant> {
        let inner = self.mutex.lock();
        inner.last_accessed
    }

    pub async fn try_get(&self) -> Option<Result<T, CachedError>> {
        let mut rx = {
            let mut inner = self.mutex.lock();
            inner.last_accessed = Some(Instant::now());

            if let Some((fetched_at, value)) = inner.last_fetched.as_ref() {
                let Some(refresh_interval) = self.refresh_interval else {
                    return Some(Ok(value.clone()));
                };

                let elapsed = fetched_at.elapsed();

                if elapsed < refresh_interval {
                    return Some(Ok(value.clone()));
                } else {
                    debug!(elapsed = ?elapsed, refresh_interval = ?refresh_interval, "Cache is stale, ignoring value");
                }
            }

            if let Some(inflight) = inner.inflight.as_ref().and_then(Weak::upgrade) {
                inflight.subscribe()
            } else {
                return None;
            }
        };

        Some(
            rx.recv()
                .await
                .map_err(|_| CachedError::new("in-flight request died"))
                .and_then(|x| x),
        )
    }

    pub fn clear(&self) -> Option<T> {
        let mut inner = self.mutex.lock();
        inner.last_fetched.take().map(|(_, v)| v)
    }
}
