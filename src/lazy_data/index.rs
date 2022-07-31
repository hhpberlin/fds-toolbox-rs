use std::{sync::{Arc}, fmt, error::Error};

use crossbeam::atomic::AtomicCell;
use dashmap::{DashMap, mapref::entry::Entry};
// use futures::{future::Shared, FutureExt};
use tokio::{sync::{RwLock, RwLockWriteGuard}, time::Instant};

use super::{serialization::{Data, Serializer}, remote::Remote};

// type BoxFuture<'a, T, E = Report> = Shared<Pin<Box<impl Future<Output = Result<T, E>> + 'a>>>;

pub enum CASValue {
    Data(Arc<dyn Data + 'static>),
    Compressed(Vec<u8>),
    Fetching,
}

// pub struct CASError;
// pub impl

#[derive(Debug)]
pub enum CASError<S: Error + 'static> {
    SerializationError(S),
    DeadFetchingThread,
}

impl<S: Error> fmt::Display for CASError<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CASError::SerializationError(err) => std::fmt::Display::fmt(&err, f),
            CASError::DeadFetchingThread => write!(f, "Dead fetching thread"),
        }
    }
}

impl<S: Error> Error for CASError<S> {
    // fn source(&self) -> Option<&(dyn Error + 'static)> {
    //     match self {
    //         CASError::RemoteError(err) => Some(&(*err)),
    //         CASError::SerializationError(err) => Some(err),
    //         CASError::DeadFetchingThread => None,
    //     }
    // }
}


#[derive(Debug)]
pub enum CASFetchError<S: Error + 'static, R: Error + 'static> {
    CacheError(CASError<S>),
    RemoteError(R),
}

impl<S: Error + 'static, R: Error + 'static> fmt::Display for CASFetchError<S, R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CASFetchError::CacheError(err) => std::fmt::Display::fmt(&err, f),
            CASFetchError::RemoteError(err) => std::fmt::Display::fmt(&err, f),
        }
    }
}

impl<S: Error + 'static, R: Error + 'static> Error for CASFetchError<S, R> {}

pub struct CASNode { 
    lock: RwLock<CASValue>, 
    last_use: AtomicCell<Instant>,
}

impl CASNode {
    pub fn new() -> Self {
        Self { lock: RwLock::new(CASValue::Fetching), last_use: Instant::now().into() }
    }

    pub async fn get_data<S: Serializer>(&self, serializer: &S) -> Result<Arc<dyn Data + 'static>, CASError<S::Error>> {
        let read_lock = self.lock.read().await;
        match *read_lock {
            CASValue::Data(ref data) => Ok(data.clone()), // Only the Arc is cloned
            CASValue::Compressed(_) => { // Can't borrow here due to the borrow depending on read_lock, which gets dropped
                drop(read_lock); // just to be sure

                // Get a write lock for the duration of the conversion
                let write_lock = self.lock.write().await;

                Self::to_value(write_lock, serializer)
            },
            CASValue::Fetching => {
                // CASValue::Fetching should always be behind a write-lock
                // if it's set while unlocked, some error has occured
                Err(CASError::DeadFetchingThread)
            }
        }
    }

    fn to_value<S: Serializer>(write_lock: RwLockWriteGuard<'_, CASValue>, serializer: &S) -> Result<Arc<dyn Data + 'static>, CASError<S::Error>> {
        // Check again if the value has been set since we dropped the read lock
        let compressed;
        match &*write_lock {
            CASValue::Data(ref data) => return Ok(data.clone()),
            CASValue::Compressed(ref compressed_vec) => compressed = compressed_vec,
            CASValue::Fetching => return Err(CASError::DeadFetchingThread),
        }

        // let converted = tokio::spawn(async move {
        //     let data = serializer.deserialize(&compressed[..])?;
        //     Ok(data.into())
        // });
        // let converted: Arc<dyn Data + 'static> = converted.await??;
        let converted = serializer.deserialize(&compressed[..]);

        let converted: Arc<dyn Data + 'static> = match converted {
            Ok(data) => data.into(),
            Err(err) => return Err(CASError::SerializationError(err)),
        };

        *write_lock = CASValue::Data(converted.clone());

        Ok(converted)
    }
}

pub struct CASIndex<S: Serializer> {
    stored: DashMap<Vec<u8>, CASNode>,
    serializer: S,
}

impl<'a, S: Serializer> CASIndex<S> {
    pub fn new(serializer: S) -> Self {
        Self {
            stored: DashMap::new(),
            serializer
        }
    }

    pub async fn get<R: Remote>(&'a self, key: &Vec<u8>, remote: &R) -> Result<Arc<dyn Data + 'static>, CASFetchError<S::Error, R::Error>> {
        let entry = self.stored.entry(key.to_vec());
        match entry {
            Entry::Occupied(entry_value) => {
                let value = entry_value.into_ref().downgrade(); // This causes locking for no good reason, TODO: fix
                value.last_use.store(Instant::now());
                let result = value.get_data(&self.serializer).await;
                result.map_err(|x| CASFetchError::CacheError(x))
            },
            Entry::Vacant(entry) => {
                let value = entry.insert(CASNode::new()).downgrade();

                let write_lock = value.lock.write().await;
                let value = value.downgrade();

                self.fetch(key, remote, write_lock).await
            },
        }
    }

    async fn fetch<R: Remote>(&'a self, key: &Vec<u8>, remote: &R, mut write_lock: RwLockWriteGuard<'_, CASValue>) -> Result<Arc<dyn Data + 'static>, CASFetchError<S::Error, R::Error>> {
        self.prefetch(key, remote, &mut write_lock).await.map_err(|x| CASFetchError::RemoteError(x))?;
        CASNode::to_value(write_lock, &self.serializer).map_err(|x| CASFetchError::CacheError(x))
    }

    async fn prefetch<R: Remote>(&'a self, key: &Vec<u8>, remote: &R, write_lock: &mut RwLockWriteGuard<'_, CASValue>) -> Result<(), R::Error> {
        let result = remote.get_async(key).await?;

        **write_lock = CASValue::Compressed(result);
        Ok(())
    }
}