/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Module which implements the [`PathBackend`], storing data in a file on the
//! file system (with a path) and featuring atomic saves.

use super::Backend;
use crate::error;
use crate::error::RustbreakErrorKind as ErrorKind;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use failure::ResultExt;
use tempfile::NamedTempFile;

/// A [`Backend`] using a file given the path.
///
/// Features atomic saves, so that the database file won't be corrupted or
/// deleted if the program panics during the save.
#[derive(Debug)]
pub struct PathBackend {
    path: PathBuf,
}

impl PathBackend {
    /// Opens a new [`PathBackend`] for a given path.
    pub fn open(path: PathBuf) -> error::Result<Self> {
        OpenOptions::new().write(true).create(true).open(path.as_path())
            .context(ErrorKind::Backend)?;
        Ok(Self {path})
    }
}

impl Backend for PathBackend {
    fn get_data(&mut self) -> error::Result<Vec<u8>> {
        use ::std::io::Read;

        let mut file = OpenOptions::new().read(true)
            .open(self.path.as_path()).context(ErrorKind::Backend)?;
        let mut buffer = vec![];
        file.read_to_end(&mut buffer).context(ErrorKind::Backend)?;
        Ok(buffer)
    }

    /// Write the byte slice to the backend. This uses and atomic save.
    ///
    /// This won't corrupt the existing database file if the program panics
    /// during the save.
    fn put_data(&mut self, data: &[u8]) -> error::Result<()> {
        use ::std::io::Write;

        #[allow(clippy::or_fun_call)] // `Path::new` is a zero cost conversion
        let mut tempf = NamedTempFile::new_in(self.path.parent().unwrap_or(Path::new(".")))
            .context(ErrorKind::Backend)?;
        tempf.write_all(data).context(ErrorKind::Backend)?;
        tempf.as_file().sync_all().context(ErrorKind::Backend)?;
        tempf.persist(self.path.as_path()).context(ErrorKind::Backend)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tempfile::NamedTempFile;
    use super::{Backend, PathBackend};

    #[test]
    fn test_path_backend_existing() {
        let file = NamedTempFile::new()
            .expect("could not create temporary file");
        let mut backend = PathBackend::open(file.path().to_owned())
            .expect("could not create backend");
        let data = [4, 5, 1, 6, 8, 1];

        backend.put_data(&data).expect("could not put data");
        assert_eq!(backend.get_data().expect("could not get data"), data);
    }

    #[test]
    fn test_path_backend_new() {
        let dir = tempfile::tempdir()
            .expect("could not create temporary directory");
        let mut file_path = dir.path().to_owned();
        file_path.push("rustbreak_path_db.db");
        let mut backend = PathBackend::open(file_path)
            .expect("could not create backend");
        let data = [4, 5, 1, 6, 8, 1];

        backend.put_data(&data).expect("could not put data");
        assert_eq!(backend.get_data().expect("could not get data"), data);
        dir.close().expect("Error while deleting temp directory!");
    }
}
