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
    clippy::panic,
    clippy::print_stdout,
    clippy::todo,
    //clippy::unwrap_used, // not yet in stable
    clippy::wrong_pub_self_convention
)]
#![warn(clippy::pedantic)]
// part of `clippy::pedantic`, causing many warnings
#![allow(clippy::missing_errors_doc, clippy::module_name_repetitions)]

//! # Rustbreak
//!
//! Rustbreak was a [Daybreak][daybreak] inspired single file Database.
//! It has now since evolved into something else. Please check v1 for a more
//! similar version.
//!
//! You will find an overview here in the docs, but to give you a more complete
//! tale of how this is used please check the [examples][examples].
//!
//! At its core, Rustbreak is an attempt at making a configurable
//! general-purpose store Database. It features the possibility of:
//!
//! - Choosing what kind of Data is stored in it
//! - Which kind of Serialization is used for persistence
//! - Which kind of persistence is used
//!
//! This means you can take any struct you can serialize and deserialize and
//! stick it into this Database. It is then encoded with Ron, Yaml, JSON,
//! Bincode, anything really that uses Serde operations!
//!
//! There are three helper type aliases [`MemoryDatabase`], [`FileDatabase`],
//! and [`PathDatabase`], each backed by their respective backend.
//!
//! The [`MemoryBackend`] saves its data into a `Vec<u8>`, which is not that
//! useful on its own, but is needed for compatibility with the rest of the
//! Library.
//!
//! The [`FileDatabase`] is a classical file based database. You give it a path
//! or a file, and it will use it as its storage. You still get to pick what
//! encoding it uses.
//!
//! The [`PathDatabase`] is very similar, but always requires a path for
//! creation. It features atomic saves, so that the old database contents won't
//! be lost when panicing during the save. It should therefore be preferred to a
//! [`FileDatabase`].
//!
//! Using the [`Database::with_deser`] and [`Database::with_backend`] one can
//! switch between the representations one needs. Even at runtime! However this
//! is only useful in a few scenarios.
//!
//! If you have any questions feel free to ask at the main [repo][repo].
//!
//! ## Quickstart
//!
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies.rustbreak]
//! version = "2"
//! features = ["ron_enc"] # You can also use "yaml_enc" or "bin_enc"
//!                        # Check the documentation to add your own!
//! ```
//!
//! ```rust
//! # extern crate rustbreak;
//! # use std::collections::HashMap;
//! use rustbreak::{deser::Ron, MemoryDatabase};
//!
//! # fn main() {
//! # let func = || -> Result<(), Box<dyn std::error::Error>> {
//! let db = MemoryDatabase::<HashMap<u32, String>, Ron>::memory(HashMap::new())?;
//!
//! println!("Writing to Database");
//! db.write(|db| {
//!     db.insert(0, String::from("world"));
//!     db.insert(1, String::from("bar"));
//! });
//!
//! db.read(|db| {
//!     // db.insert("foo".into(), String::from("bar"));
//!     // The above line will not compile since we are only reading
//!     println!("Hello: {:?}", db.get(&0));
//! })?;
//! # return Ok(()); };
//! # func().unwrap();
//! # }
//! ```
//!
//! Or alternatively:
//! ```rust
//! # extern crate rustbreak;
//! # use std::collections::HashMap;
//! use rustbreak::{deser::Ron, MemoryDatabase};
//!
//! # fn main() {
//! # let func = || -> Result<(), Box<dyn std::error::Error>> {
//! let db = MemoryDatabase::<HashMap<u32, String>, Ron>::memory(HashMap::new())?;
//!
//! println!("Writing to Database");
//! {
//!     let mut data = db.borrow_data_mut()?;
//!     data.insert(0, String::from("world"));
//!     data.insert(1, String::from("bar"));
//! }
//!
//! let data = db.borrow_data()?;
//! println!("Hello: {:?}", data.get(&0));
//! # return Ok(()); };
//! # func().unwrap();
//! # }
//! ```
//!
//! ## Error Handling
//!
//! Handling errors in Rustbreak is straightforward. Every `Result` has as its
//! fail case as [`error::RustbreakError`]. This means that you can now either
//! continue bubbling up said error case, or handle it yourself.
//!
//! ```rust
//! use rustbreak::{deser::Ron, error::RustbreakError, MemoryDatabase};
//! let db = match MemoryDatabase::<usize, Ron>::memory(0) {
//!     Ok(db) => db,
//!     Err(e) => {
//!         // Do something with `e` here
//!         std::process::exit(1);
//!     }
//! };
//! ```
//!
//! ## Panics
//!
//! This Database implementation uses [`RwLock`] and [`Mutex`] under the hood.
//! If either the closures given to [`Database::write`] or any of the Backend
//! implementation methods panic the respective objects are then poisoned. This
//! means that you *cannot panic* under any circumstances in your closures or
//! custom backends.
//!
//! Currently there is no way to recover from a poisoned `Database` other than
//! re-creating it.
//!
//! ## Examples
//!
//! There are several more or less in-depth example programs you can check out!
//! Check them out here: [Examples][examples]
//!
//! - `config.rs` shows you how a possible configuration file could be managed
//!   with rustbreak
//! - `full.rs` shows you how the database can be used as a hashmap store
//! - `switching.rs` show you how you can easily swap out different parts of the
//!   Database *Note*: To run this example you need to enable the feature `yaml`
//!   like so: `cargo run --example switching --features yaml`
//! - `server/` is a fully fledged example app written with the Rocket framework
//!   to make a form of micro-blogging website. You will need rust nightly to
//!   start it.
//!
//! ## Features
//!
//! Rustbreak comes with following optional features:
//!
//! - `ron_enc` which enables the [Ron][ron] de/serialization
//! - `yaml_enc` which enables the Yaml de/serialization
//! - `bin_enc` which enables the Bincode de/serialization
//! - 'mmap' whhich enables memory map backend.
//!
//! [Enable them in your `Cargo.toml` file to use them.][features] You can
//! safely have them all turned on per-default.
//!
//!
//! [repo]: https://github.com/TheNeikos/rustbreak
//! [daybreak]: https://propublica.github.io/daybreak
//! [examples]: https://github.com/TheNeikos/rustbreak/tree/master/examples
//! [ron]: https://github.com/ron-rs/ron
//! [features]: https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#choosing-features

