use std::{
    error::Error,
    fmt::{self, Debug},
    marker::PhantomData,
};

use async_compression::tokio::{
    bufread::{BrotliDecoder, ZstdDecoder},
    write::{BrotliEncoder, ZstdEncoder},
};

use async_trait::async_trait;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[async_trait]
pub trait Serializer<Data> {
    type SerError: Error + Send + Sync + 'static;
    type DeError: Error + Send + Sync + 'static;
    async fn serialize(&self, data: &Data) -> Result<Vec<u8>, Self::SerError>;
    async fn deserialize(&self, data: &[u8]) -> Result<Data, Self::DeError>;
}

#[async_trait]
pub trait Compressor {
    type CompError: Error + Send + Sync + 'static;
    type DeError: Error + Send + Sync + 'static;
    async fn compress(&self, data: &[u8]) -> Result<Vec<u8>, Self::CompError>;
    async fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, Self::DeError>;
}

pub struct CompressedSerializer<S: Serializer<Data> + Sync + Send, C: Compressor + Sync + Send, Data: Sync + Send> {
    serialization_algorithm: S,
    compression_algorithm: C,
    _data: PhantomData<Data>,
}

impl<S: Serializer<Data> + Sync + Send + Default, C: Compressor + Sync + Send + Default, Data: Sync + Send> Default
    for CompressedSerializer<S, C, Data>
{
    fn default() -> Self {
        Self {
            serialization_algorithm: S::default(),
            compression_algorithm: C::default(),
            _data: PhantomData,
        }
    }
}

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum CompressedSerializationError<S: Error, C: Error> {
    #[error("Serialization error: {0}")]
    SerializationError(S),
    #[error("Compression error: {0}")]
    CompressionError(C),
}

#[async_trait]
impl<S: Serializer<Data> + Sync + Send, C: Compressor + Sync + Send, Data: Sync + Send> Serializer<Data>
    for CompressedSerializer<S, C, Data>
{
    type SerError = CompressedSerializationError<S::SerError, C::CompError>;
    type DeError = CompressedSerializationError<S::DeError, C::DeError>;

    async fn serialize(&self, data: &Data) -> Result<Vec<u8>, Self::SerError> {
        let serialized = self
            .serialization_algorithm
            .serialize(data)
            .await
            .map_err(CompressedSerializationError::SerializationError)?;
        let compression = self
            .compression_algorithm
            .compress(&serialized[..])
            .await
            .map_err(CompressedSerializationError::CompressionError)?;
        Ok(compression)
    }

    async fn deserialize(&self, data: &[u8]) -> Result<Data, Self::DeError> {
        let decompressed = self
            .compression_algorithm
            .decompress(data)
            .await
            .map_err(CompressedSerializationError::CompressionError)?;
        let deserialized = self
            .serialization_algorithm
            .deserialize(&decompressed[..])
            .await
            .map_err(CompressedSerializationError::SerializationError)?;
        Ok(deserialized)
    }
}

#[derive(Default)]
pub struct MessagePackSerializer;

#[async_trait]
impl<Data: serde::de::DeserializeOwned + serde::Serialize + Sync> Serializer<Data>
    for MessagePackSerializer
{
    type SerError = rmp_serde::encode::Error;
    type DeError = rmp_serde::decode::Error;

    async fn deserialize(&self, data: &[u8]) -> Result<Data, Self::DeError> {
        rmp_serde::from_slice(data)
    }

    async fn serialize(&self, data: &Data) -> Result<Vec<u8>, Self::SerError> {
        rmp_serde::to_vec(data)
    }
}

macro_rules! async_compression_impl {
    ( $n:ident , $c:ident , $d:ident ) => {
        #[derive(Default)]
        pub struct $n;
        #[async_trait]
        impl Compressor for $n {
            type CompError = std::io::Error;
            type DeError = std::io::Error;

            //     #[tokio::main] // tokio::main just turns a async function to a sync function
            async fn compress(&self, data: &[u8]) -> Result<Vec<u8>, Self::CompError> {
                let mut encoder = $c::new(Vec::new());
                encoder.write_all(data).await?;
                encoder.shutdown().await?;
                Ok(encoder.into_inner())
            }
            async fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, Self::DeError> {
                let mut buf = Vec::new();
                let mut decoder = $d::new(data);
                decoder.read_to_end(&mut buf).await?;
                Ok(buf)
            }
        }
    };
}

async_compression_impl!(BrotliCompressor, BrotliEncoder, BrotliDecoder);
async_compression_impl!(ZstdCompressor, ZstdEncoder, ZstdDecoder);

#[derive(Default)]
pub struct NoneCompressor;

#[derive(Debug)]
pub struct NoneCompressorError;

impl fmt::Display for NoneCompressorError {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        panic!("This type is only for generic use, it should not be used directly. Something has gone horribly wrong.");
    }
}

impl Error for NoneCompressorError {}

#[async_trait]
impl Compressor for NoneCompressor {
    type CompError = NoneCompressorError;
    type DeError = NoneCompressorError;

    async fn compress(&self, data: &[u8]) -> Result<Vec<u8>, Self::CompError> {
        Ok(data.to_vec())
    }
    async fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, Self::DeError> {
        Ok(data.to_vec())
    }
}
