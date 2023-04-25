use std::{borrow::Borrow, collections::HashMap, error::Error, fmt::Debug, hash::Hash, io::Read};

use async_trait::async_trait;
use derive_more::Constructor;
use futures::future::join_all;
use get_size::GetSize;
use ndarray::Ix3;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    common::series::{TimeSeries, TimeSeriesSourceAsync},
    formats::{
        csv::{self, cpu::CpuData, devc::DeviceList, hrr::HrrStep},
        smoke::dim2::slice::{self, Slice},
        smv::{self, Smv},
    },
    geom::Bounds3I,
};

#[async_trait]
pub trait FileSystem: Send + Sync + 'static {
    type Path: Borrow<Self::PathRef> + Send + Sync + Debug + Eq + Hash;
    type PathRef: ?Sized + Send + Sync + Debug + Eq + Hash;
    type Error: Error + Send + Sync + 'static;
    type File: Read;

    async fn read(&self, path: &Self::PathRef) -> Result<Self::File, Self::Error>;
    async fn exists(&self, path: &Self::PathRef) -> Result<bool, Self::Error>;

    fn file_path(&self, directory: &Self::PathRef, file_name: &str) -> Self::Path;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

trait Parse: Sized {
    type Error;
    type Warning;

    fn parse(file: impl Read, warn: MaybeFn<Self::Warning>) -> Result<Self, Self::Error>;
}

#[derive(Error, Debug)]
pub enum ParseError<FsErr: Error, ParseErr: Error> {
    #[error("File system error: {0}")]
    Fs(FsErr),
    #[error("IO error: {0}")]
    Io(std::io::Error),
    #[error("Parse error: {0}")]
    Parse(ParseErr),
}

impl<FsErr: Error, ParseErr: Error> ParseError<FsErr, ParseErr> {
    pub fn map_parse_err<E: Error>(self, f: impl FnOnce(ParseErr) -> E) -> ParseError<FsErr, E> {
        match self {
            Self::Fs(e) => ParseError::Fs(e),
            Self::Io(e) => ParseError::Io(e),
            Self::Parse(e) => ParseError::Parse(f(e)),
        }
    }
}

type MaybeFn<T> = Option<Box<dyn Fn(T)>>;

#[derive(Debug)]
pub struct Simulation<Fs: FileSystem> {
    /// The path to the simulation files
    pub path: SimulationPath<Fs>,
    /// The simulation metadata from the .smv file
    pub smv: Smv,
    /// Maps a slices mesh-index and bounds to the index in the `smv.slices` array
    slice_index: HashMap<(i32, Bounds3I), usize>,
}

// I don't want to restrict `Fs` to be `GetSize` on the struct itself
// and the derive macro doesn't properly do this itself.
impl<Fs> GetSize for Simulation<Fs>
where
    Fs: FileSystem + GetSize,
    Fs::Path: GetSize,
{
    fn get_heap_size(&self) -> usize {
        self.path.get_heap_size() + self.smv.get_heap_size() + self.slice_index.get_heap_size()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Constructor)]
pub struct SimulationPath<Fs: FileSystem> {
    /// The file system used to read the simulation files
    pub fs: Fs,
    /// The directory containing the simulation files
    pub directory: Fs::Path,
    /// The simulation id
    pub chid: String,
}

impl<Fs: FileSystem> SimulationPath<Fs> {
    pub fn map<NewFs: FileSystem>(
        self,
        f: impl FnOnce(Fs) -> NewFs,
        fd: impl FnOnce(Fs::Path) -> NewFs::Path,
    ) -> SimulationPath<NewFs> {
        SimulationPath {
            fs: f(self.fs),
            directory: fd(self.directory),
            chid: self.chid,
        }
    }
}

impl<Fs> GetSize for SimulationPath<Fs>
where
    Fs: FileSystem + GetSize,
    Fs::Path: GetSize,
{
    fn get_heap_size(&self) -> usize {
        self.chid.get_heap_size() + self.directory.get_heap_size() + self.fs.get_heap_size()
    }
}

