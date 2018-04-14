// This just reads an example configuration. Don't forget to run with `--feature yaml`
// If it doesn't find one, it uses your default configuration
//
// You can create one by writing this file to `/tmp/config.yml`:
// ```
// ---
// user_path: /tmp/nope
// allow_overwrite: true
// ```
//

extern crate rustbreak;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate lazy_static;

use std::fs::File;
use std::path::PathBuf;
use std::default::Default;
use rustbreak::Database;
use rustbreak::deser::Yaml;

type DB = Database<Config, File, Yaml>;

lazy_static! {
    static ref CONFIG: DB = {
        let db = Database::from_path(Config::default(), "/tmp/config.yml").expect("Create database from path");
        let db = db.with_deser(Yaml);
        db.reload().expect("Config to load");
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
    let _conf : Config = CONFIG.read(|conf| {
        conf.clone()
    }).expect("Reading configuration");

    let (user_path, allow_overwrite) =
        CONFIG.read(|conf| (conf.user_path.clone(), conf.allow_overwrite.clone())).expect("Read config");

    println!("The current configuration is: {:?} and {}", user_path, allow_overwrite);
}
