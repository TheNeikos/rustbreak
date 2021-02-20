Rustbreak
=========

[![Build Status](https://travis-ci.org/TheNeikos/rustbreak.svg?branch=master)](https://travis-ci.org/TheNeikos/rustbreak)
[![Crates Link](https://img.shields.io/crates/v/rustbreak.svg)](https://crates.io/crates/rustbreak)

**[Documentation][doc]**

Rustbreak is an [Daybreak] inspired self-contained file
database. It is meant to be fast and simple to use. You add it to your
application and it should just work for you. The only thing you will have to
take care of is saving.

When to use it
--------------

This library started out because of a need to be able to quickly write an
application in rust that needed some persistence while still being able to write
arbitrary data to it.

In Ruby there is [Daybreak] however for Rust there was no similar crate, until
now!

When not to use it
------------------

Rustbreak makes several trade-offs to be easy to use and extend, so knowing of these drawbacks is important if 
you wish to use the library:

- The Database needs to fit into memory (Rustbreak cannot do partial loads/saves, so if the Database exceeds your available memory you will run OOM)
- Not all backends support atomic saves, so if your program crashes while it is saving you might save incomplete data (Notably only `PathBackend` supports atomic saves)

Features
--------

- Simple To Use, Fast, Secure
- Threadsafe
- Serde compatible storage (ron, bincode, or yaml included)

Quickstart
----------

Add this to your `Cargo.toml`:

```toml
[dependencies.rustbreak]
version = "2"
features = ["ron_enc"] # You can also use "yaml_enc" or "bin_enc"
                       # Check the documentation to add your own!
```

```rust
extern crate rustbreak;
use std::collections::HashMap;
use rustbreak::{MemoryDatabase, deser::Ron};

fn main() -> rustbreak::Result<()> {
    let db = MemoryDatabase::<HashMap<u32, String>, Ron>::memory(HashMap::new())?;

    println!("Writing to Database");
    db.write(|db| {
        db.insert(0, String::from("world"));
        db.insert(1, String::from("bar"));
    });

    db.read(|db| {
        // db.insert("foo".into(), String::from("bar"));
        // The above line will not compile since we are only reading
        println!("Hello: {:?}", db.get(&0));
    })?;

    Ok(())
}
```

Usage
-----

Usage is quite simple:

- Create/open a database using one of the Database constructors:
    - Create a `FileDatabase` with `FileDatabase::from_path`.
    - Create a `MemoryDatabase` with `MemoryDatabase::memory`.
    - Create a `MmapDatabase` with `MmapDatabase::mmap` or `MmapDatabase::mmap_with_size` with `mmap` feature.
    - Create a `Database` with `Database::from_parts`.
- `Write`/`Read` data from the Database
- Don't forget to run `save` periodically, or whenever it makes sense.
    - You can save in parallel to using the Database. However you will lock
      write acess while it is being written to storage.

```rust
# use std::collections::HashMap;
use rustbreak::{MemoryDatabase, deser::Ron};

let db = MemoryDatabase::<HashMap<String, String>, Ron>::memory(HashMap::new(), Ron);

println!("Writing to Database");
db.write(|db| {
    db.insert("hello".into(), String::from("world"));
    db.insert("foo".into(), String::from("bar"));
});

db.read(|db| {
    // db.insert("foo".into(), String::from("bar"));
    // The above line will not compile since we are only reading
    println!("Hello: {:?}", db.get("hello"));
});
```

## Encodings

The following parts explain how to enable the respective features. You can also
enable several at the same time.

### Yaml

If you would like to use yaml you need to specify `yaml_enc` as a feature:

```toml
[dependencies.rustbreak]
version = "2"
features = ["yaml_enc"]
```

You can now use `rustbreak::deser::Yaml` as deserialization struct.

### Ron

If you would like to use [`ron`](https://github.com/ron-rs/ron) you need to
specify `ron_enc` as a feature:

```toml
[dependencies.rustbreak]
version = "2"
features = ["ron_enc"]
```

You can now use `rustbreak::deser::Ron` as deserialization struct.

### Bincode

If you would like to use bincode you need to specify `bin_enc` as a feature:

```toml
[dependencies.rustbreak]
version = "2"
features = ["bin_enc"]
```

You can now use `rustbreak::deser::Bincode` as deserialization struct.


[doc]:http://neikos.me/rustbreak/rustbreak/index.html
[Daybreak]:https://propublica.github.io/daybreak/
