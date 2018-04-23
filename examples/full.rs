extern crate rustbreak;
#[macro_use] extern crate serde_derive;

use rustbreak::FileDatabase;
use rustbreak::deser::Ron;

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

    let db = FileDatabase::<HashMap<String, Person>, Ron>::from_path("test.ron", HashMap::new()).unwrap();

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

    println!("Reloading Database");
    db.reload().unwrap();

    println!("Reading from Database");
    db.read(|db| {
        println!("Results:");
        println!("{:#?}", db);
    }).unwrap();


}
