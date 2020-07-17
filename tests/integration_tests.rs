use rustbreak::backend::Backend;
use rustbreak::deser::{Bincode, DeSerializer, Ron, Yaml};
use rustbreak::{Database, FileDatabase, MemoryDatabase, MmapDatabase, PathDatabase};
use std::fmt::Debug;
use std::ops::Deref;
use tempfile::tempfile;

type Data = std::collections::HashMap<u64, String>;

fn conv(mut data: Data) -> Data {
    data.remove(&2);
    data.insert(0, "Newly inserted".to_string());
    data.insert(16, "Convertion succesful".to_string());
    data
}

fn test_basic_save_load<B: Backend + Debug, S: DeSerializer<Data> + Debug>(
    db: &Database<Data, B, S>,
) {
    db.write(|db| {
        db.insert(2, "Hello world!".to_string());
    })
    .expect("rustbreak write error");
    db.write_safe(|db| {
        db.insert(5, "Hello again".to_string());
    })
    .expect("rustbreak write error");
    db.save().expect("error while saving");
    let saved_state = db.get_data(false).expect("could not get data");

    // test that loading correctly restores the data
    db.write(|db| {
        db.clear();
    })
    .expect("rustbreak write error");
    db.load().expect("rustbreak load error");

    let len = db.read(|db| db.len()).expect("rustbreak read error");
    assert_eq!(len, 2);

    let second = db
        .read(|db| db.get(&2).cloned())
        .expect("rustbreak read error");
    assert_eq!(second, Some(String::from("Hello world!")));

    let fith = db
        .read(|db| db.get(&5).cloned())
        .expect("rustbreak read error");
    assert_eq!(fith, Some(String::from("Hello again")));

    let data = db.borrow_data().expect("rustbreak borrow error");
    assert_eq!(&saved_state, data.deref());
}

fn test_multi_borrow<B: Backend + Debug, S: DeSerializer<Data> + Debug>(db: &Database<Data, B, S>) {
    let data1 = db.borrow_data().expect("rustbreak borrow error");
    let data2 = db.borrow_data().expect("rustbreak borrow error");
    let data3 = db.borrow_data().expect("rustbreak borrow error");
    assert_eq!(data1.deref(), data2.deref());
    assert_eq!(data1.deref(), data3.deref());
}

fn test_put_data<B: Backend + Debug, S: DeSerializer<Data> + Debug>(db: &Database<Data, B, S>) {
    let backup = db.get_data(true).expect("could not get data");

    let mut other_state = Data::new();
    other_state.insert(3, "Foo".to_string());
    other_state.insert(7, "Bar".to_string());
    other_state.insert(19, "Bazz".to_string());

    db.put_data(other_state.clone(), true)
        .expect("could not put data");
    let data = db.borrow_data().expect("rustbreak borrow error");
    assert_eq!(&other_state, data.deref());
    // If we do not explicitly drop `data` here, the subsequent write will freeze
    drop(data);

    db.write(|db| {
        db.clear();
    })
    .expect("rustbreak write error");
    db.load().expect("rustbreak load error");

    let data = db.borrow_data().expect("rustbreak borrow error");
    assert_eq!(&other_state, data.deref());
    drop(data);

    db.put_data(backup, false).expect("could not put data");
}

fn test_convert_data<B: Backend + Debug, S: DeSerializer<Data> + Debug>(db: Database<Data, B, S>) {
    let db = db.convert_data(conv).expect("Could not convert data");

    let mut expected_state = Data::new();
    expected_state.insert(0, "Newly inserted".to_string());
    expected_state.insert(5, "Hello again".to_string());
    expected_state.insert(16, "Convertion succesful".to_string());
    assert_eq!(
        &expected_state,
        db.borrow_data().expect("rustbreak borrow error").deref()
    );
}

fn create_filedb<S: DeSerializer<Data> + Debug>() -> FileDatabase<Data, S> {
    FileDatabase::from_file(tempfile().expect("could not create file"), Data::default())
        .expect("could not create database")
}

fn create_filedb_from_path<S: DeSerializer<Data> + Debug>() -> FileDatabase<Data, S> {
    let file = tempfile::NamedTempFile::new().expect("could not create temporary file");
    FileDatabase::create_at_path(file.path(), Data::default()).expect("could not create database")
}

fn create_memdb<S: DeSerializer<Data> + Debug>() -> MemoryDatabase<Data, S> {
    MemoryDatabase::memory(Data::default()).expect("could not create database")
}

fn create_mmapdb<S: DeSerializer<Data> + Debug>() -> MmapDatabase<Data, S> {
    MmapDatabase::mmap(Data::default()).expect("could not create database")
}

fn create_mmapdb_with_size<S: DeSerializer<Data> + Debug>(size: usize) -> MmapDatabase<Data, S> {
    MmapDatabase::mmap_with_size(Data::default(), size).expect("could not create database")
}

fn create_pathdb<S: DeSerializer<Data> + Debug>() -> PathDatabase<Data, S> {
    let file = tempfile::NamedTempFile::new().expect("could not create temporary file");
    PathDatabase::create_at_path(file.path().to_owned(), Data::default())
        .expect("could not create database")
}

macro_rules! test_basic_save_load {
    ($name:ident, $db:expr, $enc:ty) => {
        #[test]
        #[cfg_attr(miri, ignore)]
        fn $name() {
            let db: Database<Data, _, $enc> = $db;
            test_basic_save_load(&db);
            test_put_data(&db);
            test_multi_borrow(&db);
            test_convert_data(db);
        }
    };
    ($name:ident, $db:expr, $enc:ty, miri=true) => {
        #[test]
        fn $name() {
            let db: Database<Data, _, $enc> = $db;
            test_basic_save_load(&db);
            test_put_data(&db);
            test_multi_borrow(&db);
            test_convert_data(db);
        }
    };
}

test_basic_save_load!(file_ron, create_filedb(), Ron);
test_basic_save_load!(file_yaml, create_filedb(), Yaml);
test_basic_save_load!(file_bincode, create_filedb(), Bincode);

test_basic_save_load!(filepath_ron, create_filedb_from_path(), Ron);
test_basic_save_load!(filepath_yaml, create_filedb_from_path(), Yaml);
test_basic_save_load!(filepath_bincode, create_filedb_from_path(), Bincode);

test_basic_save_load!(mem_ron, create_memdb(), Ron, miri = true);
test_basic_save_load!(mem_yaml, create_memdb(), Yaml, miri = true);
test_basic_save_load!(mem_bincode, create_memdb(), Bincode, miri = true);

test_basic_save_load!(mmap_ron, create_mmapdb(), Ron);
test_basic_save_load!(mmap_yaml, create_mmapdb(), Yaml);
test_basic_save_load!(mmap_bincode, create_mmapdb(), Bincode);

test_basic_save_load!(mmapsize_ron, create_mmapdb_with_size(10), Ron);
test_basic_save_load!(mmapsize_yaml, create_mmapdb_with_size(10), Yaml);
test_basic_save_load!(mmapsize_bincode, create_mmapdb_with_size(10), Bincode);

test_basic_save_load!(path_ron, create_pathdb(), Ron);
test_basic_save_load!(path_yaml, create_pathdb(), Yaml);
test_basic_save_load!(path_bincode, create_pathdb(), Bincode);
