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

Features
--------

- Simple To Use, Fast, Secure
- Threadsafe
- Key/Value Storage
- bincode or yaml storage

Usage
-----

Usage is quite simple:

- Create/open a database using `Database::open`
    - You can specify the kind of Key you want using this Syntax:
      `Database::<Key>::open`
- `Insert`/`Retrieve` data from the Database
- Don't forget to run `flush` periodically

```rust
use rustbreak::Database;

fn main() {
    let db = Database::open("my_contacts").unwrap();

    db.insert("Lapfox", "lapfoxtrax.com").unwrap();
    db.insert("Rust", "github.com/rust-lang/rust").unwrap();

    // we need to be explicit about the kind of type we want as println! is
    // generic
    let rust : String = db.retrieve("Rust").unwrap();
    println!("You can find Rust at: {}", rust);
    db.flush().unwrap();
}
```

### Yaml

If you would like to use yaml instead of bincode to perhaps read or modify the
database in an editor you can use it like this:

- Disable default features
- Specify yaml as a feature

```toml
[dependencies.rustbreak]
version = "1"
default-features = false
features = ["yaml"]
```

### Ron

If you would like to use [`ron`](https://github.com/ron-rs/ron) instead of bincode:

- Disable default features
- Specify ron_enc as a feature

```toml
[dependencies.rustbreak]
version = "1"
default-features = false
features = ["ron_enc"]
```

How it works
------------

Internally the Database holds a Hashmap behind a RwLock.
This Hashmap is then written to/read from and safely casted to the requested
type. This works thanks to encoding/decoding traits.



[doc]:http://neikos.me/rustbreak/rustbreak/index.html
[Daybreak]:https://propublica.github.io/daybreak/

