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
//! Rustbreak is a [Daybreak][daybreak] inspired single file Database.
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
//! Rustbreak uses the [failure][failure] crate. As such, to handle its errors you will need to
//! learn how to use it.
//!
//! If you have any questions feel free to ask at the main [repo][repo].
//!
//! ## Quickstart
//!
//! ```rust
//! # extern crate failure;
//! # extern crate rustbreak;
//! # use std::collections::HashMap;
//! use rustbreak::{MemoryDatabase, deser::Ron};
//!
//! # fn main() {
//! # let func = || -> Result<(), failure::Error> {
//! let db = MemoryDatabase::<HashMap<String, String>, Ron>::memory(HashMap::new())?;
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
//! # return Ok(()); };
//! # func().unwrap();
//! # }
//! ```
//!
//! ## Panics
//!
//! This Database implementation uses `RwLock` and `Mutex` under the hood. If either the closures
//! given to `Database::write` or any of the Backend implementation methods panic the respective
//! objects are then poisoned. This means that you *cannot panic* under any circumstances in your
//! closures or custom backends.
//!
//! Currently there is no way to recover from a poisoned `Database` other than re-creating it.
//!
//! ## Examples
//!
//! There are several more or less in-depth example programs you can check out!
//! Check them out here: https://github.com/TheNeikos/rustbreak/tree/master/examples
//!
//! - `config.rs` shows you how a possible configuration file could be managed with rustbreak
//! - `full.rs` shows you how the database can be used as a hashmap store
//! - `switching.rs` show you how you can easily swap out different parts of the Database
//!     *Note*: To run this example you need to enable the feature `yaml` like so:
//!         `cargo run --example switching --features yaml`
//! - `server/` is a fully fledged example app written with the Rocket framework to make a form of
//!     micro-blogging website. You will need rust nightly to start it.
//!
//! ## Features
//!
//! Rustbreak comes with three optional features:
//!
//! - `ron_enc` which enables the [Ron][ron] de/serialization
//! - `yaml_enc` which enables the Yaml de/serialization
//! - `bin_enc` which enables the Bincode de/serialization
//!
//! [Enable them in your `Cargo.toml` file to use them.][features] You can safely have them all
//! turned on per-default. The default feature is `ron`
//!
//!
//! [daybreak]:https://propublica.github.io/daybreak
//! [examples]: https://github.com/TheNeikos/rustbreak/tree/master/examples
//! [ron]: https://github.com/ron-rs/ron
//! [failure]: https://boats.gitlab.io/failure/intro.html
//! [features]: https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#choosing-features


extern crate serde;
#[macro_use] extern crate failure;

#[cfg(feature = "ron_enc")]
extern crate ron;

#[cfg(feature = "yaml_enc")]
extern crate serde_yaml;

#[cfg(feature = "bin_enc")]
extern crate bincode;
#[cfg(feature = "bin_enc")]
extern crate base64;

#[cfg(test)]
extern crate tempfile;

/// The rustbreak errors that can be returned
pub mod error;
/// Different storage backends
pub mod backend;
/// Different serialization and deserialization methods one can use
pub mod deser;

use std::sync::{Mutex, RwLock};
use std::fmt::Debug;

use serde::Serialize;
use serde::de::DeserializeOwned;

use deser::DeSerializer;
use backend::{Backend, MemoryBackend, FileBackend};

/// The Central Database to RustBreak
///
/// It has 3 Type Generics:
///
/// - Data: Is the Data, you must specify this
/// - Back: The storage backend.
/// - DeSer: The Serializer/Deserializer or short DeSer. Check the `deser` module for other
///     strategies.
#[derive(Debug)]
pub struct Database<Data, Back, DeSer>
    where
        Data: Serialize + DeserializeOwned + Debug + Clone + Send,
        Back: Backend + Debug,
        DeSer: DeSerializer<Data> + Debug + Send + Sync + Clone
{
    data: RwLock<Data>,
    backend: Mutex<Back>,
    deser: DeSer
}

