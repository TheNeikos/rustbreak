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
//! use rustbreak::Database;
//!
//! let db = Database::<HashMap<String, String>>::memory(HashMap::new());
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
#[cfg(feature = "yaml")]
extern crate serde_yaml;
#[cfg(feature = "bin")]
extern crate bincode;
#[cfg(feature = "bin")]
extern crate base64;
#[cfg(feature = "bin")]
#[macro_use] extern crate error_chain;

mod error;
mod backend;
/// Different serialization and deserialization methods one can use
pub mod deser;

use std::fs::{OpenOptions, File};
use std::path::Path;
use std::io::{Seek, SeekFrom, Read, Write};
use std::sync::{Mutex, RwLock};
use std::fmt::Debug;

use serde::Serialize;
use serde::de::DeserializeOwned;

use backend::RWVec;
pub use backend::Resizable;
use error::{BreakResult, BreakError};
use deser::{DeSerializer, Ron};

/// The Central Database to RustBreak
///
/// It has 5 Type Generics:
///
/// - V: Is the Data, you must specify this (usually inferred by the compiler)
/// - S: The Serializer/Deserializer or short DeSer. Per default `deser::Ron` is used. Check the
///     `deser` module for other strategies.
/// - F: The storage backend. Per default it is in memory, but can be easily used with a `File`.
#[derive(Debug)]
pub struct Database<V, S = Ron, F = RWVec>
    where
        V: Serialize + DeserializeOwned + Debug + Clone,
        S: DeSerializer<V> + Sync + Send + Debug,
        F: Write + Resizable + Debug,
        for<'r> &'r mut F: Read
{
    backing: Mutex<F>,
    data: RwLock<V>,
    deser: S,
}

impl<V> Database<V>
    where
        V: Serialize + DeserializeOwned + Debug + Clone,
{
    /// Constructs a `Database` with in-memory Storage
    pub fn memory(initial: V) -> Database<V, Ron, RWVec>
    {
        Database {
            backing: Mutex::new(RWVec::new()),
            data: RwLock::new(initial),
            deser: Ron,
        }
    }
}

impl<V> Database<V>
    where
        V: Serialize + DeserializeOwned + Debug + Clone,
{
    /// Constructs a `Database` with file-backed storage
    pub fn from_file(initial: V, file: File) -> Database<V, Ron, File> {
        Database {
            backing: Mutex::new(file),
            data: RwLock::new(initial),
            deser: Ron,
        }
    }

    /// Constructs a `Database` with file-backed storage from a given path
    pub fn from_path<P: AsRef<Path>>(initial: V, path: P) -> BreakResult<Database<V, Ron, File>,
        <Ron as DeSerializer<V>>::SerError, <Ron as DeSerializer<V>>::DeError>
    {
        let file = OpenOptions::new().read(true).write(true).create(true).open(path)?;
        Ok(Self::from_file(initial, file))
    }
}

impl<V, S, F> Database<V, S, F>
    where
        V: Serialize + DeserializeOwned + Debug + Clone,
        S: DeSerializer<V> + Sync + Send + Debug,
        F: Write + Seek + Resizable + Debug,
        for<'r> &'r mut F: Read
{
    /// Write locks the database and gives you write access to the underlying `Container`
    pub fn write<T>(&self, task: T)
        -> BreakResult<(), <S as DeSerializer<V>>::SerError, <S as DeSerializer<V>>::DeError>
        where T: FnOnce(&mut V)
    {
        let mut lock = self.data.write()?;
        task(&mut lock);
        Ok(())
    }

    /// Syncs the Database to the backing storage
    ///
    /// # Attention
    /// You __have__ to call this method yourself! Per default Rustbreak buffers everything
    /// in-memory and lets you decide when to write
    pub fn sync(&self)
        -> BreakResult<(), <S as DeSerializer<V>>::SerError, <S as DeSerializer<V>>::DeError>
    {
        let mut backing = self.backing.lock()?;
        let data = self.data.read()?;
        let s = match self.deser.serialize(&*data) {
            Ok(s) => s,
            Err(e) => return Err(BreakError::Serialize(e)),
        };
        backing.seek(SeekFrom::Start(0))?;
        backing.resize(0)?;
        backing.write_all(&s.as_bytes())?;
        backing.sync()?;
        Ok(())
    }

    /// Reloads the internal storage from the backing storage
    ///
    /// This is useful to call at startup, or your Database might be empty!
    pub fn reload(&self)
        -> BreakResult<(), <S as DeSerializer<V>>::SerError, <S as DeSerializer<V>>::DeError>
    {
        let mut backing = self.backing.lock()?;
        let mut data = self.data.write()?;
        backing.seek(SeekFrom::Start(0))?;
        let mut new_data = match self.deser.deserialize(&mut *backing) {
            Ok(s) => s,
            Err(e) => return Err(BreakError::Deserialize(e)),
        };
        ::std::mem::swap(&mut *data, &mut new_data);
        Ok(())
    }

}


impl<V, S, F> Database<V, S, F>
    where
        V: Serialize + DeserializeOwned + Debug + Clone,
        S: DeSerializer<V> + Sync + Send + Debug,
        F: Write + Resizable + Debug,
        for<'r> &'r mut F: Read
{
    /// Read locks the database and gives you read access to the underlying `Container`
    pub fn read<T, R>(&self, task: T)
        -> BreakResult<R, <S as DeSerializer<V>>::SerError, <S as DeSerializer<V>>::DeError>
        where T: FnOnce(&V) -> R
    {
        let lock = self.data.read()?;
        Ok(task(&lock))
    }
}

impl<V, S, F> Database<V, S, F>
    where
        V: Serialize + DeserializeOwned + Debug + Clone,
        S: DeSerializer<V> + Sync + Send + Debug,
        F: Write + Resizable + Debug,
        for<'r> &'r mut F: Read
{
    /// Exchanges a given deserialization method with another
    pub fn with_deser<T>(self, deser: T) -> Database<V, T, F>
        where
            T: DeSerializer<V> + Sync + Send + Debug,
    {
        Database {
            backing: self.backing,
            data: self.data,
            deser: deser,
        }
    }
}

impl<V, S, F> Database<V, S, F>
    where
        V: Serialize + DeserializeOwned + Debug + Clone,
        S: DeSerializer<V> + Sync + Send + Debug,
        F: Write + Resizable + Debug,
        for<'r> &'r mut F: Read
{
    /// Exchanges a given backing method with another
    pub fn with_backing<T>(self, backing: T) -> Database<V, S, T>
        where
            T: Write + Resizable + Debug,
            for<'r> &'r mut T: Read
    {
        Database {
            backing: Mutex::new(backing),
            data: self.data,
            deser: self.deser,
        }
    }
}
