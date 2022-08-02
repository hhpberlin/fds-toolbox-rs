use std::{collections::HashMap, hash::Hash};
use crate::sync::RwLock;

use async_trait::async_trait;
use thiserror::Error;

use crate::lazy_data::remote::Remote;

pub struct TestRemote<Key: Eq + Hash + Clone + Send> {
    data: HashMap<Key, Vec<u8>>,
    request_count: RwLock<u32>,
}

#[derive(Error, Debug, PartialEq)]
#[error("Key not present")]
pub struct RemoteNotFoundError;

impl<Key: Eq + Hash + Clone + Send> TestRemote<Key> {
    pub fn new<const N: usize>(map: [(Key, Vec<u8>); N]) -> Self {
        Self {
            data: HashMap::from(map),
            request_count: RwLock::new(0),
        }
    }

    pub async fn get_request_count(&self) -> u32 {
        *(self.request_count.read().await)
    }
}

pub fn simple_remote() -> TestRemote<&'static str> {
    TestRemote::new([("key1", vec![1, 2, 3]), ("key2", vec![4, 5, 6])])
}

#[tokio::test]
async fn basic_retrieval() {
    let remote = simple_remote();
    assert_eq!(remote.get_async(&"key1").await, Ok(vec![1, 2, 3]));
    assert_eq!(remote.get_async(&"key2").await, Ok(vec![4, 5, 6]));
}

#[tokio::test]
async fn not_found() {
    let remote = simple_remote();
    assert_eq!(
        remote.get_async(&"not_a_real_key").await,
        Err(RemoteNotFoundError)
    );
}

#[async_trait]
impl<Key: Eq + Hash + Clone + Send + Sync> Remote<Key> for TestRemote<Key> {
    type Error = RemoteNotFoundError;

    async fn get_async(&self, key: &Key) -> Result<Vec<u8>, Self::Error> {
        let mut write = self.request_count.write().await;
        *write += 1;
        match self.data.get(key) {
            Some(val) => Ok(val.clone()),
            None => Err(RemoteNotFoundError),
        }
    }
}
