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
//! Rustbreak was a [Daybreak][daybreak] inspired single file Database.
//! It has now since evolved into something else. Please check v1 for a more similar version.
//!
//! You will find an overview here in the docs, but to give you a more complete tale of how this is
//! used please check the [examples][examples].
//!
//! At its core, Rustbreak is an attempt at making a configurable general-purpose store Database.
//! It features the possibility of:
//!
//! - Choosing what kind of Data is stored in it
//! - Which kind of Serialization is used for persistence
//! - Which kind of persistence is used
//!
//! This means you can take any struct you can serialize and deserialize and stick it into this
//! Database. It is then encoded with Ron, Yaml, JSON, Bincode, anything really that uses Serde
//! operations!
//!
//! There are two helper type aliases `MemoryDatabase` and `FileDatabase`, each backed by their
//! respective backend.
//!
//! The `MemoryBackend` saves its data into a `Vec<u8>`, which is not that useful on its own, but
//! is needed for compatibility with the rest of the Library.
//!
//! The `FileDatabase` is a classical file based database. You give it a path or a file, and it
//! will use it as its storage. You still get to pick what encoding it uses.
//!
//! Using the `with_deser` and `with_backend` one can switch between the representations one needs.
//! Even at runtime! However this is only useful in a few scenarios.
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
//! # extern crate failure;
//! # extern crate rustbreak;
//! # use std::collections::HashMap;
//! use rustbreak::{MemoryDatabase, deser::Ron};
//!
//! # fn main() {
//! # let func = || -> Result<(), failure::Error> {
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
//! # extern crate failure;
//! # extern crate rustbreak;
//! # use std::collections::HashMap;
//! use rustbreak::{MemoryDatabase, deser::Ron};
//!
//! # fn main() {
//! # let func = || -> Result<(), failure::Error> {
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
//! Handling errors in Rustbreak is straightforward. Every `Result` has as its fail case as
//! `error::RustbreakError`. This means that you can now either continue bubbling up said error
//! case, or handle it yourself.
//!
//! You can simply call its `.kind()` method, giving you all the information you need to continue.
//!
//! ```rust
//! use rustbreak::{
//!     MemoryDatabase,
//!     deser::Ron,
//!     error::{
//!         RustbreakError,
//!     }
//! };
//! let db = match MemoryDatabase::<usize, Ron>::memory(0) {
//!     Ok(db) => db,
//!     Err(e) => {
//!         // Do something with `e` here
//!         ::std::process::exit(1);
//!     }
//! };
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
//! Check them out here: [Examples][examples]
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
//! Rustbreak comes with following optional features:
//!
//! - `ron_enc` which enables the [Ron][ron] de/serialization
//! - `yaml_enc` which enables the Yaml de/serialization
//! - `bin_enc` which enables the Bincode de/serialization
//! - 'mmap' whhich enables memory map backend.
//!
//! [Enable them in your `Cargo.toml` file to use them.][features] You can safely have them all
//! turned on per-default.
//!
//!
//! [repo]: https://github.com/TheNeikos/rustbreak
//! [daybreak]: https://propublica.github.io/daybreak
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

#[cfg(feature = "mmap")]
extern crate memmap;

#[cfg(test)]
extern crate tempfile;

/// The rustbreak errors that can be returned
pub mod error;
pub mod backend;
/// Different serialization and deserialization methods one can use
pub mod deser;

/// The `DeSerializer` trait used by serialization structs
pub use deser::DeSerializer;
/// The general error used by the Rustbreak Module
pub use error::RustbreakError;

