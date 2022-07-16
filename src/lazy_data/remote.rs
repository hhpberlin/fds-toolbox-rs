pub mod quic_remote;

use async_trait::async_trait;

#[async_trait]
pub trait ReadOnlyRemote {
    // fn get_data(&self, key: &[u8]) -> Option<&dyn Data>;
    fn get_async(&self, key: &[u8]) -> Result<&[u8], ()>;
    // fn get_data_async_with_timeout(&self, key: &[u8], timeout: Duration) -> Option<Box<dyn Future<Output = Result<Box<dyn Data>, Error>>>>;
}

// pub trait Remote : ReadOnlyRemote {
//     fn put_async(&self, key: &[u8]) -> Future<Result = Result<(), Error>>;
//     fn delete_async(&self, key: &[u8]) -> Future<Result = Result<(), Error>>;
// }
