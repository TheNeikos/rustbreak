/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use failure::{self, Context};

#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum RustbreakErrorKind {
    #[fail(display = "Could not serialize the value")]
    SerializationError,
    #[fail(display = "Could not deserialize the value")]
    DeserializationError,
    #[fail(display = "The database has been poisoned")]
    PoisonError
}


#[derive(Debug)]
pub struct RustbreakError {
    inner: Context<RustbreakErrorKind>,
}

impl From<RustbreakErrorKind> for RustbreakError {
    fn from(kind: RustbreakErrorKind) -> RustbreakError {
        RustbreakError { inner: Context::new(kind) }
    }
}

impl From<Context<RustbreakErrorKind>> for RustbreakError {
    fn from(inner: Context<RustbreakErrorKind>) -> RustbreakError {
        RustbreakError { inner: inner }
    }
}

pub type Result<T> = ::std::result::Result<T, failure::Error>;
