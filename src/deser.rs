/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
use std::io::Read;

use serde::Serialize;
use serde::de::DeserializeOwned;

use error;

#[cfg(feature = "ron_enc")]
pub use self::ron::Ron;

#[cfg(feature = "yaml_enc")]
pub use self::yaml::Yaml;

#[cfg(feature = "bin_enc")]
pub use self::bincode::Bincode;

/// A trait to bundle serializer and deserializer
pub trait DeSerializer<T: Serialize + DeserializeOwned> : ::std::default::Default + Send + Sync + Clone {
    /// Serializes a given value to a String
    fn serialize(&self, val: &T) -> error::Result<Vec<u8>>;
    /// Deserializes a String to a value
    fn deserialize<R: Read>(&self, s: R) -> error::Result<T>;
}


#[cfg(feature = "ron_enc")]
mod ron {
    use std::io::Read;

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
        fn serialize(&self, val: &T) -> error::Result<Vec<u8>> {
            Ok(to_ron_string(val, PrettyConfig::default()).map(String::into_bytes)
                .context(error::RustbreakErrorKind::Serialization)?)
        }
        fn deserialize<R: Read>(&self, s: R) -> error::Result<T> {
            Ok(from_ron_string(s).context(error::RustbreakErrorKind::Deserialization)?)
        }
    }
}

#[cfg(feature = "yaml_enc")]
mod yaml {
    use std::io::Read;

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
        fn serialize(&self, val: &T) -> error::Result<Vec<u8>> {
            Ok(to_yaml_string(val)
                .map(String::into_bytes)
                .context(error::RustbreakErrorKind::Serialization)?)
        }
        fn deserialize<R: Read>(&self, s: R) -> error::Result<T> {
            Ok(from_yaml_string(s).context(error::RustbreakErrorKind::Deserialization)?)
        }
    }
}

#[cfg(feature = "bin_enc")]
mod bincode {
    use std::io::Read;

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
        fn serialize(&self, val: &T) -> error::Result<Vec<u8>> {
            let res = serialize(val).context(error::RustbreakErrorKind::Serialization)?;
            Ok(res)
        }
        fn deserialize<R: Read>(&self, s: R) -> error::Result<T> {
            Ok(deserialize_from(s).context(error::RustbreakErrorKind::Deserialization)?)
        }
    }
}
