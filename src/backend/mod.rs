/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The persistence backends of the Database.
//!
//! A file is a `Backend` through the `FileBackend`, so is a `Vec<u8>` with a
//! `MemoryBackend`.
//!
//! Implementing your own Backend should be straightforward. Check the `Backend`
//! documentation for details.

use crate::error;

/// The Backend Trait.
///
/// It should always read and save in full the data that it is passed. This
/// means that a write to the backend followed by a read __must__ return the
/// same dataset.
///
/// **Important**: You can only return custom errors if the `other_errors` feature is enabled
pub trait Backend {
    /// Read the all data from the backend.
    fn get_data(&mut self) -> error::BackendResult<Vec<u8>>;

    /// Write the whole slice to the backend.
    fn put_data(&mut self, data: &[u8]) -> error::BackendResult<()>;
}

impl Backend for Box<dyn Backend> {
    fn get_data(&mut self) -> error::BackendResult<Vec<u8>> {
        use std::ops::DerefMut;
        self.deref_mut().get_data()
    }

    fn put_data(&mut self, data: &[u8]) -> error::BackendResult<()> {
        use std::ops::DerefMut;
        self.deref_mut().put_data(data)
    }
}

impl<T: Backend> Backend for Box<T> {
    fn get_data(&mut self) -> error::BackendResult<Vec<u8>> {
        use std::ops::DerefMut;
        self.deref_mut().get_data()
    }

    fn put_data(&mut self, data: &[u8]) -> error::BackendResult<()> {
        use std::ops::DerefMut;
        self.deref_mut().put_data(data)
    }
}

#[cfg(feature = "mmap")]
mod mmap;
#[cfg(feature = "mmap")]
pub use mmap::MmapStorage;

mod path;
pub use path::PathBackend;

/// A backend using a file.
#[derive(Debug)]
pub struct FileBackend(std::fs::File);

impl Backend for FileBackend {
    fn get_data(&mut self) -> error::BackendResult<Vec<u8>> {
        use std::io::{Read, Seek, SeekFrom};

        let mut buffer = vec![];
        self.0.seek(SeekFrom::Start(0))?;
        self.0.read_to_end(&mut buffer)?;
        Ok(buffer)
    }

    fn put_data(&mut self, data: &[u8]) -> error::BackendResult<()> {
        use std::io::{Seek, SeekFrom, Write};

        self.0.seek(SeekFrom::Start(0))?;
        self.0.set_len(0)?;
        self.0.write_all(data)?;
        self.0.sync_all()?;
        Ok(())
    }
}

impl FileBackend {
    /// Use an already open [`File`](std::fs::File) as the backend.
    #[must_use]
    pub fn from_file(file: std::fs::File) -> Self {
        Self(file)
    }

    /// Return the inner File.
    #[must_use]
    pub fn into_inner(self) -> std::fs::File {
        self.0
    }
}

impl FileBackend {
    /// Opens a new [`FileBackend`] for a given path.
    /// Errors when the file doesn't yet exist.
    pub fn from_path_or_fail<P: AsRef<std::path::Path>>(path: P) -> error::BackendResult<Self> {
        use std::fs::OpenOptions;

        Ok(Self(OpenOptions::new().read(true).write(true).open(path)?))
    }

    /// Opens a new [`FileBackend`] for a given path.
    /// Creates a file if it doesn't yet exist.
    ///
    /// Returns the [`FileBackend`] and whether the file already existed.
    pub fn from_path_or_create<P: AsRef<std::path::Path>>(
        path: P,
    ) -> error::BackendResult<(Self, bool)> {
        use std::fs::OpenOptions;

        let exists = path.as_ref().is_file();
        Ok((
            Self(
                OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .open(path)?,
            ),
            exists,
        ))
    }

    /// Opens a new [`FileBackend`] for a given path.
    /// Creates a file if it doesn't yet exist, and calls `closure` with it.
    pub fn from_path_or_create_and<P, C>(path: P, closure: C) -> error::BackendResult<Self>
    where
        C: FnOnce(&mut std::fs::File),
        P: AsRef<std::path::Path>,
    {
        Self::from_path_or_create(path).map(|(mut b, exists)| {
            if !exists {
                closure(&mut b.0)
            }
            b
        })
    }
}

/// An in memory backend.
///
/// It is backed by a byte vector (`Vec<u8>`).
#[derive(Debug, Default)]
pub struct MemoryBackend(Vec<u8>);

impl MemoryBackend {
    /// Construct a new Memory Database.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl Backend for MemoryBackend {
    fn get_data(&mut self) -> error::BackendResult<Vec<u8>> {
        println!("Returning data: {:?}", &self.0);
        Ok(self.0.clone())
    }

