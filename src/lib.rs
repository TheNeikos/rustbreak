/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![deny(
    missing_docs,
    non_camel_case_types,
    non_snake_case,
    path_statements,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_allocation,
    unused_import_braces,
    unused_imports,
    unused_must_use,
    unused_mut,
    while_true,
)]

//! # Rustbreak
//!
//! Rustbreak is a [Daybreak][daybreak] inspiried single file Database.
//!
//! You will find an overview here in the docs, but to give you a more complete tale of how this is
//! used please check the [examples][examples].
//!
//! At its core, Rustbreak is an attempt at making a configurable key-value store Database.
//! It features the possibility of:
//!
//! - Choosing what kind of Data is stored in it
//! - Which kind of Serialization is used for persistence
//! - Which kind of persistence is used
//!
//! Per default these options are used:
//!
//! - The Serialization is [Ron][ron], a familiar notation for Rust
//! - The persistence is in-memory, allowing for quick prototyping
//!
//! Later in the development process, the Serialization and the Persistence can be exchanged without
//! breaking the code, allowing you to be flexible.
//!
//! If you have any questions feel free to ask at the main [repo][repo].
//!
//! ## Quickstart
//!
//! ```rust
//! # use std::collections::HashMap;
//! use rustbreak::{MemoryDatabase, deser::Ron};
//!
//! let db = MemoryDatabase::<HashMap<String, String>, Ron>::memory(HashMap::new(), Ron);
//!
//! println!("Writing to Database");
//! db.write(|db| {
//!     db.insert("hello".into(), String::from("world"));
//!     db.insert("foo".into(), String::from("bar"));
//! });
//!
//! db.read(|db| {
//!     // db.insert("foo".into(), String::from("bar"));
//!     // The above line will not compile since we are only reading
//!     println!("Hello: {:?}", db.get("hello"));
//! });
//! ```
//!
//! [daybreak]:https://propublica.github.io/daybreak
//! [examples]: https://github.com/TheNeikos/rustbreak/tree/master/examples
//! [ron]: https://github.com/ron-rs/ron


extern crate serde;
extern crate ron;
#[macro_use] extern crate failure;

#[cfg(feature = "yaml")]
extern crate serde_yaml;

#[cfg(feature = "bin")]
extern crate bincode;
#[cfg(feature = "bin")]
extern crate base64;

mod error;
/// Different serialization and deserialization methods one can use
pub mod deser;

use std::io::{Read, Write};
use std::sync::{Mutex, RwLock};
use std::fmt::Debug;

use serde::Serialize;
use serde::de::DeserializeOwned;

// use error::{BreakResult, BreakError};
use deser::DeSerializer;

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
    file: std::fs::File
}

impl Backend for FileBackend {
    fn get_data(&mut self) -> error::Result<Vec<u8>> {
        use std::io::{Seek, SeekFrom};

        let mut buffer = vec![];
        self.file.seek(SeekFrom::Start(0))?;
        self.file.read_to_end(&mut buffer)?;
        Ok(buffer)
    }

    fn put_data(&mut self, data: &[u8]) -> error::Result<()> {
        use std::io::{Seek, SeekFrom};

        self.file.seek(SeekFrom::Start(0))?;
        self.file.set_len(0)?;
        self.file.write_all(data)?;
        Ok(())
    }
}

impl FileBackend {
    /// Opens a new FileBackend for a given path
    pub fn open<P: AsRef<std::path::Path>>(path: P) -> error::Result<FileBackend> {
        use std::fs::OpenOptions;

        Ok(FileBackend {
            file: OpenOptions::new().read(true).write(true).create(true).open(path)?,
        })
    }

    /// Uses an already open File
    pub fn from_file(file: std::fs::File) -> FileBackend {
        FileBackend {
            file: file
        }
    }

    /// Return the inner File
    pub fn into_inner(self) -> std::fs::File {
        self.file
    }
}

/// The Central Database to RustBreak
///
/// It has 5 Type Generics:
///
/// - V: Is the Data, you must specify this (usually inferred by the compiler)
/// - S: The Serializer/Deserializer or short DeSer. Per default `deser::Ron` is used. Check the
///     `deser` module for other strategies.
/// - F: The storage backend. Per default it is in memory, but can be easily used with a `File`.
#[derive(Debug)]
pub struct Database<Data, Back, DeSer>
    where
        Data: Serialize + DeserializeOwned + Debug + Clone + Send,
        Back: Backend,
        DeSer: DeSerializer<Data> + Send + Sync
{
    data: RwLock<Data>,
    backend: Mutex<Back>,
    deser: DeSer
}

