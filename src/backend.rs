/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::io::{Seek, SeekFrom, Read, Write, Result as IOResult};

/// Marker for Resizing action
///
/// This is used to truncate the backend to a given size
pub trait Resizable {
    /// Sets the backend to the given size
    fn resize(&mut self, size: usize) -> IOResult<()>;
    /// Syncs the given backend to the support it uses
    fn sync(&mut self) -> IOResult<()>;
}

impl Resizable for ::std::fs::File {
    fn resize(&mut self, size: usize) -> IOResult<()> {
        self.set_len(size as u64)
    }
    fn sync(&mut self) -> IOResult<()> {
        self.sync_all()
    }
}

#[derive(Debug)]
pub struct RWVec(Vec<u8>, usize);

impl Resizable for RWVec {
    fn resize(&mut self, size: usize) -> IOResult<()> {
        self.0.resize(size, 0);
        Ok(())
    }
    fn sync(&mut self) -> IOResult<()> {
        Ok(())
    }
}

impl<'r> Read for &'r mut RWVec {
    fn read(&mut self, buf: &mut [u8]) -> IOResult<usize> {
        println!("Called with: len: {} pos: {}", self.0.len(), self.1);
        let len = (&self.0[(self.1)..]).read(buf)?;
        self.1 += len;
        Ok(len)
    }
}

impl Write for RWVec {
    fn write(&mut self, buf: &[u8]) -> IOResult<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> IOResult<()> {
        self.0.flush()
    }
}

impl Seek for RWVec {
    fn seek(&mut self, seek: SeekFrom) -> IOResult<u64> {
        match seek {
            SeekFrom::Start(start) => self.1 = start as usize,
            SeekFrom::End(end) => self.1 = (self.0.len() as i64 - end) as usize,
            SeekFrom::Current(off) => self.1 = match self.1 as i64 - off {
                x if x < 0 => 0,
                x => x as usize
            }
        }
        Ok(self.1 as u64)
    }
}

impl RWVec {
    pub fn new() -> RWVec {
        RWVec(vec![], 0)
    }
}
