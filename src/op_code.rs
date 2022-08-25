use std::fmt::Display;

// operation code.
#[derive(Debug, Clone, Copy)]
pub enum OpCode {
    Call(u8),
    Constant(u8),
    Return,
}

impl Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Call(v) => write!(f, "system call {}", v),
            Self::Constant(v) => write!(f, "constant {}", v),
            Self::Return => write!(f, "system return"),
        }
    }
}
