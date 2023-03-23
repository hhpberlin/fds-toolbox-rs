use std::{error::Error, io::Read, borrow::Borrow};

use async_trait::async_trait;
use thiserror::Error;

use crate::formats::{smv::{Smv, self}, smoke::dim2::slice::{Slice, self}, csv};

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

// pub struct ParsedFile<T, Fs: FileSystem> {
//     parsed: T,
//     fs: Fs,
//     directory: Fs::Path,
// }

// impl<T, Fs: FileSystem> ParsedFile<T, Fs> {
//     pub fn new(parsed: T, path: Fs::Path) -> Self {
//         Self { parsed, path }
//     }
// }

trait Parse: Sized {
    type Error;
    type Warning;

    fn parse(file: impl Read, warn: MaybeFn<Self::Warning>) -> Result<Self, Self::Error>;
}

// impl<T: Parse> T {
//     fn parse(file: impl Read) -> Result<Self, T::Error> {
//         T::parse(file, None)
//     }
// }

#[derive(Error)]
enum ParseError<FsErr: Error, ParseErr: Error> {
    Fs(FsErr),
    Io(std::io::Error),
    Parse(ParseErr),
}

// impl<T: Parse, Fs: FileSystem> ParsedFile<T, Fs> {
//     pub fn parse(file: Fs::File) -> Result<Self, ParseError<Fs::Error, T::Error>> {
//         file.read
//     }
// }

type MaybeFn<T> = Option<Box<dyn Fn(T)>>;

// impl<Fs: FileSystem> ParsedFile<Smv, Fs> {
//     fn parse(fs: Fs, path: Fs::PathRef, warn: MaybeFn<smv::Error>) -> Result<Self, ParseError<Fs::Error, smv::Error>> {
//         let file = fs.read(path).await?;
//         let buf = String::new();
//         file.read_to_string(&mut buf)?;
//         let parsed = Smv::parse(buf, warn).map_err(ParseError::Parse)?;
//         Ok(Self { parsed, fs, path })
//     }

//     fn slice(&self) {
//         self.
//     }
// }

struct Simulation<Fs: FileSystem> {
    smv: Smv,
    fs: Fs,
    directory: Fs::Path,
    chid: String,
}

impl<Fs: FileSystem> Simulation<Fs> {
    pub async fn parse(fs: Fs, directory: Fs::Path, chid: String) -> Result<Self, ParseError<Fs::Error, smv::Error>> {
        // & doesn't seem to infer the type properly, .borrow() does (PathBuf -> &Path instead &PathBuf)
        let path = fs.file_path(directory.borrow(), &format!("{}.smv", chid));
        let mut file = fs.read(path.borrow()).await.map_err(ParseError::Fs)?;

        // TODO
        // let size = file.metadata().map(|m| m.len()).unwrap_or(0);
        let size = 0;
        let mut string = String::with_capacity(size as usize);
        file.read_to_string(&mut string).map_err(ParseError::Io)?;
       
        let smv = Smv::parse(&string).map_err(ParseError::Parse)?;

        // TODO: Add proper error handling
        debug_assert_eq!(smv.chid, chid);

        Ok(Self { smv, fs, directory, chid })
    }

    async fn file(&self, file_name: &str) -> Result<Fs::File, Fs::Error> {
        let path = self.fs.file_path(self.directory.borrow(), file_name);
        self.fs.read(path.borrow()).await
    }


    async fn slice(&self, idx: usize) -> Result<Slice, ParseError<Fs::Error, slice::Error>> {
        let slice = &self.smv.slices[idx];
        let file = self.file(&slice.file_name).await.map_err(ParseError::Fs)?;
        Slice::from_reader(file).map_err(ParseError::Parse)
    }

    // async fn s3d(&self, idx: usize) {
    //     let s3d = &self.smv.smoke3d[idx];
    //     let file = self.file(&s3d.file_name).await.map_err(ParseError::Fs)?;

    // }

    // async fn csv(&self, name: &str) -> Result<Vec<()>, ParseError<Fs::Error, ()>> {
    //     let files = self.smv.csv_files[name];

    //     // TODO: use smth like futures join_all to await in parallel
    //     // let idk = files.iter().map(|file| {
    //     //     let file = self.file(file).await.map_err(ParseError::Fs)?;
    //     // }).collect();

    //     let mut vec = Vec::new();

    //     for file in files {
    //         let file = self.file(&file).await.map_err(ParseError::Fs)?;
    //         let csv = csv::cpu::
    //     }

    //     Ok(vec)
    // }
}