impl<Data, Back, DeSer> Database<Data, Back, DeSer>
    where
        Data: Serialize + DeserializeOwned + Debug + Clone + Send,
        Back: Backend,
        DeSer: DeSerializer<Data> + Send + Sync
{
    /// Write lock the database and get write access to the `Data` container
    pub fn write<T>(&self, task: T) -> error::Result<()>
        where T: FnOnce(&mut Data)
    {
        let mut lock = self.data.write().map_err(|_| error::RustbreakErrorKind::PoisonError)?;
        task(&mut lock);
        Ok(())
    }

    /// Read lock the database and get write access to the `Data` container
    pub fn read<T, R>(&self, task: T) -> error::Result<R>
        where T: FnOnce(&Data) -> R
    {
        let mut lock = self.data.read().map_err(|_| error::RustbreakErrorKind::PoisonError)?;
        Ok(task(&mut lock))
    }

    /// Reload the Data from the backend
    pub fn reload(&self) -> error::Result<()> {
        use failure::ResultExt;

        let mut backend = self.backend.lock().map_err(|_| error::RustbreakErrorKind::PoisonError)?;
        let mut data = self.data.write().map_err(|_| error::RustbreakErrorKind::PoisonError)?;

        let mut new_data = match self.deser.deserialize(&backend.get_data()?[..]) {
            Ok(s) => s,
            Err(e) => Err(e).context(error::RustbreakErrorKind::DeserializationError)?
        };

        ::std::mem::swap(&mut *data, &mut new_data);
        Ok(())
    }

    /// Flush the data structure to the backend
    pub fn sync(&self) -> error::Result<()> {
        use failure::ResultExt;

        let mut backend = self.backend.lock().map_err(|_| error::RustbreakErrorKind::PoisonError)?;
        let data = self.data.write().map_err(|_| error::RustbreakErrorKind::PoisonError)?;

        let ser = match self.deser.serialize(&*data) {
            Ok(s) => s,
            Err(e) => Err(e).context(error::RustbreakErrorKind::SerializationError)?
        };
        backend.put_data(ser.as_bytes())?;
        Ok(())
    }
}

/// A database backed by a file
pub type FileDatabase<D, DS> = Database<D, FileBackend, DS>;

impl<Data, DeSer> Database<Data, FileBackend, DeSer>
    where
        Data: Serialize + DeserializeOwned + Debug + Clone + Send,
        DeSer: DeSerializer<Data> + Send + Sync
{
    /// Create new FileDatabase from Path
    pub fn from_path<S>(data: Data, deser: DeSer, path: S)
        -> error::Result<FileDatabase<Data, DeSer>>
        where S: AsRef<std::path::Path>
    {
        let b = FileBackend::open(path)?;

        Ok(Database {
            data: RwLock::new(data),
            backend: Mutex::new(b),
            deser: deser,
        })
    }
}

/// An in memory backend
///
/// It is backed by a `Vec<u8>`
pub struct MemoryBackend(Vec<u8>);

impl Backend for MemoryBackend {
    fn get_data(&mut self) -> error::Result<Vec<u8>> {
        Ok(self.0.clone())
    }

    fn put_data(&mut self, data: &[u8]) -> error::Result<()> {
        self.0 = data.to_owned();
        Ok(())
    }
}

/// A database backed by a file
pub type MemoryDatabase<D, DS> = Database<D, MemoryBackend, DS>;

impl<Data, DeSer> Database<Data, MemoryBackend, DeSer>
    where
        Data: Serialize + DeserializeOwned + Debug + Clone + Send,
        DeSer: DeSerializer<Data> + Send + Sync
{
    /// Create new FileDatabase from Path
    pub fn memory(data: Data, deser: DeSer) -> MemoryDatabase<Data, DeSer> {
        Database {
            data: RwLock::new(data),
            backend: Mutex::new(MemoryBackend(vec![])),
            deser: deser,
        }
    }
}
