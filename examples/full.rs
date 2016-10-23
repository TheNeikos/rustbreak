extern crate rustbreak;
#[macro_use] extern crate lazy_static;

use rustbreak::{Database, Result as BreakResult};

lazy_static! {
    static ref DB: Database<String> = {
        Database::open("music").unwrap()
    };
}

type Artist = String;
type Album = String;
type AlbumArtist = (Album, Artist);

fn add_album_to_artist(name: Artist, album: Album) -> BreakResult<()> {
    let mut lock = try!(DB.lock());
    let mut map: Vec<Album> = lock.retrieve(&name).unwrap_or_else(|_| vec![]);
    map.push(album);
    try!(lock.insert(&name, map));
    Ok(())
}

fn get_albums(name: &str) -> BreakResult<Vec<Album>> {
    DB.retrieve(name)
}

fn main() {
    let albums = [
        ("The Queenstons", "What you do EP"),
        ("The Queenstons", "Figurehead"),
        ("The Queenstons", "Undertones"),
        ("System of a Down", "Toxicity II"),
        ("System of a Down", "Mezmerize"),
    ];

    for &(artist, album) in albums.iter() {
        add_album_to_artist(artist.to_owned(), album.to_owned()).unwrap();
    }

    for al in get_albums("The Queenstons").unwrap() {
        println!("{}", al);
    }
}
