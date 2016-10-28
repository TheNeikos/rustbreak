use serde::{Serialize, Deserialize};
use bincode::serde::{serialize as bin_serialize, deserialize as bin_deserialize};
use bincode::serde::{SerializeResult, DeserializeResult};
use bincode::SizeLimit;

pub type Repr = Vec<u8>;
pub use bincode::serde::DeserializeError;
pub use bincode::serde::SerializeError;

pub fn serialize<T>(value: &T) -> SerializeResult<Vec<u8>>
    where T: Serialize
{
    bin_serialize(value, SizeLimit::Infinite)
}

pub fn deserialize<T>(bytes: &[u8]) -> DeserializeResult<T>
    where T: Deserialize
{
    bin_deserialize(bytes)
}