pub mod backend;
/// Different serialization and deserialization methods one can use
pub mod deser;
/// The rustbreak errors that can be returned
pub mod error;

/// The `DeSerializer` trait used by serialization structs
pub use crate::deser::DeSerializer;
/// The general error used by the Rustbreak Module
use std::fmt::Debug;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::{Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};

use serde::de::DeserializeOwned;
use serde::Serialize;

#[cfg(feature = "mmap")]
use crate::backend::MmapStorage;
use crate::backend::{Backend, FileBackend, MemoryBackend, PathBackend};

pub use crate::error::*;

/// The Central Database to Rustbreak.
///
/// It has 3 Type Generics:
///
/// - `Data`: Is the Data, you must specify this
/// - `Back`: The storage backend.
/// - `DeSer`: The Serializer/Deserializer or short `DeSer`. Check the [`deser`]
///   module for other strategies.
///
/// # Panics
///
/// If the backend or the de/serialization panics, the database is poisoned.
/// This means that any subsequent writes/reads will fail with an
/// [`error::RustbreakError::Poison`]. You can only recover from this by
/// re-creating the Database Object.
#[derive(Debug)]
pub struct Database<Data, Back, DeSer> {
    data: RwLock<Data>,
    backend: Mutex<Back>,
    deser: DeSer,
}

