use std::fmt::Display;

use crate::objects::{ObjClosure, ObjFunction, ObjNative};

use gc::{Finalize, Gc, Trace};

#[derive(Debug, Clone, PartialEq, PartialOrd, Trace, Finalize)]
pub enum Value {
    Deault,
    Bool(bool),
    Nil,
    Number(f64),
    String(Gc<String>),
    Function(Gc<ObjFunction>),
    NativeFunction(Gc<ObjNative>),
    Closure(Gc<ObjClosure>),
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
            Value::Function(_) => write!(f, "Function"),
            Value::Closure(_) => write!(f, "Closure"),
        }
    }
}
