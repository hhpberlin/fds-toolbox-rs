use std::{fmt::Debug, pin::Pin, sync::Arc, sync::Weak};

use futures::Future;
use parking_lot::Mutex;

use tokio::{sync::broadcast, time::Instant};

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
    inner: Arc<Mutex<CachedInner<T>>>,
}

impl<T> Cached<T>
where
    T: Clone + Send + Sync + 'static,
{
    pub fn empty() -> Self {
        Self {
            inner: Arc::new(Mutex::new(CachedInner {
                last_fetched: None,
                inflight: None,
            })),
        }
    }
}

#[derive(Debug)]
struct CachedInner<T>
where
    T: Clone + Send + Sync + 'static,
{
    last_fetched: Option<(Instant, T)>,
    inflight: Option<Weak<broadcast::Sender<Result<T, CachedError>>>>,
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
            let mut inner = self.inner.lock();

            if let Some((_fetched_at, value)) = inner.last_fetched.as_ref() {
                return Ok(value.clone());
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
}