impl<Data, Back, DeSer> Database<Data, Back, DeSer>
where
    Data: Serialize + DeserializeOwned + Clone + Send,
    Back: Backend,
    DeSer: DeSerializer<Data> + Send + Sync + Clone,
{
    /// Write lock the database and get write access to the `Data` container.
    ///
    /// This gives you an exclusive lock on the memory object. Trying to open
    /// the database in writing will block if it is currently being written
    /// to.
    ///
    /// # Panics
    ///
    /// If you panic in the closure, the database is poisoned. This means that
    /// any subsequent writes/reads will fail with an
    /// [`error::RustbreakError::Poison`]. You can only recover from
    /// this by re-creating the Database Object.
    ///
    /// If you do not have full control over the code being written, and cannot
    /// incur the cost of having a single operation panicking then use
    /// [`Database::write_safe`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[macro_use] extern crate serde_derive;
    /// # extern crate rustbreak;
    /// # extern crate serde;
    /// # extern crate tempfile;
    /// use rustbreak::{deser::Ron, FileDatabase};
    ///
    /// #[derive(Debug, Serialize, Deserialize, Clone)]
    /// struct Data {
    ///     level: u32,
    /// }
    ///
    /// # fn main() {
    /// # let func = || -> Result<(), Box<dyn std::error::Error>> {
    /// # let file = tempfile::tempfile()?;
    /// let db = FileDatabase::<Data, Ron>::from_file(file, Data { level: 0 })?;
    ///
    /// db.write(|db| {
    ///     db.level = 42;
    /// })?;
    ///
    /// // You can also return from a `.read()`. But don't forget that you cannot return references
    /// // into the structure
    /// let value = db.read(|db| db.level)?;
    /// assert_eq!(42, value);
    /// # return Ok(());
    /// # };
    /// # func().unwrap();
    /// # }
    /// ```
    pub fn write<T, R>(&self, task: T) -> error::Result<R>
    where
        T: FnOnce(&mut Data) -> R,
    {
        let mut lock = self.data.write().map_err(|_| RustbreakError::Poison)?;
        Ok(task(&mut lock))
    }

    /// Write lock the database and get write access to the `Data` container in
    /// a safe way.
    ///
    /// This gives you an exclusive lock on the memory object. Trying to open
    /// the database in writing will block if it is currently being written
    /// to.
    ///
    /// This differs to `Database::write` in that a clone of the internal data
    /// is made, which is then passed to the closure. Only if the closure
    /// doesn't panic is the internal model updated.
    ///
    /// Depending on the size of the database this can be very costly. This is a
    /// tradeoff to make for panic safety.
    ///
    /// You should read the documentation about this:
    /// [`UnwindSafe`](https://doc.rust-lang.org/std/panic/trait.UnwindSafe.html)
    ///
    /// # Panics
    ///
    /// When the closure panics, it is caught and a
    /// [`error::RustbreakError::WritePanic`] will be returned.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[macro_use] extern crate serde_derive;
    /// # extern crate rustbreak;
    /// # extern crate serde;
    /// # extern crate tempfile;
    /// use rustbreak::{
    ///     deser::Ron,
    ///     error::RustbreakError,
    ///     FileDatabase,
    /// };
    ///
    /// #[derive(Debug, Serialize, Deserialize, Clone)]
    /// struct Data {
    ///     level: u32,
    /// }
    ///
    /// # fn main() {
    /// # let func = || -> Result<(), Box<dyn std::error::Error>> {
    /// # let file = tempfile::tempfile()?;
    /// let db = FileDatabase::<Data, Ron>::from_file(file, Data { level: 0 })?;
    ///
    /// let result = db
    ///     .write_safe(|db| {
    ///         db.level = 42;
    ///         panic!("We panic inside the write code.");
    ///     })
    ///     .expect_err("This should have been caught");
    ///
    /// match result {
    ///     RustbreakError::WritePanic => {
    ///         // We can now handle this, in this example we will just ignore it
    ///     }
    ///     e => {
    ///         println!("{:#?}", e);
    ///         // You should always have generic error catching here.
    ///         // This future-proofs your code, and makes your code more robust.
    ///         // In this example this is unreachable though, and to assert that we have this
    ///         // macro here
    ///         unreachable!();
    ///     }
    /// }
    ///
    /// // We read it back out again, it has not changed
    /// let value = db.read(|db| db.level)?;
    /// assert_eq!(0, value);
    /// # return Ok(());
    /// # };
    /// # func().unwrap();
    /// # }
    /// ```
    pub fn write_safe<T>(&self, task: T) -> error::Result<()>
    where
        T: FnOnce(&mut Data) + std::panic::UnwindSafe,
    {
        let mut lock = self.data.write().map_err(|_| RustbreakError::Poison)?;
        let mut data = lock.clone();
        std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
            task(&mut data);
        }))
        .map_err(|_| RustbreakError::WritePanic)?;
        *lock = data;
        Ok(())
    }

    /// Read lock the database and get read access to the `Data` container.
    ///
    /// This gives you a read-only lock on the database. You can have as many
    /// readers in parallel as you wish.
    ///
    /// # Errors
    ///
    /// May return:
    ///
    /// - [`error::RustbreakError::Backend`]
    ///
    /// # Panics
    ///
    /// If you panic in the closure, the database is poisoned. This means that
    /// any subsequent writes/reads will fail with an
    /// [`error::RustbreakError::Poison`]. You can only recover from
    /// this by re-creating the Database Object.
    pub fn read<T, R>(&self, task: T) -> error::Result<R>
    where
        T: FnOnce(&Data) -> R,
    {
        let mut lock = self.data.read().map_err(|_| RustbreakError::Poison)?;
        Ok(task(&mut lock))
    }

    /// Read lock the database and get access to the underlying struct.
    ///
    /// This gives you access to the underlying struct, allowing for simple read
    /// only operations on it.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[macro_use] extern crate serde_derive;
    /// # extern crate rustbreak;
    /// # extern crate serde;
    /// # extern crate tempfile;
    /// use rustbreak::{deser::Ron, FileDatabase};
    ///
    /// #[derive(Debug, Serialize, Deserialize, Clone)]
    /// struct Data {
    ///     level: u32,
    /// }
    ///
    /// # fn main() {
    /// # let func = || -> Result<(), Box<dyn std::error::Error>> {
    /// # let file = tempfile::tempfile()?;
    /// let db = FileDatabase::<Data, Ron>::from_file(file, Data { level: 0 })?;
    ///
    /// db.write(|db| {
    ///     db.level = 42;
    /// })?;
    ///
    /// let data = db.borrow_data()?;
    ///
    /// assert_eq!(42, data.level);
    /// # return Ok(());
    /// # };
    /// # func().unwrap();
    /// # }
    /// ```
    pub fn borrow_data<'a>(&'a self) -> error::Result<RwLockReadGuard<'a, Data>> {
        self.data.read().map_err(|_| RustbreakError::Poison)
    }

    /// Write lock the database and get access to the underlying struct.
    ///
    /// This gives you access to the underlying struct, allowing you to modify
    /// it.
    ///
    /// # Panics
    ///
    /// If you panic while holding this reference, the database is poisoned.
    /// This means that any subsequent writes/reads will fail with an
    /// [`error::RustbreakError::Poison`]. You can only recover from
    /// this by re-creating the Database Object.
    ///
    /// If you do not have full control over the code being written, and cannot
    /// incur the cost of having a single operation panicking then use
    /// [`Database::write_safe`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[macro_use] extern crate serde_derive;
    /// # extern crate rustbreak;
    /// # extern crate serde;
    /// # extern crate tempfile;
    /// use rustbreak::{deser::Ron, FileDatabase};
    ///
    /// #[derive(Debug, Serialize, Deserialize, Clone)]
    /// struct Data {
    ///     level: u32,
    /// }
    ///
    /// # fn main() {
    /// # let func = || -> Result<(), Box<dyn std::error::Error>> {
    /// # let file = tempfile::tempfile()?;
    /// let db = FileDatabase::<Data, Ron>::from_file(file, Data { level: 0 })?;
    ///
    /// {
    ///     let mut data = db.borrow_data_mut()?;
    ///     data.level = 42;
    /// }
    ///
    /// let data = db.borrow_data()?;
    ///
    /// assert_eq!(42, data.level);
    /// # return Ok(());
    /// # };
    /// # func().unwrap();
    /// # }
    /// ```
    pub fn borrow_data_mut<'a>(&'a self) -> error::Result<RwLockWriteGuard<'a, Data>> {
        self.data.write().map_err(|_| RustbreakError::Poison)
    }

    /// Load data from backend and return this data.
    fn load_from_backend(backend: &mut Back, deser: &DeSer) -> error::Result<Data> {
        let new_data = deser.deserialize(&backend.get_data()?[..])?;

        Ok(new_data)
    }

    /// Like [`Self::load`] but returns the write lock to data it used.
    fn load_get_data_lock(&self) -> error::Result<RwLockWriteGuard<'_, Data>> {
        let mut backend_lock = self.backend.lock().map_err(|_| RustbreakError::Poison)?;

        let fresh_data = Self::load_from_backend(&mut backend_lock, &self.deser)?;
        drop(backend_lock);

        let mut data_write_lock = self.data.write().map_err(|_| RustbreakError::Poison)?;
        *data_write_lock = fresh_data;
        Ok(data_write_lock)
    }

    /// Load the data from the backend.
    pub fn load(&self) -> error::Result<()> {
        self.load_get_data_lock().map(|_| ())
    }

    /// Like [`Self::save`] but with explicit read (or write) lock to data.
    fn save_data_locked<L: Deref<Target = Data>>(&self, lock: L) -> error::Result<()> {
        let ser = self.deser.serialize(lock.deref())?;
        drop(lock);

        let mut backend = self.backend.lock().map_err(|_| RustbreakError::Poison)?;
        backend.put_data(&ser)?;
        Ok(())
    }

    /// Flush the data structure to the backend.
    pub fn save(&self) -> error::Result<()> {
        let data = self.data.read().map_err(|_| RustbreakError::Poison)?;
        self.save_data_locked(data)
    }

    /// Get a clone of the data as it is in memory right now.
    ///
    /// To make sure you have the latest data, call this method with `load`
    /// true.
    pub fn get_data(&self, load: bool) -> error::Result<Data> {
        let data = if load {
            self.load_get_data_lock()?
        } else {
            self.data.write().map_err(|_| RustbreakError::Poison)?
        };
        Ok(data.clone())
    }

    /// Puts the data as is into memory.
    ///
    /// To save the data afterwards, call with `save` true.
    pub fn put_data(&self, new_data: Data, save: bool) -> error::Result<()> {
        let mut data = self.data.write().map_err(|_| RustbreakError::Poison)?;
        *data = new_data;
        if save {
            self.save_data_locked(data)
        } else {
            Ok(())
        }
    }

    /// Create a database from its constituents.
    pub fn from_parts(data: Data, backend: Back, deser: DeSer) -> Self {
        Self {
            data: RwLock::new(data),
            backend: Mutex::new(backend),
            deser,
        }
    }

    /// Break a database into its individual parts.
    pub fn into_inner(self) -> error::Result<(Data, Back, DeSer)> {
        Ok((
            self.data.into_inner().map_err(|_| RustbreakError::Poison)?,
            self.backend
                .into_inner()
                .map_err(|_| RustbreakError::Poison)?,
            self.deser,
        ))
    }

    /// Tries to clone the Data in the Database.
    ///
    /// This method returns a `MemoryDatabase` which has an empty vector as a
    /// backend initially. This means that the user is responsible for assigning
    /// a new backend if an alternative is wanted.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[macro_use] extern crate serde_derive;
    /// # extern crate rustbreak;
    /// # extern crate serde;
    /// # extern crate tempfile;
    /// use rustbreak::{deser::Ron, FileDatabase};
    ///
    /// #[derive(Debug, Serialize, Deserialize, Clone)]
    /// struct Data {
    ///     level: u32,
    /// }
    ///
    /// # fn main() {
    /// # let func = || -> Result<(), Box<dyn std::error::Error>> {
    /// # let file = tempfile::tempfile()?;
    /// let db = FileDatabase::<Data, Ron>::from_file(file, Data { level: 0 })?;
    ///
    /// db.write(|db| {
    ///     db.level = 42;
    /// })?;
    ///
    /// db.save()?;
    ///
    /// let other_db = db.try_clone()?;
    ///
    /// // You can also return from a `.read()`. But don't forget that you cannot return references
    /// // into the structure
    /// let value = other_db.read(|db| db.level)?;
    /// assert_eq!(42, value);
    /// # return Ok(());
    /// # };
    /// # func().unwrap();
    /// # }
    /// ```
    pub fn try_clone(&self) -> error::Result<MemoryDatabase<Data, DeSer>> {
        let lock = self.data.read().map_err(|_| RustbreakError::Poison)?;

        Ok(Database {
            data: RwLock::new(lock.clone()),
            backend: Mutex::new(MemoryBackend::new()),
            deser: self.deser.clone(),
        })
    }
}

