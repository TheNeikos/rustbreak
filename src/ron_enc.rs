use serde::Serialize;
use serde::de::DeserializeOwned;
use ron::{ser, de};

pub type Repr = String;
pub type SerializeError = ser::Error;
pub type DeserializeError = de::Error;

pub fn serialize<T>(value: &T) -> ser::Result<String>
    where T: Serialize
{
    ser::to_string(value)
}

pub fn deserialize<T, I>(bytes: &I) -> Result<T, ::error::BreakError>
    where T: DeserializeOwned,
    I: AsRef<[u8]>
{
    let string = try!(String::from_utf8(bytes.as_ref().to_vec()));
    Ok(de::from_str(&string)?)
}
