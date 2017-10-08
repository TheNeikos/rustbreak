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
//! It uses [bincode][bincode] or yaml to compactly save data.
//! It is thread safe and very fast due to staying in memory until flushed to disk.
//!
//! It can be used for short-lived processes or with long-lived ones:
//!
//! ```rust
//! use rustbreak::{Database, Result};
//!
//! fn get_data(key: &str) -> Result<String> {
//!     let db = try!(Database::<String>::open("/tmp/database"));
//!     db.retrieve(key)
//! }
//! ```
//!
//! ```rust
//! # #[macro_use] extern crate lazy_static;
//! # extern crate rustbreak;
//! use rustbreak::{Database, Result};
//!
//! lazy_static! {
//!     static ref DB: Database<String> = {
//!         Database::open("/tmp/more_data").unwrap()
//!     };
//! }
//!
//! fn get_data(key: &str) -> Result<u64> {
//!     DB.retrieve(key)
//! }
//!
//! fn set_data(key: &str, d: u64) -> Result<()> {
//!     let mut lock = try!(DB.lock());
//!     let old_data : u64 = try!(lock.retrieve(key));
//!     lock.insert(key, d + old_data)
//! }
//!
//! # fn main() {}
//! ```
//!
//! [daybreak]:https://propublica.github.io/daybreak/
//! [bincode]:https://github.com/TyOverby/bincode

extern crate serde;
#[macro_use] extern crate quick_error;
extern crate fs2;
#[cfg(feature = "bin")] extern crate bincode;
#[cfg(feature = "yaml")] extern crate serde_yaml;
#[cfg(test)] extern crate tempfile;

mod error;
#[cfg(feature = "bin")] mod bincode_enc;
#[cfg(feature = "yaml")] mod yaml_enc;

mod enc {
    #[cfg(feature = "bin")] pub use bincode_enc::*;
    #[cfg(feature = "yaml")] pub use yaml_enc::*;
}

use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::sync::{RwLock, RwLockWriteGuard, Mutex};
use std::hash::Hash;
use std::borrow::Borrow;
use std::fmt::{Debug, Formatter};
use std::fmt::Error as FmtError;
use std::result::Result as RResult;

use serde::Serialize;
use serde::de::DeserializeOwned;

pub use error::BreakError;

/// Alias for our Result Type
pub type Result<T> = ::std::result::Result<T, BreakError>;

/// The Database structure
///
/// # Notes
/// One should create this once for each Database instance.
/// Subsequent tries to open the same file should fail or worse, could break the database.
///
/// # Example
///
/// ```
/// use rustbreak::Database;
///
/// let db = Database::open("/tmp/artists").unwrap();
///
/// let albums = vec![
///     ("What you do", "The Queenstons"),
///     ("Experience", "The Prodigy"),
/// ];
///
/// for (album, artist) in albums {
///     db.insert(&format!("album_{}",album), artist).unwrap();
/// }
/// db.flush().unwrap();
/// ```
pub struct Database<T: Serialize + DeserializeOwned + Eq + Hash> {
    file: Mutex<File>,
    data: RwLock<HashMap<T, enc::Repr>>,
}

impl<T: Serialize + DeserializeOwned + Eq + Hash + Debug> Debug for Database<T> {
    fn fmt(&self, _: &mut Formatter) -> RResult<(), FmtError> {
        use enc::deserialize;
        let other_map = self.data.read();

        if let Ok(m) = other_map {
            for (n, v) in m.iter() {

                #[cfg(feature = "yaml")]
                let v = deserialize::<String, String>(v)
                    .unwrap_or(String::from(""));

                #[cfg(feature = "bin")]
                let v = deserialize::<String>(v)
                    .unwrap_or(String::from(""));

                println!("{:?}\n\t{}", n, v);
            }
        }
        Ok(())
    }
}

