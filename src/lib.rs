/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![deny(
    missing_copy_implementations,
    missing_debug_implementations,
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
    unused_qualifications,
    while_true,
)]

//! # Rustbreak
//!
//! Rustbreak is an [Daybreak][daybreak] inspiried single file Database.
//! It uses [bincode][bincode] to compactly save data.
//! It is thread safe and very fast due to mostly staying in memory until flushed.
//! It is possible to periodically flush or to do it manually.
//!
//! [daybreak]:https://propublica.github.io/daybreak/
//! [bincode]:https://github.com/TyOverby/bincode

extern crate serde;
#[macro_use] extern crate quick_error;
extern crate fs2;
extern crate bincode;
#[cfg(test)] extern crate tempfile;

mod error;

use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::sync::{RwLock, Mutex};
use std::hash::Hash;
use std::borrow::Borrow;

use serde::{Serialize, Deserialize};

pub use error::BreakError;

/// Alias for our Result Type
pub type BreakResult<T> = Result<T, BreakError>;

/// The Database structure
///
/// # Notes
/// One should need to create this once for each Database instance.
/// Subsequent tries to open the same file should fail or worse, could break the database.
///
/// # Example
///
/// ```
/// use rustbreak::Database;
///
/// let mut db = Database::open("/tmp/artists").unwrap();
///
/// let albums = vec![
///     ("What you do", "The Queenstons"),
///     ("Experience", "The Prodigy"),
/// ];
///
/// for (album, artist) in albums {
/// 	db.insert(&format!("album_{}",album), artist).unwrap();
/// }
/// db.flush().unwrap();
/// ```
#[derive(Debug)]
pub struct Database<T: Serialize + Deserialize + Eq + Hash> {
    file: Mutex<File>,
    data: RwLock<HashMap<T, Vec<u8>>>,
}

impl<T: Serialize + Deserialize + Eq + Hash> Database<T> {
    /// Opens a new Database
    ///
    /// This might fail if the file is non-empty and was not created by RustBreak, or if the file
    /// is already being used by another RustBreak instance.
    ///
    /// # Example
    ///
    /// ```
    /// use rustbreak::Database;
    ///
    /// let mut db = Database::open("/tmp/artists").unwrap();
    ///
    /// let albums = vec![
    ///     ("What you do", "The Queenstons"),
    ///     ("Experience", "The Prodigy"),
    /// ];
    ///
    /// for (album, artist) in albums {
    /// 	db.insert(&format!("album_{}",album), artist).unwrap();
    /// }
    /// db.flush().unwrap();
    /// ```
    pub fn open<P: AsRef<Path>>(path: P) -> BreakResult<Database<T>> {
		use std::fs::OpenOptions;
		use fs2::FileExt;
        use std::io::Read;
        use bincode::serde::deserialize;

		let mut file = try!(OpenOptions::new().read(true).write(true).create(true).open(path));
		try!(file.try_lock_exclusive());

		let mut buf = Vec::new();
        try!(file.read_to_end(&mut buf));
        let map : HashMap<T, Vec<u8>>;
        if buf.len() > 0 {
            map = try!(deserialize(&buf));
        } else {
            map = HashMap::new();
        }

        Ok(Database {
            file: Mutex::new(file),
            data: RwLock::new(map),
        })
    }

    /// Insert a given Object into the Database at that key
    ///
    /// This will overwrite any existing objects.
    ///
    /// The Object has to be serializable. For best results do not
    /// put in anything that is expensive to clone.
    pub fn insert<S: Serialize + 'static, K: ?Sized>(&self, key: &K, obj: S) -> BreakResult<()>
        where T: Borrow<K>, K: Hash + PartialEq + ToOwned<Owned=T>
    {
        use bincode::serde::serialize;
        use bincode::SizeLimit;
        let mut map = match  self.data.write() {
            Ok(guard) => guard,
            Err(_) => unimplemented!(), // TODO: Implement recovery?
        };
        map.insert(key.to_owned(), try!(serialize(&obj, SizeLimit::Infinite)));
        Ok(())
    }

    /// Retrieves an Object from the Database
    ///
    /// # Errors
    ///
    /// This will return an `Err(BreakError::NotFound)` if there is no key behind the object.
    /// If you tried to request something that it can't be serialized to then
    /// `Err(BreakError::Deserialize)` will be returned.
    ///
    /// # Example
    ///
    /// ```
    /// use rustbreak::{Database, BreakError};
    ///
    /// let db = Database::open("/tmp/stuff").unwrap();
    ///
    /// for i in 0..5i64 {
    ///     db.insert(&format!("num_{}", i), i*i*i).unwrap();
    /// }
    ///
    /// let num : i64 = db.retrieve::<i64, str>("num_0").unwrap();
    /// assert_eq!(num, 0);
    /// match db.retrieve::<usize, str>("non-existent") {
    ///     Err(BreakError::NotFound) => {},
    ///     _ => panic!("Was still found?"),
    /// }
    ///
    /// match db.retrieve::<Vec<String>, str>("num_1") {
    ///     Err(BreakError::Deserialize(..)) => {},
    ///     _ => panic!("Was deserialized?"),
    /// }
    /// ```
    pub fn retrieve<S: Deserialize, K: ?Sized>(&self, key: &K) -> BreakResult<S>
        where T: Borrow<K>, K: Hash + Eq
    {
        use bincode::serde::deserialize;
        let map = match  self.data.read() {
            Ok(guard) => guard,
            Err(_) => unimplemented!(), // TODO: Implement recovery?
        };
        match map.get(key.borrow()) {
            Some(t) => Ok(try!(deserialize(t))),
            None => Err(BreakError::NotFound),
        }
    }

    /// Flushes the Database to disk
    pub fn flush(&mut self) -> BreakResult<()> {
        use bincode::serde::serialize;
        use bincode::SizeLimit;
        use std::io::Write;

        let map = match self.data.read() {
            Ok(guard) => guard,
            Err(_) => unimplemented!(), // TODO: Implement recovery?
        };

        let mut file = match self.file.lock() {
            Ok(guard) => guard,
            Err(_) => unimplemented!(),
        };

        let buf = try!(serialize(&*map, SizeLimit::Infinite));
        try!(file.write(&buf));
        try!(file.flush());
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::Database;
    use tempfile::NamedTempFile;

    #[test]
    fn simple_insert_and_retrieve() {
        let tmpf = NamedTempFile::new().unwrap();
        let db = Database::open(tmpf.path()).unwrap();

        db.insert("test", "Hello World!").unwrap();
        let hello : String = db.retrieve("test").unwrap();
        assert_eq!(hello, "Hello World!");
    }

    #[test]
    fn test_persistence() {
        let tmpf = NamedTempFile::new().unwrap();
        let mut db = Database::open(tmpf.path()).unwrap();
        db.insert("test", "Hello World!").unwrap();
        db.flush().unwrap();
        drop(db);
        let db : Database<String> = Database::open(tmpf.path()).unwrap();
        let hello : String = db.retrieve("test").unwrap();
        assert_eq!(hello, "Hello World!");
    }
}
