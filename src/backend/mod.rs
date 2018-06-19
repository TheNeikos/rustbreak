/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The persistence backends of the Database
//!
//! A file is a `Backend` through the `FileBackend`, so is a `Vec<u8>` with a `MemoryBackend`.
//!
//! Implementing your own Backend should be straightforward. Check the `Backend` documentation for
//! details.

use failure::ResultExt;

use error;

/// The Backend Trait
///
/// It should always read and save in full the data that it is passed. This means that a write to
/// the backend followed by a read __must__ return the same dataset.
pub trait Backend {
    /// Read the all data from the backend
    fn get_data(&mut self) -> error::Result<Vec<u8>>;

    /// Write the whole slice to the backend
    fn put_data(&mut self, data: &[u8]) -> error::Result<()>;
}

#[cfg(feature = "mmap")]
mod mmap;
#[cfg(feature = "mmap")]
pub use self::mmap::MmapStorage;

/// A backend using a file
#[derive(Debug)]
pub struct FileBackend(::std::fs::File);

impl Backend for FileBackend {
    fn get_data(&mut self) -> error::Result<Vec<u8>> {
        use ::std::io::{Seek, SeekFrom, Read};

        let mut buffer = vec![];
        self.0.seek(SeekFrom::Start(0)).context(error::RustbreakErrorKind::Backend)?;
        self.0.read_to_end(&mut buffer).context(error::RustbreakErrorKind::Backend)?;
        Ok(buffer)
    }

    fn put_data(&mut self, data: &[u8]) -> error::Result<()> {
        use ::std::io::{Seek, SeekFrom, Write};

        self.0.seek(SeekFrom::Start(0)).context(error::RustbreakErrorKind::Backend)?;
        self.0.set_len(0).context(error::RustbreakErrorKind::Backend)?;
        self.0.write_all(data).context(error::RustbreakErrorKind::Backend)?;
        self.0.sync_all().context(error::RustbreakErrorKind::Backend)?;
        Ok(())
    }
}

impl FileBackend {
    /// Opens a new FileBackend for a given path, will create it if the file doesn't exist.
    pub fn open<P: AsRef<::std::path::Path>>(path: P) -> error::Result<FileBackend> {
        use ::std::fs::OpenOptions;

        Ok(FileBackend(
            OpenOptions::new().read(true).write(true).create(true).open(path).context(error::RustbreakErrorKind::Backend)?,
        ))
    }

    /// Use an already open File as the backend
    pub fn from_file(file: ::std::fs::File) -> FileBackend {
        FileBackend(file)
    }

    /// Return the inner File
    pub fn into_inner(self) -> ::std::fs::File {
        self.0
    }
}

/// An in memory backend
///
/// It is backed by a `Vec<u8>`
#[derive(Debug)]
pub struct MemoryBackend(Vec<u8>);

impl MemoryBackend {
    /// Construct a new Memory Database
    pub fn new() -> MemoryBackend {
        MemoryBackend(vec![])
    }
}

impl Backend for MemoryBackend {
    fn get_data(&mut self) -> error::Result<Vec<u8>> {
        Ok(self.0.clone())
    }

    fn put_data(&mut self, data: &[u8]) -> error::Result<()> {
        self.0 = data.to_owned();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tempfile;
    use super::{Backend, MemoryBackend, FileBackend};

    #[test]
    fn test_memory_backend() {
        let mut backend = MemoryBackend::new();
        let data = [4, 5, 1, 6, 8, 1];

        backend.put_data(&data).unwrap();
        assert_eq!(backend.get_data().unwrap(), data);
    }

    #[test]
    fn test_file_backend() {
        let file = tempfile::tempfile().unwrap();
        let mut backend = FileBackend::from_file(file);
        let data = [4, 5, 1, 6, 8, 1];

        backend.put_data(&data).unwrap();
        assert_eq!(backend.get_data().unwrap(), data);
    }
}
