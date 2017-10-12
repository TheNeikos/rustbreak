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
//! - Choosing what kind of Data is stored in it
//! - What kind of Container is storing it
//! - Which kind of Serialization is used for persistence
//! - Which kind of persistence is used
//!
//! Per default these options are used:
//! - The Container is a HashMap<String>, leaving you the choice what you want to store in it, and
//! can access it with a String key.
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
//! use rustbreak::Database;
//!
//! let db = Database::memory();
//!
//! println!("Writing to Database");
//! db.write(|mut db| {
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
//! [examples]: https://github.com/TheNeikos/rustbreak/
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
/// Differend serialization and deserialization methods one can use
pub mod deser;

use std::collections::HashMap;
use std::fs::{OpenOptions, File};
use std::path::Path;
use std::io::{Seek, SeekFrom, Read, Write};
use std::sync::{Mutex, RwLock};
use std::hash::Hash;
use std::fmt::Debug;
use std::marker::PhantomData;

use serde::Serialize;
use serde::de::DeserializeOwned;

use backend::RWVec;
pub use backend::Resizable;
use error::{BreakResult, BreakError};
use deser::{DeSerializer, Ron};

/// A container is a Key/Value storage
pub trait Container<V : Debug, K: Hash + Eq + Debug = String> : Debug {
    /// Insert the value at the given key
    ///
    /// Returns optionally an existing value that was replaced
    fn insert(&mut self, key: K, value: V) -> Option<V>;

    /// Removes the given value from the Container
    fn remove<T: AsRef<K>>(&mut self, key: T) -> Option<V>;

    /// Borrows the given value from the Container
    fn get<T: AsRef<K>>(&self, key: T) -> Option<&V>;

    /// Gets the underlying storage container mutably
    fn borrow_mut(&mut self) -> &mut Self {
        self
    }

    /// Gets the underlying storage container
    fn borrow(&self) -> &Self {
        self
    }
}

impl<V: Debug, K: Hash + Eq + Debug> Container<V, K> for HashMap<K, V> {
    fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.insert(key, value)
    }
    fn remove<T: AsRef<K>>(&mut self, key: T) -> Option<V> {
        self.remove(key.as_ref())
    }
    fn get<T: AsRef<K>>(&self, key: T) -> Option<&V> {
        self.get(key.as_ref())
    }
}


type StringMap<D> = HashMap<String, D>;

/// The Central Database to RustBreak
///
/// It has 4 Type Generics:
///
/// - D: Is the Data, you must specify this (usually inferred by the compiler)
/// - C: Is the backing Container, per default HashMap<String, D>
/// - S: The Serializer/Deserializer or short DeSer. Per default `deser::Ron` is used
/// - F: The storage backend. Per default it is in memory, but can be easily used with a file
#[derive(Debug)]
pub struct Database<D, C = StringMap<D>, S = Ron, F = RWVec>
    where
        D: Serialize + DeserializeOwned + Debug,
        C: Serialize + DeserializeOwned + Container<D> + Debug,
        S: DeSerializer<D> + DeSerializer<C> + Sync + Send + Debug,
        F: Write + Resizable + Debug,
        for<'r> &'r mut F: Read
{
    backing: Mutex<F>,
    data: RwLock<C>,
    deser: S,
    key: PhantomData<D>
}

impl<D> Database<D, StringMap<D>>
    where
        D: Serialize + DeserializeOwned + Debug + Clone,
{
    /// Constructs a `Database` with in-memory Storage
    pub fn memory() -> Database<D, StringMap<D>, Ron, RWVec>
    {
        Database {
            backing: Mutex::new(RWVec::new()),
            data: RwLock::new(HashMap::new()),
            deser: Ron,
            key: PhantomData,
        }
    }
}

impl<D> Database<D, StringMap<D>>
    where
        D: Serialize + DeserializeOwned + Debug + Clone,
{
    /// Constructs a `Database` with file-backed storage
    pub fn from_file(file: File) -> Database<D, StringMap<D>, Ron, File> {
        Database {
            backing: Mutex::new(file),
            data: RwLock::new(HashMap::new()),
            deser: Ron,
            key: PhantomData,
        }
    }

    /// Constructs a `Database` with file-backed storage from a given path
    pub fn from_path<P: AsRef<Path>>(path: P)
        -> BreakResult<Database<D, StringMap<D>, Ron, File>,
                        <Ron as DeSerializer<D>>::SerError, <Ron as DeSerializer<D>>::DeError> {
        let file = OpenOptions::new().read(true).write(true).create(true).open(path)?;
        Ok(Self::from_file(file))
    }
}