/// A database backed by a file.
pub type FileDatabase<D, DS> = Database<D, FileBackend, DS>;

impl<Data, DeSer> Database<Data, FileBackend, DeSer>
where
    Data: Serialize + DeserializeOwned + Clone + Send,
    DeSer: DeSerializer<Data> + Send + Sync + Clone,
{
    /// Create new [`FileDatabase`] from the file at [`Path`](std::path::Path),
    /// and load the contents.
    pub fn load_from_path<S>(path: S) -> error::Result<Self>
    where
        S: AsRef<std::path::Path>,
    {
        let mut backend = FileBackend::from_path_or_fail(path)?;
        let deser = DeSer::default();
        let data = Self::load_from_backend(&mut backend, &deser)?;

        let db = Self {
            data: RwLock::new(data),
            backend: Mutex::new(backend),
            deser,
        };
        Ok(db)
    }

    /// Load [`FileDatabase`] at `path` or initialise with `data`.
    ///
    /// Create new [`FileDatabase`] from the file at [`Path`](std::path::Path),
    /// and load the contents. If the file does not exist, initialise with
    /// `data`.
    pub fn load_from_path_or<S>(path: S, data: Data) -> error::Result<Self>
    where
        S: AsRef<std::path::Path>,
    {
        let (mut backend, exists) = FileBackend::from_path_or_create(path)?;
        let deser = DeSer::default();
        if !exists {
            let ser = deser.serialize(&data)?;
            backend.put_data(&ser)?;
        }

        let db = Self {
            data: RwLock::new(data),
            backend: Mutex::new(backend),
            deser,
        };

        if exists {
            db.load()?;
        }

        Ok(db)
    }

    /// Load [`FileDatabase`] at `path` or initialise with `closure`.
    ///
    /// Create new [`FileDatabase`] from the file at [`Path`](std::path::Path),
    /// and load the contents. If the file does not exist, `closure` is
    /// called and the database is initialised with it's return value.
    pub fn load_from_path_or_else<S, C>(path: S, closure: C) -> error::Result<Self>
    where
        S: AsRef<std::path::Path>,
        C: FnOnce() -> Data,
    {
        let (mut backend, exists) = FileBackend::from_path_or_create(path)?;
        let deser = DeSer::default();
        let data = if exists {
            Self::load_from_backend(&mut backend, &deser)?
        } else {
            let data = closure();

            let ser = deser.serialize(&data)?;
            backend.put_data(&ser)?;

            data
        };

        let db = Self {
            data: RwLock::new(data),
            backend: Mutex::new(backend),
            deser,
        };
        Ok(db)
    }

    /// Create [`FileDatabase`] at `path`. Initialise with `data` if the file
    /// doesn't exist.
    ///
    /// Create new [`FileDatabase`] from the file at [`Path`](std::path::Path).
    /// Contents are not loaded. If the file does not exist, it is
    /// initialised with `data`. Frontend is always initialised with `data`.
    pub fn create_at_path<S>(path: S, data: Data) -> error::Result<Self>
    where
        S: AsRef<std::path::Path>,
    {
        let (mut backend, exists) = FileBackend::from_path_or_create(path)?;
        let deser = DeSer::default();
        if !exists {
            let ser = deser.serialize(&data)?;
            backend.put_data(&ser)?;
        }

        let db = Self {
            data: RwLock::new(data),
            backend: Mutex::new(backend),
            deser,
        };
        Ok(db)
    }

    /// Create new [`FileDatabase`] from a file.
    pub fn from_file(file: std::fs::File, data: Data) -> error::Result<Self> {
        let backend = FileBackend::from_file(file);

        Ok(Self {
            data: RwLock::new(data),
            backend: Mutex::new(backend),
            deser: DeSer::default(),
        })
    }
}

