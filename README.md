Rustbreak
=========

[![Build Status](https://travis-ci.org/TheNeikos/rustbreak.svg?branch=master)](https://travis-ci.org/TheNeikos/rustbreak)

**[Documentation][doc]**

Rustbreak is an [Daybreak](https://propublica.github.io/daybreak/) inspired file
database. It is meant to be fast and simple to use.

Features
--------

- Simple To Use, Fast, Secure
- Threadsafe

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

[doc]:http://neikos.me/rustbreak/rustbreak/index.html
