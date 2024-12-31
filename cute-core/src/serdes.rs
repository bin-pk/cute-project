use serde::{Deserialize, Serialize};

pub fn bin_serialize<T : Serialize>(input : T) -> Result<Vec<u8>, std::io::Error>{
    bincode::serialize(&input).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}

pub fn bin_deserialize<'a, T : Deserialize<'a>>(input: &'a [u8]) -> Result<T, std::io::Error>{
    bincode::deserialize(input).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}