#[derive(Debug)]
struct IndexedVec<K, V> {
    index: HashMap<K, usize>,
    vec: Vec<V>,
}

impl<K: Eq + Hash, V> IndexedVec<K, V> {
    fn new() -> Self {
        Self {
            index: HashMap::new(),
            vec: Vec::new(),
        }
    }

    fn from_vec(vec: Vec<V>, key_src: impl Fn(&V) -> K) -> Self {
        let index = vec
            .iter()
            .enumerate()
            .map(|(i, v)| (key_src(v), i))
            .collect();

        Self { index, vec }
    }

    fn get(&self, key: &K) -> Option<&V> {
        self.index.get(key).copied().map(|i| &self.vec[i])
    }

    fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.index.get(key).copied().map(move |i| &mut self.vec[i])
    }

    fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.index.iter().map(move |(k, &i)| (k, &self.vec[i]))
    }
}

#[derive(Debug, Error)]
#[error(transparent)]
pub enum SmvErr {
    Normal(smv::Error),
    // TODO: Find a way to pass the fancy errors around; `miette::Report`
    //       does not implement `Error` and therefore cannot be used as an error type.
    // FancyMiette(miette::Report),
}

impl<Fs: FileSystem> SimulationPath<Fs> {
    pub async fn parse(self) -> Result<Simulation<Fs>, ParseError<Fs::Error, SmvErr>> {
        Simulation::parse(self.fs, self.directory, self.chid).await
    }
}

impl<Fs: FileSystem> Simulation<Fs> {
    pub async fn parse(
        fs: Fs,
        directory: Fs::Path,
        chid: String,
    ) -> Result<Self, ParseError<Fs::Error, SmvErr>> {
        // & doesn't seem to infer the type properly, .borrow() does (PathBuf -> &Path instead &PathBuf)
        let path = fs.file_path(directory.borrow(), &format!("{}.smv", chid));

        Self::parse_core(fs, directory, Some(chid), path.borrow()).await
    }

    pub async fn parse_smv(
        fs: Fs,
        directory: Fs::Path,
        smv: &Fs::PathRef,
    ) -> Result<Self, ParseError<Fs::Error, SmvErr>> {
        Self::parse_core(fs, directory, None, smv).await
    }

    async fn parse_core(
        fs: Fs,
        directory: Fs::Path,
        chid: Option<String>,
        smv: &Fs::PathRef,
    ) -> Result<Self, ParseError<Fs::Error, SmvErr>> {
        let mut file = fs.read(smv).await.map_err(ParseError::Fs)?;

        // TODO: Use actual file size to pre-allocate string
        // let size = file.metadata().map(|m| m.len()).unwrap_or(0);
        let size = 0;
        let mut string = String::with_capacity(size as usize);
        file.read_to_string(&mut string).map_err(ParseError::Io)?;
        drop(file);

        let smv = Smv::parse(&string).map_err(|e| {
            // TODO: This is a hack to log errors at all, it should be cleanly passed up the stack or otherwise handled properly
            //       Maybe log via tracing? Not optimal since it doesn't strongly associate the error with the fileslice_index(mesh_index, bounds)
            // eprintln!("{:?}", e.add_src(string));

            ParseError::Parse(SmvErr::Normal(e))
        })?;
        // .map_err(|e| ParseError::Parse(SmvErr::FancyMiette(e.add_src(string))))?;

        let chid = match chid {
            Some(chid) => {
                // TODO: Add proper error handling
                debug_assert_eq!(smv.chid, chid);
                chid
            }
            None => smv.chid.clone(),
        };

        let slice_index = smv
            .slices
            .iter()
            .enumerate()
            .map(|(i, slice)| ((slice.mesh_index, slice.bounds), i))
            .collect();

        let path = SimulationPath::new(fs, directory, chid);

        Ok(Self {
            smv,
            path,
            slice_index,
        })
    }

    async fn read(&self, file_name: &str) -> Result<Fs::File, Fs::Error> {
        self.path.fs.read(self.path(file_name).borrow()).await
    }

