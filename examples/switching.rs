extern crate rustbreak;
#[macro_use] extern crate serde_derive;

use rustbreak::{FileDatabase, FileBackend};
use rustbreak::deser::{Ron, Yaml};

#[derive(Eq, PartialEq, Debug, Serialize, Deserialize, Clone)]
enum Country {
    Italy, UnitedKingdom
}

#[derive(Eq, PartialEq, Debug, Serialize, Deserialize, Clone)]
struct Person {
    name: String,
    country: Country,
}

fn main() {
    use std::collections::HashMap;

    let db = FileDatabase::<HashMap<String, Person>, Ron>::from_path(HashMap::new(),
                                                                     Ron, "test.ron").unwrap();

    println!("Writing to Database");
    db.write(|db| {
        db.insert("john".into(), Person {
            name: String::from("John Andersson"),
            country: Country::Italy
        });
        db.insert("fred".into(), Person {
            name: String::from("Fred Johnson"),
            country: Country::UnitedKingdom
        });
        println!("Entries: \n{:#?}", db);
    }).unwrap();

    println!("Syncing Database");
    db.sync().unwrap();

    // Now lets switch it

    let db = db.with_deser(Yaml).with_backend(FileBackend::open("test.yml").unwrap());
    db.sync().unwrap();

}

