/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

quick_error! {
    /// The Error type exported by BreakError, usually you only need to check against NotFound,
    /// however it might be useful sometimes to get other errors.
    #[derive(Debug)]
    pub enum BreakError {
        /// An error returned when doing file operations, this might happen by opening, closing,
        /// locking or flushing
        Io(err: ::std::io::Error) {
            from()
        }
        /// Error when reading a formatted String
        Format(err: ::std::string::FromUtf8Error) {
            from()
        }

        Serialize { }

        Deserialize { }

        Poison { }
    }
}

impl<T> From<::std::sync::PoisonError<T>> for BreakError {
    fn from(_: ::std::sync::PoisonError<T>) -> BreakError {
        BreakError::Poison
    }
}

pub type BreakResult<T> = Result<T, BreakError>;
