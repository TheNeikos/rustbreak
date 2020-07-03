/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use thiserror::Error;

/// The different kinds of errors that can be returned
#[derive(Copy, Clone, Eq, PartialEq, Debug, Error)]
#[non_exhaustive]
pub enum RustbreakError {
    /// A context error when a serialization failed
    #[error("Could not serialize the value")]
    Serialization,
    /// A context error when a deserialization failed
    #[error("Could not deserialize the value")]
    Deserialization,
    /// This error is returned if the `Database` is poisoned. See `Database::write` for details
    #[error("The database has been poisoned")]
    Poison,
    /// An error in the backend happened
    #[error("The backend has encountered an error")]
    Backend,
    /// If `Database::write_safe` is used and the closure panics, this error is returned
    #[error("The write operation paniced but got caught")]
    WritePanic,
    /// If an internal invariant is not held up, we return this instead of panicking
    #[error("An internal error, please report it to the developers. Your data should be safe.")]
    Internal,
}

/// The error type used by Rustbreak
pub type Result<T> = anyhow::Result<T>;
