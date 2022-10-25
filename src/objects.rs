use std::fmt;

use crate::{chunk::Chunk, utils::hash, value::Value};

#[derive(Hash, Eq, PartialEq, Debug, Clone, PartialOrd)]
pub struct HashKeyString {
    pub value: String,
    pub hash: u64,
}

// Define a new type for the function.
#[derive(PartialEq, Eq, Debug, Clone, PartialOrd)]
pub struct ObjFunction {
    pub arity: u8,
    pub chunk: Chunk,
    pub name: HashKeyString,
    // number of upvalues the functions uses. It is stored here because
    // we need know the count at runtime
    pub upvalue_count: usize,
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
            upvalue_count: 0,
        }
    }
}

// Define a new type for closures.
#[derive(PartialEq, Eq, Debug, Clone, PartialOrd)]
pub struct ObjClosure {
    pub function: ObjFunction, // closure shares the same code and constants as the function
}

impl ObjClosure {
    pub fn new(function: ObjFunction) -> Self {
        Self { function }
    }
}

// Define a new type for native functions
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
