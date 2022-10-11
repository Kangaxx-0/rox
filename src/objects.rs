use std::fmt::Display;

use crate::{chunk::Chunk, utils::hash};

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub struct HashKeyString {
    pub value: String,
    pub hash: u64,
}

pub struct ObjFunction {
    pub arity: u8,
    pub chunk: Chunk,
    pub name: HashKeyString,
}

impl ObjFunction {
    pub fn new(name: String) -> Self {
        Self {
            arity: 0,
            chunk: Chunk::new(),
            name: HashKeyString {
                hash: hash(&name),
                value: name,
            },
        }
    }
}

impl Display for ObjFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn {}>", self.name.value)
    }
}