impl<Data, DeSer> Database<Data, FileBackend, DeSer>
where
    Data: Serialize + DeserializeOwned + Clone + Send + Default,
    DeSer: DeSerializer<Data> + Send + Sync + Clone,
{
    /// Load [`FileDatabase`] at `path` or initialise with `Data::default()`.
    ///
    /// Create new [`FileDatabase`] from the file at [`Path`](std::path::Path),
    /// and load the contents. If the file does not exist, initialise with
    /// `Data::default`.
    pub fn load_from_path_or_default<S>(path: S) -> error::Result<Self>
    where
        S: AsRef<std::path::Path>,
    {
        Self::load_from_path_or_else(path, Data::default)
    }
}

/// A database backed by a file, using atomic saves.
pub type PathDatabase<D, DS> = Database<D, PathBackend, DS>;

impl<Data, DeSer> Database<Data, PathBackend, DeSer>
where
    Data: Serialize + DeserializeOwned + Clone + Send,
    DeSer: DeSerializer<Data> + Send + Sync + Clone,
{
    /// Create new [`PathDatabase`] from the file at [`Path`](std::path::Path),
    /// and load the contents.
    pub fn load_from_path(path: PathBuf) -> error::Result<Self> {
        let mut backend = PathBackend::from_path_or_fail(path)?;
        let deser = DeSer::default();
        let data = Self::load_from_backend(&mut backend, &deser)?;

        let db = Self {
            data: RwLock::new(data),
            backend: Mutex::new(backend),
            deser,
        };
        Ok(db)
    }

    /// Load [`PathDatabase`] at `path` or initialise with `data`.
    ///
    /// Create new [`PathDatabase`] from the file at [`Path`](std::path::Path),
    /// and load the contents. If the file does not exist, initialise with
    /// `data`.
    pub fn load_from_path_or(path: PathBuf, data: Data) -> error::Result<Self> {
        let (mut backend, exists) = PathBackend::from_path_or_create(path)?;
        let deser = DeSer::default();
        if !exists {
            let ser = deser.serialize(&data)?;
            backend.put_data(&ser)?;
        }

        let db = Self {
            data: RwLock::new(data),
            backend: Mutex::new(backend),
            deser,
        };

        if exists {
            db.load()?;
        }

        Ok(db)
    }

    /// Load [`PathDatabase`] at `path` or initialise with `closure`.
    ///
    /// Create new [`PathDatabase`] from the file at [`Path`](std::path::Path),
    /// and load the contents. If the file does not exist, `closure` is
    /// called and the database is initialised with it's return value.
    pub fn load_from_path_or_else<C>(path: PathBuf, closure: C) -> error::Result<Self>
    where
        C: FnOnce() -> Data,
    {
        let (mut backend, exists) = PathBackend::from_path_or_create(path)?;
        let deser = DeSer::default();
        let data = if exists {
            Self::load_from_backend(&mut backend, &deser)?
        } else {
            let data = closure();

            let ser = deser.serialize(&data)?;
            backend.put_data(&ser)?;

            data
        };

        let db = Self {
            data: RwLock::new(data),
            backend: Mutex::new(backend),
            deser,
        };
        Ok(db)
    }

    /// Create [`PathDatabase`] at `path`. Initialise with `data` if the file
    /// doesn't exist.
    ///
    /// Create new [`PathDatabase`] from the file at [`Path`](std::path::Path).
    /// Contents are not loaded. If the file does not exist, it is
    /// initialised with `data`. Frontend is always initialised with `data`.
    pub fn create_at_path(path: PathBuf, data: Data) -> error::Result<Self> {
        let (mut backend, exists) = PathBackend::from_path_or_create(path)?;
        let deser = DeSer::default();
        if !exists {
            let ser = deser.serialize(&data)?;
            backend.put_data(&ser)?;
        }

        let db = Self {
            data: RwLock::new(data),
            backend: Mutex::new(backend),
            deser,
        };
        Ok(db)
    }
}

impl<Data, DeSer> Database<Data, PathBackend, DeSer>
where
    Data: Serialize + DeserializeOwned + Clone + Send + Default,
    DeSer: DeSerializer<Data> + Send + Sync + Clone,
{
    /// Load [`PathDatabase`] at `path` or initialise with `Data::default()`.
    ///
    /// Create new [`PathDatabase`] from the file at [`Path`](std::path::Path),
    /// and load the contents. If the file does not exist, initialise with
    /// `Data::default`.
    pub fn load_from_path_or_default(path: PathBuf) -> error::Result<Self> {
        Self::load_from_path_or_else(path, Data::default)
    }
}

/// A database backed by a byte vector (`Vec<u8>`).
pub type MemoryDatabase<D, DS> = Database<D, MemoryBackend, DS>;

impl<Data, DeSer> Database<Data, MemoryBackend, DeSer>
where
    Data: Serialize + DeserializeOwned + Clone + Send,
    DeSer: DeSerializer<Data> + Send + Sync + Clone,
{
    /// Create new in-memory database.
    pub fn memory(data: Data) -> error::Result<Self> {
        let backend = MemoryBackend::new();

        Ok(Self {
            data: RwLock::new(data),
            backend: Mutex::new(backend),
            deser: DeSer::default(),
        })
    }
}

/// A database backed by anonymous memory map.
#[cfg(feature = "mmap")]
pub type MmapDatabase<D, DS> = Database<D, MmapStorage, DS>;

