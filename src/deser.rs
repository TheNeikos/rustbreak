/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
use std::io::Read;

use failure;
use serde::Serialize;
use serde::de::DeserializeOwned;

#[cfg(feature = "ron_enc")]
pub use self::ron::Ron;

#[cfg(feature = "yaml_enc")]
pub use self::yaml::Yaml;

#[cfg(feature = "bin_enc")]
pub use self::bincode::Bincode;

/// A trait to bundle serializer and deserializer in a simple struct
///
/// It should preferably be an struct: one that does not have any members.
///
/// # Example
///
/// For an imaginary serde compatible encoding scheme 'Frobnar', an example implementation can look
/// like this:
///
/// ```rust
/// extern crate rustbreak;
/// extern crate serde;
/// #[macro_use] extern crate failure;
///
/// use std::io::Read;
/// use serde::Serialize;
/// use serde::de::Deserialize;
/// use failure::{Context, Fail, Backtrace};
///
/// use rustbreak::error;
/// use rustbreak::deser::DeSerializer;
///
/// #[derive(Fail, Debug)]
/// #[fail(display = "A FrobnarError ocurred")]
/// struct FrobnarError;
///
/// fn to_frobnar<T: Serialize>(input: &T) -> Vec<u8> {
///     unimplemented!(); // implementation not specified
/// }
///
/// fn from_frobnar<'r, T: Deserialize<'r> + 'r, R: Read>(input: &R) -> Result<T, FrobnarError> {
///     unimplemented!(); // implementation not specified
/// }
///
/// #[derive(Debug, Default, Clone)]
/// struct Frobnar;
///
/// impl<T: Serialize> DeSerializer<T> for Frobnar
///     where for<'de> T: Deserialize<'de>
/// {
///     fn serialize(&self, val: &T) -> Result<Vec<u8>, failure::Error> {
///         Ok(to_frobnar(val))
///     }
///
///     fn deserialize<R: Read>(&self, s: R) -> Result<T, failure::Error> {
///         Ok(from_frobnar(&s)?)
///     }
/// }
///
/// fn main() {}
/// ```
pub trait DeSerializer<T: Serialize + DeserializeOwned> : ::std::default::Default + Send + Sync + Clone {
    /// Serializes a given value to a String
    fn serialize(&self, val: &T) -> Result<Vec<u8>, failure::Error>;
    /// Deserializes a String to a value
    fn deserialize<R: Read>(&self, s: R) -> Result<T, failure::Error>;
}


#[cfg(feature = "ron_enc")]
mod ron {
    use std::io::Read;

    use failure;
    use serde::Serialize;
    use serde::de::DeserializeOwned;
    use failure::ResultExt;

    use ron::ser::to_string_pretty as to_ron_string;
    use ron::ser::PrettyConfig;
    use ron::de::from_reader as from_ron_string;

    use error;
    use deser::DeSerializer;

    /// The Struct that allows you to use `ron` the Rusty Object Notation
    #[derive(Debug, Default, Clone)]
    pub struct Ron;

    impl<T: Serialize + DeserializeOwned> DeSerializer<T> for Ron {
        fn serialize(&self, val: &T) -> Result<Vec<u8>, failure::Error> {
            Ok(to_ron_string(val, PrettyConfig::default()).map(String::into_bytes)
                .context(error::RustbreakErrorKind::Serialization)?)
        }
        fn deserialize<R: Read>(&self, s: R) -> Result<T, failure::Error> {
            Ok(from_ron_string(s).context(error::RustbreakErrorKind::Deserialization)?)
        }
    }
}

#[cfg(feature = "yaml_enc")]
mod yaml {
    use std::io::Read;

    use failure;
    use serde_yaml::{to_string as to_yaml_string, from_reader as from_yaml_string};
    use serde::Serialize;
    use serde::de::DeserializeOwned;
    use failure::ResultExt;

    use error;
    use deser::DeSerializer;

    /// The struct that allows you to use yaml
    #[derive(Debug, Default, Clone)]
    pub struct Yaml;

    impl<T: Serialize + DeserializeOwned> DeSerializer<T> for Yaml {
        fn serialize(&self, val: &T) -> Result<Vec<u8>, failure::Error> {
            Ok(to_yaml_string(val)
                .map(String::into_bytes)
                .context(error::RustbreakErrorKind::Serialization)?)
        }
        fn deserialize<R: Read>(&self, s: R) -> Result<T, failure::Error> {
            Ok(from_yaml_string(s).context(error::RustbreakErrorKind::Deserialization)?)
        }
    }
}

#[cfg(feature = "bin_enc")]
mod bincode {
    use std::io::Read;

    use failure;
    use bincode::{deserialize_from, serialize};
    use serde::Serialize;
    use serde::de::DeserializeOwned;
    use failure::ResultExt;

    use error;
    use deser::DeSerializer;

    /// The struct that allows you to use bincode
    #[derive(Debug, Default, Clone)]
    pub struct Bincode;

    impl<T: Serialize + DeserializeOwned> DeSerializer<T> for Bincode {
        fn serialize(&self, val: &T) -> Result<Vec<u8>, failure::Error> {
            Ok(serialize(val).context(error::RustbreakErrorKind::Serialization)?)
        }
        fn deserialize<R: Read>(&self, s: R) -> Result<T, failure::Error> {
            Ok(deserialize_from(s).context(error::RustbreakErrorKind::Deserialization)?)
        }
    }
}
