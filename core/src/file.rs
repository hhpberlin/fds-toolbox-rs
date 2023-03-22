use std::{error::Error, io::Read, borrow::Borrow};

use async_trait::async_trait;

use crate::formats::smv::Smv;

#[async_trait]
pub trait FileSystem {
    type Path: Borrow<Self::PathRef>;
    type PathRef: ?Sized;
    type Error: Error;
    type File: Read;

    async fn read(&self, path: &Self::PathRef) -> Result<Self::File, Self::Error>;

    fn file_path(&self, directory: &Self::PathRef, file_name: &str) -> Self::Path;
}

pub struct OsFs;

#[async_trait]
impl FileSystem for OsFs {
    type Path = std::path::PathBuf;
    type PathRef = std::path::Path;
    type Error = std::io::Error;
    type File = std::fs::File;

    async fn read(&self, path: &Self::PathRef) -> Result<Self::File, Self::Error> {
        // TODO: Consider memory mapping
        //       Problem: Unsoundness due to POSIX locking being only advisory
        //                => &[u8] memory map not guaranteed to be immutable
        // std::fs
        // tokio::fs::File::open("path").await.unwrap().read_
        std::fs::File::open(path)
    }

    fn file_path(&self, directory: &Self::PathRef, file_name: &str) -> Self::Path {
        directory.join(file_name)
    }
}

pub struct ParsedFile<T, Fs: FileSystem> {
    parsed: T,
    path: Fs::Path,
}

impl<T, Fs: FileSystem> ParsedFile<T, Fs> {
    pub fn new(parsed: T, path: Fs::Path) -> Self {
        Self { parsed, path }
    }
}

// impl<T, Fs: FileSystem> ParsedFile<T, Fs> {
//     pub fn parse(file: Fs::File) -> Result<Self, Fs::Error> {
//         file.
//     }
// }