impl<T: Serialize + DeserializeOwned + Eq + Hash + Debug> Database<T> {
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
    /// let db = Database::open("/tmp/more_artists").unwrap();
    ///
    /// let albums = vec![
    ///     ("What you do", "The Queenstons"),
    ///     ("Experience", "The Prodigy"),
    /// ];
    ///
    /// for (album, artist) in albums {
    ///     db.insert(&format!("album_{}",album), artist).unwrap();
    /// }
    /// db.flush().unwrap();
    /// ```
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Database<T>> {
		use std::fs::OpenOptions;
		use fs2::FileExt;
        use std::io::Read;
        use enc::deserialize;

        let mut file = try!(OpenOptions::new().read(true).write(true).create(true).open(path));
        try!(file.try_lock_exclusive());

        let mut buf = Vec::new();
        try!(file.read_to_end(&mut buf));
        let map : HashMap<T, enc::Repr> = if !buf.is_empty() {
            try!(deserialize(&buf))
        } else {
            HashMap::new()
        };

        Ok(Database {
            file: Mutex::new(file),
            data: RwLock::new(map),
        })
    }

    /// Insert a given Object into the Database at that key
    ///
    /// This will overwrite any existing objects.
    ///
    /// The Object has to be serializable.
    pub fn insert<S: Serialize, K: ?Sized>(&self, key: &K, obj: S) -> Result<()>
        where T: Borrow<K>, K: Hash + PartialEq + ToOwned<Owned=T>
    {
        use enc::serialize;
        let mut map = try!(self.data.write());
        map.insert(key.to_owned(), try!(serialize(&obj)));
        Ok(())
    }

    /// Remove an Object at that key
    pub fn delete<K: ?Sized>(&self, key: &K) -> Result<()>
        where T: Borrow<K>, K: Hash + Eq
    {
        let mut map = try!(self.data.write());
        map.remove(key.to_owned());
        Ok(())
    }

    /// Retrieves an Object from the Database
    ///
    /// # Errors
    ///
    /// This will return an `Err(BreakError::NotFound)` if there is no key behind the object.
    /// If you tried to request something that can't be serialized to then
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
    ///     Err(_) => {},
    ///     _ => panic!("Was deserialized?"),
    /// }
    /// ```
    pub fn retrieve<S: DeserializeOwned, K: ?Sized>(&self, key: &K) -> Result<S>
        where T: Borrow<K>, K: Hash + Eq
    {
        use enc::deserialize;
        let map = try!(self.data.read());
        match map.get(key.borrow()) {
            Some(t) => Ok(try!(deserialize(t))),
            None => Err(BreakError::NotFound),
        }
    }

    /// Checks wether a given key exists in the Database
    pub fn contains_key<S: DeserializeOwned, K: ?Sized>(&self, key: &K) -> Result<bool>
        where T: Borrow<K>, K: Hash + Eq
    {
        let map = try!(self.data.read());
        Ok(map.get(key.borrow()).is_some())
    }

    /// Flushes the Database to disk
    pub fn flush(&self) -> Result<()> {
        use enc::serialize;
        use std::io::{Write, Seek, SeekFrom};

        let map = try!(self.data.read());

        let mut file = try!(self.file.lock());

        let buf = try!(serialize(&*map));
        try!(file.set_len(0));
        try!(file.seek(SeekFrom::Start(0)));
        try!(file.write(&buf.as_ref()));
        try!(file.sync_all());
        Ok(())
    }

    /// Starts a transaction
    ///
    /// A transaction passes through reads but caches writes. This means that if changes do happen
    /// they are processed at the same time. To run them you have to call `run` on the
    /// `Transaction` object.
    pub fn transaction(&self) -> Transaction<T> {
        Transaction {
            lock: &self.data,
            data: RwLock::new(HashMap::new()),
        }
    }

    /// Locks the Database, making sure only the caller can change it
    ///
    /// This write-locks the Database until the `Lock` has been dropped.
    ///
    /// # Panics
    ///
    /// If you panic while holding the lock it will get poisoned and subsequent calls to it will
    /// fail. You will have to re-open the Database to be able to continue accessing it.
    pub fn lock(&self) -> Result<Lock<T>> {
        let map = try!(self.data.write());
        Ok(Lock {
            lock: map,
        })
    }
}

/// Structure representing a lock of the Database
pub struct Lock<'a, T: Serialize + DeserializeOwned + Eq + Hash + 'a> {
    lock: RwLockWriteGuard<'a, HashMap<T, enc::Repr>>,
}

