use serde::{Deserialize, Serialize};
use crate::errors::CuteError;

pub fn bin_serialize<T : Serialize>(input : T) -> Result<Vec<u8>, CuteError>{
    bincode::serialize(&input).map_err(|e| CuteError::serialize_invalid(e.to_string()))
}

pub fn bin_deserialize<'a, T : Deserialize<'a>>(input: &'a [u8]) -> Result<T, CuteError>{
    bincode::deserialize(input).map_err(|e| CuteError::deserialize_invalid(e.to_string()))
}
