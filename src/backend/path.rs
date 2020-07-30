/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Module which implements the [`PathBackend`], storing data in a file on the
//! file system (with a path) and featuring atomic saves.

use super::Backend;
use crate::error;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
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
    /// Errors when the file doesn't yet exist.
    pub fn from_path_or_fail(path: PathBuf) -> error::BackendResult<Self> {
        OpenOptions::new().read(true).open(path.as_path())?;
        Ok(Self { path })
    }

    /// Opens a new [`PathBackend`] for a given path.
    /// Creates a file if it doesn't yet exist.
    ///
    /// Returns the [`PathBackend`] and whether the file already existed.
    pub fn from_path_or_create(path: PathBuf) -> error::BackendResult<(Self, bool)> {
        let exists = path.as_path().is_file();
        OpenOptions::new()
            .write(true)
            .create(true)
            .open(path.as_path())?;
        Ok((Self { path }, exists))
    }

    /// Opens a new [`PathBackend`] for a given path.
    /// Creates a file if it doesn't yet exist, and calls `closure` with it.
    pub fn from_path_or_create_and<C>(path: PathBuf, closure: C) -> error::BackendResult<Self>
    where
        C: FnOnce(&mut std::fs::File),
    {
        let exists = path.as_path().is_file();
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path.as_path())?;
        if !exists {
            closure(&mut file)
        }
        Ok(Self { path })
    }
}

impl Backend for PathBackend {
    fn get_data(&mut self) -> error::BackendResult<Vec<u8>> {
        use std::io::Read;

        let mut file = OpenOptions::new().read(true).open(self.path.as_path())?;
        let mut buffer = vec![];
        file.read_to_end(&mut buffer)?;
        Ok(buffer)
    }

    /// Write the byte slice to the backend. This uses and atomic save.
    ///
    /// This won't corrupt the existing database file if the program panics
    /// during the save.
    fn put_data(&mut self, data: &[u8]) -> error::BackendResult<()> {
        use std::io::Write;

        #[allow(clippy::or_fun_call)] // `Path::new` is a zero cost conversion
        let mut tempf = NamedTempFile::new_in(self.path.parent().unwrap_or(Path::new(".")))?;
        tempf.write_all(data)?;
        tempf.as_file().sync_all()?;
        tempf.persist(self.path.as_path())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{Backend, PathBackend};
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_path_backend_existing() {
        let file = NamedTempFile::new().expect("could not create temporary file");
        let (mut backend, existed) = PathBackend::from_path_or_create(file.path().to_owned())
            .expect("could not create backend");
        assert!(existed);
        let data = [4, 5, 1, 6, 8, 1];

        backend.put_data(&data).expect("could not put data");
        assert_eq!(backend.get_data().expect("could not get data"), data);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_path_backend_new() {
        let dir = tempfile::tempdir().expect("could not create temporary directory");
        let mut file_path = dir.path().to_owned();
        file_path.push("rustbreak_path_db.db");
        let (mut backend, existed) =
            PathBackend::from_path_or_create(file_path).expect("could not create backend");
        assert!(!existed);
        let data = [4, 5, 1, 6, 8, 1];

        backend.put_data(&data).expect("could not put data");
        assert_eq!(backend.get_data().expect("could not get data"), data);
        dir.close().expect("Error while deleting temp directory!");
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_path_backend_nofail() {
        let file = NamedTempFile::new().expect("could not create temporary file");
        let file_path = file.path().to_owned();
        let mut backend = PathBackend::from_path_or_fail(file_path).expect("should not fail");
        let data = [4, 5, 1, 6, 8, 1];

        backend.put_data(&data).expect("could not put data");
        assert_eq!(backend.get_data().expect("could not get data"), data);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_path_backend_fail_notfound() {
        let dir = tempfile::tempdir().expect("could not create temporary directory");
        let mut file_path = dir.path().to_owned();
        file_path.push("rustbreak_path_db.db");
        let err =
            PathBackend::from_path_or_fail(file_path).expect_err("should fail with file not found");
        if let crate::error::BackendError::Io(io_err) = &err {
            assert_eq!(std::io::ErrorKind::NotFound, io_err.kind());
        } else {
            panic!("Wrong kind of error returned: {}", err);
        };
        dir.close().expect("Error while deleting temp directory!");
    }

    // If the file already exists, the closure shouldn't be called.
    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_path_backend_create_and_existing_nocall() {
        let file = NamedTempFile::new().expect("could not create temporary file");
        let mut backend = PathBackend::from_path_or_create_and(file.path().to_owned(), |_| {
            panic!("Closure called but file already existed");
        })
        .expect("could not create backend");
        let data = [4, 5, 1, 6, 8, 1];

        backend.put_data(&data).expect("could not put data");
        assert_eq!(backend.get_data().expect("could not get data"), data);
    }

    // If the file does not yet exist, the closure should be called.
    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_path_backend_create_and_new() {
        let dir = tempfile::tempdir().expect("could not create temporary directory");
        let mut file_path = dir.path().to_owned();
        file_path.push("rustbreak_path_db.db");
        let mut backend = PathBackend::from_path_or_create_and(file_path, |f| {
            f.write_all(b"this is a new file")
                .expect("could not write to file")
        })
        .expect("could not create backend");
        assert_eq!(
            backend.get_data().expect("could not get data"),
            b"this is a new file"
        );
        let data = [4, 5, 1, 6, 8, 1];

        backend.put_data(&data).expect("could not put data");
        assert_eq!(backend.get_data().expect("could not get data"), data);
        dir.close().expect("Error while deleting temp directory!");
    }
}
