/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::io::Read;

use serde::Serialize;
use serde::de::DeserializeOwned;

use ron::ser::pretty::to_string as to_ron_string;
use ron::de::from_reader as from_ron_string;

#[cfg(feature = "yaml")]
use serde_yaml::{to_string as to_yaml_string, from_reader as from_yaml_string};

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
/// The struct that allows you to use yaml
#[derive(Debug)]
pub struct Yaml;

#[cfg(feature = "yaml")]
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
