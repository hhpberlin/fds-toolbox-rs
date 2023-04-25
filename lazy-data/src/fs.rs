use std::path::Path;

use std::io::Read;

use async_trait::async_trait;
use fds_toolbox_core::file::FileSystem;
use thiserror::Error;

use std;

use fds_toolbox_core::file::OsFs;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AnyFs {
    LocalFs(OsFs),
    // TODO: Add sftp, rescale, etc.
}

#[derive(Debug, Error)]
#[error(transparent)]
pub enum FsErr {
    Io(std::io::Error),
}

#[async_trait]
impl FileSystem for AnyFs {
    // RIP non-utf8 paths.
    // Probably will never be a problem, but fixing it would be nice.
    // TODO: Fix
    type Path = String;
    type PathRef = str;
    // TODO: Make an enum of all the possible types instead of dyn.
    type Error = FsErr;
    // TODO: Make an enum of all the possible types instead of dyn.
    type File = Box<dyn Read>;

    async fn read(&self, path: &Self::PathRef) -> Result<Self::File, Self::Error> {
        match self {
            AnyFs::LocalFs(fs) => match fs.read(Path::new(path)).await {
                Ok(file) => Ok(Box::new(file)),
                Err(err) => Err(FsErr::Io(err)),
            },
        }
    }
    async fn exists(&self, path: &Self::PathRef) -> Result<bool, Self::Error> {
        match self {
            AnyFs::LocalFs(fs) => fs.exists(Path::new(path)).await.map_err(FsErr::Io),
        }
    }

    fn file_path(&self, directory: &Self::PathRef, file_name: &str) -> Self::Path {
        match self {
            AnyFs::LocalFs(fs) => fs
                .file_path(Path::new(directory), file_name)
                .to_str()
                .expect("Non-UTF8 paths are currently not supported.")
                .to_string(),
        }
    }
}
