extern crate rustbreak;
#[macro_use] extern crate serde_derive;

use rustbreak::Database;

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
    let db = Database::from_path("test.yaml").unwrap().with_deser(rustbreak::deser::Yaml);

    println!("Writing to Database");
    db.write(|mut db| {
        db.insert("john".into(), Person {
            name: String::from("John Andersson"),
            country: Country::Italy
        });
        db.insert("fred".into(), Person {
            name: String::from("Fred Johnson"),
            country: Country::UnitedKingdom
        });
    }).unwrap();

    println!("Syncing Database");
    db.sync().unwrap();

    println!("Reloading Database");
    db.reload().unwrap();

    let mut john = None;
    let mut fred = None;

    println!("Reading from Database");
    db.read(|db| {
        // We want to take things out of the Database, so we clone
        john = db.get("john").cloned();
        fred = db.get("fred").cloned();
    }).unwrap();

    println!("Results:");
    println!("{:#?}, {:#?}", john, fred);

}
