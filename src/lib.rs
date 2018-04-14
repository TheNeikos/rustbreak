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
//! There are two helper type aliases `MemoryDatabase` and `FileDatabase`, each backed by their
//! respective backend.
//!
//! Using the `with_deser` and `with_backend` one can switch between the representations one needs.
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

#[cfg(test)]
extern crate tempfile;

mod error;
/// Different storage backends
pub mod backend;
/// Different serialization and deserialization methods one can use
pub mod deser;

use std::sync::{Mutex, RwLock};
use std::fmt::Debug;

use serde::Serialize;
use serde::de::DeserializeOwned;

// use error::{BreakResult, BreakError};
use deser::DeSerializer;
use backend::{Backend, MemoryBackend, FileBackend};

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

        let mut new_data = self.deser.deserialize(&backend.get_data()?[..])
                            .context(error::RustbreakErrorKind::DeserializationError)?;

        ::std::mem::swap(&mut *data, &mut new_data);
        Ok(())
    }

    /// Flush the data structure to the backend
    pub fn sync(&self) -> error::Result<()> {
        use failure::ResultExt;

        let mut backend = self.backend.lock().map_err(|_| error::RustbreakErrorKind::PoisonError)?;
        let data = self.data.write().map_err(|_| error::RustbreakErrorKind::PoisonError)?;

        let ser = self.deser.serialize(&*data)
                    .context(error::RustbreakErrorKind::SerializationError)?;

        backend.put_data(ser.as_bytes())?;
        Ok(())
    }

    /// Create a database from its constituents
    pub fn from_parts(data: Data, backend: Back, deser: DeSer) -> Database<Data, Back, DeSer> {
        Database {
            data: RwLock::new(data),
            backend: Mutex::new(backend),
            deser: deser,
        }
    }

    /// Break a database into its individual parts
    pub fn into_inner(self) -> error::Result<(Data, Back, DeSer)> {
        Ok((self.data.into_inner().map_err(|_| error::RustbreakErrorKind::PoisonError)?,
            self.backend.into_inner().map_err(|_| error::RustbreakErrorKind::PoisonError)?,
            self.deser))
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

/// A database backed by a `Vec<u8>`
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
            backend: Mutex::new(MemoryBackend::new()),
            deser: deser,
        }
    }
}

impl<Data, Back, DeSer> Database<Data, Back, DeSer>
    where
        Data: Serialize + DeserializeOwned + Debug + Clone + Send,
        Back: Backend,
        DeSer: DeSerializer<Data> + Send + Sync
{
    /// Exchanges the DeSerialization strategy with the given one
    pub fn with_deser<T>(self, deser: T) -> Database<Data, Back, T>
        where T: DeSerializer<Data> + Send + Sync
    {
        Database {
            backend: self.backend,
            data: self.data,
            deser: deser,
        }
    }
}

impl<Data, Back, DeSer> Database<Data, Back, DeSer>
    where
        Data: Serialize + DeserializeOwned + Debug + Clone + Send,
        Back: Backend,
        DeSer: DeSerializer<Data> + Send + Sync
{
    /// Exchanges the Backend with the given one
    pub fn with_backend<T>(self, backend: T) -> Database<Data, T, DeSer>
        where T: Backend
    {
        Database {
            backend: Mutex::new(backend),
            data: self.data,
            deser: self.deser,
        }
    }
}
