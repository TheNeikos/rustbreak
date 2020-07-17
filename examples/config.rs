// This just reads an example configuration.
// If it doesn't find one, it uses your default configuration
//
// You can create one by writing this file to `/tmp/config.ron`:
// ```
// ---
// user_path: /tmp/nope
// allow_overwrite: true
// ```
//

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;

use rustbreak::deser::Ron;
use rustbreak::FileDatabase;
use std::default::Default;
use std::path::PathBuf;

type DB = FileDatabase<Config, Ron>;

lazy_static! {
    static ref CONFIG: DB = {
        let db = FileDatabase::load_from_path_or_default("/tmp/config.ron")
            .expect("Create database from path");
        db.load().expect("Config to load");
        db
    };
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Config {
    user_path: PathBuf,
    allow_overwrite: bool,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            user_path: PathBuf::from("/tmp"),
            allow_overwrite: false,
        }
    }
}

fn main() {
    let _conf: Config = CONFIG
        .read(|conf| conf.clone())
        .expect("Reading configuration");

    let (user_path, allow_overwrite) = CONFIG
        .read(|conf| (conf.user_path.clone(), conf.allow_overwrite.clone()))
        .expect("Read config");

    println!(
        "The current configuration is: {:?} and {}",
        user_path, allow_overwrite
    );
}
