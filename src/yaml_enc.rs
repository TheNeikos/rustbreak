use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_yaml::Result as YamlResult;
use serde_yaml::Error;

pub type Repr = String;
pub type SerializeError = Error;
pub type DeserializeError = ();

pub fn serialize<T>(value: &T) -> YamlResult<String>
    where T: Serialize
{
    ::serde_yaml::to_string(value)
}

pub fn deserialize<T, I: AsRef<[u8]>>(bytes: &I) -> Result<T, ::error::BreakError>
    where T: DeserializeOwned
{

    let string = try!(String::from_utf8(bytes.as_ref().to_vec()));
    let des = try!(::serde_yaml::from_str(&string));
    Ok(des)
}
