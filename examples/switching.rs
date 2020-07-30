#[macro_use]
extern crate serde_derive;

use rustbreak::deser::{Ron, Yaml};
use rustbreak::{backend::FileBackend, FileDatabase};

#[derive(Eq, PartialEq, Debug, Serialize, Deserialize, Clone)]
enum Country {
    Italy,
    UnitedKingdom,
}

#[derive(Eq, PartialEq, Debug, Serialize, Deserialize, Clone)]
struct Person {
    name: String,
    country: Country,
}

fn do_main() -> Result<(), rustbreak::RustbreakError> {
    use std::collections::HashMap;

    let db = FileDatabase::<HashMap<String, Person>, Ron>::load_from_path_or_default("test.ron")?;

    println!("Writing to Database");
    db.write(|db| {
        db.insert(
            "john".into(),
            Person {
                name: String::from("John Andersson"),
                country: Country::Italy,
            },
        );
        db.insert(
            "fred".into(),
            Person {
                name: String::from("Fred Johnson"),
                country: Country::UnitedKingdom,
            },
        );
        println!("Entries: \n{:#?}", db);
    })?;

    println!("Syncing Database");
    db.save()?;

    // Now lets switch it

    let db = db
        .with_deser(Yaml)
        .with_backend(FileBackend::from_path_or_create("test.yml").map(|p| p.0)?);
    db.save()?;

    Ok(())
}

fn main() {
    if let Err(e) = do_main() {
        eprintln!("An error has occurred at: \n{}", e);
        std::process::exit(1);
    }
}
