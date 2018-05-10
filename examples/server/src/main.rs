#![feature(plugin, decl_macro, custom_derive)]
#![plugin(rocket_codegen)]

extern crate rustbreak;
extern crate rocket;
extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;

use std::collections::HashMap;

use rocket::{State, Outcome};
use rocket::http::{Cookies, Cookie};
use rocket::request::{self, Request, FromRequest, Form};
use rocket::response::Redirect;
use rocket_contrib::Template;
use rustbreak::FileDatabase;
use rustbreak::deser::Ron;

// We create a type alias so that we always associate the same types to it
type DB = FileDatabase<ServerData, Ron>;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Paste {
    user: String,
    body: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, FromForm)]
struct NewPaste {
    body: String
}

#[derive(Debug, Serialize, Deserialize, Clone, FromForm)]
struct User {
    username: String,
    password: String,
}

impl <'a, 'r> FromRequest<'a, 'r> for User {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<User, ()> {
        let mut cookies = request.cookies();
        let db = request.guard::<State<DB>>()?;
        match cookies.get_private("user_id") {
            Some(cookie) => {
                let mut outcome = Outcome::Forward(());
                let _ = db.read(|db| {
                    if db.users.contains_key(cookie.value()) {
                        outcome = Outcome::Success(db.users.get(cookie.value()).unwrap().clone());
                    }
                });
                return outcome;
            }
            None => return Outcome::Forward(())
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ServerData {
    pastes: Vec<Paste>,
    users: HashMap<String, User>
}

#[derive(Debug, Serialize)]
struct TemplateData {
    pastes: Vec<Paste>,
    logged_in: bool,
    user: String
}


// Routing

#[get("/")]
fn index(db: State<DB>, user: Option<User>) -> Template {
    let mut data = TemplateData {
        logged_in: user.is_some(),
        user: user.map(|u| u.username).unwrap_or_else(|| String::new()),
        pastes: vec![],
    };
    let _ = db.read(|db| {
        data.pastes = db.pastes.clone();
    });

    return Template::render("index", &data);
}

#[post("/register", data = "<req_user>")]
fn post_register(db: State<DB>, req_user: Form<User>, mut cookies: Cookies) -> Redirect {
    let user = req_user.into_inner();
    let _ = db.write(|db| {
        if db.users.contains_key(&user.username) {
            return;
        }
        db.users.insert(user.username.clone(), user.clone());
        cookies.add_private(
            Cookie::build("user_id", user.username.clone()).http_only(true).finish()
        );
    });
    let _ = db.save();

    Redirect::to("/")
}

#[post("/login", data = "<req_user>")]
fn post_login(db: State<DB>, req_user: Form<User>, mut cookies: Cookies) -> Redirect {
    let user = req_user.into_inner();
    let _ = db.read(|db| {
        match db.users.get(&user.username) {
            Some(u) => {
                if u.password == user.password {
                    cookies.add_private(
                        Cookie::build("user_id", user.username.clone()).http_only(true).finish()
                    );
                }
            }
            None => ()
        }
    });

    Redirect::to("/")
}

#[post("/paste", data = "<paste>")]
fn post_paste(db: State<DB>, user: User, paste: Form<NewPaste>) -> Redirect {
    let body : String = paste.into_inner().body.clone();
    let _ = db.write(|db| {
        let paste = Paste {
            body: body,
            user: user.username.clone(),
        };
        db.pastes.push(paste);
    });
    let _ = db.save();

    Redirect::to("/")
}

#[get("/dummy")]
fn get_dummy() -> &'static str {
    "Hello World"
}

fn main() {
    let db : DB = FileDatabase::from_path("server.ron", ServerData {
       pastes: vec![],
       users: HashMap::new(),
    }).unwrap();
    let _ = db.load();


    rocket::ignite()
        .mount("/", routes![index, post_login, post_paste, post_register, get_dummy])
        .attach(Template::fairing())
        .manage(db)
        .launch();
}