    async fn exists(&self, file_name: &str) -> Result<bool, Fs::Error> {
        self.path.fs.exists(self.path(file_name).borrow()).await
    }

    fn path(&self, file_name: &str) -> <Fs as FileSystem>::Path {
        self.path
            .fs
            .file_path(self.path.directory.borrow(), file_name)
    }

    pub async fn slice(&self, idx: usize) -> Result<Slice, ParseError<Fs::Error, slice::Error>> {
        let slice = &self.smv.slices[idx];
        let file = self.read(&slice.file_name).await.map_err(ParseError::Fs)?;
        Slice::from_reader(file).map_err(ParseError::Parse)
    }

    pub fn slice_index(&self, mesh_index: i32, bounds: Bounds3I) -> Option<usize> {
        self.slice_index.get(&(mesh_index, bounds)).copied()
    }

    // pub async fn smoke3d(&self, idx: usize) -> Result<Smoke3D, ParseError<Fs::Error, s3d::Error>> {
    //     let s3d = &self.smv.smoke3d[idx];
    //     let file = self.read(&s3d.file_name).await.map_err(ParseError::Fs)?;
    //     Smoke3D::from_reader(file).map_err(ParseError::Parse)
    // }

    // pub async fn plot3d(
    //     &self,
    //     idx: usize,
    // ) -> Result<Plot3D, ParseError<Fs::Error, plot3d::Error>> {
    //     let plot3d = &self.smv.plot3d[idx];
    //     let file = self.read(&plot3d.file_name).await.map_err(ParseError::Fs)?;
    //     Plot3D::from_reader(file).map_err(ParseError::Parse)
    // }

    // async fn s3d(&self, idx: usize) {
    //     let s3d = &self.smv.smoke3d[idx];
    //     let file = self.file(&s3d.file_name).await.map_err(ParseError::Fs)?;

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

    pub async fn csv_cpu(&self) -> Result<Option<CpuData>, ParseError<Fs::Error, csv::cpu::Error>> {
        let file_name = format!("{}_cpu.csv", self.path.chid);
        if !self.exists(&file_name).await.map_err(ParseError::Fs)? {
            return Ok(None);
        }
        let file = self.read(&file_name).await.map_err(ParseError::Fs)?;
        let data = CpuData::from_reader(file).map_err(ParseError::Parse)?;
        Ok(Some(data))
    }

    pub async fn csv_hrr(&self) -> Result<Vec<HrrStep>, ParseError<Fs::Error, csv::hrr::Error>> {
        Ok(self
            .csv("hrr", HrrStep::from_reader)
            .await?
            .into_iter()
            .flatten()
            .collect())
    }

    pub async fn csv_devc(&self) -> Result<DeviceList, ParseError<Fs::Error, csv::devc::Error>> {
        let device_lists = self
            .csv("devc", DeviceList::from_reader)
            .await
            .map_err(|e| e.map_parse_err(csv::devc::Error::ParsingError))?;

        DeviceList::merge(device_lists)
            .map_err(|e| ParseError::Parse(csv::devc::Error::JoinError(e)))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SliceSeriesIdx(pub usize);

#[async_trait]
impl<Fs: FileSystem> TimeSeriesSourceAsync<SliceSeriesIdx, f32, Ix3> for Simulation<Fs> {
    type Error = ParseError<Fs::Error, slice::Error>;

    async fn get_time_series(
        &self,
        id: SliceSeriesIdx,
    ) -> Result<TimeSeries<f32, Ix3, f32>, Self::Error> {
        Ok(self.slice(id.0).await?.data)
    }
}

pub struct HrrIdx;

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
        let _cpu = sim.csv_cpu().await.unwrap();
        let _hrr = sim.csv_hrr().await.unwrap();
        let _devc = sim.csv_devc().await.unwrap();
    }

    #[tokio::test]
    async fn slcf() {
        let sim = sim().await;
        for s in 0..sim.smv.slices.len() {
            let _slcf = sim.slice(s).await.unwrap();
        }
    }
}
