use std::{sync::{Arc}, future::Future, error::Error, pin::Pin, time::Instant};

use dashmap::DashMap;

use super::serialization::{Data, Serializer};

// use tokio::prelude::*;

type BoxFuture<'a, T, E = Box<dyn Error + 'a>> = Pin<Box<dyn Future<Output = Result<T, E>> + 'a>>;

pub enum CASValue<'a> {
    Data(Arc<dyn Data>),
    FutureData(BoxFuture<'a, Arc<dyn Data + 'a>>),
    Compressed(Vec<u8>),
    FutureCompressed(BoxFuture<'a, Vec<u8>>),
}

pub struct CASIndex<'a> {
    stored: DashMap<&'a [u8], CASValue<'a>>,
    last_uses: DashMap<&'a [u8], Instant>,
    serializer: Box<dyn Serializer>,
}

impl<'a> CASIndex<'a> {
    pub fn new(serializer: Box<dyn Serializer>) -> Self {
        Self {
            stored: DashMap::new(),
            last_uses: DashMap::new(),
            serializer
        }
    }

    pub async fn fetch(&'a self, key: &[u8]) -> Result<Arc<dyn Data + 'a>, Box<dyn Error + 'a>> {
        match self.stored.get(key) {
            Some(val) => {
                match val.value() {
                    CASValue::Data(ref data) => Ok(data.clone()), // Only the Arc is cloned
                    _ => {
                        drop(val);
                        // let a = entryContent.get_mut().get_mut()?;
                        let mut result;
                        self.stored.alter(key, |_, x| {
                            let data = self.to_data(x);
                            let data: BoxFuture<'a, Arc<dyn Data + 'a>> = Box::pin(data);
                            result = &data.shared();
                            let out = CASValue::<'a>::FutureData(data);
                            out
                        });
                        Ok(result.as_mut().await?)
                    }
                }
            },
            None => Err(Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, "Key not found"))),
            // Some(value) => {
            // },
            // None => Err(Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, "Key not found"))),
        }
    }

    async fn to_data(&'a self, val: CASValue<'a>) -> Result<Arc<dyn Data + 'a>, Box<dyn Error + 'a>> {
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