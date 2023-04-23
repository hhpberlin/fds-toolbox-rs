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
pub struct Cached<T>
where
    T: Clone + Send + Sync + 'static,
{
    // TODO: Use RwLock instead?
    inner: Arc<Mutex<CachedInner<T>>>,
    refresh_interval: Option<Duration>,
}

// (Partial)Eq and Hash all use reference equality, not value equality

// impl<T> Eq for Cached<T>
// where
//     T: Clone + Send + Sync + 'static,
//     T: Eq,
// {
// }

// impl<T> PartialEq for Cached<T>
// where
//     T: Clone + Send + Sync + 'static,
//     T: PartialEq,
// {
//     fn eq(&self, other: &Self) -> bool {
//         std::ptr::eq(self, other)
//     }
// }
// impl<T> Hash for Cached<T>
// where
//     T: Clone + Send + Sync + 'static,
//     T: Hash,
// {
//     fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
//         (self as *const _ as usize).hash(state);
//     }
// }

impl<T> Cached<T>
where
    T: Clone + Send + Sync + 'static,
{
    pub fn empty(refresh_interval: Option<Duration>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(CachedInner {
                last_fetched: None,
                inflight: None,
                last_accessed: None,
            })),
            refresh_interval,
        }
    }
}

impl<T: Send + Sync + 'static> Cached<Arc<T>> {
    pub fn empty_enrolled(refresh_interval: Option<Duration>) -> Self {
        let cached = Self::empty(refresh_interval);
        MEMORY_MANAGER.enroll(cached.inner);
        cached
    }
}

#[derive(Debug)]
struct CachedInner<T>
where
    T: Clone + Send + Sync + 'static,
{
    last_fetched: Option<(Instant, T)>,
    last_accessed: Option<Instant>,
    inflight: Option<Weak<broadcast::Sender<Result<T, CachedError>>>>,
}

// pub trait Data {
//     fn addr(&self) -> usize;
//     fn get_size(&self) -> usize;
//     fn get_last_accessed(&self) -> Option<Instant>;
//     fn free(&self);
// }

// impl<T: Data> Data for CachedInner<T>
// where
//     T: Clone + Send + Sync + 'static,
// {
//     fn addr(&self) -> usize {
//         self as *const _ as usize
//     }

//     fn get_size(&self) -> usize {
//         self.last_fetched
//             .as_ref()
//             .map(|(_, v)| v.get_size())
//             .unwrap_or(0)
//     }

//     fn get_last_accessed(&self) -> Option<Instant> {
//         self.last_accessed
//     }

//     fn free(&self) {
//         self.las
//     }
// }

// impl<T> Eq for CachedInner<T>
// where
//     T: Clone + Send + Sync + 'static,
//     T: Eq,
// {
// }

// impl<T> PartialEq for CachedInner<T>
// where
//     T: Clone + Send + Sync + 'static,
//     T: PartialEq,
// {
//     fn eq(&self, other: &Self) -> bool {
//         self as *const _ == other as *const _
//     }
// }

// // impl<T> PartialEq for CachedInner<T>
// // where
// //     T: Clone + Send + Sync + 'static,
// //     T: PartialEq,
// // {
// //     fn eq(&self, other: &Self) -> bool {
// //         self.last_fetched == other.last_fetched && self.last_accessed == other.last_accessed
// //     }
// // }

// impl<T> Hash for CachedInner<T>
// where
//     T: Clone + Send + Sync + 'static,
//     T: Hash,
// {
//     fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
//         (self as *const _ as usize).hash(state);
//     }
// }

// // impl<T> Hash for CachedInner<T>
// // where
// //     T: Clone + Send + Sync + 'static,
// //     T: Hash,
// // {
// //     fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
// //         self.last_fetched.hash(state);
// //         self.last_accessed.hash(state);
// //     }
// // }

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
            let mut inner = self.inner.lock();
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
                let inner = self.inner.clone();

                let fut = f();

                tokio::spawn(async move {
                    let res = fut.await;

                    {
                        // only sync code in this block
                        let mut inner = inner.lock();
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

    pub fn try_get_sync(&self) -> Option<Result<T, CachedError>> {
        let mut inner = self.inner.lock();
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
        let inner = self.inner.lock();
        inner.last_accessed
    }

    pub async fn try_get(&self) -> Option<Result<T, CachedError>> {
        let mut rx = {
            let mut inner = self.inner.lock();
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
        let mut inner = self.inner.lock();
        inner.last_fetched.take().map(|(_, v)| v)
    }
}
