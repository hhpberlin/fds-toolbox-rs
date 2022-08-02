use crate::sync::{Arc, AtomicU64};
use std::hash::Hash;

use crossbeam::atomic::AtomicCell;
use dashmap::{mapref::entry::Entry, DashMap};
use tokio::{sync::RwLock, time::Instant};

use super::{
    remote::Remote,
    serialization::{Data, Serializer},
};

enum CASValue {
    Value(Arc<dyn Data + 'static>),
    Serialized(Vec<u8>),
}

struct CASNode {
    value: RwLock<Option<CASValue>>,
    last_use: AtomicCell<Instant>,
}

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum CASError<SE, RE> {
    #[error("Serialization error: {0}")]
    SerializationError(SE),
    #[error("Remote error: {0}")]
    RemoteError(RE),
}

impl CASValue {
    fn materialize<S: Serializer>(
        &self,
        serializer: &S,
    ) -> Result<Arc<dyn Data + 'static>, S::Error> {
        match self {
            CASValue::Value(value) => Ok(value.clone()),
            CASValue::Serialized(compressed) => {
                let data = serializer.deserialize(compressed.as_slice())?;

                Ok(data.into())
            }
        }
    }

    async fn fetch<R: Remote<Key>, Key: Eq + Hash + Clone>(
        remote: &R,
        key: &Key,
    ) -> Result<CASValue, R::Error> {
        let data = remote.get_async(&key).await?;
        Ok(CASValue::Serialized(data))
    }
}

impl CASNode {
    fn new(value: Option<CASValue>) -> Self {
        Self {
            value: RwLock::new(value),
            // Although exact synchronization is not strictly required, it would still be UB to not sync, so lets not do that.
            last_use: AtomicCell::new(Instant::now()),
        }
    }

    fn set_last_use(&self) {
        self.last_use.store(Instant::now());
    }

    async fn get<S: Serializer>(
        &self,
        serializer: &S,
    ) -> Result<Option<Arc<dyn Data + 'static>>, S::Error> {
        let read = self.value.read().await;
        match &*read {
            Some(CASValue::Value(value)) => return Ok(Some(value.clone())),
            // Won´t be able to materialize the value here if no value is cached
            None => return Ok(None),
            _ => (),
        }
        drop(read);

        let mut write = self.value.write().await;
        match &*write {
            Some(ref value) => {
                match value {
                    // Value may have materialized while waiting for write lock
                    // => avoid rewriting the same value unnecessarily
                    CASValue::Value(value) => Ok(Some(value.clone())),
                    _ => {
                        let value = value.materialize(serializer)?;
                        *write = Some(CASValue::Value(value.clone()));
                        Ok(Some(value))
                    }
                }
            }
            None => Ok(None),
        }
    }

    async fn get_or_fetch<S: Serializer, R: Remote<Key>, Key: Eq + Hash + Clone>(
        &self,
        serializer: &S,
        remote: &R,
        key: &Key,
    ) -> Result<Arc<dyn Data + 'static>, CASError<S::Error, R::Error>> {
        let materialized = self
            .get(serializer)
            .await
            .map_err(|x| CASError::SerializationError(x))?;
        if let Some(value) = materialized {
            return Ok(value);
        }

        let mut write = self.value.write().await;

        // Recheck after acquiring write lock
        let materialized = self
            .get(serializer)
            .await
            .map_err(|x| CASError::SerializationError(x))?;
        if let Some(value) = materialized {
            return Ok(value);
        }

        let value = CASValue::fetch(remote, key)
            .await
            .map_err(|x| CASError::RemoteError(x))?;
        let value = value
            .materialize(serializer)
            .map_err(|x| CASError::SerializationError(x))?;

        *write = Some(CASValue::Value(value.clone()));

        Ok(value)
    }
}

pub struct CAS<Key: Eq + Hash + Clone> {
    nodes: DashMap<Key, CASNode>,
}

impl<Key: Eq + Hash + Clone> CAS<Key> {
    pub fn new() -> Self {
        Self {
            nodes: DashMap::new(),
        }
    }

    pub async fn get<S: Serializer>(
        &self,
        key: Key,
        serializer: &S,
    ) -> Result<Option<Arc<dyn Data + 'static>>, S::Error> {
        match self.nodes.entry(key) {
            Entry::Occupied(ref node) => {
                let node = node.get();
                node.set_last_use();
                node.get(serializer).await
            }
            Entry::Vacant(_) => Ok(None),
        }
    }

    pub async fn get_or_fetch<S: Serializer, R: Remote<Key>>(
        &self,
        key: Key,
        serializer: &S,
        remote: &R,
    ) -> Result<Arc<dyn Data + 'static>, CASError<S::Error, R::Error>> {
        let node = self.nodes.entry(key).or_insert_with(|| CASNode::new(None));
        node.set_last_use();
        node.get_or_fetch(serializer, remote, node.key()).await
    }
}