    fn put_data(&mut self, data: &[u8]) -> error::BackendResult<()> {
        println!("Writing data: {:?}", data);
        self.0 = data.to_owned();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{Backend, FileBackend, MemoryBackend};
    use std::io::{Read, Seek, SeekFrom, Write};
    use tempfile::NamedTempFile;

    #[test]
    fn test_memory_backend() {
        let mut backend = MemoryBackend::new();
        let data = [4, 5, 1, 6, 8, 1];

        backend.put_data(&data).expect("could not put data");
        assert_eq!(backend.get_data().expect("could not get data"), data);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_file_backend_from_file() {
        let file = tempfile::tempfile().expect("could not create temporary file");
        let mut backend = FileBackend::from_file(file);
        let data = [4, 5, 1, 6, 8, 1];
        let data2 = [3, 99, 127, 6];

        backend.put_data(&data).expect("could not put data");
        assert_eq!(backend.get_data().expect("could not get data"), data);

        backend.put_data(&data2).expect("could not put data");
        assert_eq!(backend.get_data().expect("could not get data"), data2);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_file_backend_from_path_existing() {
        let file = NamedTempFile::new().expect("could not create temporary file");
        let (mut backend, existed) =
            FileBackend::from_path_or_create(file.path()).expect("could not create backend");
        assert!(existed);
        let data = [4, 5, 1, 6, 8, 1];

        backend.put_data(&data).expect("could not put data");
        assert_eq!(backend.get_data().expect("could not get data"), data);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_file_backend_from_path_new() {
        let dir = tempfile::tempdir().expect("could not create temporary directory");
        let mut file_path = dir.path().to_owned();
        file_path.push("rustbreak_path_db.db");
        let (mut backend, existed) =
            FileBackend::from_path_or_create(file_path).expect("could not create backend");
        assert!(!existed);
        let data = [4, 5, 1, 6, 8, 1];

        backend.put_data(&data).expect("could not put data");
        assert_eq!(backend.get_data().expect("could not get data"), data);
        dir.close().expect("Error while deleting temp directory!");
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_file_backend_from_path_nofail() {
        let file = NamedTempFile::new().expect("could not create temporary file");
        let file_path = file.path().to_owned();
        let mut backend = FileBackend::from_path_or_fail(file_path).expect("should not fail");
        let data = [4, 5, 1, 6, 8, 1];

        backend.put_data(&data).expect("could not put data");
        assert_eq!(backend.get_data().expect("could not get data"), data);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_file_backend_from_path_fail_notfound() {
        let dir = tempfile::tempdir().expect("could not create temporary directory");
        let mut file_path = dir.path().to_owned();
        file_path.push("rustbreak_path_db.db");
        let err =
            FileBackend::from_path_or_fail(file_path).expect_err("should fail with file not found");
        if let crate::error::BackendError::Io(io_err) = &err {
            assert_eq!(std::io::ErrorKind::NotFound, io_err.kind());
        } else {
            panic!("Wrong kind of error returned: {}", err);
        };
        dir.close().expect("Error while deleting temp directory!");
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_file_backend_into_inner() {
        let file = tempfile::tempfile().expect("could not create temporary file");
        let mut backend = FileBackend::from_file(file);
        let data = [4, 5, 1, 6, 8, 1];

        backend.put_data(&data).expect("could not put data");
        assert_eq!(backend.get_data().expect("could not get data"), data);

        let mut file = backend.into_inner();
        file.seek(SeekFrom::Start(0)).unwrap();
        let mut contents = Vec::new();
        assert_eq!(file.read_to_end(&mut contents).unwrap(), 6);
        assert_eq!(&contents[..], &data[..]);
    }

    #[test]
    fn allow_boxed_backends() {
        let mut backend = Box::new(MemoryBackend::new());
        let data = [4, 5, 1, 6, 8, 1];

        backend.put_data(&data).unwrap();
        assert_eq!(backend.get_data().unwrap(), data);
    }

    // If the file already exists, the closure shouldn't be called.
    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_file_backend_create_and_existing_nocall() {
        let file = NamedTempFile::new().expect("could not create temporary file");
        let mut backend = FileBackend::from_path_or_create_and(file.path(), |_| {
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
    fn test_file_backend_create_and_new() {
        let dir = tempfile::tempdir().expect("could not create temporary directory");
        let mut file_path = dir.path().to_owned();
        file_path.push("rustbreak_path_db.db");
        let mut backend = FileBackend::from_path_or_create_and(file_path, |f| {
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
