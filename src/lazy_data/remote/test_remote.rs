use std::{collections::HashMap, hash::Hash};

use async_trait::async_trait;
use thiserror::Error;

use crate::lazy_data::remote::Remote;

pub struct TestRemote<Key: Eq + Hash + Clone + Send> {
    data: HashMap<Key, Vec<u8>>,
}

#[derive(Error, Debug, PartialEq)]
#[error("Key not present")]
pub struct RemoteNotFoundError;

impl<Key: Eq + Hash + Clone + Send> TestRemote<Key> {
    pub fn new<const N: usize>(map: [(Key, Vec<u8>); N]) -> Self {
        Self {
            data: HashMap::from(map),
        }
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
        match self.data.get(key) {
            Some(val) => Ok(val.clone()),
            None => Err(RemoteNotFoundError),
        }
    }
}
