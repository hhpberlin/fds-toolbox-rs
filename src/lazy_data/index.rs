use std::{sync::{RwLock, Arc}, future::Future, error::Error, pin::Pin};

use dashmap::DashMap;

use super::serialization::{Data, Serializer};

type BoxFuture<T, E = Box<dyn Error>> = Box<dyn Future<Output = Result<T, E>> + Unpin>;

pub enum CASValue {
    Data(Arc<dyn Data>),
    FutureData(BoxFuture<Box<dyn Data>>),
    Compressed(Vec<u8>),
    FutureCompressed(BoxFuture<Vec<u8>>),
}

pub struct CASIndex<'a> {
    stored: DashMap<&'a [u8], CASValue>,
    serializer: Box<dyn Serializer>,
}

impl CASIndex {
    pub fn new(serializer: Box<dyn Serializer>) -> Self {
        Self {
            stored: DashMap::new(),
            serializer
        }
    }

    pub async fn fetch<'a>(&'a self, key: &[u8]) -> Result<Arc<dyn Data + 'a>, Box<dyn Error + 'a>> {
        let mut entry = self.stored.entry(key);
        match entry {
            dashmap::mapref::entry::Entry::Occupied(mut entryContent) => {
                let val = entryContent.get()?;
                match &*val {
                    CASValue::Data(ref data) => Ok(data.clone()), // Only the Arc is cloned
                    _ => {
                        drop(val);
                        // let a = entryContent.get_mut().get_mut()?;
                        self.stored.alter(key, |x| CASValue::Data(self.to_data(x)));
                    }
                }
            },
            dashmap::mapref::entry::Entry::Vacant(val) => Err(Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, "Key not found"))),
            // Some(value) => {
            // },
            // None => Err(Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, "Key not found"))),
        }
    }

    async fn to_data<'a>(&'a self, val: CASValue) -> Result<Arc<dyn Data + 'a>, Box<dyn Error + 'a>> {
        match val {
            CASValue::Data(data) => Ok(data),
            CASValue::FutureData(future) => Ok(future.await?.into()),
            CASValue::Compressed(compressed) => {
                let data = compressed.clone();
                let data = (*self.serializer).deserialize(&data[..])?;
                Ok(data.into())
            }
            CASValue::FutureCompressed(future) => {
                let compressed = future.await?;
                let data = compressed.clone();
                let data = (*self.serializer).deserialize(&data[..])?;
                Ok(data.into())
            }
        }
    }
}