use std::sync::{Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::fmt::Debug;

use serde::Serialize;
use serde::de::DeserializeOwned;
use failure::ResultExt;

use backend::{Backend, MemoryBackend, FileBackend};
#[cfg(feature = "mmap")]
use backend::MmapStorage;

/// The Central Database to RustBreak
///
/// It has 3 Type Generics:
///
/// - Data: Is the Data, you must specify this
/// - Back: The storage backend.
/// - DeSer: The Serializer/Deserializer or short DeSer. Check the `deser` module for other
///     strategies.
///
/// # Panics
///
/// If the backend or the de/serialization panics, the database is poisoned. This means that any
/// subsequent writes/reads will fail with an `error::RustbreakErrorKind::PoisonError`.
/// You can only recover from this by re-creating the Database Object.
#[derive(Debug)]
pub struct Database<Data, Back, DeSer>
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
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[macro_use] extern crate serde_derive;
    /// # extern crate rustbreak;
    /// # extern crate serde;
    /// # extern crate tempfile;
    /// # extern crate failure;
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
        where T: FnOnce(&mut Data) -> R
    {
        let mut lock = self.data.write().map_err(|_| error::RustbreakErrorKind::Poison)?;
        Ok(task(&mut lock))
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
    /// You should read the documentation about this:
    /// [UnwindSafe](https://doc.rust-lang.org/std/panic/trait.UnwindSafe.html)
    ///
    /// # Panics
    ///
    /// When the closure panics, it is caught and a `error::RustbreakErrorKind::WritePanic` will be
    /// returned.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[macro_use] extern crate serde_derive;
    /// # extern crate rustbreak;
    /// # extern crate serde;
    /// # extern crate tempfile;
    /// # extern crate failure;
    /// use rustbreak::{
    ///     FileDatabase,
    ///     deser::Ron,
    ///     error::{
    ///         RustbreakError,
    ///         RustbreakErrorKind,
    ///     }
    /// };
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
    /// let result = db.write_safe(|db| {
    ///     db.level = 42;
    ///     panic!("We panic inside the write code.");
    /// }).expect_err("This should have been caught");
    ///
    /// match result.kind() {
    ///     RustbreakErrorKind::WritePanic => {
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
        where T: FnOnce(&mut Data) + std::panic::UnwindSafe,
    {
        let mut lock = self.data.write().map_err(|_| error::RustbreakErrorKind::Poison)?;
        let mut data = lock.clone();
        ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
            task(&mut data);
        })).map_err(|_| error::RustbreakErrorKind::WritePanic)?;
        *lock = data;
        Ok(())
    }

    /// Read lock the database and get write access to the `Data` container
    ///
    /// This gives you a read-only lock on the database. You can have as many readers in parallel
    /// as you wish.
    ///
    /// # Errors
    ///
    /// May return:
    ///
    /// - `error::RustbreakErrorKind::Backend`
    ///
    /// # Panics
    ///
    /// If you panic in the closure, the database is poisoned. This means that any
    /// subsequent writes/reads will fail with an `error::RustbreakErrorKind::Poison`.
    /// You can only recover from this by re-creating the Database Object.
    pub fn read<T, R>(&self, task: T) -> error::Result<R>
        where T: FnOnce(&Data) -> R
    {
        let mut lock = self.data.read().map_err(|_| error::RustbreakErrorKind::Poison)?;
        Ok(task(&mut lock))
    }

    /// Read lock the database and get access to the underlying struct
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
    /// # extern crate failure;
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
    /// let data = db.borrow_data()?;
    ///
    /// assert_eq!(42, data.level);
    /// # return Ok(());
    /// # };
    /// # func().unwrap();
    /// # }
    /// ```
    pub fn borrow_data<'a>(&'a self) -> error::Result<RwLockReadGuard<'a, Data>> {
        self.data.read().map_err(|_| error::RustbreakErrorKind::Poison.into())
    }

    /// Write lock the database and get access to the underlying struct
    ///
    /// This gives you access to the underlying struct, allowing you to modify it.
    ///
    /// # Panics
    ///
    /// If you panic while holding this reference, the database is poisoned. This means that any
    /// subsequent writes/reads will fail with an `std::sync::PoisonError`.
    /// You can only recover from this by re-creating the Database Object.
    ///
    /// If you do not have full control over the code being written, and cannot incur the cost of
    /// having a single operation panicking then use `Database::write_safe`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[macro_use] extern crate serde_derive;
    /// # extern crate rustbreak;
    /// # extern crate serde;
    /// # extern crate tempfile;
    /// # extern crate failure;
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
        self.data.write().map_err(|_| error::RustbreakErrorKind::Poison.into())
    }

    fn inner_load(backend: &mut Back, deser: &DeSer) -> error::Result<Data> {
        let new_data = deser.deserialize(
            &backend.get_data().context(error::RustbreakErrorKind::Backend)?[..]
        ).context(error::RustbreakErrorKind::Deserialization)?;

        Ok(new_data)
    }

    /// Load the Data from the backend
    pub fn load(&self) -> error::Result<()> {
        let mut data = self.data.write().map_err(|_| error::RustbreakErrorKind::Poison)?;
        let mut backend = self.backend.lock().map_err(|_| error::RustbreakErrorKind::Poison)?;

        *data = Self::inner_load(&mut backend, &self.deser).context(error::RustbreakErrorKind::Backend)?;
        Ok(())
    }

    /// Flush the data structure to the backend
    pub fn save(&self) -> error::Result<()> {
        let mut backend = self.backend.lock().map_err(|_| error::RustbreakErrorKind::Poison)?;
        let data = self.data.write().map_err(|_| error::RustbreakErrorKind::Poison)?;

        let ser = self.deser.serialize(&*data)
                    .context(error::RustbreakErrorKind::Serialization)?;

        backend.put_data(&ser).context(error::RustbreakErrorKind::Backend)?;
        Ok(())
    }

    /// Get a clone of the data as it is in memory right now
    ///
    /// To make sure you have the latest data, call this method with `load` true
    pub fn get_data(&self, load: bool) -> error::Result<Data> {
        let mut backend = self.backend.lock().map_err(|_| error::RustbreakErrorKind::Poison)?;
        let mut data = self.data.write().map_err(|_| error::RustbreakErrorKind::Poison)?;
        if load {
            *data = Self::inner_load(&mut backend, &self.deser).context(error::RustbreakErrorKind::Backend)?;
            drop(backend);
        }
        Ok(data.clone())
    }

    /// Puts the data as is into memory
    ///
    /// To save the data afterwards, call with `save` true.
    pub fn put_data(&self, new_data: Data, save: bool) -> error::Result<()> {
        let mut backend = self.backend.lock().map_err(|_| error::RustbreakErrorKind::Poison)?;
        let mut data = self.data.write().map_err(|_| error::RustbreakErrorKind::Poison)?;
        if save {
            // TODO: Spin this into its own method
            let ser = self.deser.serialize(&*data)
                        .context(error::RustbreakErrorKind::Serialization)?;

            backend.put_data(&ser).context(error::RustbreakErrorKind::Backend)?;
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
        Ok((self.data.into_inner().map_err(|_| error::RustbreakErrorKind::Poison)?,
            self.backend.into_inner().map_err(|_| error::RustbreakErrorKind::Poison)?,
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
        let lock = self.data.write().map_err(|_| error::RustbreakErrorKind::Poison)?;

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
        let backend = FileBackend::open(path).context(error::RustbreakErrorKind::Backend)?;

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

/// A database backed by anonymous memory map.
#[cfg(feature = "mmap")]
pub type MmapDatabase<D, DS> = Database<D, MmapStorage, DS>;

#[cfg(feature = "mmap")]
impl<Data, DeSer> Database<Data, MmapStorage, DeSer>
    where
        Data: Serialize + DeserializeOwned + Debug + Clone + Send,
        DeSer: DeSerializer<Data> + Debug + Send + Sync + Clone
{
    /// Create new MmapDatabase.
    pub fn mmap(data: Data) -> error::Result<MmapDatabase<Data, DeSer>> {
        let backend = MmapStorage::new()?;

        Ok(Database {
            data: RwLock::new(data),
            backend: Mutex::new(backend),
            deser: DeSer::default(),
        })
    }

    /// Create new MmapDatabase with specified initial size.
    pub fn mmap_with_size(data: Data, size: usize) -> error::Result<MmapDatabase<Data, DeSer>> {
        let backend = MmapStorage::with_size(size)?;

        Ok(Database {
            data: RwLock::new(data),
            backend: Mutex::new(backend),
            deser: DeSer::default(),
        })
    }
}

impl<Data, Back, DeSer> Database<Data, Back, DeSer> {
    /// Exchanges the DeSerialization strategy with the new one
    pub fn with_deser<T>(self, deser: T) -> Database<Data, Back, T>
    {
        Database {
            backend: self.backend,
            data: self.data,
            deser: deser,
        }
    }
}

impl<Data, Back, DeSer> Database<Data, Back, DeSer> {
    /// Exchanges the Backend with the new one
    ///
    /// The new backend does not necessarily have the latest data saved to it, so a `.save` should
    /// be called to make sure that it is saved.
    pub fn with_backend<T>(self, backend: T) -> Database<Data, T, DeSer>
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
