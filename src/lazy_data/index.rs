use std::{sync::Arc, fmt};

use color_eyre::Report;
use dashmap::{DashMap, mapref::entry::{OccupiedEntry, VacantEntry, Entry}};
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
pub enum CASNodeGetResult {
    SerializationError(Report),
    FetchingError,
}

impl fmt::Display for CASNodeGetResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CASNodeGetResult::SerializationError(err) => err.fmt(f),
            CASNodeGetResult::FetchingError => write!(f, "Error Fetching data"),
        }
    }
}

impl std::error::Error for CASNodeGetResult {}

// impl std::error::Error for CASNodeGetResult {
//     fn source(&self) -> Option<&(dyn Error + 'static)> {
//         match self {
//             CASNodeGetResult::SerializationError(e) => Some(e),
//             CASNodeGetResult::FetchingError => None,
//         }
//     }
// }

pub struct CASNode { 
    lock: RwLock<CASValue>, 
    last_use: RwLock<Instant>, // TODO: Use more lightweight locking mechanism
}

impl CASNode {
    pub fn new() -> Self {
        Self { lock: RwLock::new(CASValue::Fetching), last_use: RwLock::new(Instant::now()) }
    }

    pub async fn get_data(&self, serializer: &impl Serializer) -> Result<Arc<dyn Data + 'static>, CASNodeGetResult> {
        let read_lock = self.lock.read().await;
        match *read_lock {
            CASValue::Data(ref data) => Ok(data.clone()), // Only the Arc is cloned
            CASValue::Compressed(_) => { // Can't borrow here due to the borrow depending on read_lock, which gets dropped
                drop(read_lock); // just to be sure

                // Get a write lock for the duration of the conversion
                let write_lock = self.lock.write().await;

                Self::to_value(write_lock, serializer)
            }
            CASValue::Fetching => {
                // CASValue::Fetching should always be behind a write-lock
                // if it's set while unlocked, some error has occured
                Err(CASNodeGetResult::FetchingError)
            }
        }
    }

    fn to_value(write_lock: RwLockWriteGuard<'_, CASValue>, serializer: &impl Serializer) -> Result<Arc<dyn Data + 'static>, CASNodeGetResult> {
        // Check again if the value has been set since we dropped the read lock
        let compressed;
        match &*write_lock {
            CASValue::Data(ref data) => return Ok(data.clone()),
            CASValue::Compressed(ref compressed_vec) => compressed = compressed_vec,
            CASValue::Fetching => return Err(CASNodeGetResult::FetchingError),
        }

        // let converted = tokio::spawn(async move {
        //     let data = serializer.deserialize(&compressed[..])?;
        //     Ok(data.into())
        // });
        // let converted: Arc<dyn Data + 'static> = converted.await??;
        let converted = serializer.deserialize(&compressed[..]);

        let converted: Arc<dyn Data + 'static> = match converted {
            Ok(data) => data.into(),
            Err(err) => return Err(CASNodeGetResult::SerializationError(err)),
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

    pub async fn get(&'a self, key: &Vec<u8>, remote: &impl Remote) -> Result<Arc<dyn Data + 'static>, Report> {
        let entry = self.stored.entry(key.to_vec());
        match entry {
            Entry::Occupied(mut entry_value) => {
                let mut value = entry_value.into_ref().pair(); // This feels very wrong
                drop(entry);
                {
                    *value.last_use.write().await = Instant::now();
                }
                let result = value.get_data(&self.serializer).await;
                match result {
                    Ok(data) => Ok(data),
                    Err(CASNodeGetResult::SerializationError(err)) => Err(err),
                    Err(CASNodeGetResult::FetchingError) => {
                        let write_lock = value.lock.write().await;                        
                        self.fetch(key, remote, write_lock).await
                    },
                }
            },
            Entry::Vacant(entry) => {
                let entry = self.stored.entry(key.to_vec()).or_insert(CASNode::new());
                let write_lock = entry.lock.write().await;
                entry.lock.write();
                match node {
                    Some(_) => {
                        self.fetch(key, remote, write_lock).await
                    }
                    None => {
                    }
                }
            },
        }
    }

    async fn fetch(&'a self, key: &Vec<u8>, remote: &impl Remote, mut write_lock: RwLockWriteGuard<'_, CASValue>) -> Result<Arc<dyn Data + 'static>, Report> {
        self.prefetch(key, remote, &mut write_lock).await?;
        Ok(CASNode::to_value(write_lock, &self.serializer)?)
    }

    async fn prefetch(&'a self, key: &Vec<u8>, remote: &impl Remote, write_lock: &mut RwLockWriteGuard<'_, CASValue>) -> Result<(), Report> {
        let result = remote.get_async(key).await?;

        **write_lock = CASValue::Compressed(result);
        Ok(())
    }
}