impl<Data, Back, DeSer> Database<Data, Back, DeSer>
    where
        Data: Serialize + DeserializeOwned + Debug + Clone + Send,
        Back: Backend + Debug,
        DeSer: DeSerializer<Data> + Debug + Send + Sync + Clone
{
    /// Write lock the database and get write access to the `Data` container
    ///
    /// This gives you an exclusive lock on the memory object. Trying to open the database in
    /// writing will block if it is currently being written to.
    ///
    /// # Panics
    ///
    /// If you panic in the closure, the database is poisoned. This means that any
    /// subsequent writes/reads will fail with an `std::sync::PoisonError`.
    /// You can only recover from this by re-creating the Database Object.
    ///
    /// If you do not have full control over the code being written, and cannot incur the cost of
    /// having a single operation panicking then use `Database::write_safe`.
    pub fn write<T>(&self, task: T) -> error::Result<()>
        where T: FnOnce(&mut Data)
    {
        let mut lock = self.data.write().map_err(|_| error::RustbreakErrorKind::PoisonError)?;
        task(&mut lock);
        Ok(())
    }

    /// Write lock the database and get write access to the `Data` container in a safe way
    ///
    /// This gives you an exclusive lock on the memory object. Trying to open the database in
    /// writing will block if it is currently being written to.
    ///
    /// This differs to `Database::write` in that a clone of the internal data is made, which is
    /// then passed to the closure. Only if the closure doesn't panic is the internal model
    /// updated.
    ///
    /// Depending on the size of the database this can be very costly. This is a tradeoff to make
    /// for panic safety.
    ///
    /// # Panics
    ///
    /// When the closure panics, it is caught and a `error::RustbreakErrorkind::WritePanic` will be
    /// returned.
    pub fn write_safe<T>(&self, task: T) -> error::Result<()>
        where T: FnOnce(&mut Data) + std::panic::UnwindSafe,
              for<'r> &'r mut Data: std::panic::UnwindSafe
    {
        let mut lock = self.data.write().map_err(|_| error::RustbreakErrorKind::PoisonError)?;
        let mut data : Data = lock.clone();
        ::std::panic::catch_unwind(|| {
            task(&mut data);
        }).map_err(|_| error::RustbreakErrorKind::WritePanicError)?;
        *lock = data;
        Ok(())
    }

    /// Read lock the database and get write access to the `Data` container
    ///
    /// This gives you a read-only lock on the database. You can have as many readers in parallel
    /// as you wish.
    ///
    /// # Panics
    ///
    /// If you panic in the closure, the database is poisoned. This means that any
    /// subsequent writes/reads will fail with an `std::sync::PoisonError`.
    /// You can only recover from this by re-creating the Database Object.
    pub fn read<T, R>(&self, task: T) -> error::Result<R>
        where T: FnOnce(&Data) -> R
    {
        let mut lock = self.data.read().map_err(|_| error::RustbreakErrorKind::PoisonError)?;
        Ok(task(&mut lock))
    }

    fn load(backend: &mut Back, deser: &DeSer) -> error::Result<Data> {
        use failure::ResultExt;
        let new_data = deser.deserialize(&backend.get_data()?[..])
                            .context(error::RustbreakErrorKind::DeserializationError)?;

        Ok(new_data)
    }

    /// Reload the Data from the backend
    pub fn reload(&self) -> error::Result<()> {

        let mut data = self.data.write().map_err(|_| error::RustbreakErrorKind::PoisonError)?;
        let mut backend = self.backend.lock().map_err(|_| error::RustbreakErrorKind::PoisonError)?;

        *data = Self::load(&mut backend, &self.deser)?;
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

    /// Get a clone of the data as it is in memory right now
    ///
    /// To make sure you have the latest data, call this method with `reload` true
    pub fn get_data(&self, reload: bool) -> error::Result<Data> {
        let mut backend = self.backend.lock().map_err(|_| error::RustbreakErrorKind::PoisonError)?;
        let mut data = self.data.write().map_err(|_| error::RustbreakErrorKind::PoisonError)?;
        if reload {
            *data = Self::load(&mut backend, &self.deser)?;
            drop(backend);
        }
        Ok(data.clone())
    }

    /// Puts the data as is into memory
    ///
    /// To sync the data afterwards, call with `sync` true.
    pub fn put_data(&self, new_data: Data, sync: bool) -> error::Result<()> {
        let mut backend = self.backend.lock().map_err(|_| error::RustbreakErrorKind::PoisonError)?;
        let mut data = self.data.write().map_err(|_| error::RustbreakErrorKind::PoisonError)?;
        if sync {
            // TODO: Spin this into its own method
            use failure::ResultExt;

            let ser = self.deser.serialize(&*data)
                        .context(error::RustbreakErrorKind::SerializationError)?;

            backend.put_data(ser.as_bytes())?;
            drop(backend);
        }
        *data = new_data;
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

    /// Tries to clone the Data in the Database.
    ///
    /// This method returns a `MemoryDatabase` which has an empty vector as a
    /// backend initially. This means that the user is responsible for assigning a new backend
    /// if an alternative is wanted.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[macro_use] extern crate serde_derive;
    /// # extern crate rustbreak;
    /// # extern crate serde;
    /// # extern crate tempfile;
    /// # extern crate failure;
    ///
    /// use rustbreak::{FileDatabase, deser::Ron};
    ///
    /// #[derive(Debug, Serialize, Deserialize, Clone)]
    /// struct Data {
    ///     level: u32,
    /// }
    ///
    /// # fn main() {
    /// # let func = || -> Result<(), failure::Error> {
    /// # let file = tempfile::tempfile()?;
    /// let db = FileDatabase::<Data, Ron>::from_file(file, Data { level: 0 })?;
    ///
    /// db.write(|db| {
    ///     db.level = 42;
    /// })?;
    ///
    /// db.sync()?;
    ///
    /// let other_db = db.try_clone()?;
    ///
    /// let value = other_db.read(|db| db.level)?;
    /// assert_eq!(42, value);
    /// # return Ok(());
    /// # };
    /// # func().unwrap();
    /// # }
    /// ```
    pub fn try_clone(&self) -> error::Result<MemoryDatabase<Data, DeSer>> {
        let lock = self.data.write().map_err(|_| error::RustbreakErrorKind::PoisonError)?;

        Ok(Database {
            data: RwLock::new(lock.clone()),
            backend: Mutex::new(MemoryBackend::new()),
            deser: self.deser.clone(),
        })
    }
}

/// A database backed by a file
pub type FileDatabase<D, DS> = Database<D, FileBackend, DS>;

impl<Data, DeSer> Database<Data, FileBackend, DeSer>
    where
        Data: Serialize + DeserializeOwned + Debug + Clone + Send,
        DeSer: DeSerializer<Data> + Debug + Send + Sync + Clone
{
    /// Create new FileDatabase from Path
    pub fn from_path<S>(path: S, data: Data)
        -> error::Result<FileDatabase<Data, DeSer>>
        where S: AsRef<std::path::Path>
    {
        let backend = FileBackend::open(path)?;

        Ok(Database {
            data: RwLock::new(data),
            backend: Mutex::new(backend),
            deser: DeSer::default(),
        })
    }

    /// Create new FileDatabase from a file
    pub fn from_file(file: ::std::fs::File, data: Data) -> error::Result<FileDatabase<Data, DeSer>>
    {
        let backend = FileBackend::from_file(file);

        Ok(Database {
            data: RwLock::new(data),
            backend: Mutex::new(backend),
            deser: DeSer::default(),
        })
    }
}

/// A database backed by a `Vec<u8>`
pub type MemoryDatabase<D, DS> = Database<D, MemoryBackend, DS>;

impl<Data, DeSer> Database<Data, MemoryBackend, DeSer>
    where
        Data: Serialize + DeserializeOwned + Debug + Clone + Send,
        DeSer: DeSerializer<Data> + Debug + Send + Sync + Clone
{
    /// Create new FileDatabase from Path
    pub fn memory(data: Data) -> error::Result<MemoryDatabase<Data, DeSer>> {
        let backend = MemoryBackend::new();

        Ok(Database {
            data: RwLock::new(data),
            backend: Mutex::new(backend),
            deser: DeSer::default(),
        })
    }
}

impl<Data, Back, DeSer> Database<Data, Back, DeSer>
    where
        Data: Serialize + DeserializeOwned + Debug + Clone + Send,
        Back: Backend + Debug,
        DeSer: DeSerializer<Data> + Debug + Send + Sync + Clone
{
    /// Exchanges the DeSerialization strategy with the given one
    pub fn with_deser<T>(self, deser: T) -> Database<Data, Back, T>
        where T: DeSerializer<Data> + Debug + Send + Sync
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
        Back: Backend + Debug,
        DeSer: DeSerializer<Data> + Debug + Send + Sync + Clone
{
    /// Exchanges the Backend with the given one
    pub fn with_backend<T>(self, backend: T) -> Database<Data, T, DeSer>
        where T: Backend + Debug
    {
        Database {
            backend: Mutex::new(backend),
            data: self.data,
            deser: self.deser,
        }
    }
}

impl<Data, Back, DeSer> Database<Data, Back, DeSer>
    where
        Data: Serialize + DeserializeOwned + Debug + Clone + Send,
        Back: Backend + Debug,
        DeSer: DeSerializer<Data> + Debug + Send + Sync + Clone
{
    /// Converts from one data type to another
    ///
    /// This method is useful to migrate from one datatype to another
    pub fn convert_data<C, OutputData>(self, convert: C)
        -> error::Result<Database<OutputData, Back, DeSer>>
        where
            OutputData: Serialize + DeserializeOwned + Debug + Clone + Send,
            C: FnOnce(Data) -> OutputData,
            DeSer: DeSerializer<OutputData> + Debug + Send + Sync,
    {
        let (data, backend, deser) = self.into_inner()?;
        Ok(Database {
            data: RwLock::new(convert(data)),
            backend: Mutex::new(backend),
            deser: deser,
        })
    }
}
