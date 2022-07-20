use std::{sync::{Arc}, time::Instant, future::Future};

use color_eyre::Report;
use dashmap::DashMap;
// use futures::{future::Shared, FutureExt};
use tokio::sync::{RwLock};

use super::serialization::{Data, Serializer};

// type BoxFuture<'a, T, E = Report> = Shared<Pin<Box<impl Future<Output = Result<T, E>> + 'a>>>;

pub enum CASValue {
    Data(Arc<dyn Data + 'static>),
    Compressed(Vec<u8>),
    Fetching,
}

pub struct CASNode(RwLock<CASValue>);

impl CASValue {
    async fn as_data(&self, serializer: &impl Serializer) -> Result<Arc<dyn Data + 'static>, Report> {
        match self {
            CASValue::Data(data) => Ok(data.clone()),
            CASValue::Compressed(compressed) => {
                let data = serializer.deserialize(&compressed[..])?;
                Ok(data.into())
            }
            
        }
    }
}

impl CASNode {
    pub async fn get_data(&self, serializer: &impl Serializer) -> Result<Arc<dyn Data + 'static>, Report> {
        let read_lock = self.0.read().await;
        match *read_lock {
            CASValue::Data(ref data) => Ok(data.clone()), // Only the Arc is cloned
            CASValue::Compressed(ref compressed) => {
                drop(read_lock); // just to be sure

                // Get a write lock for the duration of the conversion
                let mut write_lock = self.0.write().await;

                // Check again if the value has been set since we dropped the read lock
                if let CASValue::Data(ref data) = &*write_lock {
                    return Ok(data.clone());
                }

                // let converted = tokio::spawn(async move {
                //     let data = serializer.deserialize(&compressed[..])?;
                //     Ok(data.into())
                // });
                // let converted: Arc<dyn Data + 'static> = converted.await??;
                let converted = serializer.deserialize(&compressed[..])?;
                let converted: Arc<dyn Data + 'static> = converted.into();

                *write_lock = CASValue::Data(converted.clone());
                
                Ok(converted)
            }
            CASValue::Fetching => {

            }
        }
    }
    
    pub async fn fetch(&self, source: fn(&[u8]) -> Box<dyn Future<Output = Result<CASNode, Report>>>) -> Result<Arc<dyn Data + 'static>, Report> {
        let read_lock = *self.0.read().await;
        match read_lock {
            CASValue::Data(ref data) => Ok(data.clone()), // Only the Arc is cloned
            _ => {
                drop(read_lock); // just to be sure

                // Get a write lock for the duration of the conversion
                let mut write = self.0.write().await;

                // Check again if the value has been set since we dropped the read lock
                if let CASValue::Data(ref data) = &*write {
                    return Ok(data.clone());
                }

                let converted = self.to_data(&*write).await?;
                *write = CASValue::Data(converted.clone());
                
                Ok(converted)
            }
        }
    }
}

pub struct CASIndex<'a> {
    stored: DashMap<&'a [u8], CASNode>,
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

    // pub fn test<'a>(fut: dyn Future<Output = Result<Arc<dyn Data + 'a>, Box<dyn Error + 'a>>>) {
    //     let etst = fut.shared();
    //     let etst = Box::pin(etst);
    //     let bf: BoxFuture<'a, Arc<dyn Data + 'a>> = etst;
    // }

    pub async fn fetch(&'a self, key: &[u8]) -> Result<Arc<dyn Data + 'a>, Report> {
        match self.stored.get(key) {
            Some(value) => {
                let value = value.value();
                match *value.0.read().await {
                    CASValue::Data(ref data) => Ok(data.clone()), // Only the Arc is cloned
                    _ => {
                        drop(value);

                        // Get a write lock for the duration of the conversion
                        let mut write = value.0.write().await;

                        // Check again if the value has been set since we dropped the read lock
                        if let CASValue::Data(ref data) = &*write {
                            return Ok(data.clone());
                        }

                        let converted = self.to_data(&*write).await?;
                        *write = CASValue::Data(converted.clone());
                        
                        Ok(converted)
                    }
                }
            },
            None => Err(Report::new(std::io::Error::new(std::io::ErrorKind::NotFound, "Key not found"))),
        }
    }

    async fn to_data(&'a self, val: &CASValue) -> Result<Arc<dyn Data + 'a>, Report> {
        match val {
            CASValue::Data(data) => Ok(data.clone()),
            CASValue::Compressed(compressed) => {
                let data = compressed.clone();
                let data = (*self.serializer).deserialize(&data[..])?;
                Ok(data.into())
            }
        }
    }
}