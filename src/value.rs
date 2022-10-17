use std::fmt::Display;

use crate::objects::{ObjFunction, ObjNative};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Value {
    Deault,
    Bool(bool),
    Nil,
    Number(f64),
    String(String),
    Function(ObjFunction),
    NativeFunction(ObjNative),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Deault => write!(f, "Default"),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Nil => write!(f, "Nil"),
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::NativeFunction(_) => write!(f, "Native Function"),
            _ => write!(f, "Function"),
        }
    }
}
