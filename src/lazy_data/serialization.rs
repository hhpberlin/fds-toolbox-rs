use std::{
    error::Error,
    fmt::{self, Debug}, any::Any,
};

use async_compression::tokio::{
    bufread::{BrotliDecoder, ZstdDecoder},
    write::{BrotliEncoder, ZstdEncoder},
};

use async_trait::async_trait;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use typetag::erased_serde;

#[typetag::serde(tag = "type")]
pub trait Data: Debug + 'static {
    fn as_any(&self) -> &dyn Any;
}

// impl<T> Data for T:

pub trait Serializer {
    type Error: Error + Send + Sync + 'static;
    fn serialize(&self, data: &Box<dyn Data>) -> Result<Vec<u8>, Self::Error>;
    fn deserialize(&self, data: &[u8]) -> Result<Box<dyn Data>, Self::Error>;
}

#[async_trait]
pub trait Compressor {
    type Error: Error + Send + Sync + 'static;
    async fn compress(&self, data: &[u8]) -> Result<Vec<u8>, Self::Error>;
    async fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, Self::Error>;
}

pub struct CompressedSerializer<S: Serializer, C: Compressor> {
    serialization_algorithm: S,
    compression_algorithm: C,
}

impl<S: Serializer + Default, C: Compressor + Default> Default for CompressedSerializer<S, C> {
    fn default() -> Self {
        Self {
            serialization_algorithm: S::default(),
            compression_algorithm: C::default(),
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

impl<S: Serializer, C: Compressor> Serializer for CompressedSerializer<S, C> {
    type Error = CompressedSerializationError<S::Error, C::Error>;

    #[tokio::main]
    async fn serialize(&self, data: &Box<dyn Data>) -> Result<Vec<u8>, Self::Error> {
        let serialized = self
            .serialization_algorithm
            .serialize(data)
            .map_err(|x| CompressedSerializationError::SerializationError(x))?;
        let compression = self
            .compression_algorithm
            .compress(&serialized[..])
            .await
            .map_err(|x| CompressedSerializationError::CompressionError(x))?;
        Ok(compression)
    }

    #[tokio::main]
    async fn deserialize(&self, data: &[u8]) -> Result<Box<dyn Data>, Self::Error> {
        let decompressed = self
            .compression_algorithm
            .decompress(data)
            .await
            .map_err(|x| CompressedSerializationError::CompressionError(x))?;
        let deserialized = self
            .serialization_algorithm
            .deserialize(&decompressed[..])
            .map_err(|x| CompressedSerializationError::SerializationError(x))?;
        Ok(deserialized)
    }
}

#[derive(Default)]
pub struct MessagePackSerializer;

impl Serializer for MessagePackSerializer {
    type Error = erased_serde::Error;

    fn deserialize(&self, data: &[u8]) -> Result<Box<dyn Data>, Self::Error> {
        let mut ser = rmp_serde::Deserializer::new(data);
        Ok(erased_serde::deserialize(
            &mut <dyn erased_serde::Deserializer>::erase(&mut ser),
        )?)
    }

    fn serialize(&self, data: &Box<dyn Data>) -> Result<Vec<u8>, Self::Error> {
        let mut buf = Vec::new();
        let mut ser = rmp_serde::Serializer::new(&mut buf);
        data.erased_serialize(&mut <dyn erased_serde::Serializer>::erase(&mut ser))?;
        Ok(buf)
    }
}

macro_rules! async_compression_impl {
    ( $n:ident , $c:ident , $d:ident ) => {
        #[derive(Default)]
        pub struct $n;
        #[async_trait]
        impl Compressor for $n {
            type Error = std::io::Error;

            //     #[tokio::main] // tokio::main just turns a async function to a sync function
            async fn compress(&self, data: &[u8]) -> Result<Vec<u8>, Self::Error> {
                let mut encoder = $c::new(Vec::new());
                encoder.write_all(data).await?;
                encoder.shutdown().await?;
                Ok(encoder.into_inner())
            }
            async fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, Self::Error> {
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
    type Error = NoneCompressorError;

    async fn compress(&self, data: &[u8]) -> Result<Vec<u8>, Self::Error> {
        Ok(data.to_vec())
    }
    async fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, Self::Error> {
        Ok(data.to_vec())
    }
}
