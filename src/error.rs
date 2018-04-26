/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use failure::{self, Context};

/// The different kinds of errors that can be returned
#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum RustbreakErrorKind {
    /// A context error when a serialization failed
    #[fail(display = "Could not serialize the value")]
    SerializationError,
    /// A context error when a deserialization failed
    #[fail(display = "Could not deserialize the value")]
    DeserializationError,
    /// This error is returned if the `Database` is poisoned. See `Database::write` for details
    #[fail(display = "The database has been poisoned")]
    PoisonError,
    /// If `Database::write_safe` is used and the closure panics, this error is returned
    #[fail(display = "The write operation paniced but got caught")]
    WritePanicError,
    /// This variant should never be used. It is meant to keep this enum forward compatible.
    #[doc(hidden)]
    #[fail(display = "You have found a secret message, please report it to the Rustbreak maintainer")]
    __Nonexhaustive,
}



/// The main error type that gets returned for errors that happen while interacting with a
/// `Database`.
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

/// A simple type alias for errors
pub type Result<T> = ::std::result::Result<T, failure::Error>;
