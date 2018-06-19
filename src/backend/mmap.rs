use memmap;
use failure;
use failure::ResultExt;

use super::Backend;

use error;

use std::cmp;
use std::io;

#[derive(Debug)]
struct Mmap {
    inner: memmap::MmapMut,
    //End of data
    pub end: usize,
    //Mmap total len
    pub len: usize
}

impl Mmap {
    fn new(len: usize) -> io::Result<Self> {
        let inner = memmap::MmapOptions::new().len(len)
            .map_anon()?;

        Ok(Self {
            inner,
            end: 0,
            len
        })
    }

    fn as_slice(&self) -> &[u8] {
        &self.inner[..self.end]
    }

    fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.inner[..self.end]
    }

    //Copies data to mmap and modifies data's end cursor.
    fn write(&mut self, data: &[u8]) -> Result<(), failure::Error> {
        if data.len() > self.len {
            return Err(failure::err_msg("Unexpected write beyond mmap's backend capacity. This is a rustbreak's bug"));
        }
        self.end = data.len();
        self.as_mut_slice().copy_from_slice(data);
        Ok(())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }

    //Increases mmap size by max(old_size*2, new_size)
    //Note that it doesn't copy original data
    fn resize_no_copy(&mut self, new_size: usize) -> io::Result<()> {
        let len = cmp::max(self.len + self.len, new_size);
        //Make sure we don't discard old mmap before creating new one;
        let new_mmap = Self::new(len)?;
        *self = new_mmap;
        Ok(())
    }
}

/// A backend that uses anonymous mmap.
///
/// The `Backend` automatically creates bigger map
/// on demand using following strategy:
///
/// - If new data size allows, multiply size by 2.
/// - Otherwise new data size is used.
///
/// Note that mmap is never shrink back.
///
/// Use `Backend` methods to read and write into it.
#[derive(Debug)]
pub struct MmapStorage {
    mmap: Mmap
}

impl MmapStorage {
    ///Creates new storage with 1024 bytes
    pub fn new() -> error::Result<Self> {
        Self::with_size(1024)
    }

    ///Creates new storage with custom size.
    pub fn with_size(len: usize) -> error::Result<Self> {
        let mmap = Mmap::new(len).context(error::RustbreakErrorKind::Backend)?;

        Ok(Self {
            mmap
        })
    }
}

impl Backend for MmapStorage {
    fn get_data(&mut self) -> error::Result<Vec<u8>> {
        let mmap = self.mmap.as_slice();
        let mut buffer = Vec::with_capacity(mmap.len());
        buffer.extend_from_slice(mmap);
        Ok(buffer)
    }

    fn put_data(&mut self, data: &[u8]) -> error::Result<()> {
        if self.mmap.len < data.len() {
            self.mmap.resize_no_copy(data.len()).context(error::RustbreakErrorKind::Backend)?;
        }
        self.mmap.write(data).context(error::RustbreakErrorKind::Backend)?;
        self.mmap.flush().context(error::RustbreakErrorKind::Backend)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{Backend, MmapStorage};

    #[test]
    fn test_mmap_storage() {
        let data = [4, 5, 1, 6, 8, 1];
        let mut storage = MmapStorage::new().expect("To crate mmap storage");

        storage.put_data(&data).expect("To put data");
        assert_eq!(storage.mmap.end, data.len());
        assert_eq!(storage.get_data().expect("To get data"), data);
    }

    #[test]
    fn test_mmap_storage_extend() {
        let data = [4, 5, 1, 6, 8, 1];
        let mut storage = MmapStorage::with_size(4).expect("To crate mmap storage");

        storage.put_data(&data).expect("To put data");
        assert_eq!(storage.mmap.end, data.len());
        assert_eq!(storage.mmap.len, 8);
        assert_eq!(storage.get_data().expect("To get data"), data);
    }

    #[test]
    fn test_mmap_storage_increase_by_new_data_size() {
        let data = [4, 5, 1, 6, 8, 1];
        let mut storage = MmapStorage::with_size(1).expect("To crate mmap storage");

        storage.put_data(&data).expect("To put data");
        assert_eq!(storage.mmap.end, data.len());
        assert_eq!(storage.mmap.len, data.len());
        assert_eq!(storage.get_data().expect("To get data"), data);
    }
}
