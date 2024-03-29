use std::{fmt::Debug, pin::Pin, sync::Arc, sync::Weak, time::Duration};

use futures::Future;
use get_size::GetSize;
use lazy_static::__Deref;
use parking_lot::Mutex;

use tokio::{sync::broadcast, time::Instant};
use tracing::{debug, error};

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

impl<T> GetSize for Cached<T>
where
    T: GetSize + Clone + Send + Sync + 'static,
{
    fn get_size(&self) -> usize {
        self.deref().get_size()
    }
}

impl<T> Cached<T>
where
    T: Clone + Send + Sync + 'static,
{
    pub fn new(value: T, refresh_interval: Option<Duration>) -> Self {
        Self(Arc::new(CachedInner {
            mutex: Mutex::new(CachedValue {
                last_fetched: Some((Instant::now(), value)),
                inflight: None,
                last_accessed: None,
            }),
            refresh_interval,
        }))
    }

    pub fn from_fut<E>(
        fut: BoxFut<'static, Result<T, E>>,
        refresh_interval: Option<Duration>,
    ) -> Self
    where
        E: std::fmt::Display + Debug + 'static,
    {
        let cached = Self::empty(refresh_interval);
        cached.attach_future(fut);
        cached
    }

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
    pub fn from_val_enrolled(value: Arc<T>, refresh_interval: Option<Duration>) -> Self {
        let cached = Self::new(value, refresh_interval);
        cached.enroll();
        cached
    }

    pub fn from_fut_enrolled<E>(
        fut: BoxFut<'static, Result<Arc<T>, E>>,
        refresh_interval: Option<Duration>,
    ) -> Self
    where
        E: std::fmt::Display + Debug + 'static,
    {
        let cached = Self::from_fut(fut, refresh_interval);
        cached.enroll();
        cached
    }

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

pub enum CacheResult<T> {
    Cached(T),
    InFlight(broadcast::Receiver<Result<T, CachedError>>),
}

impl<T> CacheResult<T>
where
    T: Clone + Send + Sync + 'static,
{
    pub fn into_val(self) -> Option<T> {
        match self {
            CacheResult::Cached(v) => Some(v),
            CacheResult::InFlight(_) => None,
        }
    }

    pub fn try_get(&self) -> Option<Result<T, CachedError>> {
        match self {
            CacheResult::Cached(v) => Some(Ok(v.clone())),
            CacheResult::InFlight(_) => None,
        }
    }

    pub async fn into_fut(self) -> Result<T, CachedError> {
        match self {
            CacheResult::Cached(v) => Ok(v),
            CacheResult::InFlight(mut rx) => rx
                .recv()
                .await
                .map_err(|_| CachedError::new("in-flight request died"))
                .and_then(|r| r),
        }
    }

    pub async fn get(&mut self) -> Result<T, CachedError> {
        match self {
            CacheResult::Cached(v) => Ok(v.clone()),
            CacheResult::InFlight(rx) => rx
                .recv()
                .await
                .map_err(|_| CachedError::new("in-flight request died"))
                .and_then(|r| r),
        }
    }
}

impl<T> Cached<T>
where
    T: Clone + Send + Sync + 'static,
{
    pub fn get_with<F, E>(&self, f: F) -> CacheResult<T>
    where
        F: FnOnce() -> BoxFut<'static, Result<T, E>>,
        E: std::fmt::Display + Debug + 'static,
    {
        let mut inner = self.mutex.lock();

        self.get_core(&mut inner)
            .unwrap_or_else(|| CacheResult::InFlight(self.attach_future_inner(&mut inner, f())))
    }

    pub fn attach_future<E>(
        &self,
        fut: BoxFut<'static, Result<T, E>>,
    ) -> broadcast::Receiver<Result<T, CachedError>>
    where
        E: std::fmt::Display + Debug + 'static,
    {
        self.attach_future_inner(&mut self.mutex.lock(), fut)
    }

    fn attach_future_inner<E>(
        &self,
        inner: &mut CachedValue<T>,
        fut: BoxFut<'static, Result<T, E>>,
    ) -> broadcast::Receiver<Result<T, CachedError>>
    where
        E: std::fmt::Display + Debug + 'static,
    {
        let (tx, rx) = broadcast::channel::<Result<T, CachedError>>(1);
        let tx = Arc::new(tx);
        inner.inflight = Some(Arc::downgrade(&tx));
        let inner = self.clone();

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
                        error!(error = ?e, "Error fetching data");
                        let _ = tx.send(Err(CachedError {
                            inner: e.to_string(),
                        }));
                    }
                };
            }
        });
        rx
    }
}

impl<T> CachedInner<T>
where
    T: Clone + Send + Sync + 'static,
{
    pub fn get(&self) -> Option<CacheResult<T>> {
        let mut inner = self.mutex.lock();

        self.get_core(&mut inner)
    }

    fn get_core(
        &self,
        inner: &mut parking_lot::lock_api::MutexGuard<parking_lot::RawMutex, CachedValue<T>>,
    ) -> Option<CacheResult<T>> {
        // let mut inner = self.mutex.lock();
        inner.last_accessed = Some(Instant::now());

        if let Some((fetched_at, value)) = inner.last_fetched.as_ref() {
            let Some(refresh_interval) = self.refresh_interval else {
                    return Some(CacheResult::Cached(value.clone()));
                };

            let elapsed = fetched_at.elapsed();

            if elapsed < refresh_interval {
                return Some(CacheResult::Cached(value.clone()));
            } else {
                inner.last_accessed = None;
                debug!(elapsed = ?elapsed, refresh_interval = ?refresh_interval, "Cache is stale, ignoring value");
            }
        }

        inner
            .inflight
            .as_ref()
            .and_then(Weak::upgrade)
            .map(|inflight| CacheResult::InFlight(inflight.subscribe()))
    }

    pub fn get_last_accessed(&self) -> Option<Instant> {
        let inner = self.mutex.lock();
        inner.last_accessed
    }

    pub fn clear(&self) -> Option<T> {
        let mut inner = self.mutex.lock();
        inner.last_fetched.take().map(|(_, v)| v)
    }
}
