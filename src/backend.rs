/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use error;

/// The Backend Trait
///
/// This trait describes a simple backend, allowing users to swap it, or
/// to implement one themselves
pub trait Backend {
    /// This method gets the data from the backend
    fn get_data(&mut self) -> error::Result<Vec<u8>>;

    /// This method
    fn put_data(&mut self, data: &[u8]) -> error::Result<()>;
}

/// A backend using a file
pub struct FileBackend {
    file: ::std::fs::File
}

impl Backend for FileBackend {
    fn get_data(&mut self) -> error::Result<Vec<u8>> {
        use ::std::io::{Seek, SeekFrom, Read};

        let mut buffer = vec![];
        self.file.seek(SeekFrom::Start(0))?;
        self.file.read_to_end(&mut buffer)?;
        Ok(buffer)
    }

    fn put_data(&mut self, data: &[u8]) -> error::Result<()> {
        use ::std::io::{Seek, SeekFrom, Write};

        self.file.seek(SeekFrom::Start(0))?;
        self.file.set_len(0)?;
        self.file.write_all(data)?;
        Ok(())
    }
}

impl FileBackend {
    /// Opens a new FileBackend for a given path
    pub fn open<P: AsRef<::std::path::Path>>(path: P) -> error::Result<FileBackend> {
        use ::std::fs::OpenOptions;

        Ok(FileBackend {
            file: OpenOptions::new().read(true).write(true).create(true).open(path)?,
        })
    }

    /// Uses an already open File
    pub fn from_file(file: ::std::fs::File) -> FileBackend {
        FileBackend {
            file: file
        }
    }

    /// Return the inner File
    pub fn into_inner(self) -> ::std::fs::File {
        self.file
    }
}

/// An in memory backend
///
/// It is backed by a `Vec<u8>`
pub struct MemoryBackend(pub(crate) Vec<u8>);

impl Backend for MemoryBackend {
    fn get_data(&mut self) -> error::Result<Vec<u8>> {
        Ok(self.0.clone())
    }

    fn put_data(&mut self, data: &[u8]) -> error::Result<()> {
        self.0 = data.to_owned();
        Ok(())
    }
}
