use std::{future::Future, error::Error, pin::Pin};

use async_compression::tokio::bufread::{BrotliDecoder};
use tokio::io::{AsyncRead, AsyncReadExt};

use dashmap::DashMap;

use super::remote::Remote;

use blake3::Hash;
use std::io::prelude::*;

// #[derive(Serialize, Deserialize)]
enum Value<'a, T> {
    Value(T),
    // CompressedRandomAccess()
    Compressed(&'a [u8]),
    Processing(Pin<Box<dyn Future<Output = Result<&'a [u8], Box<dyn Error>>>>>),
    Uninitialized,
}

pub struct VBox<T>(T);

#[typetag::serde(tag = "type")]
pub trait Data {}

// #[typetag::serde]

pub struct Index<'a>
{
    stored: DashMap<&'a Hash, VBox<Value<'a, Box<dyn Data>>>>,
}

impl<'a> Index<'a>
{
    pub fn new() -> Self {
        Index {
            stored: DashMap::new(),
        }
    }

    pub fn get(&self, key: &Hash) -> Option<&dyn Data> {
        let value = self.stored.get(key);
        
        match value {
            Some(value) => {
                match value.value().0 {
                    Value::Value(data) => Some(data.as_ref()),
                    _ => None,
                }
            },
            None => None,
        }
    }

    fn decompress() {
        // blake3::Hash
    }

    async fn fetch(&mut self, key: &Hash, remote: &impl Remote) {
        match self.stored.get_mut(key) {
            Some(currentRef) => {
                match currentRef.value().0 {
                    Value::Uninitialized => {
                        let mut val_box = currentRef.value_mut();
                        let val = Value::Processing::<Box<dyn Data>>(remote.get_async(key.as_bytes()));
                        val_box.0 = val;
                    },
                    Value::Processing(future) => {
                        future.await;
                    },
                    Value::Compressed(bytes) => {
                        // async_compression::tokio::bufread::BrotliDecoder
                        let mut decoder: BrotliDecoder<Vec<u8>> = BrotliDecoder::new(Vec::new());
                        decoder.write_all(bytes).await;
                        decoder.shutdown().await?;
                        let decoded = decoder.into_inner();
                    }
                    _ => {},
                }
            },
            None => {
                self.stored.insert(key, VBox(Value::Processing(remote.get_async(key.as_bytes()))));
            },
        }
    }
}
