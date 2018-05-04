/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::fmt::{self, Display};
use failure::{Context, Fail, Backtrace};

/// The different kinds of errors that can be returned
#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum RustbreakErrorKind {
    /// A context error when a serialization failed
    #[fail(display = "Could not serialize the value")]
    Serialization,
    /// A context error when a deserialization failed
    #[fail(display = "Could not deserialize the value")]
    Deserialization,
    /// This error is returned if the `Database` is poisoned. See `Database::write` for details
    #[fail(display = "The database has been poisoned")]
    Poison,
    /// An error in the backend happened
    #[fail(display = "The backend has encountered an error")]
    Backend,
    /// If `Database::write_safe` is used and the closure panics, this error is returned
    #[fail(display = "The write operation paniced but got caught")]
    WritePanic,
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

impl Fail for RustbreakError {
    fn cause(&self) -> Option<&Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl Display for RustbreakError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl RustbreakError {
    /// Get the kind of this error
    pub fn kind(&self) -> RustbreakErrorKind {
        *self.inner.get_context()
    }
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
pub type Result<T> = ::std::result::Result<T, RustbreakError>;