#[cfg(feature = "mmap")]
impl<Data, DeSer> Database<Data, MmapStorage, DeSer>
where
    Data: Serialize + DeserializeOwned + Clone + Send,
    DeSer: DeSerializer<Data> + Send + Sync + Clone,
{
    /// Create new [`MmapDatabase`].
    pub fn mmap(data: Data) -> error::Result<Self> {
        let backend = MmapStorage::new()?;

        Ok(Self {
            data: RwLock::new(data),
            backend: Mutex::new(backend),
            deser: DeSer::default(),
        })
    }

    /// Create new [`MmapDatabase`] with specified initial size.
    pub fn mmap_with_size(data: Data, size: usize) -> error::Result<Self> {
        let backend = MmapStorage::with_size(size)?;

        Ok(Self {
            data: RwLock::new(data),
            backend: Mutex::new(backend),
            deser: DeSer::default(),
        })
    }
}

impl<Data, Back, DeSer> Database<Data, Back, DeSer> {
    /// Exchanges the `DeSerialization` strategy with the new one.
    pub fn with_deser<T>(self, deser: T) -> Database<Data, Back, T> {
        Database {
            backend: self.backend,
            data: self.data,
            deser,
        }
    }
}

impl<Data, Back, DeSer> Database<Data, Back, DeSer> {
    /// Exchanges the `Backend` with the new one.
    ///
    /// The new backend does not necessarily have the latest data saved to it,
    /// so a `.save` should be called to make sure that it is saved.
    pub fn with_backend<T>(self, backend: T) -> Database<Data, T, DeSer> {
        Database {
            backend: Mutex::new(backend),
            data: self.data,
            deser: self.deser,
        }
    }
}

