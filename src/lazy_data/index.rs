use std::{sync::{RwLock, Arc}, future::Future, error::Error};

use dashmap::DashMap;

use super::serialization::{Data, Serializer};

pub enum CASValue {
    Data(Arc<Box<dyn Data>>),
    FutureData(Box<dyn Future<Output = Result<Box<dyn Data>, Box<dyn Error>>>>),
    Compressed(Vec<u8>),
    FutureCompressed(Box<dyn Future<Output = Result<Vec<u8>, Box<dyn Error>>>>),
}

pub struct CASIndex {
    stored: DashMap<Vec<u8>, RwLock<CASValue>>,
    serializer: Box<dyn Serializer>,
}

impl CASIndex {
    pub fn new() -> Self {
        Self {
            stored: DashMap::new(),
        }
    }

    pub async fn fetch(&self, key: &[u8]) -> Result<Arc<Box<dyn Data>>, Box<dyn Error>> {
        match self.stored.get(key) {
            Some(value) => {
                let val = value.read()?;
                match *val {
                    CASValue::Data(data) => Ok(data.clone()),
                    CASValue::FutureData(ref future) => {
                        let data = (**future).await?;
                        Ok(data)
                    }
                    CASValue::Compressed(ref compressed) => {
                        let data = compressed.clone();
                        let data = (*self.serializer).deserialize(&data[..])?;
                        Ok(Arc::new(data))
                    }
                    _ => Err(),
                }
            },
            None => Err(Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, "Key not found"))),
        }
    }
}