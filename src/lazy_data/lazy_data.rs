use std::{collections::HashMap, future::Future, error::Error};

use dashmap::DashMap;

use super::remote::ReadOnlyRemote;

// #[derive(Serialize, Deserialize)]
enum Value<T> {
    Value(T),
    // CompressedRandomAccess()
    Compressed([u8]),
    Processing(Future<Result<&[u8], Error>>),
    Uninitialized,
}

#[typetag::serde(tag = "type")]
pub trait Data {}

// #[typetag::serde]

pub struct Index<'a, Hash: std::hash::Hash + Eq> {
    stored: DashMap<&'a Hash, Value<Box<dyn Data>>>,
}

impl<'a, Hash: std::hash::Hash + Eq> Index<'a, Hash> {
    pub fn new() -> Self {
        Index {
            stored: DashMap::new(),
        }
    }

    pub fn get(&self, key: &Hash) -> Option<&dyn Data> {
        let value = self.stored.get(key);
        
        match value {
            Some(value) => {
                match value.value() {
                    Value::Value(data) => Some(data.as_ref()),
                    _ => None,
                }
            },
            None => None,
        }
    }

    async fn fetch(&mut self, key: &Hash, remote: &impl ReadOnlyRemote) {
        match self.stored.get_mut(key) {
            Some(current) => {
                match current.value() {
                    Value::Uninitialized => {
                        let mut val = current.value_mut();
                        val = Value::Processing(remote.get_async(key));
                    },
                    _ => {},
                }
            },
            None => {
                self.stored.insert(key, Value::Processing(remote.get_async(key)));
            },
        }
    }
}