impl<Data, Back, DeSer> Database<Data, Back, DeSer>
where
    Data: Serialize + DeserializeOwned + Clone + Send,
    Back: Backend,
    DeSer: DeSerializer<Data> + Send + Sync + Clone,
{
    /// Converts from one data type to another.
    ///
    /// This method is useful to migrate from one datatype to another.
    pub fn convert_data<C, OutputData>(
        self,
        convert: C,
    ) -> error::Result<Database<OutputData, Back, DeSer>>
    where
        OutputData: Serialize + DeserializeOwned + Clone + Send,
        C: FnOnce(Data) -> OutputData,
        DeSer: DeSerializer<OutputData> + Send + Sync,
    {
        let (data, backend, deser) = self.into_inner()?;
        Ok(Database {
            data: RwLock::new(convert(data)),
            backend: Mutex::new(backend),
            deser,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::NamedTempFile;

    type TestData = HashMap<usize, String>;
    type TestDb<B> = Database<TestData, B, crate::deser::Ron>;
    type TestMemDb = TestDb<MemoryBackend>;

    fn test_data() -> TestData {
        let mut data = HashMap::new();
        data.insert(1, "Hello World".to_string());
        data.insert(100, "Rustbreak".to_string());
        data
    }

    /// Used to test that `Default::default` isn't called.
    #[derive(Clone, Debug, Serialize, serde::Deserialize)]
    struct PanicDefault;
    impl Default for PanicDefault {
        fn default() -> Self {
            panic!("`default` was called but should not")
        }
    }

    #[test]
    fn create_db_and_read() {
        let db = TestMemDb::memory(test_data()).expect("Could not create database");
        assert_eq!(
            "Hello World",
            db.read(|d| d.get(&1).cloned())
                .expect("Rustbreak read error")
                .expect("Should be `Some` but was `None`")
        );
        assert_eq!(
            "Rustbreak",
            db.read(|d| d.get(&100).cloned())
                .expect("Rustbreak read error")
                .expect("Should be `Some` but was `None`")
        );
    }

    #[test]
    fn write_twice() {
        let db = TestMemDb::memory(test_data()).expect("Could not create database");
        db.write(|d| d.insert(3, "Write to db".to_string()))
            .expect("Rustbreak write error");
        db.write(|d| d.insert(3, "Second write".to_string()))
            .expect("Rustbreak write error");
        assert_eq!(
            "Hello World",
            db.read(|d| d.get(&1).cloned())
                .expect("Rustbreak read error")
                .expect("Should be `Some` but was `None`")
        );
        assert_eq!(
            "Rustbreak",
            db.read(|d| d.get(&100).cloned())
                .expect("Rustbreak read error")
                .expect("Should be `Some` but was `None`")
        );
        assert_eq!(
            "Second write",
            db.read(|d| d.get(&3).cloned())
                .expect("Rustbreak read error")
                .expect("Should be `Some` but was `None`")
        );
    }

    #[test]
    fn save_load() {
        let db = TestMemDb::memory(test_data()).expect("Could not create database");
        db.save().expect("Rustbreak save error");
        db.write(|d| d.clear()).expect("Rustbreak write error");
        db.load().expect("Rustbreak load error");
        assert_eq!(
            "Hello World",
            db.read(|d| d.get(&1).cloned())
                .expect("Rustbreak read error")
                .expect("Should be `Some` but was `None`")
        );
        assert_eq!(
            "Rustbreak",
            db.read(|d| d.get(&100).cloned())
                .expect("Rustbreak read error")
                .expect("Should be `Some` but was `None`")
        );
    }

    #[test]
    fn writesafe_twice() {
        let db = TestMemDb::memory(test_data()).expect("Could not create database");
        db.write_safe(|d| {
            d.insert(3, "Write to db".to_string());
        })
        .expect("Rustbreak write error");
        db.write_safe(|d| {
            d.insert(3, "Second write".to_string());
        })
        .expect("Rustbreak write error");
        assert_eq!(
            "Hello World",
            db.read(|d| d.get(&1).cloned())
                .expect("Rustbreak read error")
                .expect("Should be `Some` but was `None`")
        );
        assert_eq!(
            "Rustbreak",
            db.read(|d| d.get(&100).cloned())
                .expect("Rustbreak read error")
                .expect("Should be `Some` but was `None`")
        );
        assert_eq!(
            "Second write",
            db.read(|d| d.get(&3).cloned())
                .expect("Rustbreak read error")
                .expect("Should be `Some` but was `None`")
        );
    }

    #[test]
    fn writesafe_panic() {
        let db = TestMemDb::memory(test_data()).expect("Could not create database");
        let err = db
            .write_safe(|d| {
                d.clear();
                panic!("Panic should be catched")
            })
            .expect_err("Did not error on panic in safe write!");
        assert!(matches!(err, RustbreakError::WritePanic));

        assert_eq!(
            "Hello World",
            db.read(|d| d.get(&1).cloned())
                .expect("Rustbreak read error")
                .expect("Should be `Some` but was `None`")
        );
        assert_eq!(
            "Rustbreak",
            db.read(|d| d.get(&100).cloned())
                .expect("Rustbreak read error")
                .expect("Should be `Some` but was `None`")
        );
    }

    #[test]
    fn borrow_data_twice() {
        let db = TestMemDb::memory(test_data()).expect("Could not create database");
        let readlock1 = db.borrow_data().expect("Rustbreak readlock error");
        let readlock2 = db.borrow_data().expect("Rustbreak readlock error");
        assert_eq!(
            "Hello World",
            readlock1.get(&1).expect("Should be `Some` but was `None`")
        );
        assert_eq!(
            "Hello World",
            readlock2.get(&1).expect("Should be `Some` but was `None`")
        );
        assert_eq!(
            "Rustbreak",
            readlock1
                .get(&100)
                .expect("Should be `Some` but was `None`")
        );
        assert_eq!(
            "Rustbreak",
            readlock2
                .get(&100)
                .expect("Should be `Some` but was `None`")
        );
        assert_eq!(*readlock1, *readlock2);
    }

    #[test]
    fn borrow_data_mut() {
        let db = TestMemDb::memory(test_data()).expect("Could not create database");
        let mut writelock = db.borrow_data_mut().expect("Rustbreak writelock error");
        writelock.insert(3, "Write to db".to_string());
        drop(writelock);
        assert_eq!(
            "Hello World",
            db.read(|d| d.get(&1).cloned())
                .expect("Rustbreak read error")
                .expect("Should be `Some` but was `None`")
        );
        assert_eq!(
            "Rustbreak",
            db.read(|d| d.get(&100).cloned())
                .expect("Rustbreak read error")
                .expect("Should be `Some` but was `None`")
        );
        assert_eq!(
            "Write to db",
            db.read(|d| d.get(&3).cloned())
                .expect("Rustbreak read error")
                .expect("Should be `Some` but was `None`")
        );
    }

    #[test]
    fn get_data_mem() {
        let db = TestMemDb::memory(test_data()).expect("Could not create database");
        let data = db.get_data(false).expect("could not get data");
        assert_eq!(test_data(), data);
    }

    #[test]
    fn get_data_load() {
        let db = TestMemDb::memory(test_data()).expect("Could not create database");
        db.save().expect("Rustbreak save error");
        db.write(|d| d.clear()).expect("Rustbreak write error");
        let data = db.get_data(true).expect("could not get data");
        assert_eq!(test_data(), data);
    }

    #[test]
    fn put_data_mem() {
        let db = TestMemDb::memory(TestData::default()).expect("Could not create database");
        db.put_data(test_data(), false).expect("could not put data");
        assert_eq!(
            "Hello World",
            db.read(|d| d.get(&1).cloned())
                .expect("Rustbreak read error")
                .expect("Should be `Some` but was `None`")
        );
        assert_eq!(
            "Rustbreak",
            db.read(|d| d.get(&100).cloned())
                .expect("Rustbreak read error")
                .expect("Should be `Some` but was `None`")
        );
        let data = db.get_data(false).expect("could not get data");
        assert_eq!(test_data(), data);
    }

    #[test]
    fn put_data_save() {
        let db = TestMemDb::memory(TestData::default()).expect("Could not create database");
        db.put_data(test_data(), true).expect("could not put data");
        db.load().expect("Rustbreak load error");
        assert_eq!(
            "Hello World",
            db.read(|d| d.get(&1).cloned())
                .expect("Rustbreak read error")
                .expect("Should be `Some` but was `None`")
        );
        assert_eq!(
            "Rustbreak",
            db.read(|d| d.get(&100).cloned())
                .expect("Rustbreak read error")
                .expect("Should be `Some` but was `None`")
        );
        let data = db.get_data(false).expect("could not get data");
        assert_eq!(test_data(), data);
    }

    #[test]
    fn save_and_into_inner() {
        let db = TestMemDb::memory(test_data()).expect("Could not create database");
        db.save().expect("Rustbreak save error");
        let (data, mut backend, _) = db
            .into_inner()
            .expect("error calling `Database.into_inner`");
        assert_eq!(test_data(), data);
        let parsed: TestData =
            ron::de::from_reader(&backend.get_data().expect("could not get data from backend")[..])
                .expect("backend contains invalid RON");
        assert_eq!(test_data(), parsed);
    }

    #[test]
    fn clone() {
        let db1 = TestMemDb::memory(test_data()).expect("Could not create database");
        let readlock1 = db1.borrow_data().expect("Rustbreak readlock error");
        let db2 = db1.try_clone().expect("Rustbreak clone error");
        let readlock2 = db2.borrow_data().expect("Rustbreak readlock error");
        assert_eq!(test_data(), *readlock1);
        assert_eq!(*readlock1, *readlock2);
    }

    #[test]
    fn allow_databases_with_boxed_backend() {
        let db =
            MemoryDatabase::<Vec<u64>, crate::deser::Ron>::memory(vec![]).expect("To be created");
        let db: Database<_, Box<dyn Backend>, _> = db.with_backend(Box::new(MemoryBackend::new()));
        db.put_data(vec![1, 2, 3], true)
            .expect("Can save data in memory");
        assert_eq!(
            &[1, 2, 3],
            &db.get_data(true).expect("Can get data from memory")[..]
        );
    }

    /// Since `save` only needs read-access to the data we should be able to
    /// save while holding a readlock.
    #[test]
    fn save_holding_readlock() {
        let db = TestMemDb::memory(test_data()).expect("Could not create database");
        let readlock = db.borrow_data().expect("Rustbreak readlock error");
        db.save().expect("Rustbreak save error");
        assert_eq!(test_data(), *readlock);
    }

    /// Test that if the file already exists, the closure won't be called.
    #[test]
    #[cfg_attr(miri, ignore)]
    fn pathdb_from_path_or_else_existing_nocall() {
        let file = NamedTempFile::new().expect("could not create temporary file");
        let path = file.path().to_owned();
        let _ = TestDb::<PathBackend>::load_from_path_or_else(path, || {
            panic!("closure called while file existed")
        });
    }

    /// Test that if the file already exists, the closure won't be called.
    #[test]
    #[cfg_attr(miri, ignore)]
    fn filedb_from_path_or_else_existing_nocall() {
        let file = NamedTempFile::new().expect("could not create temporary file");
        let path = file.path();
        let _ = TestDb::<FileBackend>::load_from_path_or_else(path, || {
            panic!("closure called while file existed")
        });
    }

    /// Test that if the file already exists, `default` won't be called.
    #[test]
    #[cfg_attr(miri, ignore)]
    fn pathdb_from_path_or_default_existing_nocall() {
        let file = NamedTempFile::new().expect("could not create temporary file");
        let path = file.path().to_owned();
        let _ = Database::<PanicDefault, PathBackend, crate::deser::Ron>::load_from_path_or_default(
            path,
        );
    }

    /// Test that if the file already exists, the closure won't be called.
    #[test]
    #[cfg_attr(miri, ignore)]
    fn filedb_from_path_or_default_existing_nocall() {
        let file = NamedTempFile::new().expect("could not create temporary file");
        let path = file.path();
        let _ = Database::<PanicDefault, FileBackend, crate::deser::Ron>::load_from_path_or_default(
            path,
        );
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn pathdb_from_path_or_new() {
        let dir = tempfile::tempdir().expect("could not create temporary directory");
        let mut file_path = dir.path().to_owned();
        file_path.push("rustbreak_path_db.db");
        let db = TestDb::<PathBackend>::load_from_path_or(file_path, test_data())
            .expect("could not load from path");
        db.load().expect("could not load");
        let readlock = db.borrow_data().expect("Rustbreak readlock error");
        assert_eq!(test_data(), *readlock);
        dir.close().expect("Error while deleting temp directory!");
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn pathdb_from_path_or_else_new() {
        let dir = tempfile::tempdir().expect("could not create temporary directory");
        let mut file_path = dir.path().to_owned();
        file_path.push("rustbreak_path_db.db");
        let db = TestDb::<PathBackend>::load_from_path_or_else(file_path, test_data)
            .expect("could not load from path");
        db.load().expect("could not load");
        let readlock = db.borrow_data().expect("Rustbreak readlock error");
        assert_eq!(test_data(), *readlock);
        dir.close().expect("Error while deleting temp directory!");
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn filedb_from_path_or_new() {
        let dir = tempfile::tempdir().expect("could not create temporary directory");
        let mut file_path = dir.path().to_owned();
        file_path.push("rustbreak_path_db.db");
        let db = TestDb::<FileBackend>::load_from_path_or(file_path, test_data())
            .expect("could not load from path");
        db.load().expect("could not load");
        let readlock = db.borrow_data().expect("Rustbreak readlock error");
        assert_eq!(test_data(), *readlock);
        dir.close().expect("Error while deleting temp directory!");
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn filedb_from_path_or_else_new() {
        let dir = tempfile::tempdir().expect("could not create temporary directory");
        let mut file_path = dir.path().to_owned();
        file_path.push("rustbreak_path_db.db");
        let db = TestDb::<FileBackend>::load_from_path_or_else(file_path, test_data)
            .expect("could not load from path");
        db.load().expect("could not load");
        let readlock = db.borrow_data().expect("Rustbreak readlock error");
        assert_eq!(test_data(), *readlock);
        dir.close().expect("Error while deleting temp directory!");
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn pathdb_from_path_new_fail() {
        let dir = tempfile::tempdir().expect("could not create temporary directory");
        let mut file_path = dir.path().to_owned();
        file_path.push("rustbreak_path_db.db");
        let err = TestDb::<PathBackend>::load_from_path(file_path)
            .expect_err("should fail with file not found");
        if let RustbreakError::Backend(BackendError::Io(io_err)) = &err {
            assert_eq!(std::io::ErrorKind::NotFound, io_err.kind());
        } else {
            panic!("Wrong error: {}", err)
        };

        dir.close().expect("Error while deleting temp directory!");
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn filedb_from_path_new_fail() {
        let dir = tempfile::tempdir().expect("could not create temporary directory");
        let mut file_path = dir.path().to_owned();
        file_path.push("rustbreak_path_db.db");
        let err = TestDb::<FileBackend>::load_from_path(file_path)
            .expect_err("should fail with file not found");
        if let RustbreakError::Backend(BackendError::Io(io_err)) = &err {
            assert_eq!(std::io::ErrorKind::NotFound, io_err.kind());
        } else {
            panic!("Wrong error: {}", err)
        };

        dir.close().expect("Error while deleting temp directory!");
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn pathdb_from_path_existing() {
        let file = NamedTempFile::new().expect("could not create temporary file");
        let path = file.path().to_owned();
        // initialise the file
        let db = TestDb::<PathBackend>::create_at_path(path.clone(), test_data())
            .expect("could not create db");
        db.save().expect("could not save db");
        drop(db);
        // test that loading now works
        let db = TestDb::<PathBackend>::load_from_path(path).expect("could not load");
        let readlock = db.borrow_data().expect("Rustbreak readlock error");
        assert_eq!(test_data(), *readlock);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn filedb_from_path_existing() {
        let file = NamedTempFile::new().expect("could not create temporary file");
        let path = file.path();
        // initialise the file
        let db =
            TestDb::<FileBackend>::create_at_path(path, test_data()).expect("could not create db");
        db.save().expect("could not save db");
        drop(db);
        // test that loading now works
        let db = TestDb::<FileBackend>::load_from_path(path).expect("could not load");
        let readlock = db.borrow_data().expect("Rustbreak readlock error");
        assert_eq!(test_data(), *readlock);
    }
}
