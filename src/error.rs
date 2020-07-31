/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// An error returned by a `DeSer` implementor
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
#[allow(clippy::empty_enum)] // This can occur when no desers have beeen enabled
pub enum DeSerError {
    #[cfg(feature = "yaml_enc")]
    /// An error occured with Yaml
    #[error("An error with yaml occured")]
    Yaml(#[from] serde_yaml::Error),
    #[cfg(feature = "ron_enc")]
    /// An error occured with Ron
    #[error("An error with Ron occured")]
    Ron(#[from] ron::Error),
    #[cfg(feature = "bin_enc")]
    /// An error occured with Bincode
    #[error("An error with Bincode occured")]
    Bincode(#[from] std::boxed::Box<bincode::ErrorKind>),
    /// An internal error to Rustbreak occured
    #[error("An internal error to rustbreak occured, please report it to the maintainers")]
    Internal(String),
    #[cfg(feature = "other_errors")]
    /// A dynamic error occured
    ///
    /// Most likely the custom `DeSer` implementation has thrown an error, consult its documentation
    /// for more information
    ///
    /// **Important**: This can only be used if the `other_errors` feature is enabled
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// An error returned by a Backend implementor
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum BackendError {
    /// An error occured from the tempfile
    #[error("An error while persisting the file occured")]
    TempFile(#[from] tempfile::PersistError),
    /// An I/O Error occured
    #[error("An I/O Error occured")]
    Io(#[from] std::io::Error),
    /// An internal error to Rustbreak occured
    #[error("An internal error to rustbreak occured, please report it to the maintainers")]
    Internal(String),
    #[cfg(feature = "other_errors")]
    /// A dynamic error occured
    ///
    /// Most likely the custom `Backend` implementation has thrown an error, consult its documentation
    /// for more information
    ///
    /// **Important**: This can only be used if the `other_errors` feature is enabled
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// The different kinds of errors that can be returned
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum RustbreakError {
    /// A context error when a DeSerialization failed
    #[error("Could not deserialize the value")]
    DeSerialization(#[from] DeSerError),
    /// This error is returned if the `Database` is poisoned. See
    /// `Database::write` for details
    #[error("The database has been poisoned")]
    Poison,
    /// An error in the backend happened
    #[error("The backend has encountered an error")]
    Backend(#[from] BackendError),
    /// If `Database::write_safe` is used and the closure panics, this error is
    /// returned
    #[error("The write operation paniced but got caught")]
    WritePanic,
}

/// A simple type alias for errors
pub type Result<T> = std::result::Result<T, RustbreakError>;
/// The type alias used for backends
pub type BackendResult<T> = std::result::Result<T, BackendError>;
/// The type alias used for `DeSer`s
pub type DeSerResult<T> = std::result::Result<T, DeSerError>;
