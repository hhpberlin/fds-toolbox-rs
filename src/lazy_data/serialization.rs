use std::{error::Error};

use async_compression::tokio::{bufread::BrotliDecoder, write::BrotliEncoder};

use async_trait::async_trait;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use typetag::erased_serde;

#[typetag::serde(tag = "type")]
pub trait Data {}

pub trait Serializer {
    fn serialize(&self, data: &Box<dyn Data>) -> Result<Vec<u8>, Box<dyn Error>>;
    fn deserialize(&self, data: &[u8]) -> Result<Box<dyn Data>, Box<dyn Error>>;
}

#[async_trait]
pub trait Compressor {
    async fn compress(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>>;
    async fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>>;
}

pub struct CompressedSerialization<S: Serializer, C: Compressor> {
    serialization_algorithm: S,
    compression_algorithm: C,
}

impl<S: Serializer, C: Compressor> Serializer for CompressedSerialization<S, C> {
    #[tokio::main] // tokio::main just turns a async function to a sync function
    async fn serialize(&self, data: &Box<dyn Data>) -> Result<Vec<u8>, Box<dyn Error>> {
        let serialized = self.serialization_algorithm.serialize(data)?;
        let compression = self.compression_algorithm.compress(&serialized[..]).await?;
        Ok(compression)
    }

    #[tokio::main]
    async fn deserialize(&self, data: &[u8]) -> Result<Box<dyn Data>, Box<dyn Error>> {
        let decompressed = self.compression_algorithm.decompress(data).await?;
        let deserialized = self.serialization_algorithm.deserialize(&decompressed[..])?;
        Ok(deserialized)
    }
}

pub struct MessagePackSerializor;

impl Serializer for MessagePackSerializor {
    fn deserialize(&self, data: &[u8]) -> Result<Box<dyn Data>, Box<dyn Error>> {
        let mut ser = rmp_serde::Deserializer::new(data);
        Ok(erased_serde::deserialize(&mut <dyn erased_serde::Deserializer>::erase(&mut ser))?)
    }

    fn serialize(&self, data: &Box<dyn Data>) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut buf = Vec::new();
        let mut ser = rmp_serde::Serializer::new(&mut buf);
        data.erased_serialize(&mut <dyn erased_serde::Serializer>::erase(&mut ser))?;
        Ok(buf)
    }
}

// pub struct ZstdCompressor;

// impl Compressor for ZstdCompressor {
//     #[tokio::main] // tokio::main just turns a async function to a sync function
//     async fn compress(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
//         Ok(zstd::encode_all(data, level));
//     }
//     #[tokio::main]
//     async fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
//         todo!()
//     }
// }

pub struct BrotliCompressor;

#[async_trait]
impl Compressor for BrotliCompressor {
//     #[tokio::main] // tokio::main just turns a async function to a sync function
    async fn compress(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut encoder = BrotliEncoder::new(Vec::new());
        encoder.write_all(data).await?;
        encoder.shutdown().await?;
        Ok(encoder.into_inner())
    }
    async fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut buf = Vec::new();
        let mut decoder = BrotliDecoder::new(data);
        decoder.read_to_end(&mut buf).await?;
        Ok(buf)
    }
}

pub struct NoneCompressor;

#[async_trait]
impl Compressor for NoneCompressor {
    async fn compress(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        Ok(data.to_vec())
    }
    async fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        Ok(data.to_vec())
    }
}