impl<'a, T: Serialize + DeserializeOwned + Eq + Hash + 'a> Lock<'a, T> {
    /// Insert a given Object into the Database at that key
    ///
    /// See `Database::insert` for details
    pub fn insert<S: Serialize, K: ?Sized>(&mut self, key: &K, obj: S) -> Result<()>
        where T: Borrow<K>, K: Hash + PartialEq + ToOwned<Owned=T>
    {
        use enc::serialize;
        self.lock.insert(key.to_owned(), try!(serialize(&obj)));
        Ok(())
    }

    /// Retrieves an Object from the Database
    ///
    /// See `Database::retrieve` for details
    pub fn retrieve<S: DeserializeOwned, K: ?Sized>(&mut self, key: &K) -> Result<S>
        where T: Borrow<K>, K: Hash + Eq
    {
        use enc::deserialize;
        match self.lock.get(key.borrow()) {
            Some(t) => Ok(try!(deserialize(t))),
            None => Err(BreakError::NotFound),
        }
    }

    /// Starts a transaction
    ///
    /// See `Database::transaction` for details
    pub fn transaction<'b>(&'b mut self) -> TransactionLock<'a, 'b, T> {
        TransactionLock {
            lock: self,
            data: RwLock::new(HashMap::new()),
        }
    }

}

/// A `TransactionLock` that is atomic in writes and defensive
///
/// You generate this by calling `transaction` on a `Lock`
/// The transactionlock does not get automatically applied when it is dropped, you have to `run` it.
/// This allows for defensive programming where the values are only applied once it is `run`.
pub struct TransactionLock<'a: 'b, 'b, T: Serialize + DeserializeOwned + Eq + Hash + 'a> {
    lock: &'b mut Lock<'a, T>,
    data: RwLock<HashMap<T, enc::Repr>>,
}

impl<'a: 'b, 'b, T: Serialize + DeserializeOwned + Eq + Hash + 'a> TransactionLock<'a, 'b, T> {
    /// Insert a given Object into the Database at that key
    ///
    /// See `Database::insert` for details
    pub fn insert<S: Serialize, K: ?Sized>(&mut self, key: &K, obj: S) -> Result<()>
        where T: Borrow<K>, K: Hash + PartialEq + ToOwned<Owned=T>
    {
        use enc::serialize;

        let mut map = try!(self.data.write());

        map.insert(key.to_owned(), try!(serialize(&obj)));

        Ok(())
    }

    /// Retrieves an Object from the Database
    ///
    /// See `Database::retrieve` for details
    pub fn retrieve<S: DeserializeOwned, K: ?Sized>(&mut self, key: &K) -> Result<S>
        where T: Borrow<K>, K: Hash + Eq
    {
        use enc::deserialize;
        let other_map = &mut self.lock.lock;
        if other_map.contains_key(key) {
            match other_map.get(key.borrow()) {
                Some(t) => Ok(try!(deserialize(t))),
                None => Err(BreakError::NotFound),
            }
        } else {
            let map =  try!(self.data.read());
            match map.get(key.borrow()) {
                Some(t) => Ok(try!(deserialize(t))),
                None => Err(BreakError::NotFound),
            }
        }
    }

    /// Consumes the TransactionLock and runs it
    pub fn run(self) -> Result<()> {
        let other_map = &mut self.lock.lock;

        let mut map = try!(self.data.write());

        for (k, v) in map.drain() {
            other_map.insert(k, v);
        }

        Ok(())
    }
}

/// A Transaction that is atomic in writes
///
/// You generate this by calling `transaction` on a `Database`
/// The transaction does not get automatically applied when it is dropped, you have to `run` it.
/// This allows for defensive programming where the values are only applied once it is `run`.
pub struct Transaction<'a, T: Serialize + DeserializeOwned + Eq + Hash + 'a> {
    lock: &'a RwLock<HashMap<T, enc::Repr>>,
    data: RwLock<HashMap<T, enc::Repr>>,
}

