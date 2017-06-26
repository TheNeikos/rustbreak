use serde::{Serialize, Deserialize};
use bincode::{self, serialize as bin_serialize, deserialize as bin_deserialize};

pub type Repr = Vec<u8>;
pub type SerializeError = bincode::Error;
pub type DeserializeError = ();

pub fn serialize<T>(value: &T) -> bincode::Result<Vec<u8>>
    where T: Serialize
{
    bin_serialize(value, bincode::Infinite)
}

pub fn deserialize<'a, T>(bytes: &'a [u8]) -> Result<T, bincode::Error>
    where T: Deserialize<'a>
{
    bin_deserialize(bytes)
}