impl<D, C, S, F> Database<D, C, S, F>
    where
        D: Serialize + DeserializeOwned + Debug + Clone,
        C: Serialize + DeserializeOwned + Container<D> + Debug,
        S: DeSerializer<D> + DeSerializer<C> + Sync + Send + Debug,
        F: Write + Seek + Resizable + Debug,
        for<'r> &'r mut F: Read
{
    /// Write locks the database and gives you write access to the underlying `Container`
    pub fn write<T>(&self, mut task: T)
        -> BreakResult<(), <S as DeSerializer<C>>::SerError, <S as DeSerializer<C>>::DeError>
        where T: FnMut(&mut C)
    {
        let mut lock = self.data.write()?;
        task(&mut lock);
        Ok(())
    }

    /// Read locks the database and gives you read access to the underlying `Container`
    pub fn read<T>(&self, mut task: T)
        -> BreakResult<(), <S as DeSerializer<C>>::SerError, <S as DeSerializer<C>>::DeError>
        where T: FnMut(&C)
    {
        let lock = self.data.read()?;
        task(&lock);
        Ok(())
    }

    /// Directly inserts a given piece of data into the Database
    pub fn insert(&self, key: String, data: D)
        -> BreakResult<(), <S as DeSerializer<C>>::SerError, <S as DeSerializer<C>>::DeError>
    {
        let mut lock = self.data.write()?;
        lock.insert(key, data);
        Ok(())
    }

    /// Directly removes a given piece of data from the Database
    pub fn remove<K: AsRef<String>>(&self, key: K)
        -> BreakResult<Option<D>, <S as DeSerializer<C>>::SerError, <S as DeSerializer<C>>::DeError>
    {
        let mut lock = self.data.write()?;
        Ok(lock.remove(key))
    }

    /// Directly removes a given piece of data from the Database
    pub fn get<K: AsRef<String>>(&self, key: K)
        -> BreakResult<Option<D>, <S as DeSerializer<C>>::SerError, <S as DeSerializer<C>>::DeError>
    {
        let lock = self.data.read()?;
        Ok(lock.get(key).cloned())
    }

    /// Syncs the Database to the backing storage
    ///
    /// # Attention
    /// You __have__ to call this method yourself! Per default Rustbreak buffers everything
    /// in-memory and lets you decide when to write
    pub fn sync(&self)
        -> BreakResult<(), <S as DeSerializer<C>>::SerError, <S as DeSerializer<C>>::DeError>
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
        -> BreakResult<(), <S as DeSerializer<C>>::SerError, <S as DeSerializer<C>>::DeError>
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

impl<D, C, S, F> Database<D, C, S, F>
    where
        D: Serialize + DeserializeOwned + Debug,
        C: Serialize + DeserializeOwned + Container<D> + Debug,
        S: DeSerializer<D> + DeSerializer<C> + Sync + Send + Debug,
        F: Write + Resizable + Debug,
        for<'r> &'r mut F: Read
{
    /// Exchanges a given deserialization method with another
    pub fn with_deser<T>(self, deser: T) -> Database<D, C, T, F>
        where
            T: DeSerializer<D> + DeSerializer<C> + Sync + Send + Debug,
    {
        Database {
            backing: self.backing,
            data: self.data,
            deser: deser,
            key: PhantomData,
        }
    }
}

impl<D, C, S, F> Database<D, C, S, F>
    where
        D: Serialize + DeserializeOwned + Debug,
        C: Serialize + DeserializeOwned + Container<D> + Debug,
        S: DeSerializer<D> + DeSerializer<C> + Sync + Send + Debug,
        F: Write + Resizable + Debug,
        for<'r> &'r mut F: Read
{
    /// Exchanges a given backing method with another
    pub fn with_backing<T>(self, backing: T) -> Database<D, C, S, T>
        where
            T: Write + Resizable + Debug,
            for<'r> &'r mut T: Read
    {
        Database {
            backing: Mutex::new(backing),
            data: self.data,
            deser: self.deser,
            key: PhantomData,
        }
    }
}

impl<D, C, S, F> Database<D, C, S, F>
    where
        D: Serialize + DeserializeOwned + Debug,
        C: Serialize + DeserializeOwned + Container<D> + Debug,
        S: DeSerializer<D> + DeSerializer<C> + Sync + Send + Debug,
        F: Write + Resizable + Debug,
        for<'r> &'r mut F: Read
{
    /// Exchanges a given container method with another
    pub fn with_container<T>(self, data: T) -> Database<D, T, S, F>
        where
            T: Serialize + DeserializeOwned + Container<D> + Debug,
            S: DeSerializer<T> + DeSerializer<C> + Sync + Send + Debug,
    {
        Database {
            backing: self.backing,
            data: RwLock::new(data),
            deser: self.deser,
            key: PhantomData,
        }
    }
}