impl<'a, T: Serialize + DeserializeOwned + Eq + Hash + 'a> Transaction<'a, T> {
    /// Insert a given Object into the Database at that key
    ///
    /// See `Database::insert` for details
    pub fn insert<S: Serialize, K: ?Sized>(&mut self, key: &K, obj: S) -> Result<()>
        where T: Borrow<K>, K: Hash + PartialEq + ToOwned<Owned=T>
    {
        use enc::serialize;

        let mut map = try!(self.data.write());

        map.insert(key.to_owned(), try!(serialize(&obj)));

        Ok(())
    }

    /// Retrieves an Object from the Database
    ///
    /// See `Database::retrieve` for details
    pub fn retrieve<S: DeserializeOwned, K: ?Sized>(&self, key: &K) -> Result<S>
        where T: Borrow<K>, K: Hash + Eq
    {
        use enc::deserialize;
        let other_map = try!(self.lock.read());
        if other_map.contains_key(key) {
            match other_map.get(key.borrow()) {
                Some(t) => Ok(try!(deserialize(t))),
                None => Err(BreakError::NotFound),
            }
        } else {
            let map =  try!(self.data.read());
            match map.get(key.borrow()) {
                Some(t) => Ok(try!(deserialize(t))),
                None => Err(BreakError::NotFound),
            }
        }
    }

    /// Consumes the Transaction and runs it
    pub fn run(self) -> Result<()> {
        let mut other_map = try!(self.lock.write());

        let mut map = try!(self.data.write());

        for (k, v) in map.drain() {
            other_map.insert(k, v);
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::{Database,BreakError};
    use tempfile::NamedTempFile;

    #[test]
    fn insert_and_delete() {
        let tmpf = NamedTempFile::new().unwrap();
        let db = Database::open(tmpf.path()).unwrap();

        db.insert("test", "Hello World!").unwrap();
        db.delete("test").unwrap();
        let hello : Result<String,BreakError> = db.retrieve("test");
        assert!(hello.is_err())
    }

    #[test]
    fn simple_insert_and_retrieve() {
        let tmpf = NamedTempFile::new().unwrap();
        let db = Database::open(tmpf.path()).unwrap();

        db.insert("test", "Hello World!").unwrap();
        let hello : String = db.retrieve("test").unwrap();
        assert_eq!(hello, "Hello World!");
    }

    #[test]
    fn simple_insert_and_retrieve_borrow() {
        let tmpf = NamedTempFile::new().unwrap();
        let db = Database::open(tmpf.path()).unwrap();

        db.insert("test", &25).unwrap();
        let hello : u32 = db.retrieve("test").unwrap();
        assert_eq!(hello, 25);
    }

    #[test]
    fn test_persistence() {
        let tmpf = NamedTempFile::new().unwrap();
        let db = Database::open(tmpf.path()).unwrap();
        db.insert("test", "Hello World!").unwrap();
        db.flush().unwrap();
        drop(db);
        let db : Database<String> = Database::open(tmpf.path()).unwrap();
        let hello : String = db.retrieve("test").unwrap();
        assert_eq!(hello, "Hello World!");
    }

    #[test]
    fn simple_transaction() {
        let tmpf = NamedTempFile::new().unwrap();
        let db = Database::open(tmpf.path()).unwrap();
        assert!(db.retrieve::<String, str>("test").is_err());
        {
            let mut trans = db.transaction();
            trans.insert("test", "Hello World!").unwrap();
            trans.run().unwrap();
        }
        {
            let mut trans = db.transaction();
            trans.insert("test", "Hello World too!!").unwrap();
            drop(trans);
        }
        let hello : String = db.retrieve("test").unwrap();
        assert_eq!(hello, "Hello World!");
    }

    #[test]
    fn multithreaded_locking() {
        use std::sync::Arc;
        let tmpf = NamedTempFile::new().unwrap();
        let db = Arc::new(Database::open(tmpf.path()).unwrap());
        db.insert("value", 0i64).unwrap();
        let mut threads = vec![];
        for _ in 0..10 {
            use std::thread;
            let a = db.clone();
            threads.push(thread::spawn(move || {
                let mut lock = a.lock().unwrap();
                {
                    let mut trans = lock.transaction();
                    let x = trans.retrieve::<i64, str>("value").unwrap();
                    trans.insert("value", x + 1).unwrap();
                    trans.run().unwrap();
                }
                {
                    let mut trans = lock.transaction();
                    let x = trans.retrieve::<i64, str>("value").unwrap();
                    trans.insert("value", x - 1).unwrap();
                    drop(trans);
                }
            }));
        }
        for thr in threads {
            thr.join().unwrap();
        }
        let x = db.retrieve::<i64, str>("value").unwrap();
        assert_eq!(x, 10);
    }
}
