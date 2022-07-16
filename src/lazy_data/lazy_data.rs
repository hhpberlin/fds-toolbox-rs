use std::{collections::HashMap, any::Any};

// use serde::{Serialize, Deserialize, de::DeserializeOwned}; 

use rmp_serde::{Deserializer, Serializer};
use tokio::net::{TcpListener, TcpStream};

// #[derive(Serialize, Deserialize)]
enum Value<T>
{
    Value(T),
    // CompressedRandomAccess()
    // Compressed(Vec<u8>),
    Processing(),
    Uninitialized,
}

#[typetag::serde(tag = "type")]
pub trait Data {}

// #[typetag::serde]

pub struct Index<'a> {
    stored: HashMap<&'a [u8], Value<Box<dyn Data>>>,
}

impl<'a> Index<'a> {
    pub fn new() -> Self {
        Index {
            stored: HashMap::new(),
        }
    }

    pub fn get(&self, key: &[u8]) -> Option<&dyn Data> {
        let value = self.stored.get(key);
        match value {
            Some(Value::Value(value)) => Some(value.as_ref()),
            // Some(Value::LocallyCached) =>
            _ => None,
        }
    }

    async fn fetch(&self, key: &[u8]) {
        // let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    }
}