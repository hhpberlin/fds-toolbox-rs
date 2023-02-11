pub mod quic_remote;
#[cfg(test)]
pub(crate) mod test_remote;

use std::{error::Error, hash::Hash};

use async_trait::async_trait;

#[async_trait]
pub trait Remote<Key: Eq + Hash + Clone> {
    type Error: Error + Send + Sync + 'static;

    async fn get_async(&self, key: &Key) -> Result<Vec<u8>, Self::Error>;
    // fn get_data_async_with_timeout(&self, key: &[u8], timeout: Duration) -> Option<Box<dyn Future<Output = Result<Box<dyn Data>, Error>>>>;
}

// pub trait Remote : ReadOnlyRemote {
//     fn put_async(&self, key: &[u8]) -> Future<Result = Result<(), Error>>;
//     fn delete_async(&self, key: &[u8]) -> Future<Result = Result<(), Error>>;
// }
