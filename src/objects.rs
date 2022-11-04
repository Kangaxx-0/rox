use std::{fmt, ops::Deref, ops::DerefMut};

use crate::{chunk::Chunk, utils::hash, value::Value};
use gc::{Finalize, Trace};
pub const MAX_UPVALUES: usize = 256;

#[derive(Hash, Eq, PartialEq, Debug, Clone, PartialOrd, Trace, Finalize)]
pub struct HashKeyString {
    pub value: String,
    pub hash: u64,
}

// An upvalue refers to a local variable in an enclosing function.
#[derive(PartialEq, Eq, Debug, Clone, PartialOrd, Trace, Finalize)]
pub struct UpValue {
    pub index: usize, // It takes the address of the slot where the closed-over variable lives
    pub is_local: bool,
}

impl UpValue {
    pub fn new(index: usize, is_local: bool) -> Self {
        Self { index, is_local }
    }
}

#[derive(PartialEq, Debug, Clone, PartialOrd, Trace, Finalize)]
pub struct ObjUpValue {
    pub location: usize,
    pub closed: Option<Value>,
}

impl Deref for ObjUpValue {
    type Target = Value;

    fn deref(&self) -> &Self::Target {
        self.closed.as_ref().expect("Upvalue not closed")
    }
}

impl DerefMut for ObjUpValue {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self.closed {
            Some(ref mut value) => value,
            None => panic!("Cannot deref a closed upvalue"),
        }
    }
}

impl ObjUpValue {
    pub fn new(location: usize) -> Self {
        Self {
            location,
            closed: None,
        }
    }
}

impl Default for ObjUpValue {
    fn default() -> Self {
        Self::new(0)
    }
}

// Define a new type for the function.
#[derive(PartialEq, Eq, Debug, Clone, PartialOrd, Trace, Finalize)]
pub struct ObjFunction {
    pub arity: u8,
    pub chunk: Chunk,
    pub name: HashKeyString,
    // upvalues is a level of indirection to the local variable, it refers to
    // a local variable in the enclosing/parent function, it keeps track the closed-over like how stack
    // slot index works
    pub upvalues: Vec<UpValue>,
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
            upvalues: Vec::with_capacity(MAX_UPVALUES),
        }
    }
}

// Define a new type for closures.
#[derive(PartialEq, Debug, Clone, PartialOrd, Trace, Finalize)]
pub struct ObjClosure {
    pub function: ObjFunction, // closure shares the same code and constants as the function
    pub obj_upvalues: Vec<ObjUpValue>, // every closure maintains an array of upvalues
}

impl ObjClosure {
    pub fn new(function: ObjFunction) -> Self {
        let upvalues = Vec::with_capacity(function.upvalues.len());
        Self {
            function,
            obj_upvalues: upvalues,
        }
    }
}

// Define a new type for native functions
#[derive(Clone, Trace, Finalize)]
pub struct ObjNative {
    pub name: HashKeyString,
    #[unsafe_ignore_trace]
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
