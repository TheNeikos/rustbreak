extern crate rustbreak;

use rustbreak::Database;

fn main() {
    let db = Database::open("my_contacts").unwrap();

    db.insert("Lapfox", "lapfoxtrax.com").unwrap();
    db.insert("Rust", "github.com/rust-lang/rust").unwrap();

    // we need to be explicit about the kind of type we want as println! is generic
    let rust : String = db.retrieve("Rust").unwrap();
    println!("You can find Rust at: {}", rust);
    db.flush().unwrap();
}
