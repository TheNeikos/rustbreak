/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::io::Read;

use serde::Serialize;
use serde::de::DeserializeOwned;

use ron::ser::pretty::to_string as to_ron_string;
use ron::de::from_reader as from_ron_string;

#[cfg(feature = "yaml")]
pub use self::yaml::Yaml;

#[cfg(feature = "bin")]
pub use self::bincode::Bincode;

/// A trait to bundle serializer and deserializer
pub trait DeSerializer<T: Serialize + DeserializeOwned> {
    /// Associated error with serialization
    type SerError;
    /// Associated error with deserialization
    type DeError;

    /// Serializes a given value to a String
    fn serialize(&self, val: &T) -> Result<String, Self::SerError>;
    /// Deserializes a String to a value
    fn deserialize<R: Read>(&self, s: R) -> Result<T, Self::DeError>;
}

/// The Struct that allows you to use `ron` the Rusty Object Notation
#[derive(Debug)]
pub struct Ron;

impl<T: Serialize + DeserializeOwned> DeSerializer<T> for Ron {
    type SerError = ::ron::ser::Error;
    type DeError = ::ron::de::Error;
    fn serialize(&self, val: &T) -> Result<String, Self::SerError> {
        to_ron_string(val)
    }
    fn deserialize<R: Read>(&self, s: R) -> Result<T, Self::DeError> {
        from_ron_string(s)
    }
}

#[cfg(feature = "yaml")]
mod yaml {
    use std::io::Read;

    use serde_yaml::{to_string as to_yaml_string, from_reader as from_yaml_string};
    use serde::Serialize;
    use serde::de::DeserializeOwned;

    use deser::DeSerializer;

    /// The struct that allows you to use yaml
    #[derive(Debug)]
    pub struct Yaml;

    impl<T: Serialize + DeserializeOwned> DeSerializer<T> for Yaml {
        type SerError = ::serde_yaml::Error;
        type DeError = ::serde_yaml::Error;
        fn serialize(&self, val: &T) -> Result<String, Self::SerError> {
            to_yaml_string(val)
        }
        fn deserialize<R: Read>(&self, s: R) -> Result<T, Self::DeError> {
            from_yaml_string(s)
        }
    }
}

#[cfg(feature = "bin")]
mod bincode {
    use std::io::Read;

    use bincode::{serialize as to_bincode_string, deserialize as from_bincode_string};
    use base64::{encode, decode};
    use serde::Serialize;
    use serde::de::DeserializeOwned;

    use deser::DeSerializer;

    error_chain! {
        types {
            Error, ErrorKind, ResultExt;
        }

        foreign_links {
            Bincode(::bincode::Error);
            Io(::std::io::Error);
            Base64(::base64::DecodeError);
            Format(::std::string::FromUtf8Error);
        }
    }

    /// The struct that allows you to use bincode
    #[derive(Debug)]
    pub struct Bincode;

    impl<T: Serialize + DeserializeOwned> DeSerializer<T> for Bincode {
        type SerError = Error;
        type DeError = Error;
        fn serialize(&self, val: &T) -> Result<String, Self::SerError> {
            let res = to_bincode_string(val, ::bincode::Infinite)?;
            Ok(encode(&res))
        }
        fn deserialize<R: Read>(&self, mut s: R) -> Result<T, Self::DeError> {
            let mut string = String::new();
            s.read_to_string(&mut string)?;
            Ok(from_bincode_string(String::from_utf8(decode(&string)?)?.as_bytes())?)
        }
    }
}
