/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// The Error type exported by BreakError, usually you only need to check against NotFound,
/// however it might be useful sometimes to get other errors.
#[derive(Debug)]
pub enum BreakError<Se, De> {
    /// An error returned when doing file operations, this might happen by opening, closing,
    /// locking or flushing
    Io(::std::io::Error),
    /// Error when reading a formatted String
    Format(::std::string::FromUtf8Error),
    Serialize(Se),
    Deserialize(De),
    Poison
}

impl<T, Se, De> From<::std::sync::PoisonError<T>> for BreakError<Se, De> {
    fn from(_: ::std::sync::PoisonError<T>) -> BreakError<Se, De> {
        BreakError::Poison
    }
}

impl<Se, De> From<::std::io::Error> for BreakError<Se, De> {
    fn from(e: ::std::io::Error) -> BreakError<Se, De> {
        BreakError::Io(e)
    }
}


pub type BreakResult<T, Se, De> = Result<T, BreakError<Se, De>>;
