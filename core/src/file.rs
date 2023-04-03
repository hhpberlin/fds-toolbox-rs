use std::{borrow::Borrow, collections::HashMap, error::Error, fmt::Debug, io::Read, sync::Arc};

use async_trait::async_trait;
use futures::{future::join_all, SinkExt, Stream, StreamExt};
use thiserror::Error;

use crate::formats::{
    csv::{self, cpu::CpuData, devc::Devices, hrr::HRRStep},
    smoke::dim2::slice::{self, Slice},
    smv::{self, Smv},
};

#[async_trait]
pub trait FileSystem {
    type Path: Borrow<Self::PathRef>;
    type PathRef: ?Sized;
    type Error: Error;
    type File: Read;

    async fn read(&self, path: &Self::PathRef) -> Result<Self::File, Self::Error>;
    async fn exists(&self, path: &Self::PathRef) -> Result<bool, Self::Error>;

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

    async fn exists(&self, path: &Self::PathRef) -> Result<bool, Self::Error> {
        Ok(path.exists())
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

#[derive(Error, Debug)]
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
    pub smv: Smv,
    pub fs: Fs,
    pub directory: Fs::Path,
    pub chid: String,
}

impl<Fs: FileSystem> Simulation<Fs>
where
    Fs::Path: Debug,
{
    pub async fn parse(
        fs: Fs,
        directory: Fs::Path,
        chid: String,
    ) -> Result<Self, ParseError<Fs::Error, smv::Error>> {
        // & doesn't seem to infer the type properly, .borrow() does (PathBuf -> &Path instead &PathBuf)
        let path = fs.file_path(directory.borrow(), &format!("{}.smv", chid));
        let mut file = fs.read(path.borrow()).await.map_err(ParseError::Fs)?;

        // TODO
        // let size = file.metadata().map(|m| m.len()).unwrap_or(0);
        let size = 0;
        let mut string = String::with_capacity(size as usize);
        file.read_to_string(&mut string).map_err(ParseError::Io)?;
        drop(file);

        let smv = Smv::parse(&string).map_err(ParseError::Parse)?;

        // TODO: Add proper error handling
        debug_assert_eq!(smv.chid, chid);

        Ok(Self {
            smv,
            fs,
            directory,
            chid,
        })
    }

    async fn read(&self, file_name: &str) -> Result<Fs::File, Fs::Error>
    where
        Fs::Path: Debug,
    {
        self.fs.read(self.path(file_name).borrow()).await
    }

    async fn exists(&self, file_name: &str) -> Result<bool, Fs::Error> {
        self.fs.exists(self.path(file_name).borrow()).await
    }

    fn path(&self, file_name: &str) -> <Fs as FileSystem>::Path {
        self.fs.file_path(self.directory.borrow(), file_name)
    }

    async fn slice(&self, idx: usize) -> Result<Slice, ParseError<Fs::Error, slice::Error>> {
        let slice = &self.smv.slices[idx];
        let file = self.read(&slice.file_name).await.map_err(ParseError::Fs)?;
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

    async fn csv<T, Err: Error>(
        &self,
        name: &str,
        parser: impl Fn(Fs::File) -> Result<T, Err>,
    ) -> Result<Vec<T>, ParseError<Fs::Error, Err>> {
        let files = &self.smv.csv_files[name];

        let futures = files.iter().map(|file| async {
            match self.read(file).await {
                Ok(file) => parser(file).map_err(ParseError::Parse),
                Err(e) => Err(ParseError::Fs(e)),
            }
        });

        // TODO: This allocs alot
        let parsed = join_all(futures).await;

        parsed.into_iter().collect()
    }

    // #[derive(Debug, Error)]
    //     pub enum CpuError {
    //         #[error("Error occured during parsing: {0}")]
    //         Parse(csv::cpu::Error),
    //         #[error("Found multiple _cpu.csv files in .smv: {0:?}")]
    //         MultipleEntries(Vec<String>),
    //         // #[error()]
    //         // NoEntry,
    //     }

    //     async fn csv_cpu(&self) -> Result< {
    //         let files = self.smv.csv_files["cpu"];
    //         match files[..] {
    //             [a] => {},
    //             _ => {},
    //         }
    //         if files.len() > 1 {
    //             return Err(CpuError::MultipleEntries(files));
    //         }

    //         let stuff = self.smv.csv_files["cpu"].iter().map(|x| {
    //             self.csv_cpu_from_name(x)
    //         });
    //     }

    // async fn csv_cpu(&self) -> Result<Option<Vec<CpuData>>, ParseError<Fs::Error, csv::cpu::Error>> {
    //     let Some(files) = self.smv.csv_files.get("cpu") else {
    //         return Ok(None);
    //     };

    //     // use futures::stream::TryStreamExt;
    //     // let stuff = futures::stream::iter(files)
    //     //     .map(|x| async move { self.csv_cpu_from_name(&x).await })
    //     //     .collect();

    //     let mut data = Vec::new();

    //     for file in files {
    //         data.extend(self.csv_cpu_from_name(file).await?);
    //     }

    //     // let test = Self::tsrt(stuff);

    //     // let stuff = futures::stream::iter(stuff)
    //     //     .try_flatten();

    //     Ok(Some(data))
    // }

    // fn tsrt<S: Stream>(s: S) -> S::Item { () }

    async fn csv_cpu_from_name(
        &self,
        // name: &str,
    ) -> Result<Option<Vec<CpuData>>, ParseError<Fs::Error, csv::cpu::Error>> {
        // Ok(self
        //     .csv("cpu", CpuData::from_reader)
        //     .await?
        //     .into_iter()
        //     .flatten()
        //     .collect())
        let file_name = format!("{}_cpu.csv", self.chid);
        if !self.exists(&file_name).await.map_err(ParseError::Fs)? {
            return Ok(None);
        }
        let file = self.read(&file_name).await.map_err(ParseError::Fs)?;
        let data = CpuData::from_reader(file).map_err(ParseError::Parse)?;
        Ok(Some(data))
    }

    // async fn csv_hrr(&self) -> Result<Option<Vec<HRRStep>>, ParseError<Fs::Error, csv::hrr::Error>> {
    //     let Some(files) = self.smv.csv_files.get("hrr") else {
    //         return Ok(None);
    //     };

    //     let mut data = Vec::new();

    //     for file in files {
    //         data.extend(self.csv_hrr_from_name(file).await?);
    //     }

    //     Ok(Some(data))
    // }

    async fn csv_hrr_from_name(
        &self,
        // name: &str,
    ) -> Result<Vec<HRRStep>, ParseError<Fs::Error, csv::hrr::Error>> {
        Ok(self
            .csv("hrr", HRRStep::from_reader)
            .await?
            .into_iter()
            .flatten()
            .collect())
    }

    // async fn csv_devc(&self) -> Result<Option<Vec<HRRStep>>, ParseError<Fs::Error, csv::devc::Error>> {
    //     let Some(files) = self.smv.csv_files.get("devc") else {
    //         return Ok(None);
    //     };

    //     let mut data = Vec::new();

    //     for file in files {
    //         data.extend(self.csv_hrr_from_name(file).await?);
    //     }

    //     Ok(Some(data))
    // }

    async fn csv_devc_from_name(
        &self,
        // name: &str,
    ) -> Result<Vec<Devices>, ParseError<Fs::Error, csv::devc::Error>> {
        self.csv("devc", Devices::from_reader).await
    }
}

// #[derive(Default)]
// struct MemFs {
//     content: HashMap<String, Vec<u8>>,
// }

// #[derive(Debug, Error)]
// enum MemFsErr {
//     #[error("No file with path {0} found")]
//     KeyNotFound(String),
// }

// #[async_trait]
// impl FileSystem for MemFs {
//     type Path = String;
//     type PathRef = str;
//     type Error = MemFsErr;
//     type File<'a> = &'a [u8];

//     async fn read<'this: 'file, 'file>(&'this self, path: &'this Self::PathRef) -> Result<Self::File<'file>, Self::Error> {
//         match self.content.get(path) {
//             Some(file) => Ok(&file[..]),
//             None => Err(MemFsErr::KeyNotFound(path.to_string())),
//         }
//     }

//     fn file_path(&self, directory: &Self::PathRef, file_name: &str) -> Self::Path {
//         let mut path = directory.to_string();
//         path += "/";
//         path += file_name;
//         path
//     }
// }

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{OsFs, Simulation};

    fn root_path() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("demo-house")
    }

    async fn sim() -> Simulation<OsFs> {
        Simulation::parse(OsFs, root_path(), "DemoHaus2".to_string())
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn smv() {
        let sim = sim().await;
        assert_eq!(sim.smv.chid, "DemoHaus2");
    }

    #[tokio::test]
    async fn csv() {
        let sim = sim().await;
        let cpu = sim.csv_cpu_from_name().await.unwrap();
        let hrr = sim.csv_hrr_from_name().await.unwrap();
        let devc = sim.csv_devc_from_name().await.unwrap();
    }

    #[tokio::test]
    async fn slcf() {
        let sim = sim().await;
        for s in 0..sim.smv.slices.len() {
            let slcf = sim.slice(s).await.unwrap();
        }
    }
}
