use crate::{chunk::Chunk, utils::hash};

#[derive(Hash, Eq, PartialEq, Debug, Clone, PartialOrd)]
pub struct HashKeyString {
    pub value: String,
    pub hash: u64,
}

#[derive(PartialEq, Debug, Clone, PartialOrd)]
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
