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
//! See the [examples][examples] to get an idea how this library is used.
//!
//! [daybreak]:https://propublica.github.io/daybreak
//! [examples]: https://github.com/TheNeikos/rustbreak

extern crate serde;
extern crate ron;
#[cfg(feature = "yaml")]
extern crate serde_yaml;
#[macro_use] extern crate quick_error;

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
/// - F: The file backend. Per default it is in memory, but can be easily used with a file
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
        D: Serialize + DeserializeOwned + Debug,
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
        D: Serialize + DeserializeOwned + Debug,
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
    pub fn from_path<P: AsRef<Path>>(path: P) -> BreakResult<Database<D, StringMap<D>, Ron, File>> {
        let file = OpenOptions::new().read(true).write(true).create(true).open(path)?;
        Ok(Self::from_file(file))
    }
}

impl<D, C, S, F> Database<D, C, S, F>
    where
        D: Serialize + DeserializeOwned + Debug,
        C: Serialize + DeserializeOwned + Container<D> + Debug,
        S: DeSerializer<D> + DeSerializer<C> + Sync + Send + Debug,
        F: Write + Seek + Resizable + Debug,
        <S as deser::DeSerializer<C>>::SerError: std::error::Error + Send + Sync + 'static,
        <S as deser::DeSerializer<C>>::DeError: std::fmt::Debug,
        for<'r> &'r mut F: Read
{
    /// Write locks the database and gives you write access to the underlying `Container`
    pub fn write<T>(&self, mut task: T) -> BreakResult<()>
        where T: FnMut(&mut C)
    {
        let mut lock = self.data.write()?;
        task(&mut lock);
        Ok(())
    }

    /// Read locks the database and gives you read access to the underlying `Container`
    pub fn read<T>(&self, mut task: T) -> BreakResult<()>
        where T: FnMut(&C)
    {
        let lock = self.data.read()?;
        task(&lock);
        Ok(())
    }

    /// Syncs the Database to the backing storage
    ///
    /// # Attention
    /// You __have__ to call this method yourself! Per default Rustbreak buffers everything
    /// in-memory and lets you decide when to write
    pub fn sync(&self) -> BreakResult<()> {
        let mut backing = self.backing.lock()?;
        let data = self.data.read()?;
        let s = match self.deser.serialize(&*data) {
            Ok(s) => s,
            Err(_) => return Err(BreakError::Serialize),
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
    pub fn reload(&self) -> BreakResult<()> {
        let mut backing = self.backing.lock()?;
        let mut data = self.data.write()?;
        backing.seek(SeekFrom::Start(0))?;
        let mut new_data = match self.deser.deserialize(&mut *backing) {
            Ok(s) => s,
            Err(e) => {
                println!("{:?}", e);
                return Err(BreakError::Deserialize);
            }
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

