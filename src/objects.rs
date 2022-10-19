use std::fmt;

use crate::{chunk::Chunk, utils::hash, value::Value};

#[derive(Hash, Eq, PartialEq, Debug, Clone, PartialOrd)]
pub struct HashKeyString {
    pub value: String,
    pub hash: u64,
}

#[derive(PartialEq, Eq, Debug, Clone, PartialOrd)]
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

#[derive(Clone)]
pub struct ObjNative {
    pub name: HashKeyString,
    pub func: fn(&[Value]) -> Value,
}

// Impl below traits because we have a function pointer in ObjNative
impl fmt::Debug for ObjNative {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Native Function: <fn>")
    }
}

impl PartialOrd for ObjNative {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.name.value.cmp(&other.name.value))
    }
}

impl PartialEq for ObjNative {
    fn eq(&self, other: &Self) -> bool {
        self.name.value == other.name.value
    }
}

impl ObjNative {
    pub fn new(name: String, function: fn(&[Value]) -> Value) -> Self {
        Self {
            name: HashKeyString {
                hash: hash(&name),
                value: name,
            },
            func: function,
        }
    }
}
